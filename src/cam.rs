use tetra::{Context, State, graphics::Camera, input::{Key, is_key_down, is_mouse_scrolled_down, is_mouse_scrolled_up}, math::Clamp, window::{get_height, get_width}};
use crate::{V2, get_dt};

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

    pub fn project_pos(&self, ctx: &mut Context, pos: V2) -> V2 {
        self.instance.project(pos)
    }

    pub fn unproject_pos(&self, ctx: &mut Context, pos: V2) -> V2 {
        self.instance.unproject(pos)
    }

    pub fn centre_on(&mut self, pos: V2) {
        self.instance.position = pos;
    }
}

impl State for Cam {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if is_key_down(ctx, Key::W) {
            self.instance.position.y -= self.movement_speed * get_dt(ctx);
        }
        if is_key_down(ctx, Key::S) {
            self.instance.position.y += self.movement_speed * get_dt(ctx);
        }
        if is_key_down(ctx, Key::A) {
            self.instance.position.x -= self.movement_speed * get_dt(ctx);
        }
        if is_key_down(ctx, Key::D) {
            self.instance.position.x += self.movement_speed * get_dt(ctx);
        }

        if is_mouse_scrolled_up(ctx) {
            self.instance.scale += CAM_ZOOM_RATE * get_dt(ctx);
            self.instance.scale = self.instance.scale.clamped(V2::new(0.2, 0.2), V2::one() * 1.3);
        }
        else if is_mouse_scrolled_down(ctx) {
            self.instance.scale -= CAM_ZOOM_RATE * get_dt(ctx);
            self.instance.scale = self.instance.scale.clamped(V2::new(0.2, 0.2), V2::one() * 1.3);
        }

        self.instance.update();
        Ok(())
    }
}
