use std::{collections::{HashMap}};
use tetra::{Context, Event, State};
use crate::{Controller, Entity, GC, Island, Rcc, Ship, V2, wrap_rcc};
use super::scenes::{Scene, SceneType};

pub struct WorldScene {
    pub ships: HashMap<u32, Rcc<Ship>>,
    pub islands: Vec<Rcc<Island>>,
    pub controller: Controller,
    game: GC,
    ship_index: u32
}

impl WorldScene {
    pub fn new(ctx: &mut Context, game: GC) -> tetra::Result<WorldScene> {
        let mut world_scene = WorldScene {
            ships: HashMap::new(),
            islands: Vec::new(),
            controller: Controller::init(ctx, game.clone())?,
            game,
            ship_index: 0
        };
        
        let ship = world_scene.build_ship(ctx, V2::zero(), 0.0)?;
        world_scene.controller.possess_ship(ship);
        world_scene.build_ship(ctx, V2::new(250.0, 400.0), 0.8)?;
        
        world_scene.build_island(ctx, V2::new(800.0, 800.0), 0.25)?;
        Ok(world_scene)
    }

    pub fn build_ship(&mut self, ctx: &mut Context, pos: V2, rot: f32)
        -> tetra::Result<Rcc<Ship>> {
        let mut ship = Ship::init(ctx, self.game.clone())?;
        ship.transform.set_pos(pos, rot);

        let ship_ref = wrap_rcc(ship);
        self.ships.insert(self.ship_index, ship_ref.clone());
        self.ship_index += 1;
        Ok(ship_ref)
    }

    pub fn build_island(&mut self, ctx: &mut Context, pos: V2, rot: f32)
        -> tetra::Result<Rcc<Island>> {
        let mut island = Island::init(ctx, self.game.clone())?;
        island.transform.set_pos(pos, rot);

        let island_ref = wrap_rcc(island);
        self.islands.push(island_ref.clone());
        Ok(island_ref)
    }
}

impl Scene for WorldScene {
    fn poll(&self, ctx: &mut Context) -> tetra::Result<Option<Box<dyn Scene>>> {
        Ok(None)
    }

    fn get_type(&self) -> SceneType {
        SceneType::World
    }
}

impl State for WorldScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        for ship in self.ships.values() {
            ship.borrow_mut().update(ctx)?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.controller.draw(ctx)?;
        for ship in self.ships.values() {
            ship.borrow_mut().draw(ctx)?;
        }
        for island in self.islands.iter() {
            island.borrow_mut().draw(ctx)?;
        }

        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event) -> tetra::Result {
        self.controller.event(ctx, event.clone())?;
        for ship in self.ships.values() {
            ship.borrow_mut().event(ctx, event.clone())?;
        }
        Ok(())
    }
}
