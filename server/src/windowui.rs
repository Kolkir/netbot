extern crate gtk;

use gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

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
    ui_frame_width: i32,
    ui_frame_height: i32,
    pub camera_views: HashMap<u8, gtk::Image>,
    pub camera_res_combos: HashMap<u8, gtk::ComboBox>,
    pub camera_encoding_checks: HashMap<u8, gtk::CheckButton>,
    pub forward_button: gtk::Button,
    pub backward_button: gtk::Button,
    pub right_button: gtk::Button,
    pub left_button: gtk::Button,
    container: gtk::Grid,
    pub window: gtk::ApplicationWindow,
}

impl<'a> WindowUi {
    pub fn new(
        application: &gtk::Application,
        camera_list: &Vec<u8>,
        camera_resolutions: &HashMap<u8, Vec<(i32, i32)>>,
        frame_width: i32,
        frame_height: i32,
    ) -> WindowUi {
        let camera_num = camera_list.len();
        let buttons_num = 4;
        let max_cols_num = std::cmp::max(camera_num, buttons_num);
        let cols_per_camera_view = max_cols_num / camera_num;

        let mut camera_views: HashMap<u8, gtk::Image> = HashMap::new();
        let mut camera_res_combos: HashMap<u8, gtk::ComboBox> = HashMap::new();
        let mut camera_encoding_checks: HashMap<u8, gtk::CheckButton> = HashMap::new();

        let col_types: [glib::Type; 1] = [glib::Type::String];

        for cam_id in camera_list {
            let camera_view = gtk::Image::new();
            camera_view.set_halign(gtk::Align::Center);
            camera_view.set_vexpand(false);
            camera_view.set_hexpand(false);
            camera_view.set_valign(gtk::Align::Center);
            camera_view.set_size_request(frame_width, frame_height);
            camera_views.insert(*cam_id, camera_view);

            let view_model = gtk::ListStore::new(&col_types);
            for resolution in camera_resolutions.get(cam_id).unwrap() {
                let label = format!("{0} x {1}", resolution.0, resolution.1);
                view_model.insert_with_values(None, &[0], &[&label]);
            }
            let camera_res_combo = gtk::ComboBox::new();
            camera_res_combo.set_model(Some(&view_model));

            let cell = gtk::CellRendererText::new();
            camera_res_combo.pack_start(&cell, false);
            camera_res_combo.add_attribute(&cell, "text", 0);

            camera_res_combos.insert(*cam_id, camera_res_combo);

            let encoding_check = gtk::CheckButton::new();
            encoding_check.set_label("encoding");
            encoding_check.set_halign(gtk::Align::Center);
            camera_encoding_checks.insert(*cam_id, encoding_check);
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
            let key = i as u8;
            container.attach(
                camera_views.get(&key).unwrap(),
                (i * cols_per_camera_view) as i32,
                0,
                cols_per_camera_view as i32,
                1,
            );
            container.attach(
                camera_res_combos.get(&key).unwrap(),
                (i * cols_per_camera_view) as i32,
                1,
                cols_per_camera_view as i32,
                1,
            );
            container.attach(
                camera_encoding_checks.get(&key).unwrap(),
                (i * cols_per_camera_view) as i32,
                2,
                cols_per_camera_view as i32,
                1,
            );
        }

        container.attach(&forward_button, 0, 3, 1, 1);
        container.attach(&backward_button, 1, 3, 1, 1);
        container.attach(&right_button, 2, 3, 1, 1);
        container.attach(&left_button, 3, 3, 1, 1);

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
            ui_frame_width: frame_width,
            ui_frame_height: frame_height,
            camera_views: camera_views,
            camera_res_combos: camera_res_combos,
            camera_encoding_checks: camera_encoding_checks,
            forward_button: forward_button,
            backward_button: backward_button,
            right_button: right_button,
            left_button: left_button,
            container: container,
            window: window,
        }
    }

    pub fn update_image(&mut self, camera_id: u8, image_data: &mut Vec<u8>) {
        let view = (&self.camera_views).get(&camera_id).unwrap();
        let pixbuf = Pixbuf::from_mut_slice(
            image_data,
            gdk_pixbuf::Colorspace::Rgb,
            /*has alpha*/ false,
            /*bits_per_sample*/ 8,
            self.ui_frame_width,
            self.ui_frame_height,
            /*row stride*/ self.ui_frame_width * 3,
        );
        view.set_from_pixbuf(Some(&pixbuf));
    }
}

impl Drop for WindowUi {
    fn drop(&mut self) {
        println!("WindowUI dropped");
    }
}
