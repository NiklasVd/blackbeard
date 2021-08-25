use binary_stream::{BinaryStream, Serializable};
use crate::{ID, V2, deserialize_v2, peer::DisconnectReason, serialize_v2};
use std::fmt;

pub enum Packet {
    Handshake {
        name: String
    },
    HandshakeReply {
        id: u16, players: Vec<ID>, /* Game Settings */
    },
    Disconnect {
        reason: DisconnectReason
    },
    ChatMessage {
        message: String
    },
    Input {
        index: u64,
        rmb: bool, q: bool, e: bool, // q && e => Pressed space
        mouse_pos: Option<V2>
    }
}

impl Packet {
    pub fn to_num(&self) -> u8 {
        match self {
            Packet::Handshake { .. } => 0,
            Packet::HandshakeReply { .. } => 1,
            Packet::Disconnect { .. } => 2,
            Packet::ChatMessage { .. } => 3,
            Packet::Input { ..} => 4,
        }
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Packet::Handshake { name } => writeln!(f, "Handshake Packet (name: {})", name),
            Packet::HandshakeReply { id, players } => writeln!(f, "Handshake Reply Packet (ID: {:?}, players: {:?}",
                id, players),
            Packet::Disconnect { reason } => writeln!(f, "Disconnect Packet (reason: {:?}",
                reason),
            Packet::ChatMessage { message } => writeln!(f, "Chat Message Packet (message: {}",
                message),
            Packet::Input { index, rmb, q, e, mouse_pos } => writeln!(f, "Input Packet (index: {}, rmb: {}, q: {}, e: {}, mouse pos: {:?}",
                index, rmb, q, e, mouse_pos),
        }
    }
}

impl Serializable for Packet {
    fn to_stream(&self, stream: &mut BinaryStream) {
        stream.write_buffer_single(self.to_num()).unwrap();
        match self {
            Packet::Handshake { name } => {
                stream.write_string(&name).unwrap();
            },
            Packet::HandshakeReply { id, players } => {
                stream.write_vec(players).unwrap();
            },
            Packet::Disconnect { reason } => {
                reason.to_stream(stream);
            },
            Packet::ChatMessage { message } => {
                stream.write_string(message).unwrap();
            },
            Packet::Input { index, rmb, q, e, mouse_pos } => {
                stream.write_u64(*index).unwrap();
                let mut input_bits = *rmb as u8;
                input_bits |= (*q as u8) << 1;
                input_bits |= (*e as u8) << 2;
                stream.write_buffer_single(input_bits).unwrap();
                if let Some(mouse_pos) = mouse_pos {
                    serialize_v2(stream, *mouse_pos).unwrap();
                }
            },
        };
    }
    
    fn from_stream(stream: &mut BinaryStream) -> Self {
        let type_num = stream.read_buffer_single().unwrap();
        match type_num {
            0 => {
                let name = stream.read_string().unwrap();
                Packet::Handshake { name }
            },
            1 => {
                let id = stream.read_u16().unwrap();
                let players = stream.read_vec::<ID>().unwrap();
                Packet::HandshakeReply {
                    id, players
                }
            },
            2 => {
                let reason = DisconnectReason::from_stream(stream);
                Packet::Disconnect { reason }
            },
            3 => {
                let message = stream.read_string().unwrap();
                Packet::ChatMessage {
                    message
                }
            },
            4 => {
                let index = stream.read_u64().unwrap();
                let input_bits = stream.read_buffer_single().unwrap();
                // 0b00000_0_0_0
                //         e q Rmb
                let rmb = (input_bits & 0b1) != 0;
                let q = ((input_bits & 0b1) << 1) != 0;
                let e = ((input_bits & 0b1) << 2) != 0;
                let mut mouse_pos = None;
                if rmb {
                    mouse_pos = Some(deserialize_v2(stream));
                }

                Packet::Input {
                    index, rmb, q, e, mouse_pos
                }
            },
            _ => panic!("Index not assigned to any packet type")
        }
    }
}

pub fn serialize_packet(packet: Packet, sender: u16) -> Vec<u8> {
    let mut stream = BinaryStream::new();
    stream.write_u16(sender).unwrap();
    packet.to_stream(&mut stream);
    stream.get_buffer_vec()
}


// Sent by clients, as server can easily look up sender-ID from sender socket address
pub fn serialize_packet_unsigned(packet: Packet) -> Vec<u8> {
    let mut stream = BinaryStream::new();
    packet.to_stream(&mut stream);
    stream.get_buffer_vec()
}

pub fn deserialize_packet(packet_bytes: Vec<u8>) -> (Packet, u16) {
    let mut stream = BinaryStream::from_bytes(&packet_bytes);
    let sender = stream.read_u16().unwrap();
    (Packet::from_stream(&mut stream), sender)
}

pub fn deserialize_packet_unsigned(packet_bytes: Vec<u8>) -> Packet {
    let mut stream = BinaryStream::from_bytes(&packet_bytes);
    Packet::from_stream(&mut stream)
}

