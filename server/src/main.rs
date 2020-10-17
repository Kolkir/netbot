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
mod robot_interface;
mod server;

use std::cell::RefCell;
use std::error::Error;
use std::net::Ipv4Addr;
use std::rc::Rc;
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

    let application = Application::new(Some("com.github.kolkir.netbot"), Default::default())
        .expect("failed to initialize GTK application");

    application.connect_startup(move |app| {
        let mut window_ui = WindowUi::new(app, addr, port);
        window_ui.configure_robot_camera();
        let ui_container = Rc::new(RefCell::new(Some(window_ui)));
        {
            let ui_container_ref = Rc::clone(&ui_container);
            app.connect_shutdown(move |_| {
                let ui = ui_container_ref
                    .borrow_mut()
                    .take()
                    .expect("Shutdown called multiple times");
                drop(ui);
            });
        }

        use crate::gtk::ButtonExt;
        {
            let ui_container_ref = Rc::clone(&ui_container);
            let ui = ui_container.borrow_mut();
            ui.as_ref()
                .unwrap()
                .forward_button
                .connect_clicked(move |_| {
                    let mut ui = ui_container_ref.borrow_mut();
                    ui.as_mut()
                        .expect("UI is unreachable in the forward button callback")
                        .robot
                        .move_forward();
                });
        }
        {
            let ui_container_ref = Rc::clone(&ui_container);
            let ui = ui_container.borrow_mut();
            ui.as_ref()
                .unwrap()
                .backward_button
                .connect_clicked(move |_| {
                    let mut ui = ui_container_ref.borrow_mut();
                    ui.as_mut()
                        .expect("UI is unreachable in the backward button callback")
                        .robot
                        .move_backward();
                });
        }
        {
            let ui_container_ref = Rc::clone(&ui_container);
            let ui = ui_container.borrow_mut();
            ui.as_ref().unwrap().right_button.connect_clicked(move |_| {
                let mut ui = ui_container_ref.borrow_mut();
                ui.as_mut()
                    .expect("UI is unreachable in the right button callback")
                    .robot
                    .rotate_right();
            });
        }
        {
            let ui_container_ref = Rc::clone(&ui_container);
            let ui = ui_container.borrow_mut();
            ui.as_ref().unwrap().left_button.connect_clicked(move |_| {
                let mut ui = ui_container_ref.borrow_mut();
                ui.as_mut()
                    .expect("UI is unreachable in the left button callback")
                    .robot
                    .rotate_left();
            });
        }
        {
            let ui_container_ref = Rc::clone(&ui_container);
            glib::source::timeout_add_local(30, move || {
                let mut ui = ui_container_ref.borrow_mut();
                ui.as_mut().map_or_else(
                    || glib::Continue(false),
                    |v| {
                        v.update_robot_images(Duration::from_millis(10));
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
