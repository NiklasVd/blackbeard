use std::f32::consts::PI;
use tetra::{Context, State, graphics::{Color, DrawParams, Texture, text::Text}};
use crate::{Entity, EntityType, GC, MASS_FORCE_SCALE, Transform, V2, conv_vec, disassemble_iso, get_angle, get_texture_origin, pi_to_pi2_range, polar_to_cartesian};

const BASE_MOVEMENT_FORCE: f32 = 10.0 * MASS_FORCE_SCALE;
const BASE_TORQUE_FORCE: f32 = 1000.0 * MASS_FORCE_SCALE;
const TARGET_POS_DIST_MARGIN: f32 = 75.0;
const TARGET_ROT_MARGIN: f32 = PI / 34.0;

pub struct ShipAttributes {
    pub movement_speed: f32,
    pub turn_rate: f32,
    pub cannon_damage: f32,
    pub ram_damage: f32
}

impl ShipAttributes {
    pub fn caravel() -> ShipAttributes {
        ShipAttributes {
            movement_speed: 8.5, turn_rate: 6.0, cannon_damage: 3.0, ram_damage: 2.0
        }
    }
}

pub struct Ship {
    // Health
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
        let label = Text::new(format!("Cpt. {}", name), game_ref.assets.font.clone());
        std::mem::drop(game_ref);
        Ok(Ship {
            target_pos: None, attr: ShipAttributes::caravel(),
            transform: Transform::new(get_texture_origin(ship_texture.clone()),
                handle, game.clone()),
            ship_texture: ship_texture, label, game, 
        })
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
}

impl Entity for Ship {
    fn get_type(&self) -> EntityType {
        EntityType::Ship
    }
}

impl State for Ship {
    fn update(&mut self, _ctx: &mut Context) -> tetra::Result {
        self.move_to_target_pos();
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
