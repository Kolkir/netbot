use super::message;
use message::{Message, MessageId, RecvMessage, SendMessage};
use std::any::Any;

#[derive(Debug)]
pub struct GetCameraListMsg {
    pub id: u8,
    data: [u8; 1],
}

impl GetCameraListMsg {
    pub fn new() -> GetCameraListMsg {
        let id_value = MessageId::GetCameraList as u8;
        GetCameraListMsg {
            id: id_value,
            data: [id_value as u8; 1],
        }
    }
}

impl Message for GetCameraListMsg {
    fn id(&self) -> u8 {
        return self.id;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl SendMessage for GetCameraListMsg {
    fn size(&self) -> u32 {
        return 1;
    }
    fn to_bytes(&mut self) -> Option<&[u8]> {
        return Some(&self.data);
    }
}

#[derive(Debug)]
pub struct RecvCameraListMsg {
    pub id: u8,
    pub camera_list: Vec<u8>,
}

impl RecvCameraListMsg {
    pub fn new() -> RecvCameraListMsg {
        let id_value = MessageId::RecvCameraList as u8;
        RecvCameraListMsg {
            id: id_value,
            camera_list: Vec::new(),
        }
    }
}

impl Message for RecvCameraListMsg {
    fn id(&self) -> u8 {
        return self.id;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl RecvMessage for RecvCameraListMsg {
    fn from_bytes(&mut self, buf: &[u8]) {
        let size = buf[0] as usize;
        self.camera_list.resize(size, 0);
        for (i, cam_id) in buf[1..].iter().enumerate() {
            self.camera_list[i] = *cam_id;
        }
    }
}
