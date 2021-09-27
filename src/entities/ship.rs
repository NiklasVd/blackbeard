use std::{f32::consts::PI};
use binary_stream::{BinaryStream, Serializable};
use rapier2d::{data::Index};
use tetra::{Context, State, graphics::{Color}, math::Clamp};
use crate::{BbResult, Cannon, CannonSide, GC, MASS_FORCE_SCALE, Rcc, Sprite, SpriteOrigin, Timer, Transform, V2, WorldEvent, conv_vec, disassemble_iso, economy::{Deposit}, entity::{Entity, EntityType, GameState}, get_angle, health_bar::HealthBar, pi_to_pi2_range, polar_to_cartesian, ship_mod::{ShipMod, ShipModType}, vec_distance, world::World};

const BASE_STUN_LENGTH: f32 = 0.5;
const BASE_OBJECT_COLLISION_DAMAGE: u16 = 20;
const MAX_SHIP_DEFENSE: u16 = 100;
const BASE_MOVEMENT_FORCE: f32 = 10.0 * MASS_FORCE_SCALE;
const BASE_TORQUE_FORCE: f32 = 1000.0 * MASS_FORCE_SCALE;
const TARGET_POS_DIST_MARGIN: f32 = 75.0;
const TARGET_ROT_MARGIN: f32 = PI / 45.0;

const ESCUTO_RAM_STEAL_PERCENTAGE: f32 = 0.15;
const ESCUTO_SHOOT_STEAL_PERCENTAGE: f32 = 0.1;
const ESCUTO_ACCIDENT_LOSS_PERCENTAGE: f32 = 0.1;

#[derive(Debug, Clone, Copy)]
pub enum ShipType {
    Caravel,
    Schooner,
    Galleon
}

impl Serializable for ShipType {
    fn to_stream(&self, stream: &mut BinaryStream) {
        stream.write_buffer_single(match self {
            ShipType::Caravel => 0,
            ShipType::Schooner => 1,
            ShipType::Galleon => 2
        }).unwrap();
    }

    fn from_stream(stream: &mut BinaryStream) -> Self {
        match stream.read_buffer_single().unwrap() {
            0 => ShipType::Caravel,
            1 => ShipType::Galleon,
            2 => ShipType::Schooner,
            n @ _ => panic!("Index {} is not assigned to any ship type", n)
        }
    }
}

#[derive(Clone, Copy)]
pub struct ShipAttributes {
    pub health: u16,
    pub defense: u16, // 1-100
    pub movement_speed: f32,
    pub turn_rate: f32,
    pub cannon_damage: u16,
    pub cannon_reload_time: f32,
    pub ram_damage: u16
}

impl ShipAttributes {
    pub fn caravel() -> ShipAttributes {
        ShipAttributes {
            health: 140,
            defense: 60,
            movement_speed: 19.5, turn_rate: 5.25,
            cannon_damage: 15, cannon_reload_time: 5.0,
            ram_damage: 20
        }
    }

    pub fn galleon() -> ShipAttributes {
        ShipAttributes {
            health: 160,
            defense: 80,
            movement_speed: 18.0, turn_rate: 4.85,
            cannon_damage: 15, cannon_reload_time: 5.0,
            ram_damage: 30
        }
    }

    pub fn schooner() -> ShipAttributes {
        ShipAttributes {
            health: 120,
            defense: 35,
            movement_speed: 18.5, turn_rate: 3.25,
            cannon_damage: 15, cannon_reload_time: 5.0,
            ram_damage: 15
        }
    }

    pub fn get_stun_length(&self) -> f32 {
        BASE_STUN_LENGTH * (MAX_SHIP_DEFENSE / self.defense) as f32
    }
}

pub struct Ship {
    pub stype: ShipType,
    pub curr_health: u16,
    pub stun: Timer,
    pub name: String,
    pub id: u16,
    pub target_pos: Option<V2>,
    pub rotate_only: bool,
    pub attr: ShipAttributes,
    pub cannons: Vec<Cannon>,
    pub transform: Transform,
    pub treasury: Deposit,
    pub mods: Vec<Box<dyn ShipMod>>,
    pub is_in_harbour: bool,
    sprite: Sprite,
    health_bar: HealthBar,
    is_local_player: bool,
    spawn: Option<V2>,
    destroy: bool,
    game: GC
}

impl Ship {
    pub fn caravel(ctx: &mut Context, game: GC, name: String, id: u16, spawn: V2,
        respawn: bool) -> tetra::Result<Ship> {
        Self::new(ctx, ShipType::Caravel, name, id, "Caravel.png",
            ShipAttributes::caravel(), spawn, respawn, 4, 1.0, V2::new(10.0, 0.0), 1.0, game)
    }

    pub fn galleon(ctx: &mut Context, game: GC, name: String, id: u16, spawn: V2,
        respawn: bool) -> tetra::Result<Ship> {
        Self::new(ctx, ShipType::Galleon, name, id, "Galleon.png",
        ShipAttributes::galleon(), spawn, respawn, 5, 1.2, V2::new(15.0, 0.0), 1.0, game)
    }

    pub fn schooner(ctx: &mut Context, game: GC, name: String, id: u16, spawn: V2,
        respawn: bool) -> tetra::Result<Ship> {
        Self::new(ctx, ShipType::Schooner, name, id, "Schooner.png",
        ShipAttributes::schooner(), spawn, respawn, 3, 0.9, V2::new(0.0, 15.0), 1.25, game)
    }

    fn new(ctx: &mut Context, stype: ShipType, name: String, id: u16, ship_texture: &str, attr: ShipAttributes, spawn: V2,
        respawn: bool, cannons_per_side: u32, cannon_power: f32, cannon_pos: V2, mass: f32, game: GC) -> tetra::Result<Ship> {
        let mut game_ref = game.borrow_mut();
        let sprite = Sprite::new(game_ref.assets.load_texture(
            ctx, ship_texture.to_owned(), true)?, SpriteOrigin::Centre, None);
        let handle = game_ref.physics.build_ship_collider(
            sprite.texture.width() as f32 * 0.5,
            sprite.texture.height() as f32 * 0.5, mass, stype);
        let stun_length = attr.get_stun_length();
        game_ref.economy.add_deposit();
        let is_local_player = game_ref.network.as_ref().unwrap()
            .client.get_local_id().unwrap().n == id;
        std::mem::drop(game_ref);

        let mut transform = Transform::new(handle, game.clone());
        transform.set_pos(spawn, 0.0);
        let spawn = match respawn {
            true => Some(spawn),
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
            stype, curr_health: attr.health, name: name.to_owned(), id,
            target_pos: None, rotate_only: false, attr, cannons,
            transform, treasury: Deposit::default(), mods: Vec::new(),
            is_in_harbour: false, stun: Timer::new(stun_length), sprite,
            health_bar: HealthBar::new(ctx, name.as_str(), match is_local_player {
                true => Color::rgb8(0, 255, 0),
                false => Color::WHITE
            }, attr.health, game.clone())?,
            spawn, is_local_player, destroy: false, game
        })
    }

    pub fn apply_mod<T: ShipMod + 'static>(&mut self, ship_mod: T) {
        // TODO: Accept &mut Ship param to allow application of mod inside Ship method?
        // Cannot apply mod inside ship method as this will lead to a borrow conflict
        self.mods.push(Box::new(ship_mod));
    }

    pub fn remove_mod(&mut self, ship_mod_type: ShipModType)
        -> BbResult<Option<Box<dyn ShipMod + 'static>>> {
        for (i, ship_mod) in self.mods.iter_mut().enumerate() {
            if ship_mod.get_type() == ship_mod_type {
                // Called after ship borrow is dropped in upper level call:
                // ship_mod.on_remove()?;
                return Ok(Some(self.mods.remove(i)))
            }
        }
        Ok(None)
    }

    pub fn set_health(&mut self, curr_health: u16) {
        assert!(curr_health <= self.attr.health);
        self.curr_health = curr_health;
        self.health_bar.set_info(curr_health);
    }

    pub fn repair(&mut self) {
        self.set_health(self.attr.health);
    }

    pub fn stun(&mut self) {
        self.stun.reset();
    }

    pub fn is_stunned(&mut self) -> bool {
        self.stun.is_running()
    }

    pub fn take_damage(&mut self, ctx: &mut Context, damage: u16, world: &mut World)
        -> tetra::Result<bool> {
        if damage <= 0 {
            return Ok(false)
        }

        if self.curr_health < damage {
            self.curr_health = 0;
        }
        else {
            self.curr_health -= damage;
        }

        self.set_health(self.curr_health.clamped(0, self.attr.health));
        if self.curr_health == 0 {
            self.sink(ctx, world)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn sink(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result {
        println!("{} has been sunk!", self.get_name());
        let (pos, rot) = self.transform.get_translation();
        world.add_ship_wreck(ctx, pos, rot)?;

        if let Some(spawn) = self.spawn { // Respawn
            self.reset();
            let free_spawn = {
                let mut game_ref = self.game.borrow_mut();
                let free_spawn = game_ref.physics.check_for_space(
                    spawn, self.sprite.get_size() * 1.5, V2::down() /* = Up in Tetra space */);
                if self.is_local_player {
                    game_ref.cam.centre_on(free_spawn.clone());
                }
                free_spawn
            };
            self.transform.set_pos(free_spawn.clone(), 0.0);
        }
        else {
            self.set_health(0);
            self.destroy = true;
        }
        
        Ok(())
    }

    pub fn is_sunk(&self) -> bool {
        self.curr_health == 0
    }

    pub fn reset(&mut self) {
        self.transform.reset_velocity();
        self.reset_target_pos();
        self.stun.end();
        self.repair();
    }

    pub fn take_cannon_ball_hit(&mut self, ctx: &mut Context, dmg: u16,
        shooter_index: Index, world: &mut World) -> tetra::Result {
        let shooter = world.get_ship(shooter_index).unwrap();
        let mut shooter_ref = shooter.borrow_mut(); // Bold unwrap but what else...
        println!("{} was hit by {} cannon and took {} damage!",
            self.get_name(), shooter_ref.get_name(), dmg);
        match self.take_damage(ctx, dmg, world) {
            Ok(true) => {
                let forfeited_escudos = (self.treasury.balance as f32 * ESCUTO_SHOOT_STEAL_PERCENTAGE) as u32;
                let generated_payout =  self.game.borrow_mut()
                    .economy.total_payout(self.treasury.networth);
                shooter_ref.treasury.add(forfeited_escudos + generated_payout);
                self.treasury.lose(forfeited_escudos);
                self.game.borrow_mut().world.add_event(
                    WorldEvent::PlayerSunkByCannon(shooter_ref.get_name(), self.get_name()));
                Ok(())
            },
            Ok(false) => Ok(()),
            Err(e) => Err(e)
        }
    }

    pub fn shoot_cannons_on_side(&mut self, ctx: &mut Context,
        side: CannonSide, world: &mut World) -> tetra::Result {
        for cannon in self.cannons.iter_mut() {
            if cannon.side == side {
                cannon.shoot(ctx, world)?;
            }
        }
        Ok(())
    }

    pub fn shoot_cannons(&mut self, ctx: &mut Context, world: &mut World)
        -> tetra::Result {
        for cannon in self.cannons.iter_mut() {
            cannon.shoot(ctx, world)?;
        }
        Ok(())
    }

    pub fn set_target_pos(&mut self, pos: V2, rotate_only: bool) {
        self.target_pos = Some(pos);
        self.rotate_only = rotate_only;
    }

    pub fn reset_target_pos(&mut self) {
        self.target_pos = None;
    }

    fn move_to_target_pos(&mut self) {
        if self.is_stunned() {
            return;
        }
        
        if let Some(target_pos) = self.target_pos {
            let mut game_ref = self.game.borrow_mut();
            let rb = game_ref.physics.get_rb_mut(self.transform.handle.0);
            let (pos, rot) = disassemble_iso(rb.position());
            if vec_distance(pos, target_pos) <= TARGET_POS_DIST_MARGIN {
                self.target_pos = None;
                return;
            }

            if !self.rotate_only {
                let mut facing_dir = polar_to_cartesian(1.0, rot);
                facing_dir *= BASE_MOVEMENT_FORCE * self.attr.movement_speed;
                rb.apply_impulse(conv_vec(facing_dir), true);
            }

            let target_dir = target_pos - pos;
            let target_rot = pi_to_pi2_range(get_angle(target_dir));
            let delta_rot = target_rot - pi_to_pi2_range(rot);
            let mut applied_torque = self.attr.turn_rate * BASE_TORQUE_FORCE;
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
        //format!("Cpt. {}' Ship", self.name)
        self.name.clone()
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

    fn collide_with_ship(&mut self, ctx: &mut Context, other: Rcc<Ship>, world: &mut World) -> tetra::Result {
        let mut other_ref = other.borrow_mut();
        println!("{} collided with {} and dealt {} ram damage!",
            self.get_name(), other_ref.get_name(), self.attr.ram_damage);
        
        other_ref.stun();
        match other_ref.take_damage(ctx, self.attr.ram_damage, world) {
            Ok(true) => {
                let forfeited_escudos = (other_ref.treasury.balance as f32 *
                    ESCUTO_RAM_STEAL_PERCENTAGE) as u32;
                let generated_payout = self.game.borrow_mut()
                    .economy.total_payout(other_ref.treasury.networth);
                other_ref.treasury.lose(forfeited_escudos);
                self.treasury.add(forfeited_escudos + generated_payout);
                // println!("{} sunk {}'s ship via ramming and stole {} escudos!",
                //     self.get_name(), other_ref.get_name(), forfeited_escudos);
                self.game.borrow_mut().world.add_event(
                    WorldEvent::PlayerSunkByRamming(self.get_name(), other_ref.get_name()));
                Ok(())
            },
            Ok(false) => Ok(()),
            Err(e) => Err(e)
        }
    }

    fn collide_with_entity(&mut self, ctx: &mut Context, other: Rcc<dyn Entity>, world: &mut World)
        -> tetra::Result {
        let other_ref = other.borrow();
        if self.is_stunned() ||
            other_ref.get_type() == EntityType::CannonBall /* Cannon ball does the damage part */ {
            return Ok(())
        }

        // TODO: Rework
        let absorption = BASE_OBJECT_COLLISION_DAMAGE *
            (self.attr.defense / MAX_SHIP_DEFENSE);
        let damage = BASE_OBJECT_COLLISION_DAMAGE - absorption;
        if damage == 0 {
            return Ok(())
        }
        println!("{} collided with object {}!", self.get_name(), other_ref.get_name());

        self.stun();
        match self.take_damage(ctx, damage, world) {
            Ok(true) => {
                let forfeited_escudos = (self.treasury.balance as f32 * ESCUTO_ACCIDENT_LOSS_PERCENTAGE) as u32;
                self.game.borrow_mut().economy.remove(forfeited_escudos); // Lost to the sea...
                self.treasury.lose(forfeited_escudos);
                // println!("{} lost {} escudos after sinking their ship in an accident!",
                //     self.get_name(), forfeited_escudos);
                self.game.borrow_mut().world.add_event(
                    WorldEvent::PlayerSunkByAccident(self.get_name()));
                Ok(())
            },
            Ok(false) => Ok(()),
            Err(e) => Err(e)
        }
    }

    fn collide_with_neutral(&mut self, ctx: &mut Context)
        -> tetra::Result {
        Ok(())
    }

    fn intersect_with_entity(&mut self, ctx: &mut Context, state: bool,
        other: Rcc<dyn Entity>) -> tetra::Result {
        if other.borrow().get_type() == EntityType::Harbour {
            self.is_in_harbour = state;
        }
        Ok(())
    }
}

impl GameState for Ship {
    fn update(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result {
        self.stun.update(ctx);
        for cannon in self.cannons.iter_mut() {
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
            cannon.set_ship_translation(translation);
            cannon.draw(ctx)?;
        }
        self.health_bar.draw(ctx, translation.0);
        Ok(())
    }
}
