use rapier2d::{data::Index, na::{Isometry2, Vector2}};
use tetra::graphics::DrawParams;
use crate::{GC, PhysicsHandle, V2, conv_vec};

pub struct Transform {
    pub handle: PhysicsHandle,
    game: GC
}

impl Transform {
    pub fn new(handle: PhysicsHandle, game: GC) -> Transform {
        Transform {
            handle, game
        }
    }

    pub fn set_pos(&mut self, pos: V2, rot: f32) {
        let mut game_ref = self.game.borrow_mut();
        let rb = game_ref.physics.get_rb_mut(self.handle.0);
        rb.set_position(
            Isometry2::new(conv_vec(pos), rot), true)
    }

    pub fn reset(&mut self) {
        let mut game_ref = self.game.borrow_mut();
        let rb = game_ref.physics.get_rb_mut(self.handle.0);
        rb.set_linvel(Vector2::new(0.0, 0.0), true);
        rb.set_angvel(0.0, true);
    }

    pub fn get_index(&self) -> Index {
        self.handle.1.0
    }

    pub fn get_translation(&self) -> (V2, f32) {
        self.game.borrow().physics.get_converted_rb_iso(self.handle.0)
    }

    pub fn get_draw_params(&self, texture_origin: V2) -> DrawParams {
        self.game.borrow().physics.get_rb_draw_params(
            self.handle.0, texture_origin)
    }

    pub fn destroy(&mut self) {
        self.game.borrow_mut().physics.remove_collider(self.handle);
    }
}
