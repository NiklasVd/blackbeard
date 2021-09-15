use std::{f32::consts::PI};
use binary_stream::{BinaryStream, Serializable};
use rapier2d::{data::Index};
use tetra::{Context, State, graphics::{text::Text}, math::Clamp};
use crate::{BbResult, Cannon, CannonSide, GC, MASS_FORCE_SCALE, Rcc, Sprite, SpriteOrigin, Timer, Transform, V2, WorldEvent, conv_vec, disassemble_iso, entity::{Entity, EntityType, GameState}, get_angle, pi_to_pi2_range, polar_to_cartesian, ship_mod::{ShipMod, ShipModType}, world::World};

const BASE_STUN_LENGTH: f32 = 2.0;
const BASE_OBJECT_COLLISION_DAMAGE: u16 = 20;
const MAX_SHIP_DEFENSE: u16 = 100;
const BASE_MOVEMENT_FORCE: f32 = 10.0 * MASS_FORCE_SCALE;
const BASE_TORQUE_FORCE: f32 = 1000.0 * MASS_FORCE_SCALE;
const TARGET_POS_DIST_MARGIN: f32 = 75.0;
const TARGET_ROT_MARGIN: f32 = PI / 45.0;
const ESCUTO_RAM_STEAL_PERCENTAGE: f32 = 0.4;
const ESCUTO_SHOOT_STEAL_PERCENTAGE: f32 = 0.3;
const ESCUTO_ACCIDENT_LOSS_PERCENTAGE: f32 = 0.2;

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
            movement_speed: 8.5, turn_rate: 5.0,
            cannon_damage: 20, cannon_reload_time: 5.0,
            ram_damage: 20
        }
    }

    pub fn galleon() -> ShipAttributes {
        ShipAttributes {
            health: 180,
            defense: 80,
            movement_speed: 6.0, turn_rate: 3.85,
            cannon_damage: 20, cannon_reload_time: 5.0,
            ram_damage: 35
        }
    }

    pub fn schooner() -> ShipAttributes {
        ShipAttributes {
            health: 115,
            defense: 35,
            movement_speed: 10.0, turn_rate: 5.0,
            cannon_damage: 20, cannon_reload_time: 5.0,
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
    pub target_pos: Option<V2>,
    pub attr: ShipAttributes,
    pub cannons: Vec<Cannon>,
    pub transform: Transform,
    pub escudos: u32,
    pub mods: Vec<Box<dyn ShipMod>>,
    pub is_in_harbour: bool,
    sprite: Sprite,
    label: Text,
    spawn: Option<V2>,
    destroy: bool,
    game: GC
}

impl Ship {
    pub fn caravel(ctx: &mut Context, game: GC, name: String, spawn: V2,
        respawn: bool) -> tetra::Result<Ship> {
        Self::new(ctx, name, "Caravel.png", ShipAttributes::caravel(), spawn, respawn,
            3, 1.0, game)
    }

    pub fn galleon(ctx: &mut Context, game: GC, name: String, spawn: V2, respawn: bool)
        -> tetra::Result<Ship> {
        Self::new(ctx, name, "Galleon.png", ShipAttributes::galleon(), spawn, respawn,
            4, 1.0, game)
    }

    pub fn schooner(ctx: &mut Context, game: GC, name: String, spawn: V2, respawn: bool)
        -> tetra::Result<Ship> {
        Self::new(ctx, name, "Schooner.png", ShipAttributes::schooner(), spawn, respawn,
            2, 1.5 /* Small collider size warrants greater density as a balance */, game)
    }

    fn new(ctx: &mut Context, name: String, ship_texture: &str, attributes: ShipAttributes, spawn: V2,
        respawn: bool, cannons_per_side: u32, mass: f32, game: GC) -> tetra::Result<Ship> {
        let mut game_ref = game.borrow_mut();
        let sprite = Sprite::new(game_ref.assets.load_texture(
            ctx, ship_texture.to_owned(), true)?, SpriteOrigin::Centre, None);
        let handle = game_ref.physics.build_ship_collider(
            sprite.texture.width() as f32 * 0.5, sprite.texture.height() as f32 * 0.5, mass);
        let label = Text::new("", game_ref.assets.font.clone());
        let attr = ShipAttributes::caravel();
        let stun_length = attr.get_stun_length();
        std::mem::drop(game_ref);

        let mut transform = Transform::new(handle, game.clone());
        transform.set_pos(spawn, 0.0);
        let spawn = match respawn {
            true => Some(spawn),
            false => None
        };

        let index = transform.get_index();
        let mut cannons = Vec::new();
        let mut bow_pos = V2::new(48.0, -50.0);
        let bow_rot = PI * 1.5;
        for _ in 0..cannons_per_side {
            cannons.push(Cannon::new(ctx, bow_pos, bow_rot, attr.cannon_damage,
                CannonSide::Bowside, attr.cannon_reload_time, index, game.clone())?);
            bow_pos -= V2::new(45.0, 0.0);
        }
        let mut port_pos = V2::new(48.0, 50.0);
        let port_rot = PI / 2.0;
        for _ in 0..cannons_per_side {
            cannons.push(Cannon::new(ctx, port_pos, port_rot, attr.cannon_damage,
                CannonSide::Portside, attr.cannon_reload_time, index, game.clone())?);
            port_pos -= V2::new(45.0, 0.0);
        }

        Ok(Ship {
            stype: ShipType::Caravel, curr_health: attr.health, name,
            target_pos: None, attr, cannons,
            transform, escudos: 100, mods: Vec::new(), is_in_harbour: false,
            stun: Timer::new(stun_length), sprite, label, spawn,
            destroy: false, game
        })
    }

    pub fn apply_mod<T: ShipMod + 'static>(&mut self, ship_mod: T) {
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

    pub fn repair(&mut self) {
        self.curr_health = self.attr.health;
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

        self.curr_health = self.curr_health.clamped(0, self.attr.health);
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
            let free_spawn = self.game.borrow().physics.check_for_space(
                spawn, self.sprite.get_size() * 1.5, V2::right());
            self.transform.set_pos(free_spawn, 0.0);
        }
        else {
            self.curr_health = 0;
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
        self.curr_health = self.attr.health;
    }

    pub fn take_cannon_ball_hit(&mut self, ctx: &mut Context, dmg: u16,
        shooter_index: Index, world: &mut World) -> tetra::Result {
        let shooter = world.get_ship(shooter_index).unwrap();
        let mut shooter_ref = shooter.borrow_mut(); // Bold unwrap but what else...
        println!("{} was hit by {} cannon and took {} damage!",
            self.get_name(), shooter_ref.get_name(), dmg);
        match self.take_damage(ctx, dmg, world) {
            Ok(true) => {
                let forfeited_escudos = (self.escudos as f32 * ESCUTO_SHOOT_STEAL_PERCENTAGE) as u32;
                shooter_ref.escudos += forfeited_escudos;
                self.escudos -= forfeited_escudos;
                // println!("{} sunk {}'s ship with a cannon shot and stole {} escudos!",
                //     shooter_ref.get_name(), self.get_name(), forfeited_escudos);
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
        -> tetra::Result { // Add option to choose cannon side
        for cannon in self.cannons.iter_mut() {
            cannon.shoot(ctx, world)?;
        }
        Ok(())
    }

    pub fn set_target_pos(&mut self, pos: V2) {
        self.target_pos = Some(pos);
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
            if pos.distance(target_pos) <= TARGET_POS_DIST_MARGIN {
                self.target_pos = None;
            }

            let mut facing_dir = polar_to_cartesian(1.0, rot);
            facing_dir *= BASE_MOVEMENT_FORCE * self.attr.movement_speed;
            rb.apply_impulse(conv_vec(facing_dir), true);

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

    fn update_label(&mut self) {
        let mut stunned = "";
        if self.is_stunned() {
            stunned = "*";
        }
        self.label.set_content(format!("Cpt. {} [{}/{} HP] {}", self.name,
            self.curr_health, self.attr.health, stunned));
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
                let forfeited_escudos = (other_ref.escudos as f32 * ESCUTO_RAM_STEAL_PERCENTAGE) as u32;
                other_ref.escudos -= forfeited_escudos;
                self.escudos += forfeited_escudos;
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
                let forfeited_escudos = (self.escudos as f32 * ESCUTO_ACCIDENT_LOSS_PERCENTAGE) as u32;
                self.escudos -= forfeited_escudos;
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
        self.update_label();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        let translation = self.transform.get_translation();
        self.sprite.draw2(ctx, translation);
        for cannon in self.cannons.iter_mut() {
            cannon.set_ship_translation(translation);
            cannon.draw(ctx)?;
        }
        // Ship mod draw call unneccessary
        self.label.draw(ctx, translation.0 - V2::new(90.0, 15.0));
        Ok(())
    }
}
