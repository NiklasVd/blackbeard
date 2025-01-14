use tetra::{Context, State, graphics::Camera, input::{Key, is_key_down, is_mouse_scrolled_down, is_mouse_scrolled_up}, math::Clamp, window::{get_height, get_width}};
use crate::{UNIT_FRAMERATE_TIMESTEP, V2};

pub const CAM_ZOOM_RATE: f32 = 2.0;

pub struct Cam {
    pub instance: Camera,
    pub movement_speed: f32
}

impl Cam {
    pub fn setup(ctx: &mut Context, movement_speed: f32) -> Cam {
        Cam {
            instance: Camera::new(get_width(ctx) as f32, get_height(ctx) as f32),
            movement_speed
        }
    }

    pub fn get_mouse_pos(&self, ctx: &mut Context) -> V2 {
        self.instance.mouse_position(ctx)
    }

    pub fn project_pos(&self, _ctx: &mut Context, pos: V2) -> V2 {
        self.instance.project(pos)
    }

    pub fn unproject_pos(&self, _ctx: &mut Context, pos: V2) -> V2 {
        self.instance.unproject(pos)
    }

    pub fn centre_on(&mut self, pos: V2) {
        self.instance.position = pos;
    }
}

impl State for Cam {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if is_key_down(ctx, Key::W) {
            self.instance.position.y -= self.movement_speed * UNIT_FRAMERATE_TIMESTEP;
        }
        if is_key_down(ctx, Key::S) {
            self.instance.position.y += self.movement_speed * UNIT_FRAMERATE_TIMESTEP;
        }
        if is_key_down(ctx, Key::A) {
            self.instance.position.x -= self.movement_speed * UNIT_FRAMERATE_TIMESTEP;
        }
        if is_key_down(ctx, Key::D) {
            self.instance.position.x += self.movement_speed * UNIT_FRAMERATE_TIMESTEP;
        }

        if is_mouse_scrolled_up(ctx) {
            self.instance.scale += CAM_ZOOM_RATE * UNIT_FRAMERATE_TIMESTEP;
            self.instance.scale = self.instance.scale.clamped(V2::new(0.1, 0.1),
                V2::one() * 1.5);
        }
        else if is_mouse_scrolled_down(ctx) {
            self.instance.scale -= CAM_ZOOM_RATE * UNIT_FRAMERATE_TIMESTEP;
            self.instance.scale = self.instance.scale.clamped(V2::new(0.1, 0.1),
                V2::one() * 1.5);
        }

        self.instance.update();
        Ok(())
    }
}
