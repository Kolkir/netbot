use super::message;
use message::{Message, MessageId, RecvMessage, SendMessage};
use std::any::Any;

#[derive(Debug)]
pub struct GetCameraPropMsg {
    pub id: u8,
    pub camera_id: u8,
    data: Vec<u8>,
}

impl GetCameraPropMsg {
    pub fn new() -> GetCameraPropMsg {
        let id_value = MessageId::GetCameraProp as u8;
        GetCameraPropMsg {
            id: id_value,
            camera_id: 0,
            data: Vec::new(),
        }
    }
}

impl Message for GetCameraPropMsg {
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

impl SendMessage for GetCameraPropMsg {
    fn size(&self) -> u32 {
        return 1;
    }
    fn to_bytes(&mut self) -> Option<&[u8]> {
        self.data.push(self.camera_id);
        return Some(&self.data);
    }
}

#[derive(Debug)]
pub struct RecvCameraPropMsg {
    pub id: u8,
    pub camera_prop: Vec<u16>,
}

impl RecvCameraPropMsg {
    pub fn new() -> RecvCameraPropMsg {
        let id_value = MessageId::RecvCameraProp as u8;
        RecvCameraPropMsg {
            id: id_value,
            camera_prop: Vec::new(),
        }
    }
}

impl Message for RecvCameraPropMsg {
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

impl RecvMessage for RecvCameraPropMsg {
    fn from_bytes(&mut self, buf: &[u8]) {
        let size: usize;
        {
            let tmp = slice_as_array!(&buf[0..2], [u8; 2]).expect("RecvCameraPropMsg wrong data");
            size = u16::from_be_bytes(*tmp) as usize;
        }
        self.camera_prop.resize(size, 0);
        for i in 0..size {
            let s = 2 + i * 2;
            let e = s + 2;
            let tmp = slice_as_array!(&buf[s..e], [u8; 2]).expect("RecvCameraPropMsg wrong data");
            self.camera_prop[i] = u16::from_be_bytes(*tmp);
        }
    }
}
