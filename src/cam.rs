use tetra::{Context, State, graphics::Camera, input::{Key, is_key_down, is_mouse_scrolled_down, is_mouse_scrolled_up}, math::Clamp};
use crate::{UPDATE_TICK_RATE, V2, WINDOW_HEIGHT, WINDOW_WIDTH};

pub const CAM_ZOOM_RATE: f32 = 2.0;

pub struct Cam {
    pub instance: Camera,
    pub movement_speed: f32
}

impl Cam {
    pub fn setup(movement_speed: f32) -> Cam {
        Cam {
            instance: Camera::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            movement_speed
        }
    }

    pub fn get_mouse_pos(&self, ctx: &mut Context) -> V2 {
        self.instance.mouse_position(ctx)
    }
}

impl State for Cam {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if is_key_down(ctx, Key::Up) {
            self.instance.position.y -= self.movement_speed * UPDATE_TICK_RATE;
        }
        if is_key_down(ctx, Key::Down) {
            self.instance.position.y += self.movement_speed * UPDATE_TICK_RATE;
        }
        if is_key_down(ctx, Key::Left) {
            self.instance.position.x -= self.movement_speed * UPDATE_TICK_RATE;
        }
        if is_key_down(ctx, Key::Right) {
            self.instance.position.x += self.movement_speed * UPDATE_TICK_RATE;
        }

        if is_mouse_scrolled_up(ctx)   {
            self.instance.scale += CAM_ZOOM_RATE * UPDATE_TICK_RATE;
            self.instance.scale = self.instance.scale.clamped(V2::new(0.2, 0.2), V2::one() * 1.3);
        }
        else if is_mouse_scrolled_down(ctx) {
            self.instance.scale -= CAM_ZOOM_RATE * UPDATE_TICK_RATE;
            self.instance.scale = self.instance.scale.clamped(V2::new(0.2, 0.2), V2::one() * 1.3);
        }

        self.instance.update();
        Ok(())
    }
}
