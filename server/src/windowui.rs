extern crate fltk;
use fltk::{app::*, button::*, frame::*, group::*, image::*, window::*};

use super::ui;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use ui::UI as LocalUI;
use ui::{CaptureImageFn, GetCameraListFn, GetCameraPropFn, MoveFn};

extern crate opencv;
use opencv::{core, imgproc, prelude::*};

#[derive(Debug)]
enum UiErrors {
    MissedCameraList,
    ImageFrameMissed,
}
impl fmt::Display for UiErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: {:?}", self)
    }
}
impl Error for UiErrors {}

#[derive(Clone)]
pub struct WindowUi {
    get_camera_list: Rc<Option<GetCameraListFn>>,
    capture_image: Rc<Option<CaptureImageFn>>,
    move_fn: Rc<Option<MoveFn>>,
    get_camera_prop: Rc<Option<GetCameraPropFn>>,
    camera_list: Vec<u8>,
    image_data: HashMap<u8, Vec<u8>>,
    image_frames: HashMap<u8, Frame>,
    image_res: HashMap<u8, (u16, u16)>,
    ui_frame_width: i32,
    ui_frame_height: i32,
    move_speed: u8,
}

impl LocalUI for WindowUi {
    fn set_get_camera_list_fn(&mut self, get_camera_list_fn: GetCameraListFn) {
        self.get_camera_list = Rc::new(Some(get_camera_list_fn));
    }
    fn set_capture_img_fn(&mut self, capture_image_fn: CaptureImageFn) {
        self.capture_image = Rc::new(Some(capture_image_fn));
    }
    fn set_move_fn(&mut self, move_fn: MoveFn) {
        self.move_fn = Rc::new(Some(move_fn));
    }
    fn set_get_camera_prop_fn(&mut self, get_camera_prop_fn: GetCameraPropFn) {
        self.get_camera_prop = Rc::new(Some(get_camera_prop_fn));
    }
    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.camera_list = self.get_camera_list.as_ref().as_ref().unwrap()()?;
        if !self.camera_list.is_empty() {
            let mut win_width: i32 = 0;
            let max_frame_height = self.ui_frame_height;
            for cam_id in &self.camera_list {
                let prop = self.get_camera_prop.as_ref().as_ref().unwrap()(*cam_id)?;
                println!("{:?}", prop);
                let (width, height): (Vec<_>, Vec<_>) = prop
                    .into_iter()
                    .enumerate()
                    .partition(|&(i, _)| (i % 2) == 0);
                let max_w = width.iter().max_by(|a, b| a.1.cmp(&b.1)).unwrap();
                let height_index = max_w.0 / 2;
                self.image_res
                    .insert(*cam_id, (max_w.1, height[height_index].1));
                win_width += self.ui_frame_width;
            }
            let win_height = max_frame_height + 40;
            let app = App::default().with_scheme(Scheme::Gtk);
            let mut wind = Window::new(100, 100, win_width, win_height + 40, "NetBot server");
            let global_pack = Pack::default()
                .with_size(win_width, win_height)
                .center_of(&wind);
            let mut cameras_pack = Pack::default()
                .with_size(win_width, max_frame_height as i32)
                .center_of(&wind);
            for cam_id in &self.camera_list {
                let frame = Frame::default().with_size(self.ui_frame_width, self.ui_frame_height);
                self.image_frames.insert(*cam_id, frame);
            }
            cameras_pack.set_type(PackType::Horizontal);
            cameras_pack.end();

            let mut btns_pack = Pack::default().with_size(win_width, 40).center_of(&wind);
            let btn_width = 100;
            let mut rot_left_but = Button::default()
                .with_size(btn_width, 40)
                .with_label("Rotate left");
            {
                let mut self_clone = self.clone();
                rot_left_but.set_callback(Box::new(move || self_clone.rotate_left()));
            }
            let mut mv_fwd_but = Button::default()
                .with_size(btn_width, 40)
                .with_label("Forward");
            {
                let mut self_clone = self.clone();
                mv_fwd_but.set_callback(Box::new(move || self_clone.move_forward()));
            }
            let mut mv_bck_but = Button::default()
                .with_size(btn_width, 40)
                .with_label("Backward");
            {
                let mut self_clone = self.clone();
                mv_bck_but.set_callback(Box::new(move || self_clone.move_backward()));
            }
            let mut rot_right_but = Button::default()
                .with_size(btn_width, 40)
                .with_label("Rotate right");
            {
                let mut self_clone = self.clone();
                rot_right_but.set_callback(Box::new(move || self_clone.rotate_right()));
            }
            btns_pack.set_type(PackType::Horizontal);
            btns_pack.end();
            btns_pack.set_align(fltk::Align::Center);

            global_pack.end();
            // wind.make_resizable(true);
            wind.end();
            wind.show();
            self.update_images_callback();
            Ok(app.run()?)
        } else {
            Err(Box::new(UiErrors::MissedCameraList))
        }
    }
}

impl WindowUi {
    pub fn new() -> WindowUi {
        WindowUi {
            get_camera_list: Rc::new(None),
            capture_image: Rc::new(None),
            move_fn: Rc::new(None),
            get_camera_prop: Rc::new(None),
            camera_list: Vec::new(),
            image_data: HashMap::new(),
            image_frames: HashMap::new(),
            image_res: HashMap::new(),
            ui_frame_width: 640,
            ui_frame_height: 480,
            move_speed: 10,
        }
    }

    pub fn rotate_left(&mut self) {
        let res = self.move_fn.as_ref().as_ref().unwrap()(0, 0, self.move_speed, 1);
        match res {
            Err(err) => println!("Failed rotate right {:?}", err),
            Ok(_) => (),
        }
    }

    pub fn rotate_right(&mut self) {
        let res = self.move_fn.as_ref().as_ref().unwrap()(self.move_speed, 1, 0, 0);
        match res {
            Err(err) => println!("Failed rotate left {:?}", err),
            Ok(_) => (),
        }
    }

    pub fn move_forward(&mut self) {
        let res = self.move_fn.as_ref().as_ref().unwrap()(self.move_speed, 1, self.move_speed, 1);
        match res {
            Err(err) => println!("Failed move forward {:?}", err),
            Ok(_) => (),
        }
    }

    pub fn move_backward(&mut self) {
        let res = self.move_fn.as_ref().as_ref().unwrap()(self.move_speed, 0, self.move_speed, 0);
        match res {
            Err(err) => println!("Failed move backward {:?}", err),
            Ok(_) => (),
        }
    }
    pub fn update_images_callback(&mut self) {
        let mut self_clone = self.clone();
        match self.update_images() {
            Ok(()) => add_timeout(0.1, Box::new(move || self_clone.update_images_callback())),
            Err(err) => println!("Image update error {:?}", err),
        }
    }

    pub fn update_images(&mut self) -> Result<(), Box<dyn Error>> {
        for cam_id in &self.camera_list {
            let (width, height) = self.image_res.entry(*cam_id).or_default();
            let image_data =
                self.capture_image.as_ref().as_ref().unwrap()(*cam_id, *width, *height)?;
            let img_mat = Mat::from_slice(&image_data).unwrap();
            let mut img_mat_reduced = Mat::default()?;
            imgproc::resize(
                &img_mat,
                &mut img_mat_reduced,
                core::Size {
                    width: self.ui_frame_width,
                    height: self.ui_frame_height,
                },
                0.0,
                0.0,
                imgproc::INTER_LINEAR,
            )?;
            self.image_data
                .entry(*cam_id)
                .and_modify(|entry| {
                    entry.clone_from_slice(img_mat_reduced.data_typed::<u8>().unwrap())
                })
                .or_insert(image_data);
            let img = RgbImage::new(
                &self.image_data.get(cam_id).unwrap(),
                *width as u32,
                *height as u32,
                3, /*channels*/
            )?;

            match self.image_frames.get_mut(cam_id) {
                Some(frame) => {
                    frame.set_image(Some(img));
                    frame.redraw();
                }
                None => return Err(Box::new(UiErrors::ImageFrameMissed)),
            }
        }

        Ok(())
    }
}
