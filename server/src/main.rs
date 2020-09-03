#![allow(dead_code)]

#[macro_use]
extern crate slice_as_array;

mod message;
use message::{MessageId, RecvMessage, SendMessage};

mod camera_msg;
use camera_msg::{GetCameraListMsg, RecvCameraListMsg};

mod image_msg;
use image_msg::{CaptureImageMsg, RecvImageMsg};

use std::error::Error;
use std::io::{stdin, stdout, BufRead, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};

extern crate opencv;
use opencv::{highgui, prelude::*};

fn capture_image(tcp_stream: &mut TcpStream) -> opencv::Result<(), Box<dyn Error>> {
    let mut get_msg = CaptureImageMsg::new();
    get_msg.camera_id = 0;
    get_msg.frame_width = 640;
    get_msg.frame_height = 480;
    tcp_stream.write(get_msg.to_bytes().unwrap())?;
    let mut recv_msg = RecvImageMsg::new();
    let mut id: [u8; 1] = [0; 1];
    tcp_stream.read(&mut id)?;
    assert_eq!(id[0], MessageId::RecvImage as u8);
    let mut size: [u8; 4] = [0; 4];
    tcp_stream.read(&mut size)?;
    let msg_size = u32::from_be_bytes(size);
    println!("RecvImageMsg size = {}", msg_size);
    let mut data: Vec<u8> = Vec::new();
    data.resize(msg_size as usize, 0);
    tcp_stream.read_exact(&mut data)?;
    recv_msg.from_bytes(&data);

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

fn get_camera_list(tcp_stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let mut get_msg = GetCameraListMsg::new();
    tcp_stream.write(get_msg.to_bytes().unwrap())?;
    let mut recv_msg = RecvCameraListMsg::new();
    let mut id_size: [u8; 2] = [0; 2];
    tcp_stream.read(&mut id_size)?;
    assert_eq!(id_size[0], MessageId::RecvCameraList as u8);
    let mut data: Vec<u8> = Vec::new();
    data.resize(id_size[1] as usize, 0);
    tcp_stream.read(&mut data)?;
    recv_msg.from_bytes(&data);
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
    let loopback = Ipv4Addr::new(127, 0, 0, 1);
    let socket = SocketAddrV4::new(loopback, 8080);
    let listener = TcpListener::bind(socket)?;
    let port = listener.local_addr()?;
    println!("Listening on {}, access this port to end the program", port);
    let (mut tcp_stream, addr) = listener.accept()?; //block  until requested
    println!("Connection received! {:?}", addr);
    // handshake
    let mut data = [0; 1];
    let _ = tcp_stream.read(&mut data);
    if data[0] == 1 {
        tcp_stream.write(&data)?;
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
                                1 => get_camera_list(&mut tcp_stream)?,
                                2 => capture_image(&mut tcp_stream)?,
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
    } else {
        println!("Handshake failed");
    }
    Ok(())
}
