use super::message;
use message::{Message, MessageId, RecvMessage, SendMessage};
use std::any::Any;

#[derive(Debug)]
pub struct CaptureImageMsg {
    pub id: u8,
    pub camera_id: u8,
    data: Vec<u8>,
}

impl CaptureImageMsg {
    pub fn new() -> CaptureImageMsg {
        let id_value = MessageId::CaptureImage as u8;
        CaptureImageMsg {
            id: id_value,
            camera_id: 0,
            data: Vec::new(),
        }
    }
}

impl Message for CaptureImageMsg {
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

impl SendMessage for CaptureImageMsg {
    fn size(&self) -> u32 {
        return 1 + 2 + 2;
    }

    fn to_bytes(&mut self) -> Option<&[u8]> {
        self.data.push(self.camera_id);
        return Some(&self.data);
    }
}

#[derive(Debug)]
pub struct RecvImageMsg {
    pub id: u8,
    pub camera_id: u8,
    pub channels: u16,
    pub frame_width: u16,
    pub frame_height: u16,
    pub data: Vec<u8>,
}

impl RecvImageMsg {
    pub fn new() -> RecvImageMsg {
        let id_value = MessageId::RecvImage as u8;
        RecvImageMsg {
            id: id_value,
            camera_id: 0,
            channels: 0,
            frame_width: 0,
            frame_height: 0,
            data: Vec::new(),
        }
    }
}

impl Message for RecvImageMsg {
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

impl RecvMessage for RecvImageMsg {
    fn from_bytes(&mut self, _buf: &[u8]) {
        {
            self.camera_id = _buf[0];
        }
        {
            let tmp = slice_as_array!(&_buf[1..3], [u8; 2]).expect("RecvImageMsg wrong data");
            self.channels = u16::from_be_bytes(*tmp);
        }
        {
            let tmp = slice_as_array!(&_buf[3..5], [u8; 2]).expect("RecvImageMsg wrong data");
            self.frame_width = u16::from_be_bytes(*tmp);
        }
        {
            let tmp = slice_as_array!(&_buf[5..7], [u8; 2]).expect("RecvImageMsg wrong data");
            self.frame_height = u16::from_be_bytes(*tmp);
        }
        self.data.extend_from_slice(&_buf[7..]);
    }
}
