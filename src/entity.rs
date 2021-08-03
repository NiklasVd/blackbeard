use std::{collections::HashMap};
use rapier2d::data::Index;
use tetra::{Context, Event};
use crate::{Object, Rcc, Ship, Transform};

pub enum EntityType {
    Ship = 0,
    Object = 1
}

impl EntityType {
    pub fn to_num(&self) -> u128 {
        match self {
            EntityType::Ship => 0,
            EntityType::Object => 1
        }
    }

    pub fn to_entity_type(n: u128) -> EntityType {
        match n {
            0 => EntityType::Ship,
            1 => EntityType::Object,
            _ => panic!("Number does not correspond with any entity type")
        }
    }
}

pub trait Entity {
    fn get_type(&self) -> EntityType;
    fn get_transform(&self) -> &Transform;
    fn get_transform_mut(&mut self) -> &mut Transform;

    fn get_index(&self) -> Index {
        self.get_transform().handle.0.0
    }

    fn update(&mut self, ctx: &mut Context, ships: &HashMap<Index, Rcc<Ship>>,
        objects: &HashMap<Index, Rcc<Object>>) -> tetra::Result {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }
    fn event(&mut self, ctx: &mut Context, event: Event, ships: &HashMap<Index, Rcc<Ship>>,
        objects: &HashMap<Index, Rcc<Object>>) -> tetra::Result {
        Ok(())
    }
}
