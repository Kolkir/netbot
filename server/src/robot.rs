extern crate opencv;
use super::camera_msg;
use super::camera_prop_msg;
use super::image_msg;
use super::message;
use super::move_msg;
use super::server;
use camera_msg::{GetCameraListMsg, RecvCameraListMsg};
use camera_prop_msg::{GetCameraPropMsg, RecvCameraPropMsg, SetCameraPropMsg};
use image_msg::{CaptureImageMsg, RecvImageMsg};
use message::{HelloMsg, Message, MessageId, StopMsg};
use move_msg::MoveMsg;
use opencv::{core, imgcodecs, imgproc, prelude::*};
use server::Server;
use std::collections::HashMap;
use std::error::Error;
use std::net::Ipv4Addr;

#[derive(Clone)]
pub struct Robot {
    camera_list: Vec<u8>,
    image_data: HashMap<u8, Mat>,
    image_res: HashMap<u8, (u16, u16)>,
    move_speed: u8,
    server: Server,
    img_mat_reduced: Mat,
}

impl Robot {
    pub fn new(addr: Ipv4Addr, port: u16) -> Result<Robot, Box<dyn Error>> {
        Ok(Robot {
            camera_list: Vec::new(),
            image_res: HashMap::new(),
            image_data: HashMap::new(),
            move_speed: 10,
            server: Server::new(addr, port)?,
            img_mat_reduced: Mat::default()?,
        })
    }

    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.server.wait_client()?;

        // handshake
        let mut hello_msg = HelloMsg {};
        self.server.recv(&mut hello_msg)?;
        if hello_msg.id() == MessageId::Hello as u8 {
            println!("Handshaking started");
            self.server.send(Box::new(HelloMsg {}))?;
        } else {
            panic!("Handshake failed!");
        }
        println!("Handshake completed");
        // receive bot cameras properties
        self.recv_camera_list()?;
        self.recv_camera_prop()?;
        // set bot cameras resolution
        self.set_max_camera_prop()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        self.server.send(Box::new(StopMsg {}))?;
        Ok(())
    }

    fn recv_camera_prop(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.camera_list.is_empty() {
            for cam_id in &self.camera_list {
                let mut get_msg = GetCameraPropMsg::new();
                get_msg.camera_id = *cam_id;
                self.server.send(Box::new(get_msg))?;
                let mut recv_msg = RecvCameraPropMsg::new();
                self.server.recv(&mut recv_msg)?;
                let (width, height): (Vec<_>, Vec<_>) = recv_msg
                    .camera_prop
                    .into_iter()
                    .enumerate()
                    .partition(|&(i, _)| (i % 2) == 0);
                let max_w = width.iter().max_by(|a, b| a.1.cmp(&b.1)).unwrap();
                let height_index = max_w.0 / 2;
                self.image_res
                    .insert(*cam_id, (max_w.1, height[height_index].1));
            }
        }
        Ok(())
    }

    pub fn get_max_frame_resolution(&self, cam_id: u8) -> Option<(u16, u16)> {
        match self.image_res.get(&cam_id) {
            Some(res) => Some(res.clone()),
            None => None,
        }
    }

    fn recv_camera_list(&mut self) -> Result<(), Box<dyn Error>> {
        let get_msg = GetCameraListMsg::new();
        self.server.send(Box::new(get_msg))?;
        let mut recv_msg = RecvCameraListMsg::new();
        self.server.recv(&mut recv_msg)?;
        self.camera_list = recv_msg.camera_list;
        Ok(())
    }

    pub fn get_camera_list(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(self.camera_list.clone())
    }

    pub fn set_max_camera_prop(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.camera_list.is_empty() {
            for cam_id in &self.camera_list {
                let mut set_msg = SetCameraPropMsg::new();
                set_msg.camera_id = *cam_id;
                let frame_resolution = self.get_max_frame_resolution(*cam_id).unwrap();
                set_msg.frame_width = frame_resolution.0;
                set_msg.frame_height = frame_resolution.1;
                self.server.send(Box::new(set_msg))?;
            }
        }
        Ok(())
    }
    pub fn capture_frame(
        &mut self,
        cam_id: u8,
        out_resolution: (u16, u16),
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut get_msg = CaptureImageMsg::new();
        get_msg.camera_id = cam_id;
        self.server.send(Box::new(get_msg))?;

        let mut recv_msg = RecvImageMsg::new();
        self.server.recv(&mut recv_msg)?;

        // println!(
        //     "Recv image : {0} x {1} x {2}",
        //     recv_msg.channels, recv_msg.frame_width, recv_msg.frame_height
        // );
        // decode image
        let cv_data_vector = core::Vector::<u8>::from(recv_msg.data);
        let img_mat = imgcodecs::imdecode(&cv_data_vector, imgcodecs::IMREAD_UNCHANGED)?;
        if img_mat.empty()? {
            panic!("Filed to decode frame");
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
        // save last frame for camera
        self.image_data.insert(cam_id, img_mat);
        // scale image
        //let mut img_mat = Mat::from_slice(self.image_data.get(&cam_id).unwrap()).unwrap();
        //img_mat = img_mat.reshape(3, recv_msg.frame_height as i32)?;
        let new_size = core::Size {
            width: out_resolution.0 as i32,
            height: out_resolution.1 as i32,
        };
        imgproc::resize(
            &self.image_data.get(&cam_id).unwrap(),
            &mut self.img_mat_reduced,
            new_size,
            0.0,
            0.0,
            imgproc::INTER_AREA,
        )?;

        self.img_mat_reduced = self
            .img_mat_reduced
            .reshape(1, self.img_mat_reduced.rows() * 3)?;
        Ok(self.img_mat_reduced.data_typed::<u8>().unwrap().to_vec())
    }

    pub fn move_bot(
        &mut self,
        left_speed: u8,
        left_dir: u8,
        right_speed: u8,
        right_dir: u8,
    ) -> Result<(), Box<dyn Error>> {
        let mut move_msg = MoveMsg::new();
        move_msg.left_speed = left_speed;
        move_msg.left_dir = left_dir;
        move_msg.right_speed = right_speed;
        move_msg.right_dir = right_dir;
        self.server.send(Box::new(move_msg))?;
        Ok(())
    }
}
