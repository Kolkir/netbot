use std::cmp::Ordering;
use std::collections::binary_heap::BinaryHeap;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::error::Error;
use std::net::Ipv4Addr;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use super::camera_msg;
use super::camera_prop_msg;
use super::image_msg;
use super::message;
use super::move_msg;
use super::robot;
use camera_msg::{GetCameraListMsg, RecvCameraListMsg};
use camera_prop_msg::SetCameraPropMsg;
use image_msg::{CaptureImageMsg, RecvImageMsg};
use message::{MessageId, RecvMessage, SendMessage, StopMsg};
use move_msg::MoveMsg;
use robot::Robot;

impl Ord for Box<dyn SendMessage + Send> {
    fn cmp(&self, other: &Box<dyn SendMessage + Send>) -> Ordering {
        match MessageId::from(other.id()) {
            MessageId::CaptureImage => Ordering::Less,
            _ => Ordering::Equal,
        }
    }
}

impl PartialOrd for Box<dyn SendMessage + Send> {
    fn partial_cmp(&self, other: &Box<dyn SendMessage + Send>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Box<dyn SendMessage + Send> {
    fn eq(&self, other: &Box<dyn SendMessage + Send>) -> bool {
        match MessageId::from(other.id()) {
            MessageId::CaptureImage => false,
            _ => true,
        }
    }
}

impl Eq for Box<dyn SendMessage + Send> {}

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
    let mut message_queue: BinaryHeap<Box<dyn SendMessage + Send>> = BinaryHeap::new();
    let mut frame_width = 640;
    let mut frame_height = 480;

    while !stop_thread {
        let input_msg = rx
            .recv()
            .expect("Failed to get message in the Robot thread");
        message_queue.push(input_msg);
        let out_msg = message_queue.pop();
        if out_msg.is_none() {
            continue;
        }
        let msg = out_msg.unwrap();

        match MessageId::from(msg.id()) {
            MessageId::Stop => {
                robot.stop()?;
                stop_thread = true;
            }
            MessageId::SetCameraProp => {
                let camera_prop_msg = msg.as_any().downcast_ref::<SetCameraPropMsg>().unwrap();
                frame_width = camera_prop_msg.frame_width;
                frame_height = camera_prop_msg.frame_height;
            }
            MessageId::CaptureImage => {
                let capture_image_msg = msg.as_any().downcast_ref::<CaptureImageMsg>().unwrap();
                let frame_data = robot
                    .capture_frame(capture_image_msg.camera_id, (frame_width, frame_height))?;
                let mut send_image_msg = RecvImageMsg::new();
                send_image_msg.channels = 3;
                send_image_msg.frame_width = frame_width;
                send_image_msg.frame_height = frame_height;
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
    in_messages_queue: RobotMessages,
    image_request_status: HashMap<u8, i32>,
    move_speed: u8,
    max_image_request_num: i32,
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
            max_image_request_num: 2,
            in_messages_queue: RobotMessages::new(),
            image_request_status: HashMap::new(),
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
                self.send_message_to_robot(Box::new(stop_msg));
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
            self.in_messages_queue
                .entry(MessageId::from(msg.id()))
                .or_default()
                .push_back(msg);
        }
    }

    fn send_message_to_robot(&mut self, msg: Box<dyn SendMessage + Send>) {
        self.to_robot_tx
            .send(msg)
            .expect("Failed send a message to robot thread");
    }

    fn ask_camera_list(&mut self) {
        let get_msg = GetCameraListMsg::new();
        self.send_message_to_robot(Box::new(get_msg));
    }

    fn get_camera_list_impl(&mut self) -> Option<Vec<u8>> {
        let key = MessageId::RecvCameraList;
        let values = self.in_messages_queue.get_mut(&key);
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
        self.ask_camera_list();
        loop {
            let camera_list = self.get_camera_list_impl();
            match camera_list {
                Some(list) => return Ok(list),
                None => self.get_robot_msg(timeout),
            }
        }
    }

    pub fn set_robot_ui_camera_resolution(&mut self, frame_width: i32, frame_height: i32) {
        let mut set_msg = SetCameraPropMsg::new();
        set_msg.camera_id = 0;
        set_msg.frame_width = frame_width as u16;
        set_msg.frame_height = frame_height as u16;
        self.send_message_to_robot(Box::new(set_msg));
    }

    pub fn ask_image(&mut self, camera_id: u8) {
        let request = self.image_request_status.get_mut(&camera_id);
        match request {
            Some(value) => {
                if *value > self.max_image_request_num {
                    return ();
                } else {
                    *value += 1;
                }
            }
            None => {
                let _res = self.image_request_status.insert(camera_id, 1);
            }
        };
        let mut get_msg = CaptureImageMsg::new();
        get_msg.camera_id = camera_id;
        self.send_message_to_robot(Box::new(get_msg));
    }

    pub fn get_image(&mut self) -> Option<(u8, Vec<u8>)> {
        let key = MessageId::RecvImage;
        let values = self.in_messages_queue.get_mut(&key);
        match values {
            Some(queue) => {
                if !queue.is_empty() {
                    let mut msg = queue.pop_front().unwrap();
                    let image_msg = msg.as_mut_any().downcast_mut::<RecvImageMsg>().unwrap();
                    let image_data = std::mem::replace(&mut image_msg.data, Vec::new());

                    self.image_request_status
                        .entry(image_msg.camera_id)
                        .and_modify(|entry| *entry -= 1);

                    Some((image_msg.camera_id, image_data))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn move_bot_impl(&mut self, left_speed: u8, left_dir: u8, right_speed: u8, right_dir: u8) {
        let mut move_msg = MoveMsg::new();
        move_msg.left_speed = left_speed;
        move_msg.left_dir = left_dir;
        move_msg.right_speed = right_speed;
        move_msg.right_dir = right_dir;
        self.send_message_to_robot(Box::new(move_msg));
    }

    pub fn rotate_left(&mut self) {
        self.move_bot_impl(0, 0, self.move_speed, 1);
    }

    pub fn rotate_right(&mut self) {
        self.move_bot_impl(self.move_speed, 1, 0, 0);
    }

    pub fn move_forward(&mut self) {
        self.move_bot_impl(self.move_speed, 1, self.move_speed, 1);
    }

    pub fn move_backward(&mut self) {
        self.move_bot_impl(self.move_speed, 0, self.move_speed, 0);
    }
}

impl Drop for RobotInterface {
    fn drop(&mut self) {
        println!("Robot interface dropped");
        self.stop_robot_thread().unwrap();
    }
}
