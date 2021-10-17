use crate::{Timer, V2};

pub struct ShipStatus {
    pub stun: Timer,
    pub target_pos: Option<V2>,
    pub rotate_only: bool,
    pub is_in_harbour: bool,
}

impl ShipStatus {
    pub fn stun(&mut self) {
        self.stun.reset();
    }

    pub fn is_stunned(&mut self) -> bool {
        self.stun.is_running()
    }

    pub fn reset_stun(&mut self) {
        self.stun.end();
    }

    pub fn set_target_pos(&mut self, pos: V2, rotate_only: bool) {
        self.target_pos = Some(pos);
        self.rotate_only = rotate_only;
    }

    pub fn reset_target_pos(&mut self) {
        self.target_pos = None;
        self.rotate_only = false;
    }
}
