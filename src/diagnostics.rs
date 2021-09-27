use std::{collections::VecDeque, fs::File, io::{self, Write}, iter::FromIterator, path::Path, time::{SystemTime, UNIX_EPOCH}};
use crate::{V2, sync_checker::SyncState};

pub const DIAGNOSTICS_LOG_PATH: &str = "log";
pub const CACHED_SYNC_STATES_BACKUP_FILE_PATH: &str = "sync_states";
const MAX_CACHED_SYNC_STATES: usize = 60 * 4;

pub struct SyncStateShipData {
    id: u16,
    pos: V2, rot: f32, curr_health: u16
}

impl SyncStateShipData {
    pub fn new(id: u16, pos: V2, rot: f32, curr_health: u16) -> SyncStateShipData {
        SyncStateShipData {
            id, pos, rot, curr_health
        }
    }
}

struct SyncStateCache {
    t: u64,
    hash: u64,
    ship_data: VecDeque<SyncStateShipData>
}

impl SyncStateCache {
    pub fn new(t: u64, hash: u64, ship_data: Vec<SyncStateShipData>) -> SyncStateCache {
        SyncStateCache {
            t,
            hash,
            ship_data: VecDeque::from_iter(ship_data.into_iter())
        }
    }
}

pub struct Diagnostics {
    cached_sync_states: VecDeque<SyncStateCache>
}

impl Diagnostics {
    pub fn new() -> Diagnostics {
        Diagnostics {
            cached_sync_states: VecDeque::new()
        }
    }

    pub fn add_sync_state(&mut self, state: SyncState, ship_data: Vec<SyncStateShipData>) {
        if self.cached_sync_states.len() >= MAX_CACHED_SYNC_STATES {
            self.cached_sync_states.pop_front();
        }
        self.cached_sync_states.push_back(SyncStateCache::new(state.t, state.hash, ship_data));
    }

    pub fn backup_sync_states(&mut self) -> io::Result<()> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut file = File::create(Path::new(DIAGNOSTICS_LOG_PATH)
            .join(format!("{}-{}.csv", CACHED_SYNC_STATES_BACKUP_FILE_PATH, timestamp)))?;
        writeln!(file, "t,Hash,ID,Pos,Rot,CurrHealth")?;
        while let Some(mut sync_state) = self.cached_sync_states.pop_front() {
            writeln!(file, "{},{}", sync_state.t, sync_state.hash)?;
            while let Some(ship_data) = sync_state.ship_data.pop_front() {
                writeln!(file, ",,{},{:.2}|{:.2},{:.2},{}", ship_data.id,
                    ship_data.pos.x, ship_data.pos.y, ship_data.rot, ship_data.curr_health)?;
            }
        }
        file.flush()
    }
}

pub fn log(prefix: &str, text: &str) {
    println!("[{}] {}", prefix, text);
}

pub fn log_warning(text: &str) {
    log("W", text);
}

pub fn log_err(text: &str) {
    log("E", text);
}
