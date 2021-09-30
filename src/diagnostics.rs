use std::{fs::File, io::{self, Write}, path::Path, time::{SystemTime, UNIX_EPOCH}};
use indexmap::IndexMap;
use crate::{GC, V2, sync_checker::SyncState};

pub const DIAGNOSTICS_LOG_PATH: &str = "log";
const MAX_CACHE_SIZE: usize = 100;

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

pub enum StateEvent {
    SyncState(u64, Vec<SyncStateShipData>),
    ShipShoot(u16, bool, bool),
    ShipMotion(u16, bool, V2)
}

impl StateEvent {
    pub fn log(self, file: &mut File) -> io::Result<()> {
        match self {
            StateEvent::SyncState(hash, mut ship_data) => {
                writeln!(file, "SyncState;{}", hash)?;
                for ship_data in ship_data.drain(..) {
                    writeln!(file, ";;;Ship;^{};{:.2},{:.2};{:.2};{}", ship_data.id,
                        ship_data.pos.x, ship_data.pos.y, ship_data.rot, ship_data.curr_health)?;
                }
                Ok(())
            },
            StateEvent::ShipShoot(id, q, e) =>
                writeln!(file, "Shoot;^{};{};{}", id, q, e),
            StateEvent::ShipMotion(id, rotate, target_pos, ) =>
                writeln!(file, "Move;^{};{};{:.2},{:.2}", id, match rotate {
                    true => "Rot",
                    false => "Move"
                }, target_pos.x, target_pos.y)
        }
    }
}

pub struct DiagnosticState {
    pub t: u64,
    pub curr_frame: u64,
    pub event: StateEvent
}

impl DiagnosticState {
    pub fn new(t: u64, curr_frame: u64, event: StateEvent) -> Self {
        DiagnosticState {
            t, curr_frame, event
        }
    }

    pub fn new_sync_state(t: u64, curr_frame: u64, state: SyncState,
        ship_data: Vec<SyncStateShipData>) -> DiagnosticState {
        Self::new(t, curr_frame, StateEvent::SyncState(state.hash, ship_data))
    }

    pub fn log(self, file: &mut File) -> io::Result<()> {
        write!(file, "{};{};", self.t, self.curr_frame)?;
        self.event.log(file)
    }
}

pub struct Diagnostics {
    states: IndexMap<u64, Vec<DiagnosticState>>
}

impl Diagnostics {
    pub fn new() -> Diagnostics {
        Diagnostics {
            states: IndexMap::new()
        }
    }

    pub fn add_state(&mut self, state: DiagnosticState) {
        if self.states.len() >= MAX_CACHE_SIZE {
            // Are index maps ordered inherently by the key, if implementing Ord?
            self.states.drain(..10);
        }

        if let Some(curr_gen_states) = self.states.get_mut(&state.t) {
            curr_gen_states.push(state);
        } else {
            self.states.insert(state.t, vec![state]);
        }
    }

    pub fn backup_states(&mut self) -> io::Result<()> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut file = File::create(Path::new(DIAGNOSTICS_LOG_PATH)
            .join(format!("{}-{}.csv", "states", timestamp)))?;
        writeln!(&mut file, "t;CurrFrame;Event")?;
        for (_, mut states) in self.states.drain(..) {
            for state in states.drain(..) {
                state.log(&mut file)?;
            }
        }
        file.flush()
    }
}

pub fn log_state_event(game: GC, state: StateEvent) {
    let mut game_ref = game.borrow_mut();
    let curr_gen = game_ref.simulation_settings.curr_gen;
    let curr_frames = game_ref.simulation_settings.curr_frames;
    game_ref.diagnostics.add_state(DiagnosticState::new(curr_gen, curr_frames, state));
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
