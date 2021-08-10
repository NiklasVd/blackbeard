use std::any::Any;

use rapier2d::{na::Vector2, prelude::{Ray}};
use tetra::{Context, State};
use crate::{AnimatedSprite, Entity, EntityType, GC, GameState, MASS_FORCE_SCALE, Rcc, Ship, Sprite, Timer, Transform, V2, build_water_splash_sprite, conv_vec, conv_vec_point, get_angle, polar_to_cartesian, world_scene::Entities};

pub const POWER_FORCE_FACTOR: f32 = 10.0 * MASS_FORCE_SCALE;
pub const POWER_DROP_THRESHOLD: f32 = 1.0;

pub enum CannonSide {
    Bowside,
    Portside,
    Nose,
    Stern
}

pub struct Cannon {
    pub pos: V2,
    pub rot: f32,
    pub facing_rot: f32,
    pub power: f32,
    pub dmg: u16,
    pub side: CannonSide,
    pub reload: Timer,
    shoot_effect: Option<AnimatedSprite>
}

impl Cannon {
    pub fn new(pos: V2, facing_rot: f32, power: f32, dmg: u16, side: CannonSide,
        reload_time: f32) -> Cannon {
        Cannon {
            pos, rot: get_angle(pos), facing_rot, power, dmg, side,
            reload: Timer::new(reload_time), shoot_effect: None
        }
    }

    pub fn shoot(&mut self, ship_translation: (V2, f32)) -> Ray {
        let local_translation = self.get_translation(ship_translation);
        let facing_dir = polar_to_cartesian(1.0, local_translation.1);
        self.reload.reset();
        Ray::new(conv_vec_point(local_translation.0), conv_vec(facing_dir))
    }

    pub fn get_reload_time(&self) -> f32 {
        return self.reload.max
    }

    pub fn get_translation(&self, ship_translation: (V2, f32)) -> (V2, f32) {
        (ship_translation.0 + polar_to_cartesian(self.pos.magnitude(),
            self.rot + ship_translation.1), self.facing_rot + ship_translation.1) 
    }
}

impl State for Cannon {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.reload.update(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        // Shoot effect
        Ok(())
    }
}

pub enum CannonBallStatus {
    Travelling,
    Hit,
    Miss
}

pub struct CannonBall {
    pub power: f32,
    pub dmg: u16,
    pub shooter: Rcc<Ship>,
    pub transform: Transform,
    pub status: CannonBallStatus,
    cannon_sprite: Sprite,
    water_splash_miss: Option<AnimatedSprite>,
    game: GC
}

impl CannonBall {
    pub fn shoot(ctx: &mut Context, ship: Rcc<Ship>, cannon: &Cannon, game: GC)
        -> tetra::Result<CannonBall> {
        todo!()
    }

    pub fn on_hit_ship(&mut self, ctx: &mut Context, ship: Rcc<Ship>,
        entities: &mut Entities) -> tetra::Result {
        self.status = CannonBallStatus::Hit;
        ship.borrow_mut().take_cannon_ball_hit(ctx, self.dmg, self.shooter.clone(), entities)
    }

    pub fn is_finished(&self) -> bool {
        match self.status {
            CannonBallStatus::Travelling => false,
            _ => true
        }
    }

    fn fly(&mut self, ctx: &mut Context) -> tetra::Result {
        let mut game_ref = self.game.borrow_mut();
        let rb = game_ref.physics.get_rb_mut(self.transform.handle.0);

        if rb.linvel().magnitude() <= POWER_DROP_THRESHOLD {
            self.status = CannonBallStatus::Miss;
            self.water_splash_miss = Some(build_water_splash_sprite(
                ctx, self.game.clone(), self.transform.get_translation().0)?);
            rb.set_linvel(Vector2::new(0.0, 0.0), true);
        }
        Ok(())
    }
}

impl Entity for CannonBall {
    fn get_type(&self) -> EntityType {
        EntityType::CannonBall
    }

    fn get_name(&self) -> String {
        format!("{} Cannon Ball", self.shooter.borrow().get_name())
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
}

impl GameState for CannonBall {
    fn update(&mut self, ctx: &mut Context, entities: &mut Entities) -> tetra::Result {
        match self.status {
            CannonBallStatus::Travelling => self.fly(ctx)?,
            _ => ()
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        match self.status {
            CannonBallStatus::Travelling => {
                self.cannon_sprite.draw2(ctx, self.transform.get_translation());
            },
            CannonBallStatus::Miss => {
                self.water_splash_miss.as_mut().unwrap()
                    .draw(ctx, self.transform.get_translation());
            },
            _ => ()
        }
        Ok(())
    }
}
