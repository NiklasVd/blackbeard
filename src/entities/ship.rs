use std::{f32::consts::PI};
use rapier2d::{data::Index};
use tetra::{Context, State, graphics::{Color}};
use crate::{Cannon, CannonSide, GC, MASS_FORCE_SCALE, Rcc, Sprite, SpriteOrigin, StateEvent, Timer, Transform, V2, WorldEvent, conv_vec, disassemble_iso, economy::{Deposit}, entity::{Entity, EntityType, GameState}, get_angle, health_bar::HealthBar, log_state_event, pi_to_pi2_range, polar_to_cartesian, ship_data::{DamageResult, ShipAttributes, ShipData, ShipID, ShipType}, ship_mod::{ShipMod, ShipModType}, ship_status::ShipStatus, vec_distance, world::World};

pub const BASE_STUN_LENGTH: f32 = 0.5;
pub const MAX_SHIP_DEFENSE: u16 = 100;

const BASE_OBJECT_COLLISION_DAMAGE: u16 = 20;
const BASE_MOVEMENT_FORCE: f32 = 10.0 * MASS_FORCE_SCALE;
const BASE_TORQUE_FORCE: f32 = 1000.0 * MASS_FORCE_SCALE;
const TARGET_POS_DIST_MARGIN: f32 = 75.0;
const TARGET_ROT_MARGIN: f32 = PI / 42.0;

const ESCUDO_RAM_STEAL_PERCENTAGE: f32 = 0.15;
const ESCUDO_SHOOT_STEAL_PERCENTAGE: f32 = 0.1;
const ESCUDO_ACCIDENT_LOSS_PERCENTAGE: f32 = 0.1;

pub struct Ship {
    pub data: ShipData,
    pub status: ShipStatus,
    pub cannons: Vec<Cannon>,
    pub transform: Transform,
    pub treasury: Deposit,
    pub mods: Vec<Box<dyn ShipMod>>,
    sprite: Sprite,
    health_bar: HealthBar,
}

impl Ship {
    pub fn caravel(ctx: &mut Context, game: GC, controller: ShipID, spawn: V2,
        respawn: bool) -> tetra::Result<Ship> {
        Self::new(ctx, ShipType::Caravel, controller, "Caravel.png",
            ShipAttributes::caravel(), spawn, respawn, 4, 1.0, V2::new(10.0, 0.0), 1.0, game)
    }

    pub fn galleon(ctx: &mut Context, game: GC, controller: ShipID, spawn: V2,
        respawn: bool) -> tetra::Result<Ship> {
        Self::new(ctx, ShipType::Galleon, controller, "Galleon.png",
        ShipAttributes::galleon(), spawn, respawn, 5, 1.2, V2::new(15.0, 0.0), 1.0, game)
    }

    pub fn schooner(ctx: &mut Context, game: GC, controller: ShipID, spawn: V2,
        respawn: bool) -> tetra::Result<Ship> {
        Self::new(ctx, ShipType::Schooner, controller, "Schooner.png",
        ShipAttributes::schooner(), spawn, respawn, 3, 0.9, V2::new(0.0, 15.0), 1.25, game)
    }

    fn new(ctx: &mut Context, ship_type: ShipType, controller: ShipID,
        ship_texture: &str, attr: ShipAttributes, spawn_pos: V2,
        respawn: bool, cannons_per_side: u32, cannon_power: f32, cannon_pos: V2,
        mass: f32, game: GC) -> tetra::Result<Ship> {
        let mut game_ref = game.borrow_mut();
        let sprite = Sprite::new(game_ref.assets.load_texture(
            ctx, ship_texture.to_owned(), true)?, SpriteOrigin::Centre, None);
        let handle = game_ref.physics.build_ship_collider(
            sprite.texture.width() as f32 * 0.5,
            sprite.texture.height() as f32 * 0.5, mass, ship_type);
        let stun_length = attr.get_stun_length();
        game_ref.economy.add_deposit();
        std::mem::drop(game_ref);

        let mut transform = Transform::new(handle, game.clone());
        transform.set_pos(spawn_pos, 0.0);
        let spawn_pos = match respawn {
            true => Some(spawn_pos),
            false => None
        };

        let index = transform.get_index();
        let mut cannons = Vec::new();
        let mut bow_pos = V2::new(48.0, -50.0) + cannon_pos;
        let bow_rot = PI * 1.5;
        for _ in 0..cannons_per_side {
            cannons.push(Cannon::new(ctx, bow_pos, bow_rot, attr.cannon_damage,
                CannonSide::Bowside, attr.cannon_reload_time, cannon_power, index,
                game.clone())?);
            bow_pos -= V2::new(45.0, 0.0);
        }
        let mut port_pos = V2::new(48.0, 50.0) + V2::new(cannon_pos.x, -cannon_pos.y);
        let port_rot = PI / 2.0;
        for _ in 0..cannons_per_side {
            cannons.push(Cannon::new(ctx, port_pos, port_rot, attr.cannon_damage,
                CannonSide::Portside, attr.cannon_reload_time, cannon_power, index,
                game.clone())?);
            port_pos -= V2::new(45.0, 0.0);
        }

        Ok(Ship {
            data: ShipData {
                curr_health: attr.health, ship_type, id: controller.clone(),
                attr, spawn_pos, destroy: false, game: game.clone()
            },
            status: ShipStatus {
                target_pos: None, rotate_only: false,
                is_in_harbour: false, stun: Timer::new(stun_length)
            }, cannons,
            transform, treasury: Deposit::default(), mods: Vec::new(), sprite,
            health_bar: HealthBar::new(ctx, controller.to_string(), Color::WHITE /* Customise for local player? */,
            attr.health, game.clone())?
        })
    }

    pub fn apply_mod<T: ShipMod + 'static>(&mut self, ship_mod: T) {
        // TODO: Accept &mut Ship param to allow application of mod inside Ship method?
        // Cannot apply mod inside ship method as this will lead to a borrow conflict
        self.mods.push(Box::new(ship_mod));
    }

    pub fn remove_mod(&mut self, ship_mod_type: ShipModType)
        -> Option<Box<dyn ShipMod + 'static>> {
        for (i, ship_mod) in self.mods.iter_mut().enumerate() {
            if ship_mod.get_type() == ship_mod_type {
                // Called after ship borrow is dropped in upper level call:
                // ship_mod.on_remove()?;
                return Some(self.mods.remove(i))
            }
        }
        None
    }

    pub fn set_health(&mut self, val: u16) {
        self.data.set_health(val);
        self.health_bar.set_info(val);
    }

    pub fn take_damage(&mut self, ctx: &mut Context, mut damage: u16, world: &mut World)
        -> tetra::Result<DamageResult> {
        if damage <= 0 {
            return Ok(DamageResult::Empty)
        }
        
        if self.data.curr_health < damage {
            damage = self.data.curr_health;
        }
        let remaining_health = self.data.curr_health - damage;
        self.set_health(remaining_health);

        match self.data.is_sunk() {
            true => self.sink(ctx, world).and(Ok(DamageResult::Sink)),
            false => Ok(DamageResult::Hit(damage))
        }
    }

    pub fn sink(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result {
        println!("{} has been sunk!", self.get_name());
        let (pos, rot) = self.transform.get_translation();
        world.add_ship_wreck(ctx, pos, rot)?;

        if let Some(spawn) = self.data.spawn_pos { // Respawn
            self.reset();
            let free_spawn = {
                let mut game_ref = self.data.game.borrow_mut();
                let free_spawn = game_ref.physics.check_for_space(
                    spawn, self.sprite.get_size() * 1.5, V2::down() /* = Up in Tetra space */);
                if self.data.id.is_local_player() {
                    game_ref.cam.centre_on(free_spawn.clone());
                }
                free_spawn
            };
            self.transform.set_pos(free_spawn.clone(), 0.0);
        }
        else {
            self.set_health(0);
            self.data.destroy = true;
        }
        
        Ok(())
    }

    pub fn repair(&mut self) {
        self.set_health(self.data.attr.health);
    }

    pub fn reset(&mut self) {
        self.transform.reset_velocity();
        self.status.reset_target_pos();
        self.status.reset_stun();
        self.repair();
    }

    pub fn take_cannon_ball_hit(&mut self, ctx: &mut Context, dmg: u16,
        shooter_index: Index, world: &mut World) -> tetra::Result {
        let shooter = world.get_ship(shooter_index).unwrap(); // Bold unwrap but what else...
        let mut shooter_ref = shooter.borrow_mut();
        log_state_event(self.data.game.clone(), StateEvent::ShipCannonBallCollision(
                shooter_ref.data.id.clone(), self.data.id.clone(), dmg));

        match self.take_damage(ctx, dmg, world)? {
            DamageResult::Sink => {
                let forfeited_escudos = (self.treasury.balance as f32 * ESCUDO_SHOOT_STEAL_PERCENTAGE) as u32;
                let generated_payout =  self.data.game.borrow_mut()
                    .economy.total_payout(self.treasury.networth);
                shooter_ref.treasury.add(forfeited_escudos + generated_payout);
                self.treasury.lose(forfeited_escudos);
                self.data.game.borrow_mut().world.add_event(
                    WorldEvent::PlayerSunkByCannon(shooter_ref.get_name(), self.get_name()));
                Ok(())
            },
            _ => Ok(())
        }
    }

    pub fn shoot_cannons(&mut self, ctx: &mut Context, side: Option<CannonSide>,
        world: &mut World)
        -> tetra::Result {
        let cannons: Vec<_> = match side {
            Some(side) => self.cannons.iter_mut().filter(|c| c.side == side).collect(),
            None => self.cannons.iter_mut().collect()
        };
        for cannon in cannons {
            if let Some(cannonball) = cannon.shoot(ctx, world)? {
                let cannonball_ref = cannonball.borrow();
                log_state_event(self.data.game.clone(), StateEvent::ShipShootCannon(
                    self.data.id.clone(), cannonball_ref.transform.get_translation().0, cannonball_ref.dmg));
            }
        }
        Ok(())
    }

    fn move_to_target_pos(&mut self) {
        if self.status.is_stunned() {
            return;
        }
        
        if let Some(target_pos) = self.status.target_pos {
            let mut game_ref = self.data.game.borrow_mut();
            let rb = game_ref.physics.get_rb_mut(self.transform.handle.0);
            let (pos, rot) = disassemble_iso(rb.position());
            if vec_distance(pos, target_pos) <= TARGET_POS_DIST_MARGIN {
                self.status.target_pos = None;
                return;
            }

            if !self.status.rotate_only {
                let mut facing_dir = polar_to_cartesian(1.0, rot);
                facing_dir *= BASE_MOVEMENT_FORCE * self.data.attr.movement_speed;
                rb.apply_impulse(conv_vec(facing_dir), true);
            }

            let target_dir = target_pos - pos;
            let target_rot = pi_to_pi2_range(get_angle(target_dir));
            let delta_rot = target_rot - pi_to_pi2_range(rot);
            let mut applied_torque = self.data.attr.turn_rate * BASE_TORQUE_FORCE;
            if delta_rot < PI && delta_rot > 0.0 || delta_rot < -PI { // Clockwise rotation
            }
            else { // Counter-clockwise rotation
                applied_torque *= -1.0;
            }

            if delta_rot.abs() >= TARGET_ROT_MARGIN {
                rb.apply_torque_impulse(applied_torque, true);
            }
        }
    }
}

impl Entity for Ship {
    fn get_type(&self) -> EntityType {
        EntityType::Ship
    }

    fn get_name(&self) -> String {
        self.data.id.to_string()
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn marked_destroy(&self) -> bool {
        self.data.destroy
    }

    fn destroy(&mut self) {
        self.data.destroy = true;
    }

    fn collide_with_ship(&mut self, ctx: &mut Context, other: Rcc<Ship>, world: &mut World) -> tetra::Result {
        // ---
        // TODO: Rewrite logic to apply ram effects to oneself instead of opponent
        // ---
        let mut other_ref = other.borrow_mut();
        println!("{} collided with {} and dealt {} ram damage!",
            self.get_name(), other_ref.get_name(), self.data.attr.ram_damage);
        log_state_event(self.data.game.clone(), StateEvent::ShipShipCollision(
            self.data.id.clone(), other_ref.data.id.clone(), self.data.attr.ram_damage));
        
        other_ref.status.stun();
        match other_ref.take_damage(ctx, self.data.attr.ram_damage, world)? {
            DamageResult::Sink => {
                let forfeited_escudos = (other_ref.treasury.balance as f32 *
                    ESCUDO_RAM_STEAL_PERCENTAGE) as u32;
                let generated_payout = self.data.game.borrow_mut()
                    .economy.total_payout(other_ref.treasury.networth);
                println!("{} loses {}c. {} earns lost coins + {}c.",
                    other_ref.get_name(), forfeited_escudos,
                    self.get_name(), generated_payout);
                
                other_ref.treasury.lose(forfeited_escudos);
                self.treasury.add(forfeited_escudos + generated_payout);
                self.data.game.borrow_mut().world.add_event(
                    WorldEvent::PlayerSunkByRamming(self.get_name(), other_ref.get_name()));
                Ok(())
            },
            _ => Ok(())
        }
    }

    fn collide_with_entity(&mut self, ctx: &mut Context, other: Rcc<dyn Entity>, world: &mut World)
        -> tetra::Result {
        let other_ref = other.borrow();
        let other_entity_type = other_ref.get_type();
        if self.status.is_stunned() ||
            other_entity_type == EntityType::CannonBall /* Cannon ball does the damage part */ {
            return Ok(())
        }

        // TODO: Rework defence system
        let absorption = BASE_OBJECT_COLLISION_DAMAGE *
            (self.data.attr.defense / MAX_SHIP_DEFENSE);
        let damage = BASE_OBJECT_COLLISION_DAMAGE - absorption;
        if damage == 0 {
            return Ok(())
        }
        println!("{} collided with object {}!", self.get_name(), other_ref.get_name());
        log_state_event(self.data.game.clone(), StateEvent::ShipEntityCollision(
            self.data.id.clone(), other_entity_type, damage));

        self.status.stun();
        match self.take_damage(ctx, damage, world)? {
            DamageResult::Sink => {
                let forfeited_escudos = (self.treasury.balance as f32 * ESCUDO_ACCIDENT_LOSS_PERCENTAGE) as u32;
                self.data.game.borrow_mut().economy.remove(forfeited_escudos); // Lost to the sea...
                self.treasury.lose(forfeited_escudos);
                // println!("{} lost {} escudos after sinking their ship in an accident!",
                //     self.get_name(), forfeited_escudos);
                self.data.game.borrow_mut().world.add_event(
                    WorldEvent::PlayerSunkByAccident(self.get_name()));
                Ok(())
            },
            _ => Ok(())
        }
    }

    fn collide_with_neutral(&mut self, _: &mut Context)
        -> tetra::Result {
        Ok(())
    }

    fn intersect_with_entity(&mut self, _: &mut Context, state: bool,
        other: Rcc<dyn Entity>) -> tetra::Result {
        if other.borrow().get_type() == EntityType::Harbour {
            self.status.is_in_harbour = state;
        }
        Ok(())
    }
}

impl GameState for Ship {
    fn update(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result {
        self.status.stun.update(ctx);
        let translation = self.transform.get_translation();
        for cannon in self.cannons.iter_mut() {
            cannon.set_ship_translation(translation);
            cannon.update(ctx)?;
        }
        for ship_mod in self.mods.iter_mut() {
            ship_mod.update(ctx, world)?;
        }
        self.move_to_target_pos();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        let translation = self.transform.get_translation();
        self.sprite.draw2(ctx, translation);
        for cannon in self.cannons.iter_mut() {
            cannon.draw(ctx)?;
        }
        self.health_bar.draw(ctx, translation.0);
        Ok(())
    }
}
