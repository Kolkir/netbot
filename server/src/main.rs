#![allow(dead_code)]

#[macro_use]
extern crate slice_as_array;

use std::env;
mod ui;
mod windowui;
use ui::UI;
use windowui::WindowUi;
mod camera_msg;
mod camera_prop_msg;
mod image_msg;
mod message;
mod move_msg;
mod robot;
mod server;

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::error::Error;
use std::net::Ipv4Addr;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use camera_msg::{GetCameraListMsg, RecvCameraListMsg};
use image_msg::{CaptureImageMsg, RecvImageMsg};
use message::{MessageId, RecvMessage, SendMessage};
use move_msg::MoveMsg;
use robot::Robot;

fn ask_image(
    tx: std::sync::mpsc::Sender<Box<dyn SendMessage + Send>>,
    camera_id: u8,
    frame_width: u16,
    frame_height: u16,
) -> Result<(), Box<dyn Error>> {
    let mut get_msg = CaptureImageMsg::new();
    get_msg.camera_id = camera_id;
    get_msg.frame_width = frame_width;
    get_msg.frame_height = frame_height;
    tx.send(Box::new(get_msg))
        .expect("Failed to ask for a new image for camera");
    Ok(())
}

pub fn ask_camera_list(
    tx: std::sync::mpsc::Sender<Box<dyn SendMessage + Send>>,
) -> Result<(), Box<dyn Error>> {
    let get_msg = GetCameraListMsg::new();
    tx.send(Box::new(get_msg))
        .expect("Failed to ask for a camera list");
    Ok(())
}

pub fn move_bot(
    tx: std::sync::mpsc::Sender<Box<dyn SendMessage + Send>>,
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
    tx.send(Box::new(move_msg)).expect("Failed to move a bot");
    Ok(())
}

fn robot_thread(
    addr: Ipv4Addr,
    port: u16,
    rx: std::sync::mpsc::Receiver<Box<dyn SendMessage + Send>>,
    tx: std::sync::mpsc::Sender<Box<dyn RecvMessage + Send>>,
) -> Result<(), Box<dyn Error>> {
    let mut robot = Robot::new(addr, port)?;
    robot.init()?;

    loop {
        let msg = rx
            .recv()
            .expect("Failed to get message in the Robot thread");
        match MessageId::from(msg.id()) {
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
}

type RobotMessages = HashMap<MessageId, VecDeque<Box<dyn RecvMessage>>>;

fn get_robot_msg(
    messages: &mut RobotMessages,
    rx: &std::sync::mpsc::Receiver<Box<dyn RecvMessage + Send>>,
    timeout: Duration,
) {
    let recv_result = rx.recv_timeout(timeout);
    if recv_result.is_ok() {
        let msg = recv_result.unwrap();
        messages
            .entry(MessageId::from(msg.id()))
            .or_default()
            .push_back(msg);
    }
}

fn get_camera_list(messages: &mut RobotMessages) -> Option<Vec<u8>> {
    let key = MessageId::RecvCameraList;
    let values = messages.get_mut(&key);
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

fn get_image(messages: &mut RobotMessages) -> Option<Vec<u8>> {
    let key = MessageId::RecvImage;
    let values = messages.get_mut(&key);
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut addr = Ipv4Addr::new(192, 168, 88, 184);
    let mut port = 2345;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        addr = args[1].parse::<Ipv4Addr>().unwrap();
    }
    if args.len() >= 2 {
        port = args[2].parse::<u16>().unwrap();
    }

    // Start Robot instance
    let robot_messages = RobotMessages::new();

    let (to_robot_tx, in_robot_rx): (
        std::sync::mpsc::Sender<Box<dyn SendMessage + Send>>,
        std::sync::mpsc::Receiver<Box<dyn SendMessage + Send>>,
    ) = channel();

    let (from_robot_tx, in_ui_rx): (
        std::sync::mpsc::Sender<Box<dyn RecvMessage + Send>>,
        std::sync::mpsc::Receiver<Box<dyn RecvMessage + Send>>,
    ) = channel();

    let robot = thread::spawn(move || {
        robot_thread(addr, port, in_robot_rx, from_robot_tx).expect_err("Robot thread failed");
    });
    println!("Robot thread started");

    // Initialize UI
    let mut ui: Box<dyn UI> = Box::new(WindowUi::new());

    {
        let to_robot_tx_clone = to_robot_tx.clone();
        let ask_image_fn = move |camera_id, frame_width, frame_height| {
            ask_image(
                to_robot_tx_clone.clone(),
                camera_id,
                frame_width,
                frame_height,
            )
        };
        ui.set_ask_img_fn(Box::new(ask_image_fn));
    }

    {
        let to_robot_tx_clone = to_robot_tx.clone();
        let ask_camera_list_fn = move || ask_camera_list(to_robot_tx_clone.clone());
        ui.set_ask_camera_list_fn(Box::new(ask_camera_list_fn));
    }

    {
        let to_robot_tx_clone = to_robot_tx.clone();
        let move_bot_fn = move |left_speed, left_dir, right_speed, right_dir| {
            move_bot(
                to_robot_tx_clone.clone(),
                left_speed,
                left_dir,
                right_speed,
                right_dir,
            )
        };
        ui.set_move_fn(Box::new(move_bot_fn));
    }

    let ui_rx = Rc::new(in_ui_rx);
    let messages = Rc::new(RefCell::new(robot_messages));
    {
        let in_ui_rx_clone = ui_rx.clone();
        let messages_clone = messages.clone();
        let get_camera_list_fn = move || {
            get_robot_msg(
                &mut messages_clone.borrow_mut(),
                in_ui_rx_clone.as_ref(),
                Duration::from_micros(10),
            );
            get_camera_list(&mut messages_clone.borrow_mut())
        };
        ui.set_get_camera_list_fn(Box::new(get_camera_list_fn));
    }

    {
        let in_ui_rx_clone = ui_rx.clone();
        let messages_clone = messages.clone();
        let get_image_fn = move || {
            get_robot_msg(
                &mut messages_clone.borrow_mut(),
                in_ui_rx_clone.as_ref(),
                Duration::from_micros(10),
            );
            get_image(&mut messages_clone.borrow_mut())
        };
        ui.set_get_img_fn(Box::new(get_image_fn));
    }
    println!("Starting UI ...");
    ui.run()?;

    robot.join().unwrap();
    println!("UI was stopped");
    Ok(())
}
