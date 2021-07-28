use tetra::{State};

use crate::Rcc;

pub enum EntityType {
    Ship = 0,
    Island = 1
}

impl EntityType {
    pub fn to_num(&self) -> u128 {
        match self {
            EntityType::Ship => 0,
            EntityType::Island => 1
        }
    }

    pub fn to_entity_type(n: u128) -> EntityType {
        match n {
            0 => EntityType::Ship,
            1 => EntityType::Island,
            _ => panic!("Number does not correspond with any entity type")
        }
    }
}

pub trait Entity : State {
    fn get_type(&self) -> EntityType;
}
