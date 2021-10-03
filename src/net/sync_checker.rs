use core::fmt;
use std::{collections::HashMap, iter::FromIterator};
use binary_stream::{BinaryStream, Serializable};
use crate::{Rcc, input_pool::STEP_PHASE_FRAME_LENGTH, round_f32, ship::Ship};

pub const SYNC_STATE_GEN_INTERVAL: u64 = 1/*60 / STEP_PHASE_FRAME_LENGTH as u64*/;

#[derive(Clone, Copy)]
pub struct SyncState {
    pub t: u64,
    pub hash: u64
}

impl SyncState {
    pub fn new(t: u64, hash: u64) -> SyncState {
        SyncState {
            t, hash
        }
    }

    pub fn gen(t: u64, buffer: &[u8]) -> SyncState {
        Self::new(t, seahash::hash(buffer))
    }

    pub fn gen_from_ships(t: u64, ships: Vec<Rcc<Ship>>) -> SyncState {
        let mut buffer = Vec::new();
        ships.into_iter().for_each(|ship| Self::serialize_ship(&mut buffer, ship));
        Self::gen(t, &buffer)
    }

    pub fn serialize_ship(buffer: &mut Vec<u8>, ship: Rcc<Ship>) {
        let ship_ref = ship.borrow();
        buffer.extend(ship_ref.curr_health.to_le_bytes());
        let translation = ship_ref.transform.get_translation();
        
        let x_state = round_f32(translation.0.x);
        let y_state = round_f32(translation.0.y);
        let rot_state = round_f32(translation.1);
        
        buffer.extend(x_state.to_le_bytes());
        buffer.extend(y_state.to_le_bytes());
        buffer.extend(rot_state.to_le_bytes());
    }
}

impl Serializable for SyncState {
    fn to_stream(&self, stream: &mut BinaryStream) {
        stream.write_u64(self.t).unwrap();
        stream.write_u64(self.hash).unwrap();
    }

    fn from_stream(stream: &mut BinaryStream) -> Self {
        let gen = stream.read_u64().unwrap();
        let hash = stream.read_u64().unwrap();
        Self::new(gen, hash)
    }
}

impl PartialEq for SyncState {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl fmt::Debug for SyncState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Gen = {}, Hash = {}", self.t, self.hash)
    }
}

pub struct SyncChecker {
    states: HashMap<u64, HashMap<u16, SyncState>>
}

impl SyncChecker {
    pub fn new() -> SyncChecker {
        SyncChecker {
            states: HashMap::new()
        }
    }

    pub fn add_state(&mut self, sender: u16, state: SyncState) {
        if let Some(gen_states) = self.states.get_mut(&state.t) {
            gen_states.insert(sender, state);
        } else {
            self.states.insert(state.t, HashMap::from_iter([(sender, state); 1]));
        }
    }

    pub fn review_desyncs(&mut self, t: u64) -> Vec<u16> {
        if let Some(gen_states) = self.states.get(&t) {
            if let Some(auth_client_state) = gen_states.get(&0) {
                gen_states
                    .iter()
                    .filter(|(_, s)| s.hash != auth_client_state.hash)
                    .map(|(&id, _)| id).collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
}
