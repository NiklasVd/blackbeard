use rapier2d::data::Index;

use crate::{Entity, ID, Rcc, Ship};

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
