use std::{collections::VecDeque, time::Instant};
use tetra::{Context, State, math::Clamp};
use crate::{DEFAULT_SIMULATION_TIMESTEP, input_pool::{STEP_PHASE_FRAME_LENGTH, STEP_PHASE_TIME_SECS}, packet::{InputStep}};

const MAX_BUFFER_SIZE: usize = (DEFAULT_SIMULATION_TIMESTEP * 0.5) as usize
    / STEP_PHASE_FRAME_LENGTH as usize;
const MIN_BUFFER_SIZE: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepPhase {
    Running,
    Imminent,
    Over
}

pub struct PlaybackBuffer {
    pub curr_frames: u64,
    steps: VecDeque<InputStep>,
    curr_frame_index: u32,
    received_steps: u64,
    step_wait_time: f32,
    startup_time: Instant
}

impl PlaybackBuffer {
    pub fn new() -> PlaybackBuffer {
        PlaybackBuffer {
            curr_frames: 0, steps: VecDeque::new(), curr_frame_index: 0, received_steps: 0,
            step_wait_time: 0.0, startup_time: Instant::now()
        }
    }

    pub fn add_step(&mut self, step: InputStep) {
        self.steps.push_back(step);
        self.received_steps += 1;

        let curr_step_wait_time = self.startup_time.elapsed().as_secs_f32();
        self.step_wait_time += curr_step_wait_time.min(STEP_PHASE_TIME_SECS);
        self.startup_time = Instant::now();
    }

    pub fn get_buffer_size(&self) -> usize {
        self.steps.len()
    }

    pub fn get_curr_phase(&self) -> StepPhase {
        // Alternative: check with modulo
        if self.curr_frame_index >= STEP_PHASE_FRAME_LENGTH {
            StepPhase::Over
        } else if self.curr_frame_index == STEP_PHASE_FRAME_LENGTH - 1 {
            StepPhase::Imminent
        } else {
            StepPhase::Running
        }
    }

    pub fn is_next_step_ready(&self) -> bool {
        self.get_buffer_size() > 0
    }

    pub fn get_next_step(&mut self) -> Option<InputStep> {
        if let Some(step) = self.steps.pop_front() {
            self.curr_frame_index = 0;
            Some(step)
        } else {
            None
        }
    }

    pub fn get_latency(&self) -> f32 {
        if self.received_steps == 0 {
            0.0
        } else {
            self.step_wait_time / self.received_steps as f32
        }
    }

    pub fn estimate_optimal_buffer_size(&self) -> usize {
        let latency = self.get_latency();
        let optimal_latency = STEP_PHASE_TIME_SECS;
        ((latency / optimal_latency).round() as usize).clamped(MIN_BUFFER_SIZE, MAX_BUFFER_SIZE)
    }
}

impl State for PlaybackBuffer {
    fn update(&mut self, _ctx: &mut Context) -> tetra::Result {
        self.curr_frame_index += 1;
        self.curr_frames += 1;
        Ok(())
    }
}
