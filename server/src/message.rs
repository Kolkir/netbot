use std::any::Any;

#[derive(PartialEq, Eq, Hash)]
pub enum MessageId {
    Hello = 1,
    CaptureImage = 2,
    RecvImage = 3,
    GetCameraList = 4,
    RecvCameraList = 5,
    Move = 6,
    GetCameraProp = 7,
    RecvCameraProp = 8,
    Stop = 9,
    Unknown,
}
impl From<u8> for MessageId {
    fn from(orig: u8) -> Self {
        match orig {
            1 => return MessageId::Hello,
            2 => return MessageId::CaptureImage,
            3 => return MessageId::RecvImage,
            4 => return MessageId::GetCameraList,
            5 => return MessageId::RecvCameraList,
            6 => return MessageId::Move,
            7 => return MessageId::GetCameraProp,
            8 => return MessageId::RecvCameraProp,
            9 => return MessageId::Stop,
            _ => return MessageId::Unknown,
        };
    }
}
pub trait Message {
    fn id(&self) -> u8;
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

pub trait SendMessage: Message {
    fn size(&self) -> u32;
    fn to_bytes(&mut self) -> Option<&[u8]>;
}

pub trait RecvMessage: Message {
    fn from_bytes(&mut self, _buf: &[u8]) {}
}

pub struct HelloMsg {}

impl Message for HelloMsg {
    fn id(&self) -> u8 {
        return MessageId::Hello as u8;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl RecvMessage for HelloMsg {
    fn from_bytes(&mut self, _buf: &[u8]) {}
}

impl SendMessage for HelloMsg {
    fn size(&self) -> u32 {
        return 0;
    }
    fn to_bytes(&mut self) -> Option<&[u8]> {
        None
    }
}

pub struct StopMsg {}

impl Message for StopMsg {
    fn id(&self) -> u8 {
        return MessageId::Stop as u8;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl SendMessage for StopMsg {
    fn size(&self) -> u32 {
        return 0;
    }
    fn to_bytes(&mut self) -> Option<&[u8]> {
        None
    }
}
