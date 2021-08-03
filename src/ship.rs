use std::{collections::HashMap, f32::consts::PI};
use rapier2d::data::Index;
use tetra::{Context, graphics::{text::Text}, math::Clamp};
use crate::{Entity, EntityType, GC, MASS_FORCE_SCALE, Object, Rcc, Sprite, SpriteOrigin, Timer, Transform, V2, conv_vec, disassemble_iso, get_angle, pi_to_pi2_range, polar_to_cartesian};

const BASE_STUN_LENGTH: f32 = 2.0;
const BASE_OBJECT_COLLISION_DAMAGE: u16 = 20;
const MAX_SHIP_DEFENSE: u16 = 100;
const BASE_MOVEMENT_FORCE: f32 = 10.0 * MASS_FORCE_SCALE;
const BASE_TORQUE_FORCE: f32 = 1000.0 * MASS_FORCE_SCALE;
const TARGET_POS_DIST_MARGIN: f32 = 75.0;
const TARGET_ROT_MARGIN: f32 = PI / 30.0;

pub struct ShipAttributes {
    pub health: u16,
    pub defense: u16, // 1-100
    pub movement_speed: f32,
    pub turn_rate: f32,
    pub cannon_damage: u16,
    pub ram_damage: u16
}

impl ShipAttributes {
    pub fn caravel() -> ShipAttributes {
        ShipAttributes {
            health: 100,
            defense: 60,
            movement_speed: 8.5, turn_rate: 5.0,
            cannon_damage: 30, ram_damage: 20
        }
    }

    pub fn get_stun_length(&self) -> f32 {
        BASE_STUN_LENGTH * (MAX_SHIP_DEFENSE / self.defense) as f32
    }
}

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
    pub range: f32,
    pub dmg: f32,
    pub side: CannonSide,
    pub reload: Timer
}

impl Cannon {
    pub fn new(pos: V2, facing_rot: f32, range: f32, dmg: f32, side: CannonSide, reload_time: f32)
        -> Cannon {
        Cannon {
            pos, rot: get_angle(pos), facing_rot, range, dmg, side,
            reload: Timer::new(reload_time)
        }
    }

    pub fn get_reload_time(&self) -> f32 {
        return self.reload.max
    }

    pub fn get_translation(&self, ship_translation: (V2, f32)) -> (V2, f32) {
        (ship_translation.0 + polar_to_cartesian(self.pos.magnitude(),
            self.rot + ship_translation.1), self.facing_rot + ship_translation.1) 
    }
}

pub struct Ship {
    pub curr_health: u16,
    pub stun: Timer,
    pub name: String,
    pub target_pos: Option<V2>,
    pub attr: ShipAttributes,
    pub cannons: Vec<Cannon>,
    pub transform: Transform,
    sprite: Sprite,
    cannon_sprite: Sprite,
    label: Text,
    game: GC
}

impl Ship {
    pub fn caravel(ctx: &mut Context, game: GC, name: String) -> tetra::Result<Ship> {
        let mut game_ref = game.borrow_mut();
        let sprite = Sprite::new(game_ref.assets.load_texture(
            ctx, "Caravel.png".to_owned(), true)?, SpriteOrigin::Centre);
        let cannon_sprite = Sprite::new(game_ref.assets.load_texture(
            ctx, "Cannon.png".to_owned(), true)?, SpriteOrigin::Centre);
        let handle = game_ref.physics.build_ship_collider(
            sprite.texture.width() as f32 * 0.5, sprite.texture.height() as f32 * 0.5);
        let label = Text::new("", game_ref.assets.font.clone());
        let attr = ShipAttributes::caravel();
        let stun_length = attr.get_stun_length();
        std::mem::drop(game_ref);

        let mut cannons = Vec::new();
        cannons.push(Cannon::new(V2::new(40.0, -50.0), -PI / 2.0, 200.0, 20.0,
            CannonSide::Bowside, 5.0));

        Ok(Ship {
            curr_health: attr.health, name,
            target_pos: None, attr, cannons,
            transform: Transform::new(handle, game.clone()),
            stun: Timer::new(stun_length),
            sprite, cannon_sprite, label, game
        })
    }

    pub fn stun(&mut self) {
        self.stun.reset();
    }

    pub fn is_stunned(&mut self) -> bool {
        self.stun.is_running()
    }

    pub fn take_damage(&mut self, ctx: &mut Context, damage: u16) {
        if damage <= 0 {
            return;
        }

        println!("Cpt. {}'s ship took {} damage.", self.name, damage);
        if self.curr_health < damage {
            self.curr_health = 0;
        }
        else {
            self.curr_health -= damage;
        }

        self.curr_health = self.curr_health.clamped(0, self.attr.health);
        if self.curr_health == 0 {
            self.destroy(ctx);
        }
    }

    pub fn destroy(&mut self, ctx: &mut Context) {
        self.curr_health = 0;
        println!("Cpt. {}'s ship has been destroyed!", self.name);
    }

    pub fn is_destroyed(&self) -> bool {
        self.curr_health == 0
    }

    pub fn reset(&mut self) {
        self.transform.reset();
        self.reset_target_pos();
        self.stun.end();
        self.curr_health = self.attr.health;
    }

    pub fn collision_with_ship(&mut self, ctx: &mut Context, other: Rcc<Ship>) {
        if self.is_stunned() {
            return;
        }

        println!("Cpt .{}'s and Cpt. {}'s ships collided!",
            self.name, other.borrow().name);
        self.take_damage(ctx, other.borrow().attr.ram_damage);
        self.stun();
    }

    pub fn collision_with_object(&mut self, ctx: &mut Context) {
        if self.is_stunned() {
            return;
        }

        let absorption = BASE_OBJECT_COLLISION_DAMAGE *
            (self.attr.defense / MAX_SHIP_DEFENSE);
        let damage = BASE_OBJECT_COLLISION_DAMAGE - absorption;
        if damage == 0 {
            return;
        }
        println!("Cpt. {}'s ship collided with an object!", self.name);

        self.take_damage(ctx, damage);
        self.stun();
    }

    pub fn shoot_cannons(&mut self, side: CannonSide) {
        todo!()
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

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn update(&mut self, ctx: &mut Context, ships: &HashMap<Index, Rcc<Ship>>,
        objects: &HashMap<Index, Rcc<Object>>) -> tetra::Result {
        self.stun.update();
        self.move_to_target_pos();
        self.update_label();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        let translation = self.transform.get_translation();
        self.sprite.draw2(ctx, translation);
        for cannon in self.cannons.iter() {
            self.cannon_sprite.draw2(ctx, cannon.get_translation(
                self.transform.get_translation()));
        }
        self.label.draw(ctx, translation.0 - V2::new(90.0, 15.0));
        Ok(())
    }
}
