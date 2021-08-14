use std::any::Any;

use rapier2d::{data::Index, na::Vector2};
use tetra::{Context, State, graphics::text::Text};
use crate::{AnimatedSprite, Entity, EntityType, GC, GameState, MASS_FORCE_SCALE, Rcc, Ship, Sprite, SpriteOrigin, Timer, Transform, V2, build_water_splash_sprite, cast_entity, conv_vec, get_angle, get_decal_coll_group, get_empty_coll_group, polar_to_cartesian, world_scene::{Entities, WorldScene}};

pub const POWER_FORCE_FACTOR: f32 = 25.0 * MASS_FORCE_SCALE;
pub const POWER_DROP_THRESHOLD: f32 = 5.0 * POWER_FORCE_FACTOR / MASS_FORCE_SCALE;

#[derive(PartialEq)]
pub enum CannonSide {
    Bowside,
    Portside,
    Nose,
    Stern
}

pub struct Cannon {
    pub translation: (V2, f32),
    pub relative_rot: f32,
    pub dmg: u16,
    pub side: CannonSide,
    pub reload: Timer,
    ship_translation: (V2, f32),
    ship_index: Index,
    cannon_sprite: Sprite,
    shoot_effect: AnimatedSprite,
    reload_label: Text,
    shoot: bool,
    game: GC
}

impl Cannon {
    pub fn new(ctx: &mut Context, relative_pos: V2, relative_rot: f32, dmg: u16,
        side: CannonSide, reload_time: f32, ship_index: Index, game: GC) -> tetra::Result<Cannon> {
        let mut game_ref = game.borrow_mut();
        let cannon_tex = game_ref.assets.load_texture(ctx, "Cannon.png".to_owned(), true)?;
        let shoot_tex = game_ref.assets.load_texture(ctx, "Shoot Cannon.png".to_owned(), true)?;
        let reload_label = Text::new("*", game_ref.assets.font.clone());
        std::mem::drop(game_ref);

        let cannon_sprite = Sprite::new(cannon_tex, SpriteOrigin::Centre, None);
        let shoot_effect = AnimatedSprite::new(game.borrow_mut().assets.load_texture(
            ctx, "Shoot Cannon.png".to_owned(), true)?, 5, 15.0, 15.0, 0.2, false, None);
        
        Ok(Cannon {
            translation: (relative_pos, get_angle(relative_pos)), relative_rot, dmg, side,
            reload: Timer::new(reload_time), ship_translation: (V2::zero(), 0.0), ship_index,
            cannon_sprite, shoot_effect, reload_label, shoot: false, game
        })
    }

    pub fn shoot(&mut self, ctx: &mut Context, entities: &mut Entities)
        -> tetra::Result<Option<Rcc<CannonBall>>> {
        if !self.can_shoot() {
            println!("Cannon isn't ready yet. Time to reload: {:.1}",
                self.reload.time_until_over());
            return Ok(None);
        }

        let curr_translation = self.get_world_translation();
        let facing_dir = polar_to_cartesian(1.0, curr_translation.1);
        let cannon_ball = WorldScene::build_cannon_ball(CannonBall::new(ctx,
            self.dmg, self.ship_index, curr_translation.0 + facing_dir, facing_dir,
            self.game.clone())?, entities)?;
        // Shoot effect
        self.reload.reset();
        Ok(Some(cannon_ball))
    }

    pub fn can_shoot(&self) -> bool {
        self.reload.is_over()
    }

    pub fn get_reload_time(&self) -> f32 {
        return self.reload.max
    }

    pub fn set_ship_translation(&mut self, ship_translation: (V2, f32)) {
        self.ship_translation = ship_translation;
    }

    pub fn get_world_translation(&self) -> (V2, f32) {
        (self.ship_translation.0 + polar_to_cartesian(self.translation.0.magnitude(),
            self.translation.1 + self.ship_translation.1), self.relative_rot + self.ship_translation.1)
    }
}

impl State for Cannon {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.reload.update(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        let curr_translation = self.get_world_translation();
        self.cannon_sprite.draw2(ctx, curr_translation);
        if self.shoot && !self.shoot_effect.is_finished() {
            self.shoot_effect.draw(ctx, curr_translation);
        }
        if !self.can_shoot() {
            self.reload_label.draw(ctx, curr_translation.0);
        }
        Ok(())
    }
}

#[derive(PartialEq)]
pub enum CannonBallState {
    Travelling,
    Hit,
    Miss
}

pub struct CannonBall {
    pub dmg: u16,
    pub shooter_index: Index,
    pub state: CannonBallState,
    pub transform: Transform,
    sprite: Sprite,
    miss_effect: Option<AnimatedSprite>,
    game: GC
}

impl CannonBall {
    pub fn new(ctx: &mut Context, dmg: u16, shooter_index: Index,
        starting_pos: V2, dir: V2, game: GC) -> tetra::Result<CannonBall> {
        let physics_handle = game.borrow_mut().physics.build_cannon_ball();
        let mut transform = Transform::new(physics_handle, game.clone());
        transform.set_pos(starting_pos, 0.0);
        
        let mut game_ref = game.borrow_mut();
        let rb = game_ref.physics.get_rb_mut(physics_handle.0);
        rb.apply_impulse(conv_vec(dir * POWER_FORCE_FACTOR), true); // Or use impulse?
        
        let sprite = Sprite::new(game_ref.assets.load_texture(ctx,
            "Cannon Ball.png".to_owned(), true)?, SpriteOrigin::Centre, None);
        std::mem::drop(game_ref);

        Ok(CannonBall {
            dmg, shooter_index, transform, state: CannonBallState::Travelling,
            sprite, miss_effect: None, game
        })
    }

    fn check_trajectory(&mut self, ctx: &mut Context) -> tetra::Result {
        let game_ref = self.game.borrow();
        let rb = game_ref.physics.get_rb(self.transform.handle.0);
        if rb.linvel().magnitude() <= POWER_DROP_THRESHOLD
            && self.state == CannonBallState::Travelling {
            std::mem::drop(game_ref);
            self.on_drop(ctx)?;
        }
        Ok(())
    }

    fn on_hit_ship(&mut self, ctx: &mut Context, ship: &mut Ship, entities: &mut Entities)
        -> tetra::Result {
        self.state = CannonBallState::Hit;
        WorldScene::remove_entity(ctx, self.get_index(), self.transform.handle,
            entities, self.game.clone())?;
        ship.take_cannon_ball_hit(ctx, self.dmg, self.shooter_index, entities)       
    }

    fn miss(&mut self, ctx: &mut Context, miss_effect: AnimatedSprite) -> tetra::Result {
        self.state = CannonBallState::Miss;
        let mut game_ref = self.game.borrow_mut();
        let rb = game_ref.physics.get_rb_mut(
            self.transform.handle.0);
        rb.set_linvel(Vector2::new(0.0, 0.0), true);
        game_ref.physics.set_coll_group(self.transform.handle.1,
            get_decal_coll_group(), get_empty_coll_group());
        self.miss_effect = Some(miss_effect);
        Ok(())
    }

    fn on_drop(&mut self, ctx: &mut Context) -> tetra::Result {
        let water_splash_effect = build_water_splash_sprite(ctx, self.game.clone(),
            self.transform.get_translation().0)?;
        self.miss(ctx, water_splash_effect)
    }

    fn on_hit_object(&mut self, ctx: &mut Context) -> tetra::Result {
        let curr_pos = self.transform.get_translation().0;
        let water_splash_effect = build_water_splash_sprite(ctx, self.game.clone(),
            curr_pos)?; // Add different effect
        self.miss(ctx, water_splash_effect)
    }

    fn check_miss_lifetime(&mut self, ctx: &mut Context, entities: &mut Entities) -> tetra::Result {
        if let Some(miss_effect) = self.miss_effect.as_ref() {
            if miss_effect.is_finished() {
                WorldScene::remove_entity(ctx, self.get_index(), self.transform.handle,
                    entities, self.game.clone())?;
            }
        }
        Ok(())
    }
}

impl Entity for CannonBall {
    fn get_type(&self) -> EntityType {
        EntityType::CannonBall
    }

    fn get_name(&self) -> String {
        "Cannon Ball".to_owned()
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn collide_with_entity(&mut self, ctx: &mut Context, other: Rcc<dyn Entity>,
        entities: &mut Entities) -> tetra::Result {
        let mut entity_ref = other.borrow_mut();
        if entity_ref.get_index() == self.shooter_index { // Ignore if hitting own ship
            return Ok(())
        }

        let entity_type = entity_ref.get_type();
        println!("Cannon ball collided with {}.", entity_ref.get_name());
        match entity_type {
            EntityType::Ship => {
                self.on_hit_ship(ctx,
                    cast_entity(entity_ref.as_any_mut()), entities)
            },
            EntityType::Object => {
                self.on_hit_object(ctx)
            },
            EntityType::CannonBall => { // Wtf
                self.on_hit_object(ctx)
            }
        }
    }
}

impl GameState for CannonBall {
    fn update(&mut self, ctx: &mut Context, entities: &mut Entities) -> tetra::Result {
        self.check_trajectory(ctx)?;
        self.check_miss_lifetime(ctx, entities)
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        if let Some(miss_effect) = self.miss_effect.as_mut() {
            miss_effect.draw(ctx, self.transform.get_translation());
        }
        else {
            self.sprite.draw2(ctx, self.transform.get_translation());
        }
        Ok(())
    }
}
