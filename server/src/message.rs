pub enum MessageId {
    Hello = 1,
    CaptureImage = 2,
    RecvImage = 3,
    GetCameraList = 4,
    RecvCameraList = 5,
    Move = 6,
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
            _ => return MessageId::Unknown,
        };
    }
}
pub trait Message {
    fn id(&self) -> u8;
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
