use std::{collections::VecDeque, time::Instant};
use tetra::{Context, State};
use crate::{input_pool::STEP_PHASE_FRAME_LENGTH, packet::{InputStep}};

const MAX_TIMESTAMPS_CACHED: usize = 500;

pub struct PlaybackBuffer {
    steps: VecDeque<InputStep>,
    curr_frame_index: u32,
    timestamps: VecDeque<f32>,
    startup_time: Instant
}

impl PlaybackBuffer {
    pub fn new() -> PlaybackBuffer {
        PlaybackBuffer {
            steps: VecDeque::new(), curr_frame_index: 0,
            timestamps: VecDeque::new(), startup_time: Instant::now()
        }
    }

    pub fn add_step(&mut self, step: InputStep) {
        self.steps.push_back(step);
        
        if self.timestamps.len() > MAX_TIMESTAMPS_CACHED {
            self.timestamps.pop_front();
        }
        self.timestamps.push_back(self.startup_time.elapsed().as_secs_f32());
        self.startup_time = Instant::now();
    }

    pub fn get_buffer_size(&self) -> usize {
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

    pub fn calc_latency(&mut self) -> (f32, f32, f32) {
        if self.timestamps.len() == 0 {
            return (0.0, 0.0, 0.0)
        }
        let min = self.timestamps.iter().map(|t| *t).reduce(f32::min).unwrap();
        let max = self.timestamps.iter().map(|t| *t).reduce(f32::max).unwrap();
        let avg: f32 = self.timestamps.iter().sum::<f32>() / self.timestamps.len() as f32;
        self.timestamps.drain(0..(self.timestamps.len() / 2));
        (min, max, avg)
    }
}

impl State for PlaybackBuffer {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.curr_frame_index += 1;
        Ok(())
    }
}
