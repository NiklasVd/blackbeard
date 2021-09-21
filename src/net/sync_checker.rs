use core::fmt;
use std::{collections::HashMap, iter::FromIterator};
use binary_stream::{BinaryStream, Serializable};
use nalgebra::{ComplexField};
use crate::{Rcc, ship::Ship};

pub const SYNC_STATE_GEN_INTERVAL: u64 = 100;

#[derive(Clone, Copy)]
pub struct SyncState {
    pub gen: u64,
    pub hash: u64
}

impl SyncState {
    pub fn new(gen: u64, hash: u64) -> SyncState {
        SyncState {
            gen, hash
        }
    }

    pub fn gen(gen: u64, buffer: &[u8]) -> SyncState {
        Self::new(gen, seahash::hash(buffer))
    }

    pub fn gen_from_ships(gen: u64, ships: Vec<Rcc<Ship>>) -> SyncState {
        let mut buffer = Vec::new();
        ships.into_iter().for_each(|ship| Self::serialize_ship(&mut buffer, ship));
        Self::gen(gen, &buffer)
    }

    pub fn serialize_ship(buffer: &mut Vec<u8>, ship: Rcc<Ship>) {
        let ship_ref = ship.borrow();
        buffer.extend(ship_ref.curr_health.to_le_bytes());
        let translation = ship_ref.transform.get_translation();
        buffer.extend(ComplexField::round(translation.0.x).to_le_bytes());
        buffer.extend(ComplexField::round(translation.0.y).to_le_bytes());
        buffer.extend(ComplexField::round(translation.1).to_le_bytes());
    }
}

impl Serializable for SyncState {
    fn to_stream(&self, stream: &mut BinaryStream) {
        stream.write_u64(self.gen).unwrap();
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
        write!(f, "Gen = {}, Hash = {}", self.gen, self.hash)
    }
}

pub struct SyncChecker {
    cache: HashMap<u64, HashMap<u16, SyncState>>,
}

impl SyncChecker {
    pub fn new() -> SyncChecker {
        SyncChecker {
            cache: HashMap::new()
        }
    }

    pub fn add_state(&mut self, sender: u16, state: SyncState) {
        if let Some(gen_states) = self.cache.get_mut(&state.gen) {
            gen_states.insert(sender, state);
        } else {
            self.cache.insert(state.gen, HashMap::from_iter([(sender, state); 1]));
        }
    }

    pub fn check_states(&mut self, gen: u64) -> Option<Vec<u16>> {
        // Check if auth client (ID == 0) already sent their sync state
        // If so, check states of all other clients, else wait.
        if let Some(gen_states) = self.cache.get_mut(&gen) {
            if let Some((_, auth_client_state)) = gen_states.iter_mut()
                .find(|(&id, _)| id == 0) {
                let auth_client_state = *auth_client_state;
                Some(gen_states.iter()
                    .filter(|(&id, &state)| id != 0 && state != auth_client_state)
                    .map(|(id, state)| *id).collect())
            } else {
                None // Auth client has not sent their state, from which all other states will be judged
            }
        } else {
            None // No state for this generation exists yet
        }
    }
}
