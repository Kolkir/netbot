#![allow(dead_code)]

#[macro_use]
extern crate slice_as_array;

// mod cmdui;
mod ui;
mod windowui;
// use cmdui::CmdUi;
use ui::UI;
use windowui::WindowUi;
mod camera_msg;
mod image_msg;
mod message;
mod move_msg;
mod server;
use server::Server;

use std::error::Error;
use std::net::Ipv4Addr;

use camera_msg::{GetCameraListMsg, RecvCameraListMsg};
use image_msg::{CaptureImageMsg, RecvImageMsg};
use message::{HelloMsg, Message, MessageId};
use move_msg::MoveMsg;

fn capture_image<'a>(
    server: &mut Server,
    camera_id: u8,
    frame_width: u16,
    frame_height: u16,
) -> opencv::Result<Vec<u8>, Box<dyn Error>> {
    let mut get_msg = CaptureImageMsg::new();
    get_msg.camera_id = camera_id;
    get_msg.frame_width = frame_width;
    get_msg.frame_height = frame_height;
    server.send(Box::new(get_msg))?;

    let mut recv_msg = RecvImageMsg::new();
    server.recv(&mut recv_msg)?;

    println!(
        "Recv image : {0} x {1} x {2}",
        recv_msg.channels, recv_msg.frame_width, recv_msg.frame_height
    );

    Ok(recv_msg.data)
}

pub fn get_camera_list(server: &mut Server) -> Result<Vec<u8>, Box<dyn Error>> {
    let get_msg = GetCameraListMsg::new();
    server.send(Box::new(get_msg))?;
    let mut recv_msg = RecvCameraListMsg::new();
    server.recv(&mut recv_msg)?;
    Ok(recv_msg.camera_list)
}

pub fn move_bot(
    server: &mut Server,
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
    server.send(Box::new(move_msg))?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let addr = Ipv4Addr::new(127, 0, 0, 1);
    let port = 8080;
    let mut server = Server::new(addr, port)?;
    server.wait_client()?;
    // handshake
    let mut hello_msg = HelloMsg {};
    server.recv(&mut hello_msg)?;
    if hello_msg.id() == MessageId::Hello as u8 {
        println!("Handshaking started");
        server.send(Box::new(HelloMsg {}))?;
    } else {
        panic!("Handshake failed!");
    }
    println!("Handshake completed");
    // let mut ui: Box<dyn UI> = Box::new(CmdUi::new());
    let mut ui: Box<dyn UI> = Box::new(WindowUi::new());

    let srv1 = server.clone();
    let capture_img = move |camera_id, frame_width, frame_height| {
        let mut srv = srv1.clone();
        capture_image(&mut srv, camera_id, frame_width, frame_height)
    };
    ui.set_capture_img_fn(Box::new(capture_img));

    let srv2 = server.clone();
    let get_cam_list = move || {
        let mut srv = srv2.clone();
        get_camera_list(&mut srv)
    };
    ui.set_get_camera_list_fn(Box::new(get_cam_list));

    let srv3 = server.clone();
    let move_bot_fn = move |left_speed, left_dir, right_speed, right_dir| {
        let mut srv = srv3.clone();
        move_bot(&mut srv, left_speed, left_dir, right_speed, right_dir)
    };
    ui.set_move_fn(Box::new(move_bot_fn));
    ui.run()
}
