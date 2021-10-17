use core::fmt;
use binary_stream::{BinaryStream, Serializable};
use crate::{GC, ID, V2, ship::{BASE_STUN_LENGTH, MAX_SHIP_DEFENSE}};

#[derive(Debug, Clone, Copy)]
pub enum ShipType {
    Caravel,
    Schooner,
    Galleon
}

impl Serializable for ShipType {
    fn to_stream(&self, stream: &mut BinaryStream) {
        stream.write_buffer_single(match self {
            ShipType::Caravel => 0,
            ShipType::Galleon => 1,
            ShipType::Schooner => 2
        }).unwrap();
    }

    fn from_stream(stream: &mut BinaryStream) -> Self {
        match stream.read_buffer_single().unwrap() {
            0 => ShipType::Caravel,
            1 => ShipType::Galleon,
            2 => ShipType::Schooner,
            n @ _ => panic!("Index {} is not assigned to any ship type", n)
        }
    }
}

#[derive(Clone, Copy)]
pub struct ShipAttributes {
    pub health: u16,
    pub defense: u16, // 1-100
    pub movement_speed: f32,
    pub turn_rate: f32,
    pub cannon_damage: u16,
    pub cannon_reload_time: f32,
    pub ram_damage: u16
}

impl ShipAttributes {
    pub fn caravel() -> ShipAttributes {
        ShipAttributes {
            health: 140,
            defense: 60,
            movement_speed: 19.8, turn_rate: 5.25,
            cannon_damage: 15, cannon_reload_time: 5.0,
            ram_damage: 20
        }
    }

    pub fn galleon() -> ShipAttributes {
        ShipAttributes {
            health: 160,
            defense: 80,
            movement_speed: 18.5, turn_rate: 5.1,
            cannon_damage: 15, cannon_reload_time: 5.0,
            ram_damage: 30
        }
    }

    pub fn schooner() -> ShipAttributes {
        ShipAttributes {
            health: 120,
            defense: 35,
            movement_speed: 16.5, turn_rate: 3.1,
            cannon_damage: 15, cannon_reload_time: 5.0,
            ram_damage: 15
        }
    }

    pub fn get_stun_length(&self) -> f32 {
        BASE_STUN_LENGTH * (MAX_SHIP_DEFENSE / self.defense) as f32
    }
}

#[derive(Clone)]
pub enum ShipID {
    Player(ID, bool /* is local player? */),
    Computer(String)
}

impl ShipID {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }

    pub fn is_local_player(&self) -> bool {
        match self {
            ShipID::Player(_, is_local_player) => *is_local_player,
            _ => false
        }
    }
}

impl fmt::Debug for ShipID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShipID::Player(id, _) => write!(f, "{:?}", id),
            ShipID::Computer(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DamageResult {
    Hit(u16),
    Sink,
    Empty
}

pub struct ShipData {
    pub curr_health: u16,
    pub ship_type: ShipType,
    pub id: ShipID,
    pub attr: ShipAttributes,
    pub spawn_pos: Option<V2>,
    pub destroy: bool,
    pub game: GC
}

impl ShipData {
    pub fn is_sunk(&self) -> bool {
        self.curr_health == 0
    }

    pub fn set_health(&mut self, curr_health: u16) {
        assert!(curr_health <= self.attr.health);
        self.curr_health = curr_health;
    }
}
