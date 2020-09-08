#![allow(dead_code)]

#[macro_use]
extern crate slice_as_array;

mod camera_msg;
mod image_msg;
mod message;
mod server;
use server::Server;

use std::error::Error;
use std::io::{stdin, stdout, BufRead, Write};
use std::net::Ipv4Addr;

use camera_msg::{GetCameraListMsg, RecvCameraListMsg};
use image_msg::{CaptureImageMsg, RecvImageMsg};
use message::{HelloMsg, Message, MessageId};

extern crate opencv;
use opencv::{highgui, prelude::*};

fn capture_image(server: &mut Server) -> opencv::Result<(), Box<dyn Error>> {
    let mut get_msg = CaptureImageMsg::new();
    get_msg.camera_id = 0;
    get_msg.frame_width = 640;
    get_msg.frame_height = 480;
    server.send(Box::new(get_msg))?;

    let mut recv_msg = RecvImageMsg::new();
    server.recv(&mut recv_msg)?;

    println!(
        "Recv image : {0} x {1} x {2}",
        recv_msg.channels, recv_msg.frame_width, recv_msg.frame_height
    );

    let mut frame = Mat::from_exact_iter(recv_msg.data.into_iter())?;
    frame = frame.reshape(3, recv_msg.frame_height as i32)?;

    let window = "Captured image";
    highgui::named_window(window, 1)?;
    if frame.size()?.width > 0 {
        highgui::imshow(window, &mut frame)?;
        highgui::wait_key(0)?;
        highgui::destroy_all_windows()?;
    }
    Ok(())
}

pub fn get_camera_list(server: &mut Server) -> Result<(), Box<dyn Error>> {
    let get_msg = GetCameraListMsg::new();
    server.send(Box::new(get_msg))?;
    let mut recv_msg = RecvCameraListMsg::new();
    server.recv(&mut recv_msg)?;
    println!("Camera list {:?}", recv_msg);
    Ok(())
}

fn print_commands() -> Result<(), std::io::Error> {
    println!(
        "Press [1] for getting a camera list\nPress [2] for the image capture\nPress [3] to exit\n"
    );
    stdout().flush()?;
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
        server.send(Box::new(HelloMsg {}))?;
    } else {
        panic!("Handshake failed!");
    }
    println!("Handshake completed");
    print_commands()?;
    let stdin = stdin();
    for line in stdin.lock().lines() {
        print_commands()?;
        match line {
            Ok(text) => {
                let trimmed = text.trim();
                match trimmed.parse::<u32>() {
                    Ok(i) => {
                        println!("your input: {}", i);
                        match i {
                            1 => get_camera_list(&mut server)?,
                            2 => capture_image(&mut server)?,
                            3 => return Ok(()),
                            _ => println!("Incorrect command"),
                        }
                    }
                    Err(..) => println!("this was not an integer"),
                };
            }
            Err(err) => println!("Failed to read an input {:?}", err),
        };
    }
    Ok(())
}
