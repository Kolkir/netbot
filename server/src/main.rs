#![allow(dead_code)]

#[macro_use]
extern crate slice_as_array;
extern crate cairo;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gio;
extern crate glib;
extern crate gtk;

use gio::prelude::*;
use gtk::Application;
use std::env;
mod windowui;
use windowui::WindowUi;
mod camera_msg;
mod camera_prop_msg;
mod image_msg;
mod message;
mod move_msg;
mod robot;
mod server;

use robot::Robot;
use std::cell::RefCell;
use std::error::Error;
use std::net::Ipv4Addr;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize UI
    let mut addr = Ipv4Addr::new(192, 168, 88, 184);
    let mut port = 2345;
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        addr = args[1].parse::<Ipv4Addr>().unwrap();
    }
    if args.len() >= 3 {
        port = args[2].parse::<u16>().unwrap();
    }

    let robot = Rc::new(RefCell::new(Robot::new()?));
    robot.borrow_mut().init(addr, port)?;
    {
        println!("Getting camera list...");
        robot.borrow_mut().ask_camera_list()?;
        let mut camera_list: Option<Vec<u8>>;
        loop {
            camera_list = robot.borrow_mut().get_camera_list();
            if camera_list.is_none() {
                thread::sleep(Duration::from_millis(100));
            } else {
                break;
            }
        }

        println!("Getting cameras resolutions...");
        for camera_id in camera_list.as_ref().unwrap() {
            robot.borrow_mut().ask_camera_prop(*camera_id)?;
        }

        let mut cam_num: usize = 0;
        loop {
            for camera_id in camera_list.as_ref().unwrap() {
                if robot
                    .borrow_mut()
                    .get_camera_resolutions(*camera_id)
                    .is_none()
                {
                    thread::sleep(Duration::from_millis(100));
                } else {
                    cam_num += 1;
                }
            }
            if cam_num == camera_list.as_ref().unwrap().len() {
                break;
            }
        }
    }
    println!("UI initialization...");

    let application = Application::new(Some("com.github.kolkir.netbot"), Default::default())
        .expect("failed to initialize GTK application");

    let robot_ui = Rc::clone(&robot);
    application.connect_startup(move |app| {
        let camera_list = robot_ui.borrow_mut().get_camera_list();
        let cameras_resolutions = robot_ui.borrow_mut().get_cameras_resolutions();
        let window_ui = WindowUi::new(
            app,
            camera_list.as_ref().unwrap(),
            &cameras_resolutions,
            640,
            480,
        );
        let ui_container = Rc::new(RefCell::new(Some(window_ui)));

        {
            let robot_ref = Rc::clone(&robot_ui);
            let ui_container_ref = Rc::clone(&ui_container);
            app.connect_shutdown(move |_| {
                robot_ref
                    .borrow_mut()
                    .stop()
                    .expect("Failed to stop the bot");
                let ui = ui_container_ref
                    .borrow_mut()
                    .take()
                    .expect("Shutdown called multiple times");
                drop(ui);
            });
        }

        {
            use crate::gtk::ToggleButtonExt;
            let ui = ui_container.borrow_mut();
            for check_btn in &ui.as_ref().unwrap().camera_encoding_checks {
                let robot_ref = Rc::clone(&robot_ui);
                let cam_id = *check_btn.0;
                check_btn.1.connect_clicked(move |btn| {
                    robot_ref
                        .borrow_mut()
                        .ask_set_camera_prop(cam_id, 0, 0, 0, btn.get_active())
                        .expect("Failed to set camera prop");
                });
            }
        }

        {
            use crate::gtk::ComboBoxExt;
            use crate::gtk::ToggleButtonExt;
            use crate::gtk::TreeModelExt;
            let ui = ui_container.borrow_mut();
            for res_combo in &ui.as_ref().unwrap().camera_res_combos {
                let robot_ref = Rc::clone(&robot_ui);
                let ui_container_ref = Rc::clone(&ui_container);
                let cam_id = *res_combo.0;
                res_combo.1.connect_changed(move |combo| {
                    let ui = ui_container_ref.borrow_mut();
                    let encoded = ui
                        .as_ref()
                        .unwrap()
                        .camera_encoding_checks
                        .get(&cam_id)
                        .as_ref()
                        .unwrap()
                        .get_active();
                    let model = combo.get_model().unwrap();
                    let iter = combo.get_active_iter();
                    match iter {
                        Some(tree_iter) => {
                            let res_val = model
                                .get_value(&tree_iter, 0)
                                .get::<String>()
                                .expect("Failed to get resolution in UI");
                            res_val.map(|res_str| {
                                let mut split = res_str.split("x");
                                let width = split
                                    .next()
                                    .unwrap()
                                    .trim()
                                    .parse::<u16>()
                                    .expect("Failed to get resolution width in UI");
                                let height = split
                                    .next()
                                    .unwrap()
                                    .trim()
                                    .parse::<u16>()
                                    .expect("Failed to get resolution height in UI");
                                robot_ref
                                    .borrow_mut()
                                    .ask_set_camera_prop(cam_id, width, height, 0, encoded)
                                    .expect("Failed to set camera prop");
                            });
                        }
                        None => (),
                    }
                });
            }
        }

        use crate::gtk::ButtonExt;
        {
            let robot_ref = Rc::clone(&robot_ui);
            let ui = ui_container.borrow_mut();
            ui.as_ref()
                .unwrap()
                .forward_button
                .connect_clicked(move |_| {
                    robot_ref.borrow_mut().move_forward();
                });
        }
        {
            let robot_ref = Rc::clone(&robot_ui);
            let ui = ui_container.borrow_mut();
            ui.as_ref()
                .unwrap()
                .backward_button
                .connect_clicked(move |_| {
                    robot_ref.borrow_mut().move_backward();
                });
        }
        {
            let robot_ref = Rc::clone(&robot_ui);
            let ui = ui_container.borrow_mut();
            ui.as_ref().unwrap().right_button.connect_clicked(move |_| {
                robot_ref.borrow_mut().rotate_right();
            });
        }
        {
            let robot_ref = Rc::clone(&robot_ui);
            let ui = ui_container.borrow_mut();
            ui.as_ref().unwrap().left_button.connect_clicked(move |_| {
                robot_ref.borrow_mut().rotate_left();
            });
        }
        {
            let robot_ref = Rc::clone(&robot_ui);
            let ui_container_ref = Rc::clone(&ui_container);
            let camera_list_clone = camera_list.unwrap();
            glib::source::timeout_add_local(30, move || {
                let mut ui = ui_container_ref.borrow_mut();
                ui.as_mut().map_or_else(
                    || glib::Continue(false),
                    |v| {
                        for camera_id in &camera_list_clone {
                            let img = robot_ref.borrow_mut().get_image(*camera_id);
                            img.map(|mut data| v.update_image(*camera_id, &mut data));
                        }
                        glib::Continue(true)
                    },
                )
            });
        }
    });
    application.connect_activate(|_| {});
    // application.run(&std::env::args().collect::<Vec<_>>());
    application.run(&[]);
    Ok(())
}
