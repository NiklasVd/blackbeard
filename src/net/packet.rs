use binary_stream::{BinaryStream, Serializable};
use tetra::{Context, input::{Key, MouseButton, get_mouse_position, is_key_down, is_mouse_button_down}};
use crate::{PlayerParams, V2, deserialize_v2, game_settings::GameSettings, peer::DisconnectReason, serialize_v2, ship_data::ShipType, ship_mod::ShipModType, sync_checker::SyncState};
use std::fmt;

#[derive(Clone)]
pub enum Packet {
    Handshake {
        name: String
    },
    HandshakeReply {
        players: Vec<PlayerParams>,
        /* + Game Settings */
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
    },
    InputStep {
        step: InputStep
    },
    Game {
        phase: GamePhase,
    },
    Sync {
        state: SyncState
    },
    Selection {
        mode: bool,
        ship: Option<ShipType>,
        settings: Option<GameSettings>
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
            Packet::Input { .. } => 5,
            Packet::InputStep { .. } => 6,
            Packet::Game { .. } => 7,
            Packet::Sync { .. } => 8,
            Packet::Selection { .. } => 9
        }
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Packet::Handshake { name } => write!(f, "Handshake Packet (name: {})", name),
            Packet::HandshakeReply { players } => write!(f, "Handshake Reply Packet (players: {:?})",
                players),
            Packet::PlayerConnect { name } => write!(f, "Player Connect Packet (name: {})", name),
            Packet::PlayerDisconnect { reason } => write!(f, "Player Disconnect Packet (reason: {:?})",
                reason),
            Packet::ChatMessage { message } => write!(f, "Chat Message Packet (message: {})",
                message),
            Packet::Input { state } => write!(f, "Input State Packet ({:?})", state),
            Packet::InputStep { step } => write!(f, "Input Step Packet (states: {:?}, gen: {})", step.states, step.gen),
            Packet::Game { phase } => write!(f, "Game Packet (phase: {:?})", phase),
            Packet::Sync { state } => write!(f, "Sync Packet (state: {:?})", state),
            Packet::Selection { ship, settings, .. } => write!(f, "Selection Packet (ship: {:?}, settings: {:?}", ship, settings)
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
            }
            Packet::InputStep { step } => {
                step.to_stream(stream);
            },
            Packet::Game { phase } => {
                phase.to_stream(stream);
            },
            Packet::Sync { state } => {
                state.to_stream(stream);
            },
            Packet::Selection { mode, ship, settings } => {
                stream.write_bool(*mode).unwrap();
                if *mode {
                    ship.as_ref().unwrap().to_stream(stream);
                } else {
                    settings.as_ref().unwrap().to_stream(stream);
                }
            }
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
                let players = stream.read_vec::<PlayerParams>().unwrap();
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
                Packet::ChatMessage {
                    message: stream.read_string().unwrap()
                }
            },
            5 => {
                Packet::Input {
                    state: InputState::from_stream(stream)
                }
            }
            6 => {
                Packet::InputStep {
                    step: InputStep::from_stream(stream)
                }
            },
            7 => {
                let phase = GamePhase::from_stream(stream);
                Packet::Game {
                    phase
                }
            },
            8 => {
                Packet::Sync {
                    state: SyncState::from_stream(stream)
                }
            },
            9 => {
                let mode = stream.read_bool().unwrap();
                if mode {
                    Packet::Selection {
                        mode, ship: Some(ShipType::from_stream(stream)), settings: None
                    }
                } else {
                    Packet::Selection {
                        mode, ship: None, settings: Some(GameSettings::from_stream(stream))
                    }
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

// Send rate: every 15 frames at 60 fixed FPS = 1/4 = 0.25 secs
// Packet size:
//      3 booleans (bitpacked) = 1 byte
//      Vec2 if rmb == true (2 f32) = 8 bytes
// => 1-9 bytes (5 on avg.)
// 5 * 4 = 20 bytes/sec
#[derive(Clone)]
pub struct InputState {
    pub rmb: bool,
    pub r: bool,
    pub q: bool,
    pub e: bool,
    pub buy_mod: bool,
    pub disconnect: bool,
    pub mod_type: Option<ShipModType>,
    pub mouse_pos: Option<V2>
}

impl InputState {
    pub fn new(rmb: bool, r: bool, q: bool, e: bool, buy_mod: bool, disconnect: bool, mod_type: Option<ShipModType>,
        mouse_pos: Option<V2>) -> InputState {
        InputState {
            rmb, r, q, e, disconnect, buy_mod, mod_type, mouse_pos
        }
    }

    pub fn discover(ctx: &mut Context) -> InputState {
        let rmb = is_mouse_button_down(ctx, MouseButton::Right);
        // is_key_down: Returns true if the specified key is currently down.
        // is_key_pressed: Returns true if the specified key was pressed since the last update.
        let r = is_key_down(ctx, Key::R);
        let q = is_key_down(ctx, Key::Q);
        let e = is_key_down(ctx, Key::E);
        let mouse_pos = match rmb || r {
            true => Some(get_mouse_position(ctx)),
            false => None
        };
        InputState::new(rmb, r, q, e, false, false, None, mouse_pos)
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new(false, false, false, false, false, false, None, None)
    }
}

impl Serializable for InputState {
    fn to_stream(&self, stream: &mut BinaryStream) {
        let mut input_bits = self.rmb as u8;
        input_bits |= (self.r as u8) << 1;
        input_bits |= (self.q as u8) << 2;
        input_bits |= (self.e as u8) << 3;
        input_bits |= (self.buy_mod as u8) << 4;
        input_bits |= (self.disconnect as u8) << 5;
        stream.write_buffer_single(input_bits).unwrap();

        if self.buy_mod {
            if let Some(mod_type) = self.mod_type.as_ref() {
                mod_type.to_stream(stream);
            }
        }
        if self.rmb || self.r {
            if let Some(mouse_pos) = self.mouse_pos.as_ref() {
                serialize_v2(stream, mouse_pos.clone()).unwrap();
            }
        }
    }

    fn from_stream(stream: &mut BinaryStream) -> Self {
        let input_bits = stream.read_buffer_single().unwrap();
        // 0b00000_0_0_0
        //       e q r rmb
        let rmb = (input_bits & 0b1) != 0;
        let r = (input_bits & (0b1 << 1u8)) != 0;
        let q = (input_bits & (0b1 << 2u8)) != 0;
        let e = (input_bits & (0b1 << 3u8)) != 0;
        let buy_mod = (input_bits & (0b1 << 4u8)) != 0;
        let disconnect = (input_bits & (0b1 << 5u8)) != 0;

        let mod_type = match buy_mod {
            true => Some(ShipModType::from_stream(stream)),
            false => None
        };
        let mouse_pos = match rmb || r {
            true => Some(deserialize_v2(stream)),
            false => None
        };
        InputState::new(rmb, r, q, e, buy_mod, disconnect, mod_type, mouse_pos)
    }
}

impl fmt::Debug for InputState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Rmb: {}, R: {}, Q: {}, E: {}, Buy Mod: {}, Disconnect: {}, Mod Type: {:?}, Mouse Pos.: {:?}",
            self.rmb, self.r, self.q, self.e, self.buy_mod, self.disconnect, self.mod_type, self.mouse_pos)
    }
}

#[derive(Clone)]
pub struct InputStep {
    pub states: Vec<(u16, InputState)>,
    pub gen: u64
}

impl InputStep {
    pub fn new(states: Vec<(u16, InputState)>, gen: u64) -> InputStep {
        InputStep {
            states, gen
        }
    }

    pub fn add_state(&mut self, sender: u16, state: InputState) {
        self.states.push((sender, state));
    }
}

impl Serializable for InputStep {
    fn to_stream(&self, stream: &mut BinaryStream) {
        stream.write_buffer_single(self.states.len() as u8).unwrap();
        for (sender, state) in self.states.iter() {
            stream.write_u16(*sender).unwrap();
            state.to_stream(stream);
        }
        stream.write_u64(self.gen).unwrap();
    }

    fn from_stream(stream: &mut BinaryStream) -> Self {
        let len = stream.read_buffer_single().unwrap() as usize;
        let mut states = Vec::with_capacity(len);
        for _ in 0..len {
            let sender = stream.read_u16().unwrap();
            let state = InputState::from_stream(stream);
            states.push((sender, state));
        }
        let gen = stream.read_u64().unwrap();
        InputStep::new(states, gen)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GamePhase {
    World(u64),
    Score
}

impl Serializable for GamePhase {
    fn to_stream(&self, stream: &mut BinaryStream) {
        match self {
            GamePhase::World(world_seed) => {
                stream.write_buffer_single(0).unwrap();
                stream.write_u64(*world_seed).unwrap();
            },
            GamePhase::Score => stream.write_buffer_single(1).unwrap(),
        }
    }

    fn from_stream(stream: &mut BinaryStream) -> Self {
        match stream.read_buffer_single().unwrap() {
            0 => {
                let world_seed = stream.read_u64().unwrap();
                GamePhase::World(world_seed)
            },
            1 => GamePhase::Score,
            n @ _ => panic!("Index {} not assigned to any game phase", n)
        }
    }
}
