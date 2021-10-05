use std::{collections::VecDeque, fs::File, io::{self, Write}, path::Path, time::{SystemTime, UNIX_EPOCH}};
use crate::{GC, V2, entity::EntityType, ship_data::ShipID, sync_checker::SyncState};

pub const DIAGNOSTICS_LOG_PATH: &str = "log";
const MAX_CACHE_SIZE: usize = 220;

pub struct SyncStateShipData {
    id: u16,
    pos: V2, rot: f32,
    curr_health: u16,
    cash_balance: u32
}

impl SyncStateShipData {
    pub fn new(id: u16, pos: V2, rot: f32, curr_health: u16, cash_balance: u32) -> SyncStateShipData {
        SyncStateShipData {
            id, pos, rot, curr_health, cash_balance
        }
    }
}

pub enum StateEvent {
    SyncState(u64, Vec<SyncStateShipData>),
    ShipShipCollision(ShipID, ShipID, u16),
    ShipEntityCollision(ShipID, EntityType, u16),
    ShipCannonBallCollision(ShipID, ShipID, u16),
    ShipShootCannon(ShipID, V2, u16),
}

impl StateEvent {
    pub fn log(self, file: &mut File) -> io::Result<()> {
        match self {
            StateEvent::SyncState(hash, mut ship_data) => {
                writeln!(file, "SyncState;{}", hash)?;
                for ship_data in ship_data.drain(..) {
                    writeln!(file, ";;;Ship;^{};{:.2},{:.2};{:.2};{};{}c", ship_data.id,
                        ship_data.pos.x, ship_data.pos.y, ship_data.rot, ship_data.curr_health, ship_data.cash_balance)?;
                }
                Ok(())
            },
            StateEvent::ShipShipCollision(player1_id, player2_id, dmg) =>
                writeln!(file, "ShipColl;{:?};collided with;{:?};{} dmg",
                player1_id, player2_id, dmg),
            StateEvent::ShipEntityCollision(player_id, entity_type, dmg) =>
                writeln!(file, "EntityColl;{:?};collided with;{:?};{} dmg",
                player_id, entity_type, dmg),
            StateEvent::ShipCannonBallCollision(player_id, shooter_id, dmg) =>
                writeln!(file, "CannonColl;{:?};shot by;{:?};{} dmg",
                player_id, shooter_id, dmg),
            StateEvent::ShipShootCannon(shooter_id, starting_pos, dmg) =>
                writeln!(file, "Shoot;{:?};From;{:.2},{:.2};{} dmg",
                shooter_id, starting_pos.x, starting_pos.y, dmg),
            
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
    states: VecDeque<DiagnosticState>
}

impl Diagnostics {
    pub fn new() -> Diagnostics {
        Diagnostics {
            states: VecDeque::new()
        }
    }

    pub fn add_state(&mut self, state: DiagnosticState) {
        if self.states.len() >= MAX_CACHE_SIZE {
            self.states.pop_front();
        }
        self.states.push_back(state);
    }

    pub fn backup_states(&mut self, name: &str) -> io::Result<()> {
        if self.states.len() == 0 {
            return Ok(())
        }

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut file = File::create(Path::new(DIAGNOSTICS_LOG_PATH)
            .join(format!("{}-{}-{}.csv", "states", name, timestamp)))?;
        writeln!(&mut file, "t;CurrFrame;Event")?;
        for state in self.states.drain(..) {
            state.log(&mut file)?;
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
