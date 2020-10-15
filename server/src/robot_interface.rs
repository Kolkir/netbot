use std::collections::HashMap;
use std::collections::VecDeque;
use std::error::Error;
use std::net::Ipv4Addr;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use super::camera_msg;
use super::image_msg;
use super::message;
use super::move_msg;
use super::robot;
use camera_msg::{GetCameraListMsg, RecvCameraListMsg};
use image_msg::{CaptureImageMsg, RecvImageMsg};
use message::{MessageId, RecvMessage, SendMessage, StopMsg};
use move_msg::MoveMsg;
use robot::Robot;

fn robot_thread(
    addr: Ipv4Addr,
    port: u16,
    rx: std::sync::mpsc::Receiver<Box<dyn SendMessage + Send>>,
    tx: std::sync::mpsc::Sender<Box<dyn RecvMessage + Send>>,
) -> Result<(), Box<dyn Error>> {
    println!("Robot thread started!");
    let mut robot = Robot::new(addr, port)?;
    robot.init()?;
    let mut stop_thread = false;

    while !stop_thread {
        // TODO: add priority queue for messages
        let msg = rx
            .recv()
            .expect("Failed to get message in the Robot thread");
        match MessageId::from(msg.id()) {
            MessageId::Stop => {
                stop_thread = true;
            }
            MessageId::CaptureImage => {
                let capture_image_msg = msg.as_any().downcast_ref::<CaptureImageMsg>().unwrap();
                let frame_data = robot.capture_frame(
                    capture_image_msg.camera_id,
                    (
                        capture_image_msg.frame_width,
                        capture_image_msg.frame_height,
                    ),
                )?;
                let mut send_image_msg = RecvImageMsg::new();
                send_image_msg.channels = 3;
                send_image_msg.frame_width = capture_image_msg.frame_width;
                send_image_msg.frame_height = capture_image_msg.frame_height;
                send_image_msg.data = frame_data;
                tx.send(Box::new(send_image_msg))?;
            }
            MessageId::GetCameraList => {
                let camera_list = robot.get_camera_list()?;
                let mut send_camera_list_msg = RecvCameraListMsg::new();
                send_camera_list_msg.camera_list = camera_list;
                tx.send(Box::new(send_camera_list_msg))?;
            }
            MessageId::Move => {
                let move_msg = msg.as_any().downcast_ref::<MoveMsg>().unwrap();
                robot.move_bot(
                    move_msg.left_speed,
                    move_msg.left_dir,
                    move_msg.right_speed,
                    move_msg.right_dir,
                )?;
            }
            _ => panic!("Message is unsupported by robot thread"),
        }
    }
    println!("Robot thread stopped");
    Ok(())
}

type RobotMessages = HashMap<MessageId, VecDeque<Box<dyn RecvMessage>>>;

pub struct RobotInterface {
    thread_join_handle: Option<thread::JoinHandle<()>>,
    to_robot_tx: std::sync::mpsc::Sender<Box<dyn SendMessage + Send>>,
    from_robot_rx: std::sync::mpsc::Receiver<Box<dyn RecvMessage + Send>>,
    messages: RobotMessages,
    move_speed: u8,
}

impl RobotInterface {
    pub fn new(addr: Ipv4Addr, port: u16) -> RobotInterface {
        let (to_robot_tx, in_robot_rx): (
            std::sync::mpsc::Sender<Box<dyn SendMessage + Send>>,
            std::sync::mpsc::Receiver<Box<dyn SendMessage + Send>>,
        ) = channel();
        let (from_robot_tx, from_robot_rx): (
            std::sync::mpsc::Sender<Box<dyn RecvMessage + Send>>,
            std::sync::mpsc::Receiver<Box<dyn RecvMessage + Send>>,
        ) = channel();

        RobotInterface {
            move_speed: 10,
            messages: RobotMessages::new(),
            to_robot_tx: to_robot_tx,
            from_robot_rx: from_robot_rx,
            thread_join_handle: Some(thread::spawn(move || {
                robot_thread(addr, port, in_robot_rx, from_robot_tx).expect("Robot thread failed");
            })),
        }
    }

    pub fn stop_robot_thread(&mut self) -> Result<(), Box<dyn Error>> {
        let stop_msg = StopMsg {};
        match self.thread_join_handle.take() {
            Some(handle) => {
                self.to_robot_tx
                    .send(Box::new(stop_msg))
                    .expect("Can't send the stop message into robot channel");
                handle.join().expect("Can't stop robot thread");
            }
            None => (),
        }
        Ok(())
    }

    pub fn get_robot_msg(&mut self, timeout: Duration) {
        let recv_result = self.from_robot_rx.recv_timeout(timeout);
        if recv_result.is_ok() {
            let msg = recv_result.unwrap();
            self.messages
                .entry(MessageId::from(msg.id()))
                .or_default()
                .push_back(msg);
        }
    }

    fn ask_camera_list(&mut self) -> Result<(), Box<dyn Error>> {
        let get_msg = GetCameraListMsg::new();
        self.to_robot_tx.send(Box::new(get_msg))?;
        Ok(())
    }

    fn get_camera_list_impl(&mut self) -> Option<Vec<u8>> {
        let key = MessageId::RecvCameraList;
        let values = self.messages.get_mut(&key);
        match values {
            Some(queue) => {
                if !queue.is_empty() {
                    let mut msg = queue.pop_front().unwrap();
                    let cam_list_msg = msg
                        .as_mut_any()
                        .downcast_mut::<RecvCameraListMsg>()
                        .unwrap();
                    let list = std::mem::replace(&mut cam_list_msg.camera_list, Vec::new());
                    let camera_list = list;
                    Some(camera_list)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn get_camera_list(&mut self, timeout: Duration) -> Result<Vec<u8>, Box<dyn Error>> {
        self.ask_camera_list()?;
        loop {
            let camera_list = self.get_camera_list_impl();
            match camera_list {
                Some(list) => return Ok(list),
                None => self.get_robot_msg(timeout),
            }
        }
    }

    pub fn ask_image(
        &mut self,
        camera_id: u8,
        frame_width: u16,
        frame_height: u16,
    ) -> Result<(), Box<dyn Error>> {
        let mut get_msg = CaptureImageMsg::new();
        get_msg.camera_id = camera_id;
        get_msg.frame_width = frame_width;
        get_msg.frame_height = frame_height;
        self.to_robot_tx.send(Box::new(get_msg))?;
        Ok(())
    }

    pub fn get_image(&mut self) -> Option<Vec<u8>> {
        let key = MessageId::RecvImage;
        let values = self.messages.get_mut(&key);
        match values {
            Some(queue) => {
                if !queue.is_empty() {
                    let mut msg = queue.pop_front().unwrap();
                    let image_msg = msg.as_mut_any().downcast_mut::<RecvImageMsg>().unwrap();
                    let image_data = std::mem::replace(&mut image_msg.data, Vec::new());
                    Some(image_data)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn move_bot_impl(
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
        self.to_robot_tx.send(Box::new(move_msg))?;
        Ok(())
    }

    pub fn rotate_left(&mut self) -> Result<(), Box<dyn Error>> {
        self.move_bot_impl(0, 0, self.move_speed, 1)
    }

    pub fn rotate_right(&mut self) -> Result<(), Box<dyn Error>> {
        self.move_bot_impl(self.move_speed, 1, 0, 0)
    }

    pub fn move_forward(&mut self) -> Result<(), Box<dyn Error>> {
        self.move_bot_impl(self.move_speed, 1, self.move_speed, 1)
    }

    pub fn move_backward(&mut self) -> Result<(), Box<dyn Error>> {
        self.move_bot_impl(self.move_speed, 0, self.move_speed, 0)
    }
}

impl Drop for RobotInterface {
    fn drop(&mut self) {
        println!("Robot interface dropped");
        self.stop_robot_thread().unwrap();
    }
}
