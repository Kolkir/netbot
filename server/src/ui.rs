use std::error::Error;

pub type GetCameraListFn = Box<dyn Fn() -> Result<Vec<u8>, Box<dyn Error>>>;
pub type CaptureImageFn = Box<dyn Fn(u8, u16, u16) -> Result<Vec<u8>, Box<dyn Error>>>;
pub type MoveFn = Box<dyn Fn(u8, u8, u8, u8) -> Result<(), Box<dyn Error>>>;
pub type GetCameraPropFn = Box<dyn Fn(u8) -> Result<Vec<u16>, Box<dyn Error>>>;

pub trait UI {
    fn set_get_camera_list_fn(&mut self, get_camera_list_fn: GetCameraListFn);
    fn set_capture_img_fn(&mut self, capture_image_fn: CaptureImageFn);
    fn set_move_fn(&mut self, move_fn: MoveFn);
    fn set_get_camera_prop_fn(&mut self, get_camera_prop_fn: GetCameraPropFn);
    fn run(&mut self) -> Result<(), Box<dyn Error>>;
}
