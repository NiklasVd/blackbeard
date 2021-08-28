use std::{collections::HashMap, time::Duration};
use crate::packet::InputState;

struct InputDelayedState {
    input: InputState,
    delay: Duration
}

impl InputDelayedState {
    fn new(input: InputState, delay: Duration) -> InputDelayedState {
        InputDelayedState {
            input, delay
        }
    }
}

pub struct PlaybackBuffer {
    pub default_delay: Duration,
    inputs: HashMap<u16, InputDelayedState>,
}

