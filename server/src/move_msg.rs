use super::message;
use message::{Message, MessageId, SendMessage};

#[derive(Debug)]
pub struct MoveMsg {
    pub id: u8,
    pub left_speed: u8,
    pub left_dir: u8,
    pub right_speed: u8,
    pub right_dir: u8,
    data: Vec<u8>,
}

impl MoveMsg {
    pub fn new() -> MoveMsg {
        let id_value = MessageId::Move as u8;
        MoveMsg {
            id: id_value,
            left_speed: 0,
            left_dir: 0,
            right_speed: 0,
            right_dir: 0,
            data: Vec::new(),
        }
    }
}

impl<'a> Message for MoveMsg {
    fn id(&self) -> u8 {
        return self.id;
    }
}

impl<'a> SendMessage for MoveMsg {
    fn size(&self) -> u32 {
        return 1 + 1 + 1 + 1;
    }

    fn to_bytes(&mut self) -> Option<&[u8]> {
        self.data.push(self.left_speed);
        self.data.push(self.left_dir);
        self.data.push(self.right_speed);
        self.data.push(self.right_dir);
        return Some(&self.data[..]);
    }
}
