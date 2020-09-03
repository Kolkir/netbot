pub enum MessageId {
    Hello = 1,
    CaptureImage = 2,
    RecvImage = 3,
    GetCameraList = 4,
    RecvCameraList = 5,
}

pub trait Message<'a> {
    fn id(&self) -> u8;
}

pub trait SendMessage<'a>: Message<'a> {
    fn size(&self) -> Option<u32>;
    fn to_bytes(&'a mut self) -> Option<&'a [u8]>;
}

pub trait RecvMessage<'a>: Message<'a> {
    fn from_bytes(&mut self, _buf: &[u8]) {}
}
