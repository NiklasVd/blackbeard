use crate::{Rcc, Ship};

pub struct Player {
    pub name: String,
    pub possessed_ship: Rcc<Ship>
}
