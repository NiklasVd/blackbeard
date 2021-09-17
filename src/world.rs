use std::collections::HashMap;
use indexmap::IndexMap;
use rapier2d::{data::Index, prelude::ContactEvent};
use tetra::{Context, Event, State};
use crate::{CannonBall, GC, ID, Rcc, V2, entity::{Entity, EntityType}, harbour::Harbour, object::Object, ship::{Ship, ShipType}, wrap_rcc};

pub type EntityMap<T = dyn Entity + 'static> = HashMap<Index, Rcc<T>>;

pub struct World {
    entities: IndexMap<Index, Rcc<dyn Entity>>,
    sensors: EntityMap,
    ships: EntityMap<Ship>,
    game: GC
}

impl World {
    pub fn new(ctx: &mut Context, game: GC) -> World  {
        World {
            entities: IndexMap::new(), sensors: HashMap::new(),
            ships: HashMap::new(), game
        }
    }

    pub fn add_player_ship(&mut self, ctx: &mut Context, id: ID, ship_type: ShipType) -> tetra::Result<Rcc<Ship>> {
        self.add_ship(ctx, ship_type, id.name,
            V2::right() * id.n as f32 * 1000.0, // Initial spawn position
            true)
    }

    pub fn add_island(&mut self, ctx: &mut Context, pos: V2, rot: f32, island_type: u32)
        -> tetra::Result<Rcc<Object>> {
        let island = Object::build_island(ctx, self.game.clone(), pos, rot, island_type)?;
        Ok(self.add_entity(island).unwrap())
    }

    pub fn add_ship_wreck(&mut self, ctx: &mut Context, pos: V2, rot: f32
        /* Ship Type */) -> tetra::Result<Rcc<Object>> {
        let ship_wreck = Object::build_ship_wreck(ctx, self.game.clone(), pos, rot)?;
        Ok(self.add_entity(ship_wreck).unwrap())
    }

    pub fn add_harbour(&mut self, ctx: &mut Context, name: &str, pos: V2, rot: f32)
        -> tetra::Result<Rcc<Harbour>> {
        let harbour = Harbour::new(ctx, name.to_owned(), pos, rot, self.game.clone())?;
        let index = harbour.get_index();
        let zone_index = harbour.zone_handle.0;
        let harbour_ref = wrap_rcc(harbour);
        
        self.add_entity_unchecked(index, harbour_ref.clone());
        self.sensors.insert(zone_index, harbour_ref.clone());
        Ok(harbour_ref)
    }

    pub fn add_cannon_ball(&mut self, ctx: &mut Context, cannon_ball: CannonBall) -> Rcc<CannonBall> {
        let index = cannon_ball.get_index();
        let cannon_ball_ref = wrap_rcc(cannon_ball);
        self.add_entity_unchecked(index, cannon_ball_ref.clone());
        cannon_ball_ref
    }

    pub fn get_entity(&mut self, index: Index) -> Option<Rcc<dyn Entity>> {
        self.entities.get(&index).and_then(|entity| Some(entity.clone()))
    }

    pub fn get_entity_unchecked(&mut self, index: Index) -> Rcc<dyn Entity> {
        self.entities[&index].clone()
    }

    pub fn get_ship(&mut self, index: Index) -> Option<Rcc<Ship>> {
        self.ships.get(&index).and_then(|ship| Some(ship.clone()))
    }

    pub fn get_ship_unchecked(&mut self, index: Index) -> Rcc<Ship> {
        self.ships[&index].clone()
    }

    pub fn remove_entity(&mut self, index: Index) -> Option<Rcc<dyn Entity>> {
        if let Some(entity) = self.entities.remove(&index) {
            {
                let entity_ref = entity.borrow();
                self.game.borrow_mut().physics.remove_collider(entity_ref.get_transform().handle);
                match entity_ref.get_type() {
                    EntityType::Ship => { self.ships.remove(&index); },
                    _ => ()
                };
            }
            Some(entity)
        } else {
            None
        }
    }

    fn add_entity<T: Entity + 'static>(&mut self, entity: T) -> Option<Rcc<T>> {
        let index = entity.get_index();
        if self.entities.contains_key(&index) {
            None
        } else {
            let entity_ref = wrap_rcc(entity);
            self.add_entity_unchecked(index, entity_ref.clone());
            Some(entity_ref)
        }
    }

    fn add_entity_unchecked<T: Entity + 'static>(&mut self, index: Index, entity: Rcc<T>) {
        self.entities.insert(index, entity);
    }

    fn add_ship(&mut self, ctx: &mut Context, ship_type: ShipType, name: String, spawn: V2,
        respawn: bool) -> tetra::Result<Rcc<Ship>> {
        let ship = match ship_type {
            ShipType::Caravel => Ship::caravel(ctx, self.game.clone(),
                name, spawn, respawn),
            ShipType::Galleon => Ship::galleon(ctx, self.game.clone(),
                name, spawn, respawn),
            ShipType::Schooner => Ship::schooner(ctx, self.game.clone(),
                name, spawn, respawn)
        }?;
        let index = ship.get_index();
        let ship_ref = self.add_entity::<Ship>(ship).unwrap();
        self.ships.insert(index, ship_ref.clone());
        Ok(ship_ref)
    }

    fn handle_intersections(&mut self, ctx: &mut Context) -> tetra::Result {
        let intersections = self.game.borrow().physics.get_intersections();
        for intersection in intersections.iter() {
            let coll1_sensor = self.game.borrow().physics
                    .get_coll(intersection.collider1).is_sensor();
            
            let (entity1, entity2) = {
                if coll1_sensor { // Only one of the two colliders ever is a sensor
                    if let Some(entity1) = self.sensors.get(&intersection.collider1.0) {
                        let entity1 = entity1.clone();
                        if let Some(entity2) = self.get_entity(intersection.collider2.0) {
                            (entity1, entity2)
                        } else {
                            continue
                        }
                    } else {
                        continue
                    }
                } else { // If the first isn't, then the second collider will be the sensor
                    if let Some(entity2) = self.sensors.get(&intersection.collider2.0) {
                        let entity2 = entity2.clone();
                        if let Some(entity1) = self.get_entity(intersection.collider1.0) {
                            (entity1, entity2)
                        } else {
                            continue
                        }
                    } else {
                        continue
                    }
                }
            };

            entity1.borrow_mut().intersect_with_entity(ctx,
                intersection.intersecting, entity2.clone())?;
            entity2.borrow_mut().intersect_with_entity(ctx,
                intersection.intersecting, entity1)?;
        }
        Ok(())
    }

    fn handle_contacts(&mut self, ctx: &mut Context) -> tetra::Result {
        let contacts = self.game.borrow().physics.get_contacts();
        for contact in contacts.iter() {
            match contact {
                ContactEvent::Started(coll1_handle, coll2_handle) => {
                    let entity1 = self.get_entity(coll1_handle.0);
                    let entity2 = self.get_entity(coll2_handle.0);
                    if let Some(entity1) = entity1 {
                        if let Some(entity2) = entity2 {
                            self.handle_contact_with(ctx, entity1.clone(), entity2.clone())?;
                            self.handle_contact_with(ctx, entity2, entity1)?;
                        }
                        else {
                            entity1.borrow_mut().collide_with_neutral(ctx)?;
                        }
                    }
                    else {
                        if let Some(entity2) = entity2 {
                            entity2.borrow_mut().collide_with_neutral(ctx)?;
                        }
                    }
                },
                _ => ()
            }
        }
        Ok(())
    }

    fn handle_contact_with(&mut self, ctx: &mut Context, a: Rcc<dyn Entity>, b: Rcc<dyn Entity>)
        -> tetra::Result {
        let b_ship = {
            let b_ref = b.borrow();
            match b_ref.get_type() {
                EntityType::Ship => Some(self.get_ship_unchecked(b_ref.get_index())),
                _ => None
            }
        };
        if let Some(b_ship) = b_ship { // b is a ship
            a.borrow_mut().collide_with_ship(ctx, b_ship, self)
        } else {
            a.borrow_mut().collide_with_entity(ctx, b, self)
        }
    }
}

impl State for World {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.handle_intersections(ctx)?;
        self.handle_contacts(ctx)?;

        let entities = &mut self.entities.clone(); // Performance?
        for entity in entities.values() {
            let mut entity_ref = entity.borrow_mut();
            entity_ref.update(ctx, self)?;
            if entity_ref.marked_destroy() {
                let index = entity_ref.get_index();
                std::mem::drop(entity_ref);
                self.remove_entity(index);
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        for entity in self.entities.values() {
            entity.borrow_mut().draw(ctx)?;
        }
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event) -> tetra::Result {
        let entities = &mut self.entities.clone(); // Performance?
        for entity in entities.values() {
            entity.borrow_mut().event(ctx, event.clone(), self)?;
        }
        Ok(())
    }
}
