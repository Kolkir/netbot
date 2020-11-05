extern crate opencv;
use super::camera_msg;
use super::camera_prop_msg;
use super::image_msg;
use super::message;
use super::move_msg;
use super::server;
use camera_msg::{GetCameraListMsg, RecvCameraListMsg};
use camera_prop_msg::{GetCameraPropMsg, RecvCameraPropMsg, SetCameraPropMsg};
use image_msg::RecvImageMsg;
use message::{HelloMsg, MessageId, RecvMessage, StopMsg};
use move_msg::MoveMsg;
use opencv::{core, imgcodecs, imgproc, prelude::*};
use server::Server;
use std::collections::HashMap;
use std::error::Error;
use std::net::Ipv4Addr;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread;

type ResolutionsMap = HashMap<u8, Vec<(i32, i32)>>;

struct ImageProcessor {
    images: HashMap<u8, Mat>,
    scaled_images: HashMap<u8, Mat>,
    out_resolution: (i32, i32),
}

pub struct Robot {
    image_processor: Arc<Mutex<ImageProcessor>>,
    camera_list: Arc<Mutex<Option<Vec<u8>>>>,
    camera_resolutions: Arc<Mutex<ResolutionsMap>>,
    move_speed: u8,
    server: Server,
    recv_thread_handle: Option<thread::JoinHandle<()>>,
    stop_thread_flag: Arc<AtomicBool>,
    bot_is_moving: bool,
}

fn recv_thread(
    stop_flag: Arc<AtomicBool>,
    mut server: Server,
    image_processor: Arc<Mutex<ImageProcessor>>,
    camera_list: Arc<Mutex<Option<Vec<u8>>>>,
    camera_resolutions: Arc<Mutex<ResolutionsMap>>,
) {
    println!("Robot thread started!");
    while !stop_flag.as_ref().load(std::sync::atomic::Ordering::SeqCst) {
        let (id, data) = server
            .recv()
            .expect("Failed to receive message from tcp stream");
        let msg_id = MessageId::from(id);
        match msg_id {
            MessageId::RecvCameraList => {
                let mut cam_list_msg = RecvCameraListMsg::new();
                cam_list_msg.from_bytes(&data);
                *camera_list.lock().unwrap() = Some(cam_list_msg.camera_list);
            }
            MessageId::RecvCameraProp => {
                let mut cam_prop_msg = RecvCameraPropMsg::new();
                cam_prop_msg.from_bytes(&data);
                let (width, height): (Vec<_>, Vec<_>) = cam_prop_msg
                    .camera_prop
                    .into_iter()
                    .enumerate()
                    .partition(|&(i, _)| (i % 2) == 0);
                let resolutions: Vec<(i32, i32)> = width
                    .iter()
                    .map(|v| v.1 as i32)
                    .zip(height.iter().map(|v| v.1 as i32))
                    .collect();

                camera_resolutions
                    .lock()
                    .unwrap()
                    .insert(cam_prop_msg.camera_id, resolutions);
            }
            MessageId::RecvImage => {
                let mut image_msg = RecvImageMsg::new();
                image_msg.from_bytes(&data);
                image_processor
                    .lock()
                    .unwrap()
                    .process_recv_image_msg(image_msg)
                    .expect("Failed to process image");
            }
            _ => unreachable!(),
        }
    }
}

impl ImageProcessor {
    pub fn new() -> Result<ImageProcessor, Box<dyn Error>> {
        Ok(ImageProcessor {
            images: HashMap::new(),
            scaled_images: HashMap::new(),
            out_resolution: (640, 489),
        })
    }

    pub fn set_out_resolution(&mut self, width: i32, height: i32) {
        self.out_resolution = (width, height);
    }

    pub fn get_scaled_image_data(&self, camera_id: u8) -> Option<Vec<u8>> {
        let img_mat_scaled = self.scaled_images.get(&camera_id);
        match img_mat_scaled {
            Some(mat) => Some(mat.data_typed::<u8>().unwrap().to_vec()),
            None => None,
        }
    }

    pub fn process_recv_image_msg(
        &mut self,
        recv_img_msg: RecvImageMsg,
    ) -> Result<(), Box<dyn Error>> {
        // println!(
        //     "Recv image : {0} x {1} x {2}",
        //     recv_img_msg.channels, recv_img_msg.frame_width, recv_img_msg.frame_height
        // );

        let img_mat = self
            .images
            .entry(recv_img_msg.camera_id)
            .or_insert(Mat::default()?);
        if recv_img_msg.encoded == 1 {
            let cv_data_vector = core::Vector::<u8>::from(recv_img_msg.data);
            *img_mat = imgcodecs::imdecode(&cv_data_vector, imgcodecs::IMREAD_UNCHANGED)?;
        } else {
            *img_mat = Mat::from_slice(&recv_img_msg.data)?;
            *img_mat = img_mat.reshape(3, recv_img_msg.frame_height as i32)?;
            let mut rgb_mat = Mat::default()?;
            imgproc::cvt_color(img_mat, &mut rgb_mat, imgproc::COLOR_BGR2RGB, 3)?;
            *img_mat = rgb_mat;
        }
        if img_mat.empty()? {
            panic!("Failed to decode frame");
        }
        // println!(
        //     "depth {0} channels {1} width {2} height {3} size {4} step {5}",
        //     img_mat.depth()?,
        //     img_mat.channels()?,
        //     img_mat.cols(),
        //     img_mat.rows(),
        //     img_mat.total()? * img_mat.elem_size()?,
        //     img_mat.step1(0)?
        // );

        // scale image
        let new_size = core::Size {
            width: self.out_resolution.0,
            height: self.out_resolution.1,
        };

        let img_mat_scaled = self
            .scaled_images
            .entry(recv_img_msg.camera_id)
            .or_insert(Mat::default()?);
        imgproc::resize(
            &self.images.get(&recv_img_msg.camera_id).unwrap(),
            img_mat_scaled,
            new_size,
            0.0,
            0.0,
            imgproc::INTER_AREA,
        )?;
        *img_mat_scaled = img_mat_scaled.reshape(1, img_mat_scaled.rows() * 3)?;
        Ok(())
    }
}

impl Robot {
    pub fn new() -> Result<Robot, Box<dyn Error>> {
        Ok(Robot {
            image_processor: Arc::new(Mutex::new(ImageProcessor::new()?)),
            camera_list: Arc::new(Mutex::new(None)),
            camera_resolutions: Arc::new(Mutex::new(HashMap::new())),
            move_speed: 10,
            server: Server::new(),
            recv_thread_handle: None,
            stop_thread_flag: Arc::new(AtomicBool::new(false)),
            bot_is_moving: false,
        })
    }

    pub fn init(&mut self, addr: Ipv4Addr, port: u16) -> Result<(), Box<dyn Error>> {
        self.server.wait_client(addr, port)?;

        let (id, _) = self
            .server
            .recv()
            .expect("Failed to receive message from tcp stream");
        if MessageId::from(id) == MessageId::Hello {
            let msg = Box::new(HelloMsg {});
            println!("Handshake received!");
            self.server.send(msg).expect("Failed to send Hello message");
        } else {
            panic!("Handshake failed");
        }

        let server_recv = self.server.clone();
        let stop_flag = Arc::clone(&self.stop_thread_flag);
        let image_processor_clone = Arc::clone(&self.image_processor);
        let camera_list_clone = Arc::clone(&self.camera_list);
        let camera_resolutions_clone = Arc::clone(&self.camera_resolutions);

        self.recv_thread_handle = Some(thread::spawn(move || {
            recv_thread(
                stop_flag,
                server_recv,
                image_processor_clone,
                camera_list_clone,
                camera_resolutions_clone,
            )
        }));
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        self.stop_thread_flag
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.recv_thread_handle.take().map(thread::JoinHandle::join);
        self.server.send(Box::new(StopMsg {}))?;
        Ok(())
    }

    pub fn ask_camera_prop(&mut self, camera_id: u8) -> Result<(), Box<dyn Error>> {
        let mut get_msg = GetCameraPropMsg::new();
        get_msg.camera_id = camera_id;
        self.server.send(Box::new(get_msg))?;
        self.camera_resolutions
            .lock()
            .unwrap()
            .remove_entry(&camera_id);
        Ok(())
    }

    pub fn ask_camera_list(&mut self) -> Result<(), Box<dyn Error>> {
        let get_msg = GetCameraListMsg::new();
        self.server.send(Box::new(get_msg))?;
        *self.camera_list.lock().unwrap() = None;
        Ok(())
    }

    pub fn ask_set_camera_prop(
        &mut self,
        camera_id: u8,
        frame_width: u16,
        frame_height: u16,
        fps: u8,
        do_encode: bool,
    ) -> Result<(), Box<dyn Error>> {
        let mut set_msg = SetCameraPropMsg::new();
        set_msg.camera_id = camera_id;
        set_msg.frame_width = frame_width;
        set_msg.frame_height = frame_height;
        set_msg.fps = fps;
        set_msg.encode = if do_encode { 1 } else { 0 };
        self.server.send(Box::new(set_msg))?;
        Ok(())
    }

    pub fn ask_move_bot(&mut self, left_speed: u8, left_dir: u8, right_speed: u8, right_dir: u8) {
        let mut move_msg = MoveMsg::new();
        move_msg.left_speed = left_speed;
        move_msg.left_dir = left_dir;
        move_msg.right_speed = right_speed;
        move_msg.right_dir = right_dir;
        self.server
            .send(Box::new(move_msg))
            .expect("Failed to send move command");
        self.bot_is_moving = true;
    }

    pub fn rotate_left(&mut self) {
        if !self.bot_is_moving {
            self.ask_move_bot(0, 0, self.move_speed, 1);
        }
    }

    pub fn rotate_right(&mut self) {
        if !self.bot_is_moving {
            self.ask_move_bot(self.move_speed, 1, 0, 0);
        }
    }

    pub fn move_forward(&mut self) {
        if !self.bot_is_moving {
            self.ask_move_bot(self.move_speed, 1, self.move_speed, 1);
        }
    }

    pub fn move_backward(&mut self) {
        if !self.bot_is_moving {
            self.ask_move_bot(self.move_speed, 0, self.move_speed, 0);
        }
    }

    pub fn stop_moving(&mut self) {
        if self.bot_is_moving {
            self.ask_move_bot(0, 0, 0, 0);
            self.bot_is_moving = false;
        }
    }

    pub fn set_out_resolution(&mut self, width: i32, height: i32) {
        self.image_processor
            .lock()
            .unwrap()
            .set_out_resolution(width, height);
    }

    pub fn get_camera_resolutions(&self, camera_id: u8) -> Option<Vec<(i32, i32)>> {
        let cam_res_guard = self.camera_resolutions.lock();
        let resolutions = cam_res_guard.as_ref().unwrap().get(&camera_id);
        match resolutions {
            Some(res) => Some(res.clone()),
            None => None,
        }
    }

    pub fn get_cameras_resolutions(&self) -> ResolutionsMap {
        let cam_res_guard = self.camera_resolutions.lock();
        cam_res_guard.unwrap().clone()
    }

    pub fn get_camera_list(&self) -> Option<Vec<u8>> {
        self.camera_list.lock().unwrap().clone()
    }

    pub fn get_image(&self, camera_id: u8) -> Option<Vec<u8>> {
        self.image_processor
            .lock()
            .unwrap()
            .get_scaled_image_data(camera_id)
    }
}
