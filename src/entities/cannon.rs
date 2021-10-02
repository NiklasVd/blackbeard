use rapier2d::{data::Index, na::Vector2};
use tetra::{Context, State, graphics::text::Text};
use crate::{AnimatedSprite, CANNON_BALL_COLL_GROUP, EMPTY_COLL_GROUP, GC, MASS_FORCE_SCALE, Rcc, Sprite, SpriteOrigin, Timer, Transform, V2, build_water_splash_sprite, conv_vec, entity::{Entity, EntityType, GameState}, get_angle, polar_to_cartesian, ship::Ship, ship_mod::Attribute, world::World};

pub const POWER_FORCE_FACTOR: f32 = 40.0 * MASS_FORCE_SCALE;
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
    pub dmg: Attribute<u16>,
    pub side: CannonSide,
    pub reload: Timer,
    pub reload_time: Attribute<f32>,
    pub shooting_power: Attribute<f32>, // in %
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
        side: CannonSide, reload_time: f32, shooting_power: f32, ship_index: Index, game: GC) -> tetra::Result<Cannon> {
        let mut game_ref = game.borrow_mut();
        let cannon_tex = game_ref.assets.load_texture(ctx, "Cannon.png".to_owned(), true)?;
        let shoot_tex = game_ref.assets.load_texture(ctx, "Shoot Cannon.png".to_owned(), true)?;
        let reload_label = Text::new("*", game_ref.assets.font.clone());
        std::mem::drop(game_ref);

        let cannon_sprite = Sprite::new(cannon_tex, SpriteOrigin::Centre, None);
        let shoot_effect = AnimatedSprite::new(game.borrow_mut().assets.load_texture(
            ctx, "Shoot Cannon.png".to_owned(), true)?, 5, 15.0, 15.0, 0.2, false, None);
        
        Ok(Cannon {
            translation: (relative_pos, get_angle(relative_pos)), relative_rot,
            dmg: Attribute::setup(dmg), side,
            reload: Timer::new(reload_time), reload_time: Attribute::setup(reload_time),
            shooting_power: Attribute::setup(shooting_power),
            ship_translation: (V2::zero(), 0.0), ship_index,
            cannon_sprite, shoot_effect, reload_label, shoot: false, game
        })
    }

    pub fn shoot(&mut self, ctx: &mut Context, world: &mut World)
        -> tetra::Result<Option<Rcc<CannonBall>>> {
        if !self.can_shoot() {
            return Ok(None);
        }

        let curr_translation = self.get_world_translation();
        let facing_dir = polar_to_cartesian(1.0, curr_translation.1);
        let starting_pos = curr_translation.0 + facing_dir;
        let cannon_ball = CannonBall::new(ctx, self.dmg.total(), self.shooting_power.total(),
            self.ship_index, starting_pos, facing_dir, self.game.clone())?;
        let cannon_ball = world.add_cannon_ball(ctx, cannon_ball);

        // Shoot effect
        self.reload.reset();
        Ok(Some(cannon_ball))
    }

    pub fn can_shoot(&self) -> bool {
        self.reload.is_over()
    }

    pub fn change_reload_time(&mut self, val: f32) {
        self.reload_time.add(val);
        self.reload.max = self.reload_time.total();
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
    destroy: bool,
    game: GC
}

impl CannonBall {
    pub fn new(ctx: &mut Context, dmg: u16, shooting_power: f32, shooter_index: Index,
        starting_pos: V2, dir: V2, game: GC) -> tetra::Result<CannonBall> {
        let mut game_ref = game.borrow_mut();
        let sprite = Sprite::new(game_ref.assets.load_texture(ctx,
            "Cannon Ball.png".to_owned(), true)?, SpriteOrigin::Centre, None);
        
        let physics_handle = game_ref.physics.build_cannon_ball(
            sprite.texture.width() as f32, 0.1);
        std::mem::drop(game_ref);

        let mut transform = Transform::new(physics_handle, game.clone());
        transform.set_pos(starting_pos, 0.0);
        {
            let mut game_ref = game.borrow_mut();
            let rb = game_ref.physics.get_rb_mut(physics_handle.0);
            rb.apply_impulse(conv_vec(dir * POWER_FORCE_FACTOR * shooting_power), true);
        };

        Ok(CannonBall {
            dmg, shooter_index, transform, state: CannonBallState::Travelling,
            sprite, miss_effect: None, destroy: false, game
        })
    }

    fn check_trajectory(&mut self, ctx: &mut Context) -> tetra::Result {
        if self.transform.get_lin_velocity().magnitude() <= POWER_DROP_THRESHOLD
            && self.state == CannonBallState::Travelling {
            self.on_drop(ctx)
        } else {
            Ok(())
        }
    }

    fn on_hit_ship(&mut self, ctx: &mut Context, ship: Rcc<Ship>, world: &mut World)
        -> tetra::Result {
        // Issue: When two cannon balls hit a ship in immediate succession, and the first
        // sinks the ship, the second still applies damage right afterward. This results
        // in the ship spawning with less than 100% health, which is suboptimal.
        // Even potential desync issues?
        self.state = CannonBallState::Hit;
        self.destroy = true;
        ship.borrow_mut().take_cannon_ball_hit(ctx, self.dmg, self.shooter_index, world)       
    }

    fn miss(&mut self, ctx: &mut Context, miss_effect: AnimatedSprite) -> tetra::Result {
        self.state = CannonBallState::Miss;
        let mut game_ref = self.game.borrow_mut();
        let rb = game_ref.physics.get_rb_mut(
            self.transform.handle.0);
        rb.set_linvel(Vector2::new(0.0, 0.0), true);
        game_ref.physics.set_coll_group(self.transform.handle.1,
            CANNON_BALL_COLL_GROUP, EMPTY_COLL_GROUP);
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

    fn check_miss_lifetime(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result {
        if let Some(miss_effect) = self.miss_effect.as_ref() {
            if miss_effect.is_finished() {
                self.destroy();
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

    fn marked_destroy(&self) -> bool {
        self.destroy
    }

    fn destroy(&mut self) {
        self.destroy = true;
    }

    fn collide_with_ship(&mut self, ctx: &mut Context, other: Rcc<Ship>,
        world: &mut World) -> tetra::Result {
        let entity_ref = other.borrow_mut();
        if entity_ref.get_index() == self.shooter_index { // Ignore if hitting own ship
            Ok(())
        } else {
            std::mem::drop(entity_ref);
            self.on_hit_ship(ctx, other, world)
        }
    }

    fn collide_with_entity(&mut self, ctx: &mut Context, other: Rcc<dyn Entity>,
        world: &mut World) -> tetra::Result {
        self.on_hit_object(ctx)
    }
}

impl GameState for CannonBall {
    fn update(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result {
        self.check_trajectory(ctx)?;
        self.check_miss_lifetime(ctx, world)
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
