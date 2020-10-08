extern crate opencv;
use super::camera_msg;
use super::camera_prop_msg;
use super::image_msg;
use super::message;
use super::move_msg;
use super::server;
use camera_msg::{GetCameraListMsg, RecvCameraListMsg};
use camera_prop_msg::{GetCameraPropMsg, RecvCameraPropMsg};
use image_msg::{CaptureImageMsg, RecvImageMsg};
use message::{HelloMsg, Message, MessageId};
use move_msg::MoveMsg;
use opencv::{core, imgproc, prelude::*};
use server::Server;
use std::collections::HashMap;
use std::error::Error;
use std::net::Ipv4Addr;

#[derive(Clone)]
struct Robot {
    camera_list: Vec<u8>,
    image_data: HashMap<u8, Vec<u8>>,
    image_res: HashMap<u8, (u16, u16)>,
    move_speed: u8,
    server: Server,
}

impl Robot {
    pub fn new(addr: Ipv4Addr, port: u16) -> Result<Robot, Box<dyn Error>> {
        Ok(Robot {
            camera_list: Vec::new(),
            image_res: HashMap::new(),
            image_data: HashMap::new(),
            move_speed: 10,
            server: Server::new(addr, port)?,
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

    pub fn capture_frame(
        &mut self,
        cam_id: u8,
        out_resolution: (u16, u16),
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut get_msg = CaptureImageMsg::new();
        get_msg.camera_id = cam_id;
        let frame_resolution = self.get_max_frame_resolution(cam_id).unwrap();
        get_msg.frame_width = frame_resolution.0;
        get_msg.frame_height = frame_resolution.1;
        self.server.send(Box::new(get_msg))?;

        let mut recv_msg = RecvImageMsg::new();
        self.server.recv(&mut recv_msg)?;

        println!(
            "Recv image : {0} x {1} x {2}",
            recv_msg.channels, recv_msg.frame_width, recv_msg.frame_height
        );
        // save last frame for camera
        self.image_data
            .entry(cam_id)
            .and_modify(|entry| entry.clone_from_slice(&recv_msg.data))
            .or_insert(recv_msg.data);

        // scale image
        let img_mat = Mat::from_slice(self.image_data.get(&cam_id).unwrap()).unwrap();
        let mut img_mat_reduced = Mat::default()?;
        imgproc::resize(
            &img_mat,
            &mut img_mat_reduced,
            core::Size {
                width: out_resolution.0 as i32,
                height: out_resolution.1 as i32,
            },
            0.0,
            0.0,
            imgproc::INTER_LINEAR,
        )?;

        Ok(img_mat_reduced.data_typed::<u8>().unwrap().to_vec())
    }

    fn move_bot(
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

    pub fn move_forward(&mut self) -> Result<(), Box<dyn Error>> {
        self.move_bot(self.move_speed, 1, self.move_speed, 1)
    }

    pub fn move_backward(&mut self) -> Result<(), Box<dyn Error>> {
        self.move_bot(self.move_speed, 0, self.move_speed, 0)
    }

    pub fn rotate_right(&mut self) -> Result<(), Box<dyn Error>> {
        self.move_bot(self.move_speed, 1, 0, 0)
    }

    pub fn rotate_left(&mut self) -> Result<(), Box<dyn Error>> {
        self.move_bot(0, 0, self.move_speed, 1)
    }
}
