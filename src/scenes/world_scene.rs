use std::{collections::{HashMap}};
use rapier2d::{data::Index, prelude::ContactEvent};
use tetra::{Context, Event, State};
use crate::{Controller, GC, Island, Rcc, Ship, V2, wrap_rcc};
use super::scenes::{Scene, SceneType};

pub struct WorldScene {
    pub ships: HashMap<Index, Rcc<Ship>>,
    pub islands: HashMap<Index, Rcc<Island>>,
    pub controller: Controller,
    game: GC
}

impl WorldScene {
    pub fn new(ctx: &mut Context, game: GC) -> tetra::Result<WorldScene> {
        let mut world_scene = WorldScene {
            ships: HashMap::new(),
            islands: HashMap::new(),
            controller: Controller::new(ctx, game.clone())?,
            game
        };
        
        let ship = world_scene.build_caravel(ctx, "Niklas".to_owned(), V2::zero(), 0.0)?;
        world_scene.controller.possess_ship(ship);
        world_scene.build_caravel(ctx, "Antonia".to_owned(), V2::new(250.0, 400.0), 0.8)?;
        
        world_scene.build_island(ctx, V2::new(1000.0, 800.0), 0.12)?;
        Ok(world_scene)
    }

    pub fn build_caravel(&mut self, ctx: &mut Context, name: String, pos: V2, rot: f32)
        -> tetra::Result<Rcc<Ship>> {
        let mut ship = Ship::caravel(ctx, self.game.clone(), name)?;
        let ship_coll_index = ship.transform.handle.1.0;
        ship.transform.set_pos(pos, rot);

        let ship_ref = wrap_rcc(ship);
        self.ships.insert(ship_coll_index, ship_ref.clone());
        Ok(ship_ref)
    }

    pub fn build_island(&mut self, ctx: &mut Context, pos: V2, rot: f32)
        -> tetra::Result<Rcc<Island>> {
        let mut island = Island::build(ctx, self.game.clone())?;
        let island_coll_index = island.transform.handle.1.0;
        island.transform.set_pos(pos, rot);

        let island_ref = wrap_rcc(island);
        self.islands.insert(island_coll_index, island_ref.clone());
        Ok(island_ref)
    }

    pub fn get_ship(&self, index: Index) -> Rcc<Ship> {
        self.ships.get(&index).unwrap().clone()
    }

    fn handle_intersections(&self) {
        // CHECK: RefCell locked during iteration?
        for intersection in self.game.borrow().physics.get_intersections().iter() {
            if !intersection.intersecting {
                continue
            }

            println!("{:?} and {:?} intersect!", intersection.collider1, intersection.collider2);
        }
    }

    fn handle_contacts(&self) {
        for contact in self.game.borrow().physics.get_contacts().iter() {
            match contact {
                ContactEvent::Started(coll1, coll2) => {
                    println!("{:?} and {:?} have contact!", coll1, coll2);
                },
                _ => ()
            }
        }
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
        self.handle_intersections();
        self.handle_contacts();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.controller.draw(ctx)?;
        for ship in self.ships.values() {
            ship.borrow_mut().draw(ctx)?;
        }
        for island in self.islands.values() {
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
