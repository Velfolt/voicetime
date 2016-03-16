use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub enum MessageType {
    Connect,
    Disconnect,
    Audio
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub struct Message {
    pub message_type: MessageType,
    pub data: Vec<i16>
}

impl Message {
    pub fn new(message_type: MessageType, data: Option<Vec<i16>>) -> Message {
        Message { message_type: message_type, data: data.unwrap_or(vec!()) }
    }

    pub fn encoded(message_type: MessageType, data: Option<Vec<i16>>) -> Vec<u8> {
        encode(&Message::new(message_type, data), SizeLimit::Infinite).unwrap()
    }

    pub fn decoded(buffer: &[u8]) -> Message {
        decode(&buffer).unwrap()
    }
}
