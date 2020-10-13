use std::error::Error;

pub type AskCameraListFn = Box<dyn Fn() -> Result<(), Box<dyn Error>>>;
pub type AskImageFn = Box<dyn Fn(u8, u16, u16) -> Result<(), Box<dyn Error>>>;
pub type MoveFn = Box<dyn Fn(u8, u8, u8, u8) -> Result<(), Box<dyn Error>>>;
pub type GetCameraListFn = Box<dyn FnMut() -> Option<Vec<u8>>>;
pub type GetImageFn = Box<dyn FnMut() -> Option<Vec<u8>>>;

pub trait UI {
    fn set_ask_img_fn(&mut self, ask_image_fn: AskImageFn);
    fn set_ask_camera_list_fn(&mut self, ask_camera_list_fn: AskCameraListFn);
    fn set_get_camera_list_fn(&mut self, get_camera_list_fn: GetCameraListFn);
    fn set_get_img_fn(&mut self, get_image_fn: GetImageFn);
    fn set_move_fn(&mut self, move_fn: MoveFn);
    fn run(&mut self) -> Result<(), Box<dyn Error>>;
}
