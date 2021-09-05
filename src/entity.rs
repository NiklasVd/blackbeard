use rapier2d::data::Index;
use tetra::{Context, Event};
use crate::{Rcc, Ship, Transform, world::World};

#[derive(Debug, PartialEq, Eq)]
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
            n @ _ => panic!("Index {} does not correspond with any entity type", n)
        }
    }
}

pub trait GameState {
    fn update(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event, world: &mut World)
        -> tetra::Result {
        Ok(())
    }
}

pub trait Entity : GameState {
    fn get_type(&self) -> EntityType;
    fn get_name(&self) -> String;
    fn get_transform(&self) -> &Transform;
    fn get_transform_mut(&mut self) -> &mut Transform;
    fn marked_destroy(&self) -> bool {
        false
    }
    fn destroy(&mut self);

    fn collide_with_ship(&mut self, ctx: &mut Context, other: Rcc<Ship>,
        world: &mut World) -> tetra::Result {
        Ok(())
    }

    fn collide_with_entity(&mut self, ctx: &mut Context, other: Rcc<dyn Entity>,
        world: &mut World) -> tetra::Result {
        Ok(())
    }
    fn collide_with_neutral(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }

    fn get_index(&self) -> Index {
        self.get_transform().get_index()
    }
}
