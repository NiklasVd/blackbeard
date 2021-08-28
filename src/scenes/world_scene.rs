use std::{collections::{HashMap}};
use rapier2d::{data::Index, prelude::ContactEvent};
use tetra::{Context, Event, State};
use crate::{BbResult, CannonBall, Controller, TransformResult, Entity, GC, GameState, ID, Object, PhysicsHandle, Player, Rcc, Ship, V2, grid::{Grid, UIAlignment}, wrap_rcc};
use super::scenes::{Scene, SceneType};

pub type Entities = HashMap<Index, Rcc<dyn Entity>>;

pub struct WorldScene {
    pub controller: Controller,
    entities: Entities,
    grid: Grid,
    game: GC
}

impl WorldScene {
    pub fn new(ctx: &mut Context, game: GC) -> BbResult<WorldScene> {
        let grid = Grid::new(ctx, UIAlignment::Vertical,
            V2::zero(), V2::one(), 0.0).convert()?;
        let mut world_scene = WorldScene {
            controller: Controller::new(ctx, game.clone()).convert()?,
            entities: HashMap::new(),
            grid, game
        };
        
        let local_player = world_scene.build_player_ship(ctx,
            ID::new("Niklas".to_owned(), 0), V2::zero()).convert()?;
        world_scene.controller.set_local_player(local_player);
        let test_ship = world_scene.build_caravel(ctx, "Antonia".to_owned(),
            V2::new(100.0, 300.0), false).convert()?;
        world_scene.build_island_object(ctx, V2::new(1000.0, 800.0), 0.12).convert()?;

        Ok(world_scene)
    }

    pub fn build_caravel(&mut self, ctx: &mut Context, name: String, spawn: V2,
        respawn: bool) -> tetra::Result<Rcc<Ship>> {
        let ship_ref = self.add_new_entity(Ship::caravel(
            ctx, self.game.clone(), name, spawn, respawn)?);
        Ok(ship_ref)
    }

    pub fn build_player_ship(&mut self, ctx: &mut Context, player_id: ID,
        spawn: V2) -> tetra::Result<Rcc<Player>> {
        let ship = self.build_caravel(ctx, player_id.name.clone(), spawn, true)?;
        Ok(self.controller.add_player(Player::new(player_id, ship)))
    }

    pub fn build_island_object(&mut self, ctx: &mut Context, pos: V2, rot: f32)
        -> tetra::Result<Rcc<Object>> {
        Ok(self.add_new_entity(Object::build_island(ctx, self.game.clone(), pos, rot)?))
    }

    pub fn build_ship_wreck_object(ctx: &mut Context, pos: V2, rot: f32, game: GC,
        entities: &mut HashMap<Index, Rcc<dyn Entity>>) -> tetra::Result<Rcc<Object>> {
        let obj = Object::build_ship_wreck(ctx, game, pos, rot)?;
        let index = obj.get_index();
        let obj_ref = wrap_rcc(obj);
        entities.insert(index, obj_ref.clone());
        Ok(obj_ref)
    }

    pub fn build_cannon_ball(cannon_ball: CannonBall,
        entities: &mut HashMap<Index, Rcc<dyn Entity>>) -> tetra::Result<Rcc<CannonBall>> {
        let index = cannon_ball.get_index();
        let cannon_ball_ref = wrap_rcc(cannon_ball);
        entities.insert(index, cannon_ball_ref.clone());
        Ok(cannon_ball_ref)
    }

    pub fn remove_entity(ctx: &mut Context, index: Index, handle: PhysicsHandle,
        entities: &mut Entities, game: GC) -> tetra::Result {
        /* Unsafe to return Rcc<dyn Entity>,
        as the RefCell is already borrowed during this call (in world_scene::update()).
        Hence, accessing entity using borrow(_mut) is impossible.
        This renders an Entity::on:destroy() unfeasible too. */
        let entity = entities.remove(&index);
        if let Some(entity) = entity {
            game.borrow_mut().physics.remove_collider(handle);
            return Ok(())
        }
        Ok(())
    }

    fn add_new_entity<T: Entity + 'static>(&mut self, entity: T) -> Rcc<T> {
        let index = entity.get_index();
        let entity_ref = wrap_rcc(entity);
        self.entities.insert(index, entity_ref.clone());
        entity_ref
    }

    fn get_entity(&self, index: Index) -> Option<Rcc<dyn Entity>> {
        self.entities.get(&index).and_then(|rcc| Some(rcc.clone()))
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

    fn handle_contacts(&mut self, ctx: &mut Context) -> tetra::Result {
        let contacts = self.game.borrow().physics.get_contacts();
        for contact in contacts.iter() {
            match contact {
                ContactEvent::Started(coll1_handle, coll2_handle) => {
                    let entity1 = self.get_entity(coll1_handle.0);
                    let entity2 = self.get_entity(coll2_handle.0);
                    if let Some(entity1) = entity1 {
                        if let Some(entity2) = entity2 {
                            entity1.borrow_mut().collide_with_entity(ctx,
                                entity2.clone(), &mut self.entities)?;
                            entity2.borrow_mut().collide_with_entity(ctx,
                                entity1, &mut self.entities)?;
                        }
                        else {
                            entity1.borrow_mut().collide_with_neutral(ctx,
                                &mut self.entities)?;
                        }
                    }
                    else {
                        if let Some(entity2) = entity2 {
                            entity2.borrow_mut().collide_with_neutral(ctx,
                                &mut self.entities)?;
                        }
                    }
                },
                _ => ()
            }
        }
        Ok(())
    }

    fn get_entities(&self) -> Vec<Rcc<dyn Entity>> {
        self.entities.values()
            .map(|e| e.clone()).collect()
    }
}

impl Scene for WorldScene {
    fn get_grid(&self) -> &Grid {
        &self.grid
    }

    fn get_grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    fn poll(&self, ctx: &mut Context) -> BbResult<Option<Box<dyn Scene>>> {
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
        
        for entity in self.get_entities() {
            entity.borrow_mut().update(ctx, &mut self.entities)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.controller.draw(ctx)?;
        for entity in self.entities.values() {
            entity.borrow_mut().draw(ctx)?;
        }
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event)
        -> tetra::Result {
        self.controller.event(ctx, event.clone(), &mut self.entities)?;
        for entity in self.get_entities() {
            entity.borrow_mut().event(ctx, event.clone(), &mut self.entities)?;
        }
        Ok(())
    }
}
