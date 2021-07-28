use std::f32::consts::PI;
use tetra::{Context, State, graphics::{Color, DrawParams, Texture, text::Text}, math::Clamp};
use crate::{Entity, EntityType, GC, MASS_FORCE_SCALE, Rcc, Transform, V2, conv_vec, disassemble_iso, get_angle, get_texture_origin, pi_to_pi2_range, polar_to_cartesian};

const BASE_MOVEMENT_FORCE: f32 = 10.0 * MASS_FORCE_SCALE;
const BASE_TORQUE_FORCE: f32 = 1000.0 * MASS_FORCE_SCALE;
const TARGET_POS_DIST_MARGIN: f32 = 75.0;
const TARGET_ROT_MARGIN: f32 = PI / 34.0;

pub struct ShipAttributes {
    pub health: u16,
    pub stability: u16,
    pub movement_speed: f32,
    pub turn_rate: f32,
    pub cannon_damage: u16,
    pub ram_damage: u16
}

impl ShipAttributes {
    pub fn caravel() -> ShipAttributes {
        ShipAttributes {
            health: 100, stability: 80,
            movement_speed: 8.5, turn_rate: 6.0,
            cannon_damage: 30, ram_damage: 20
        }
    }
}

pub struct Ship {
    pub curr_health: u16,
    pub name: String,
    pub target_pos: Option<V2>,
    pub attr: ShipAttributes,
    pub transform: Transform,
    ship_texture: Texture,
    label: Text,
    game: GC
}

impl Ship {
    pub fn caravel(ctx: &mut Context, game: GC, name: String) -> tetra::Result<Ship> {
        let mut game_ref = game.borrow_mut();
        let ship_texture = game_ref.assets.load_texture(ctx, "Caravel.png".to_owned(), true)?;
        let handle = game_ref.physics.build_ship_collider(
            ship_texture.width() as f32 * 0.5, ship_texture.height() as f32 * 0.5);
        let label = Text::new("", game_ref.assets.font.clone());
        let attr = ShipAttributes::caravel();
        std::mem::drop(game_ref);
        Ok(Ship {
            curr_health: attr.health, name,
            target_pos: None, attr,
            transform: Transform::new(get_texture_origin(ship_texture.clone()),
                handle, game.clone()),
            ship_texture: ship_texture, label, game
        })
    }

    pub fn take_damage(&mut self, ctx: &mut Context, damage: u16) -> tetra::Result {
        println!("Cpt. {}'s ship took {} damage.", self.name, damage);
        if self.curr_health < damage {
            self.curr_health = 0;
        }
        else {
            self.curr_health -= damage;
        }

        self.curr_health = self.curr_health.clamped(0, self.attr.health);
        if self.curr_health == 0 {
            self.destroy(ctx)?;
        }
        Ok(())
    }

    pub fn destroy(&mut self, ctx: &mut Context) -> tetra::Result {
        self.curr_health = 0;
        self.ship_texture = self.game.borrow_mut().assets.load_texture(ctx, 
            "Destroyed Caravel.png".to_owned(), true)?;
        println!("Cpt. {}'s ship has been destroyed!", self.name);
        Ok(())
    }

    pub fn collision_with_ship(&mut self, ctx: &mut Context, other: Rcc<Ship>)
        -> tetra::Result {
        println!("Cpt .{}'s and Cpt. {}'s ships collided!",
            self.name, other.borrow().name);
        self.take_damage(ctx, other.borrow().attr.ram_damage)
    }

    pub fn collision_with_object(&mut self, ctx: &mut Context) -> tetra::Result {
        println!("Cpt. {}'s ship collided with an object!", self.name);
        self.take_damage(ctx, 100 - self.attr.stability) // ?
    }

    pub fn set_target_pos(&mut self, pos: V2) {
        self.target_pos = Some(pos);
    }

    fn move_to_target_pos(&mut self) {
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
        self.label.set_content(format!("Cpt. {} [{}/{} HP]", self.name,
            self.curr_health, self.attr.health));
    }
}

impl Entity for Ship {
    fn get_type(&self) -> EntityType {
        EntityType::Ship
    }
}

impl State for Ship {
    fn update(&mut self, _ctx: &mut Context) -> tetra::Result {
        self.move_to_target_pos();
        self.update_label();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        let draw_params = self.transform.get_draw_params();
        self.ship_texture.draw(ctx, draw_params.clone());
        self.label.draw(ctx, DrawParams {
            position: draw_params.position, rotation: 0.0,
            scale: V2::one(), origin: V2::zero(), color: Color::WHITE
        });
        Ok(())
    }
}
