use super::message;
use message::{Message, MessageId, RecvMessage, SendMessage};

#[derive(Debug)]
pub struct CaptureImageMsg {
    pub id: u8,
    pub camera_id: u8,
    pub frame_width: u16,
    pub frame_height: u16,
    data: Vec<u8>,
}

impl CaptureImageMsg {
    pub fn new() -> CaptureImageMsg {
        let id_value = MessageId::CaptureImage as u8;
        CaptureImageMsg {
            id: id_value,
            camera_id: 0,
            frame_width: 0,
            frame_height: 0,
            data: Vec::new(),
        }
    }
}

impl<'a> Message for CaptureImageMsg {
    fn id(&self) -> u8 {
        return self.id;
    }
}

impl<'a> SendMessage for CaptureImageMsg {
    fn size(&self) -> u32 {
        return 1 + 1 + 2 + 2;
    }
    fn to_bytes(&mut self) -> Option<&[u8]> {
        self.data.push(self.id);
        self.data.push(self.camera_id);
        let width_bytes = self.frame_width.to_be_bytes();
        self.data.extend_from_slice(&width_bytes);
        let height_bytes = self.frame_height.to_be_bytes();
        self.data.extend_from_slice(&height_bytes);
        return Some(&self.data[..]);
    }
}

#[derive(Debug)]
pub struct RecvImageMsg {
    pub id: u8,
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
            channels: 0,
            frame_width: 0,
            frame_height: 0,
            data: Vec::new(),
        }
    }
}

impl<'a> Message for RecvImageMsg {
    fn id(&self) -> u8 {
        return self.id;
    }
}

impl RecvMessage for RecvImageMsg {
    fn from_bytes(&mut self, _buf: &[u8]) {
        {
            let tmp = slice_as_array!(&_buf[0..2], [u8; 2]).expect("RecvImageMsg wrong data");
            self.channels = u16::from_be_bytes(*tmp);
        }
        {
            let tmp = slice_as_array!(&_buf[2..4], [u8; 2]).expect("RecvImageMsg wrong data");
            self.frame_width = u16::from_be_bytes(*tmp);
        }
        {
            let tmp = slice_as_array!(&_buf[4..6], [u8; 2]).expect("RecvImageMsg wrong data");
            self.frame_height = u16::from_be_bytes(*tmp);
        }
        self.data.extend_from_slice(&_buf[6..]);
    }
}
