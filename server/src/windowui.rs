extern crate iui;
use iui::controls::{Button, Group, Label, VerticalBox};
use iui::prelude::*;

pub struct WindowUI {
    ui: UI,
}

impl WindowUI {
    pub fn new() -> WindowUI {
        let ui = UI::init().expect("Couldn't initialize UI library");
        let mut win = Window::new(&ui, "Test App", 200, 200, WindowType::NoMenubar);
        let mut vbox = VerticalBox::new(&ui);
        vbox.set_padded(&ui, true);
        let mut button = Button::new(&ui, "Button");
        button.on_clicked(&ui, {
            let ui = ui.clone();
            move |btn| {
                btn.set_text(&ui, "Clicked!");
            }
        });
        vbox.append(&ui, button, LayoutStrategy::Compact);
        win.set_child(&ui, vbox);
        win.show(&ui);
        ui.main();
    }
}
