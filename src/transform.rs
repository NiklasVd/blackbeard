use rapier2d::na::Isometry2;
use tetra::graphics::DrawParams;

use crate::{GC, PhysicsHandle, V2, conv_vec};

pub struct Transform {
    pub texture_origin: V2,
    pub handle: PhysicsHandle,
    game: GC
}

impl Transform {
    pub fn new(texture_origin: V2, handle: PhysicsHandle, game: GC) -> Transform {
        Transform {
            texture_origin, handle, game
        }
    }

    pub fn set_pos(&mut self, pos: V2, rot: f32) {
        self.game.borrow_mut().physics.get_rb_mut(self.handle.0).set_position(
            Isometry2::new(conv_vec(pos), rot), true)
    }

    pub fn get_draw_params(&self) -> DrawParams {
        self.game.borrow_mut().physics.get_rb_draw_params(
            self.handle.0, self.texture_origin)
    }
}
