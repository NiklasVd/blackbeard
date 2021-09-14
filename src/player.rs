use binary_stream::{BinaryStream, Serializable};
use rapier2d::data::Index;
use crate::{ID, Rcc, entity::Entity, ship::{Ship, ShipType}};

pub struct Player {
    pub id: ID,
    pub possessed_ship_index: Index,
    pub possessed_ship: Rcc<Ship>
}

impl Player {
    pub fn new(id: ID, possessed_ship: Rcc<Ship>) -> Player {
        let possessed_ship_index = possessed_ship.borrow().get_index();
        Player {
            id, possessed_ship_index, possessed_ship
        }
    }

    pub fn possess_ship(&mut self, possessed_ship: Rcc<Ship>) {
        self.possessed_ship_index = possessed_ship.borrow().get_index();
        self.possessed_ship = possessed_ship;
    }
}

#[derive(Debug, Clone)]
pub struct PlayerParams {
    pub id: u16,
    pub ship_type: ShipType
}

impl PlayerParams {
    pub fn new(id: u16, ship_type: ShipType) -> PlayerParams {
        PlayerParams {
            id, ship_type
        }
    }
}

impl Serializable for PlayerParams {
    fn to_stream(&self, stream: &mut BinaryStream) {
        stream.write_u16(self.id).unwrap();
        self.ship_type.to_stream(stream);
    }

    fn from_stream(stream: &mut BinaryStream) -> Self {
        let id = stream.read_u16().unwrap();
        let ship_type = ShipType::from_stream(stream);
        PlayerParams::new(id, ship_type)
    }
}
