use binary_stream::{BinaryStream, Serializable};
use tetra::{Context, input::{Key, MouseButton, is_key_down, is_mouse_button_down}};
use crate::{GC, ID, V2, deserialize_v2, peer::DisconnectReason, serialize_v2};
use std::fmt;

pub enum Packet {
    Handshake {
        name: String
    },
    HandshakeReply {
        players: Vec<ID>, /* Game Settings */
    },
    PlayerConnect {
        name: String
    },
    PlayerDisconnect {
        reason: DisconnectReason
    },
    ChatMessage {
        message: String
    },
    Input {
        state: InputState
    }
}

impl Packet {
    pub fn to_num(&self) -> u8 {
        match self {
            Packet::Handshake { .. } => 0,
            Packet::HandshakeReply { .. } => 1,
            Packet::PlayerConnect { .. } => 2,
            Packet::PlayerDisconnect { .. } => 3,
            Packet::ChatMessage { .. } => 4,
            Packet::Input { ..} => 5,
        }
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Packet::Handshake { name } => writeln!(f, "Handshake Packet (name: {})", name),
            Packet::HandshakeReply { players } => writeln!(f, "Handshake Reply Packet (players: {:?})",
                players),
            Packet::PlayerConnect { name } => writeln!(f, "Player Connect Packet (name: {})", name),
            Packet::PlayerDisconnect { reason } => writeln!(f, "Player Disconnect Packet (reason: {:?})",
                reason),
            Packet::ChatMessage { message } => writeln!(f, "Chat Message Packet (message: {})",
                message),
            Packet::Input { state } => writeln!(f, "{:?}", state),
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
            Packet::HandshakeReply { players } => {
                stream.write_vec(players).unwrap();
            },
            Packet::PlayerConnect { name } => {
                stream.write_string(name).unwrap();
            }
            Packet::PlayerDisconnect { reason } => {
                reason.to_stream(stream);
            },
            Packet::ChatMessage { message } => {
                stream.write_string(message).unwrap();
            },
            Packet::Input { state } => {
                state.to_stream(stream);
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
                let players = stream.read_vec::<ID>().unwrap();
                Packet::HandshakeReply {
                    players
                }
            },
            2 => {
                let name = stream.read_string().unwrap();
                Packet::PlayerConnect { name }
            }
            3 => {
                let reason = DisconnectReason::from_stream(stream);
                Packet::PlayerDisconnect { reason }
            },
            4 => {
                let message = stream.read_string().unwrap();
                Packet::ChatMessage {
                    message
                }
            },
            5 => {
                Packet::Input {
                    state: InputState::from_stream(stream)
                }
            },
            n @ _ => panic!("Index {} not assigned to any packet type", n)
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

pub struct InputState {
    pub rmb: bool,
    pub q: bool,
    pub e: bool,
    pub mouse_pos: Option<V2>
}

impl InputState {
    pub fn new(rmb: bool, q: bool, e: bool, mouse_pos: Option<V2>) -> InputState {
        InputState {
            rmb, q, e, mouse_pos
        }
    }

    pub fn discover(ctx: &mut Context, game: GC) -> InputState {
        let rmb = is_mouse_button_down(ctx, MouseButton::Right);
        let q = is_key_down(ctx, Key::Q); // is_key_down: Returns true if the specified key is currently down. is_key_pressed: Returns true if the specified key was pressed since the last update.
        let e = is_key_down(ctx, Key::E);
        let mouse_pos = match rmb {
            true => Some(game.borrow().cam.get_mouse_pos(ctx)),
            false => None
        };
        InputState::new(q, e, rmb, mouse_pos)
    }
}

impl Serializable for InputState {
    fn to_stream(&self, stream: &mut BinaryStream) {
        let mut input_bits = self.rmb as u8;
        input_bits |= (self.q as u8) << 1;
        input_bits |= (self.e as u8) << 2;
        stream.write_buffer_single(input_bits).unwrap();
        if let Some(mouse_pos) = self.mouse_pos {
            serialize_v2(stream, mouse_pos).unwrap();
        }
    }

    fn from_stream(stream: &mut BinaryStream) -> Self {
        let input_bits = stream.read_buffer_single().unwrap();
        // 0b00000_0_0_0
        //         e q Rmb
        let rmb = (input_bits & 0b1) != 0;
        let q = ((input_bits & 0b1) << 1) != 0;
        let e = ((input_bits & 0b1) << 2) != 0;
        let mouse_pos = match rmb {
            true => Some(deserialize_v2(stream)),
            false => None
        };
        InputState::new(q, e, rmb, mouse_pos)
    }
}

impl fmt::Debug for InputState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Rmb: {}, Q: {}, E: {}, Mouse-pos: {:?}", self.rmb, self.q, self.e, self.mouse_pos)
    }
}
