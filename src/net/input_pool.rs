use std::{collections::{HashMap, HashSet}, iter::FromIterator};
use crate::packet::{InputState, InputStep};

pub const STEP_PHASE_FRAME_LENGTH: u32 = 10;
pub const STEP_PHASE_TIME_SECS: f32 = STEP_PHASE_FRAME_LENGTH as f32 / 60.0;
pub const MAX_CLIENT_STATE_SEND_DELAY: u32 = 60 * 15; // 5 secs

pub struct InputPool {
    pub curr_gen: u64,
    pub curr_frame_index: u32,
    players: HashSet<u16>,
    player_states: HashSet<u16>,
    input_states: HashMap<u16, InputState>
}

impl InputPool {
    pub fn new(players: Vec<u16>) -> Self {
        Self {
            curr_gen: 0, curr_frame_index: 0,
            players: HashSet::from_iter(players.into_iter()),
            player_states: HashSet::new(), input_states: HashMap::new()
        }
    }

    pub fn add_state(&mut self, sender: u16, state: InputState) {
        self.player_states.insert(sender);
        self.input_states.insert(sender, state); // If client sends state more than once during step, overwrite
    }

    pub fn remove_player(&mut self, id: u16) {
        self.players.remove(&id);
        self.player_states.remove(&id);
        self.input_states.remove(&id);
    }

    pub fn is_step_phase_over(&self) -> bool {
        self.curr_frame_index >= STEP_PHASE_FRAME_LENGTH
    }

    pub fn is_max_delay_exceeded(&self) -> bool {
        self.curr_frame_index >= MAX_CLIENT_STATE_SEND_DELAY
    }

    pub fn check_delayed_players(&mut self) -> Vec<u16> {
        self.players.iter().filter(|id| !self.player_states.contains(id)).map(|id| *id).collect()
    }

    pub fn update_states(&mut self) {
        self.curr_frame_index += 1;
    }

    pub fn flush_states(&mut self) -> InputStep {
        self.player_states.clear();
        self.curr_frame_index = 0;
        self.curr_gen += 1;

        let states = self.input_states.drain().collect::<Vec<_>>();
        InputStep::new(states, self.curr_gen)
    }
}
