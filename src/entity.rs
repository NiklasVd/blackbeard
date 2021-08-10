use std::any::Any;

use rapier2d::data::Index;
use tetra::{Context, Event};
use crate::{Rcc, Transform, world_scene::Entities};

pub enum EntityType {
    Ship = 0,
    Object = 1,
    CannonBall = 2
}

impl EntityType {
    pub fn to_num(&self) -> u128 {
        match self {
            EntityType::Ship => 0,
            EntityType::Object => 1,
            &EntityType::CannonBall => 2
        }
    }

    pub fn to_entity_type(n: u128) -> EntityType {
        match n {
            0 => EntityType::Ship,
            1 => EntityType::Object,
            2 => EntityType::CannonBall,
            _ => panic!("Number does not correspond with any entity type")
        }
    }
}

pub trait GameState {
    fn update(&mut self, ctx: &mut Context, entities: &mut Entities) -> tetra::Result {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event, entities: &mut Entities)
        -> tetra::Result {
        Ok(())
    }
}

pub trait Entity : GameState {
    fn get_type(&self) -> EntityType;
    fn get_name(&self) -> String;
    fn get_transform(&self) -> &Transform;
    fn get_transform_mut(&mut self) -> &mut Transform;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_any(&self) -> &dyn Any;

    fn collide_with_entity(&mut self, ctx: &mut Context, other: Rcc<dyn Entity>,
        entities: &mut Entities) -> tetra::Result {
        Ok(())
    }
    fn collide_with_neutral(&mut self, ctx: &mut Context, entities: &mut Entities)
        -> tetra::Result {
        Ok(())
    }
    fn on_destroy(&mut self, ctx: &mut Context, entities: &mut Entities) -> tetra::Result {
        Ok(())
    }

    fn get_index(&self) -> Index {
        self.get_transform().handle.1.0
    }
}

pub fn cast_entity<'a, T: Entity + 'static>(entity_any: &'a mut dyn Any) -> &'a mut T {
    match entity_any.downcast_mut::<T>() {
        Some(e) => e,
        None => panic!("Unable to cast any to entity type.")
    }
}
