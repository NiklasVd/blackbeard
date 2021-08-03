use std::{collections::{HashMap}};
use rapier2d::{data::Index, prelude::ContactEvent};
use tetra::{Context, Event, State};
use crate::{Controller, Entity, GC, ID, Object, Player, Rcc, Ship, V2, wrap_rcc};
use super::scenes::{Scene, SceneType};

pub struct WorldScene {
    pub ships: HashMap<Index, Rcc<Ship>>,
    pub objects: HashMap<Index, Rcc<Object>>,
    pub controller: Controller,
    game: GC
}

impl WorldScene {
    pub fn new(ctx: &mut Context, game: GC) -> tetra::Result<WorldScene> {
        let mut world_scene = WorldScene {
            ships: HashMap::new(),
            objects: HashMap::new(),
            controller: Controller::new(ctx, game.clone())?,
            game
        };
        
        let local_player = world_scene.build_player_ship(ctx,
            ID::new("Niklas".to_owned(), 0))?;
        world_scene.controller.set_local_player(local_player);
        world_scene.build_island_object(ctx, V2::new(1000.0, 800.0), 0.12)?;

        Ok(world_scene)
    }

    pub fn build_caravel(&mut self, ctx: &mut Context, name: String, pos: V2, rot: f32)
        -> tetra::Result<Rcc<Ship>> {
        let mut ship = Ship::caravel(ctx, self.game.clone(), name)?;
        let ship_rb_index = ship.get_index();
        ship.transform.set_pos(pos, rot);

        let ship_ref = wrap_rcc(ship);
        self.ships.insert(ship_rb_index, ship_ref.clone());
        Ok(ship_ref)
    }

    pub fn build_player_ship(&mut self, ctx: &mut Context, player_id: ID)
        -> tetra::Result<Rcc<Player>> {
        let ship = self.build_caravel(ctx, player_id.name.clone(), V2::zero(), 0.0)?;
        Ok(self.controller.add_player(Player::new(player_id, ship)))
    }

    pub fn build_island_object(&mut self, ctx: &mut Context, pos: V2, rot: f32)
        -> tetra::Result<Rcc<Object>> {
        Ok(self.add_object(Object::build_island(ctx, self.game.clone(), pos, rot)?))
    }

    pub fn build_ship_wreck_object(&mut self, ctx: &mut Context, pos: V2, rot: f32)
        -> tetra::Result<Rcc<Object>> {
        Ok(self.add_object(Object::build_ship_wreck(ctx, self.game.clone(), pos, rot)?))
    }

    pub fn get_ship(&self, index: Index) -> Rcc<Ship> {
        self.ships.get(&index).unwrap().clone()
    }

    fn add_object(&mut self, obj: Object) -> Rcc<Object> {
        let index = obj.get_index();
        let obj_ref = wrap_rcc(obj);
        self.objects.insert(index, obj_ref.clone());
        obj_ref
    }

    fn remove_object(&mut self, obj_index: Index) {
        self.objects.remove(&obj_index);
    }

    fn handle_intersections(&self) {
        let intersections = self.game.borrow().physics.get_intersections();
        for intersection in intersections.iter() {
            if !intersection.intersecting {
                continue
            }

            println!("{:?} and {:?} intersect!", intersection.collider1, intersection.collider2);
        }
    }

    fn handle_contacts(&self, ctx: &mut Context) -> tetra::Result {
        let contacts = self.game.borrow().physics.get_contacts();
        for contact in contacts.iter() {
            match contact {
                ContactEvent::Started(coll1, coll2) => {
                    // TODO: Rewrite using collider user-data to see if ship or object
                    let ship1 = self.ships.get(&coll1.0);
                    let ship2 = self.ships.get(&coll2.0);
                    if ship1.is_some() {
                        let ship1 = ship1.unwrap();
                        if ship2.is_some() {
                            let ship2 = ship2.unwrap();
                            ship1.borrow_mut()
                                .collision_with_ship(ctx, ship2.clone());
                            ship2.borrow_mut().collision_with_ship(ctx, ship1.clone());
                        }
                        else {
                            ship1.borrow_mut().collision_with_object(ctx);
                        }
                    }
                    else {
                        if ship2.is_some() {
                            let ship2 = ship2.unwrap();
                            ship2.borrow_mut().collision_with_object(ctx);
                        }
                    }
                },
                _ => ()
            }
        }
        Ok(())
    }

    fn on_ship_destroyed(&mut self, ctx: &mut Context, ship: Rcc<Ship>)
        -> tetra::Result<Rcc<Object>> {
        let mut ship_ref = ship.borrow_mut();
        let (pos, rot) = ship_ref.transform.get_translation();
        ship_ref.reset();
        std::mem::drop(ship_ref);

        self.build_ship_wreck_object(ctx, pos, rot)
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
        self.handle_intersections();
        self.handle_contacts(ctx)?;
        
        let mut destroyed_ships = Vec::new();
        for ship in self.ships.values() {
            let mut ship_ref = ship.borrow_mut();
            ship_ref.update(ctx, &self.ships, &self.objects)?;

            if ship_ref.is_destroyed() {
                std::mem::drop(ship_ref);
                destroyed_ships.push(ship.clone());
            }
        }
        for destroyed_ship in destroyed_ships.iter() {
            self.on_ship_destroyed(ctx, destroyed_ship.clone())?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.controller.draw(ctx)?;
        for obj in self.objects.values() {
            obj.borrow_mut().draw(ctx)?;
        }
        for ship in self.ships.values() {
            ship.borrow_mut().draw(ctx)?;
        }
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event) -> tetra::Result {
        self.controller.event(ctx, event.clone())?;
        for ship in self.ships.values() {
            ship.borrow_mut().event(ctx, event.clone(), &self.ships, &self.objects)?;
        }
        Ok(())
    }
}
