use core::fmt;
use std::{collections::HashMap, iter::FromIterator};
use binary_stream::{BinaryStream, Serializable};
use nalgebra::ComplexField;
use crate::{Rcc, input_pool::STEP_PHASE_FRAME_LENGTH, ship::Ship};

pub const SYNC_STATE_GEN_INTERVAL: u64 = 1/*60 / STEP_PHASE_FRAME_LENGTH as u64*/;
// First desync might be small inaccuracy (well, technically not). Second casts doubt. Third means it spiralled out of control.
pub const MAX_DESYNC_INTERVAL: u16 = 3;

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
        
        let x_state = ComplexField::round(translation.0.x);
        let y_state = ComplexField::round(translation.0.y);
        let rot_state = ComplexField::round(translation.1);
        
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
    cache: HashMap<u64, HashMap<u16, SyncState>>,
    pub desync_counter: HashMap<u16, u16>
}

impl SyncChecker {
    pub fn new() -> SyncChecker {
        SyncChecker {
            cache: HashMap::new(), desync_counter: HashMap::new()
        }
    }

    pub fn add_state(&mut self, sender: u16, state: SyncState) {
        let t = state.t;
        if let Some(gen_states) = self.cache.get_mut(&t) {
            gen_states.insert(sender, state);

        } else {
            self.cache.insert(t, HashMap::from_iter([(sender, state); 1]));
        }
        self.check_desync(sender, t);
    }

    pub fn get_desynced_players(&mut self) -> Vec<u16> {
        let players: Vec<_> = self.desync_counter.iter()
            .filter(|(id, &counter)| counter >= MAX_DESYNC_INTERVAL)
            .map(|(id, _)| *id).collect();
        players.iter()
            .for_each(|id| {
                self.desync_counter.remove(id);
            });
        players
    }

    fn check_desync(&mut self, client_id: u16, t: u64) {
        // Fetch states of generation at t
        if let Some(gen_states) = self.cache.get_mut(&t) {
            // Check if auth client has already sent their sync state.
            // This is the objective base from which other clients will be judged
            if let Some((_, auth_client_state)) = gen_states.iter_mut()
                .find(|(&id, _)| id == 0) {
                let auth_client_state = *auth_client_state;
                // Find sync state of respective player and compare
                if let Some(client_state) = gen_states.get(&client_id) {
                    let is_desync = *client_state != auth_client_state;
                    if let Some(client_desync_counter) = self.desync_counter.get_mut(&client_id) {
                        if is_desync {
                            *client_desync_counter += 1;
                            println!("Player with ID {} is out of sync for {}. time.",
                                client_id, client_desync_counter)
                        } else if *client_desync_counter > 0 {
                            println!("Player with ID {} is not out of sync anymore.", client_id);
                            *client_desync_counter = 0;
                        }
                    } else if is_desync {
                        self.desync_counter.insert(client_id, 1);
                        println!("Player with ID {} is out of sync for 1. time.", client_id);
                    }
                }
            }
        }
    }

    // fn record_desyncs(&mut self, t: u64) {
    //     if let Some(gen_states) = self.cache.get_mut(&t) {
    //         // Check if auth client (ID == 0) already sent their sync state
    //         if let Some((_, auth_client_state)) = gen_states.iter_mut()
    //             .find(|(&id, _)| id == 0) {
    //             let auth_client_state = *auth_client_state;

    //             // If so, check states of all other clients
    //             let (desynced_players, synced_players): (Vec<_>, Vec<_>) = gen_states.iter()
    //                 .partition(|(&id, &state)| id != 0 && state != auth_client_state);
    //             // Increment desync counter of all selected players
    //             let desync_counter = &mut self.desync_counter;
    //             desynced_players.into_iter().for_each(|(&id, _)| {
    //                 if let Some(counter) = desync_counter.get_mut(&id) {
    //                     *counter += 1;
    //                     println!("Player with ID {} is out of sync for {}. time.",
    //                         id, counter)
    //                 } else {
    //                     desync_counter.insert(id, 1);
    //                     println!("Player with ID {} is out of sync for 1. time.", id)
    //                 }
    //             });
    //             // Reset desync counter for all filtered players
    //             synced_players.into_iter().for_each(|(&id, _)| {
    //                 if let Some(counter) = desync_counter.get_mut(&id) {
    //                     if *counter > 0 {
    //                         println!("Player with ID {} is not out of sync anymore!", id);
    //                     }
    //                     *counter = 0;
    //                 }
    //             })
    //         }
    //     }
    // }
}
