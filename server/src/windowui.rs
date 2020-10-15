extern crate gtk;

use gtk::prelude::*;
use std::error::Error;
use std::fmt;
use std::net::Ipv4Addr;
use std::time::Duration;

use super::robot_interface;
use robot_interface::RobotInterface;

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

pub struct WindowUi {
    pub robot: RobotInterface,
    ui_frame_width: i32,
    ui_frame_height: i32,
    camera_views: Vec<gtk::Label>,
    pub forward_button: gtk::Button,
    pub backward_button: gtk::Button,
    pub right_button: gtk::Button,
    pub left_button: gtk::Button,
    container: gtk::Grid,
    window: gtk::ApplicationWindow,
}

impl WindowUi {
    pub fn new(application: &gtk::Application, addr: Ipv4Addr, port: u16) -> WindowUi {
        let mut robot = RobotInterface::new(addr, port);
        let camera_list = robot
            .get_camera_list(Duration::from_millis(100))
            .expect("Failed to get list of bot cameras");

        let camera_num = camera_list.len();
        let buttons_num = 4;
        let max_cols_num = std::cmp::max(camera_num, buttons_num);
        let cols_per_camera_view = max_cols_num / camera_num;

        let mut camera_views: Vec<gtk::Label> = Vec::new();
        for cam_id in camera_list {
            let camera_view = gtk::Label::new(None);
            camera_view.set_markup("Cam"); //format!("Camera {}", cam_id.to_string()));
            camera_view.set_halign(gtk::Align::Center);
            camera_view.set_valign(gtk::Align::Center);
            camera_view.set_vexpand(true);
            camera_view.set_hexpand(true);
            camera_views.push(camera_view);
        }

        let forward_button = gtk::Button::new();
        forward_button.set_label("forward");
        forward_button.set_halign(gtk::Align::Center);

        let backward_button = gtk::Button::new();
        backward_button.set_label("backward");
        backward_button.set_halign(gtk::Align::Center);
        let right_button = gtk::Button::new();
        right_button.set_label("right");
        right_button.set_halign(gtk::Align::Center);

        let left_button = gtk::Button::new();
        left_button.set_label("left");
        left_button.set_halign(gtk::Align::Center);

        let container = gtk::Grid::new();
        container.set_vexpand(true);
        container.set_hexpand(true);

        for i in 0..camera_num {
            container.attach(
                camera_views.get(i).unwrap(),
                (i * cols_per_camera_view) as i32,
                0,
                cols_per_camera_view as i32,
                1,
            );
        }

        container.attach(&forward_button, 0, 1, 1, 1);
        container.attach(&backward_button, 1, 1, 1, 1);
        container.attach(&right_button, 2, 1, 1, 1);
        container.attach(&left_button, 3, 1, 1, 1);

        let window = gtk::ApplicationWindow::new(application);
        window.set_icon_name(Some("package-x-generic"));
        window.set_property_window_position(gtk::WindowPosition::Center);
        window.add(&container);
        window.show_all();
        window.connect_delete_event(move |window, _| {
            window.close();
            Inhibit(false)
        });
        WindowUi {
            robot: robot,
            ui_frame_width: 640,
            ui_frame_height: 480,
            camera_views: camera_views,
            forward_button: forward_button,
            backward_button: backward_button,
            right_button: right_button,
            left_button: left_button,
            container: container,
            window: window,
        }
    }
}

impl Drop for WindowUi {
    fn drop(&mut self) {
        println!("WindowUI dropped");
    }
}
