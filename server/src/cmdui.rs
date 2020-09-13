use super::ui;
use std::error::Error;
use std::io::{stdin, stdout, BufRead, Write};
use ui::{CaptureImageFn, GetCameraListFn, UI};

extern crate opencv;
use opencv::{highgui, prelude::*};

pub struct CmdUi {
    get_camera_list: Option<GetCameraListFn>,
    capture_image: Option<CaptureImageFn>,
    camera_id: u8,
    frame_width: u16,
    frame_height: u16,
}

impl UI for CmdUi {
    fn set_get_camera_list_fn(&mut self, get_camera_list_fn: GetCameraListFn) {
        self.get_camera_list = Some(get_camera_list_fn);
    }
    fn set_capture_img_fn(&mut self, capture_image_fn: CaptureImageFn) {
        self.capture_image = Some(capture_image_fn);
    }
    fn run(&mut self) -> Result<(), Box<dyn Error>> {
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
                                1 => {
                                    let camera_list = self.get_camera_list.as_ref().unwrap()()?;
                                    println!("Camera list {:?}", camera_list);
                                }
                                2 => {
                                    let img_data = self.capture_image.as_ref().unwrap()(
                                        self.camera_id,
                                        self.frame_width,
                                        self.frame_height,
                                    )?;
                                    self.show_image(&img_data)?
                                }
                                3 => {
                                    println!("Exiting!");
                                    return Ok(());
                                }
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
}

fn print_commands() -> Result<(), std::io::Error> {
    println!(
        "Press [1] for getting a camera list\nPress [2] for the image capture\nPress [3] to exit\n"
    );
    stdout().flush()?;
    Ok(())
}

impl CmdUi {
    pub fn new() -> CmdUi {
        CmdUi {
            get_camera_list: None,
            capture_image: None,
            camera_id: 0,
            frame_width: 640,
            frame_height: 480,
        }
    }
    fn show_image(&self, data: &[u8]) -> opencv::Result<(), Box<dyn Error>> {
        let mut frame = Mat::from_slice(data)?;
        frame = frame.reshape(3, self.frame_height as i32)?;

        let window = "Captured image";
        highgui::named_window(window, 1)?;
        if frame.size()?.width > 0 {
            highgui::imshow(window, &mut frame)?;
            highgui::wait_key(0)?;
            highgui::destroy_all_windows()?;
        }
        Ok(())
    }
}
