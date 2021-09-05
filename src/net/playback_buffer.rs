use std::collections::VecDeque;

use tetra::{Context, State};
use crate::{packet::{InputStep}, server::STEP_PHASE_FRAME_LENGTH};

pub struct PlaybackBuffer {
    steps: VecDeque<InputStep>,
    curr_frame_index: u32
}

impl PlaybackBuffer {
    pub fn new() -> PlaybackBuffer {
        PlaybackBuffer {
            steps: VecDeque::new(), curr_frame_index: 0
        }
    }

    pub fn add_step(&mut self, step: InputStep) {
        self.steps.push_back(step);
    }

    pub fn get_buffered_step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn is_phase_over(&self) -> bool {
        self.curr_frame_index >= STEP_PHASE_FRAME_LENGTH
    }

    pub fn get_next_step(&mut self) -> Option<InputStep> {
        if let Some(step) = self.steps.pop_front() {
            self.curr_frame_index = 0;
            Some(step)
        } else {
            None
        }
    }
}

impl State for PlaybackBuffer {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.curr_frame_index += 1;
        Ok(())
    }
}
