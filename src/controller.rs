use std::{time::Instant};
use indexmap::IndexMap;
use tetra::{Context, Event, State, input::{Key, MouseButton}, time::{Timestep, set_timestep}};
use crate::{BbResult, DiagnosticState, GC, Player, Rcc, Sprite, SpriteOrigin, SyncStateShipData, TransformResult, V2, entity::GameState, input_pool::STEP_PHASE_TIME_SECS, packet::{InputState, InputStep, Packet}, playback_buffer::{PlaybackBuffer, StepPhase}, ship_mod::ShipModType, sync_checker::{SYNC_STATE_GEN_INTERVAL, SyncState}, world::World, wrap_rcc};

pub const MAX_INPUT_STEP_BLOCK_TIME: f32 = 20.0;
pub const DEFAULT_SIMULATION_TIMESTEP: f64 = 60.0;
pub const MAX_ACCELERATED_TIMESTEP: f64 = DEFAULT_SIMULATION_TIMESTEP * 6.0; // Stability?

pub struct Controller {
    pub players: IndexMap<u16, Rcc<Player>>,
    pub local_player: Option<Rcc<Player>>,
    pub catch_input: bool,
    pub input_buffer: PlaybackBuffer,
    curr_input_state: InputState,
    curr_gen: u64,
    blocking_time: Instant,
    target_x: Sprite,
    curr_target_pos: Option<V2>,
    game: GC
}

impl Controller {
    pub fn new(ctx: &mut Context, game: GC) -> tetra::Result<Controller> {
        let target_x = game.borrow_mut().assets.load_texture(
            ctx, "UI/X.png".to_owned(), false)?;
        let mut controller = Controller {
            players: IndexMap::new(), local_player: None, catch_input: true,
            input_buffer: PlaybackBuffer::new(),
            curr_input_state: InputState::default(),
            curr_gen: 0, blocking_time: Instant::now(),
            target_x: Sprite::new(target_x, SpriteOrigin::Centre, None),
            curr_target_pos: None, game
        };
        controller.send_curr_state().convert()?; // Notify server we are finished loading
        Ok(controller)
    }

    pub fn add_player(&mut self, player: Player) -> Rcc<Player> {
        let player_idn = player.id.n;
        let player_ref = wrap_rcc(player);
        self.players.insert(player_idn, player_ref.clone());
        player_ref
    }

    pub fn remove_player(&mut self, id: u16) -> Option<Rcc<Player>> {
        self.players.remove(&id)
    }

    pub fn set_local_player(&mut self, local_player: Rcc<Player>) {
        self.local_player = Some(local_player)
    }

    pub fn buy_ship_mod(&mut self, mod_type: ShipModType) {
        self.curr_input_state.buy_mod = true;
        self.curr_input_state.mod_type = Some(mod_type);
    }

    pub fn add_step(&mut self, step: InputStep) {
        self.input_buffer.add_step(step);
    }

    pub fn is_next_frame_ready(&self) -> bool {
        // The simulation is ready for the next frame, if...
        match self.input_buffer.get_curr_phase() {
            // ...the current phase is imminent or over and the next step is ready
            StepPhase::Over | StepPhase::Imminent => self.input_buffer.get_buffer_size() > 0,
            // ...or the phase is still going on
            StepPhase::Running => true
        }
    }

    pub fn is_block_timed_out(&mut self) -> bool {
        let elapsed_time = self.blocking_time.elapsed();
        if elapsed_time.as_millis() % 2000 <= 25 {
            println!("Blocking simulation until next input step arrives...")
        }
        elapsed_time.as_secs_f32() >= MAX_INPUT_STEP_BLOCK_TIME
    }

    pub fn calc_input_feedback_latency(&self) -> f32 {
        (self.input_buffer.get_buffer_size() + 1) as f32 * STEP_PHASE_TIME_SECS
    }

    pub fn get_curr_gen(&self) -> u64 {
        self.curr_gen
    }

    fn adjust_simulation(&mut self, ctx: &mut Context) {
        let buffered_steps = self.input_buffer.get_buffer_size();
        let optimal_buffer_size = self.input_buffer.estimate_optimal_buffer_size();
        let timestep = {
            // > optimal buffer size means simulation will not catch up to the very latest step,
            // and instead keep the local simulation a couple steps in the past. This shields the player
            // from some stutters and consequent speed-ups by putting a bit of distance between
            // actual and local steps.
            // Optimally, the playback buffer should adjust the gap between actual and
            // local step according to the connection quality/simulation performance.
            if buffered_steps > optimal_buffer_size * 2 { // Hard catch-up required
                DEFAULT_SIMULATION_TIMESTEP * 10.0
            } else if buffered_steps > optimal_buffer_size { // Gently catch up to optimal state
                let acceleration = (buffered_steps - optimal_buffer_size) as f64 * 5.0;
                (DEFAULT_SIMULATION_TIMESTEP
                    + acceleration).min(MAX_ACCELERATED_TIMESTEP) as f64
            } else {
                DEFAULT_SIMULATION_TIMESTEP
            }
        };
        set_timestep(ctx, Timestep::Fixed(timestep));
    }

    fn update_step(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result {
        if self.input_buffer.get_curr_phase() == StepPhase::Over {
            if let Some(next_step) = self.input_buffer.get_next_step() {
                self.apply_step(ctx, next_step, world)?;
                self.send_curr_state().convert()?;
                self.check_sync_state().convert()?;
                self.adjust_simulation(ctx);
            }
        }
        Ok(())
    }

    fn apply_step(&mut self, ctx: &mut Context, step: InputStep, world: &mut World) -> tetra::Result {
        self.curr_gen += 1;
        self.blocking_time = Instant::now();
        assert!(self.curr_gen == step.gen);

        for (sender, state) in step.states.into_iter() {
            self.apply_state(ctx, sender, state, world)?;
        }
        Ok(())
    }

    fn apply_state(&mut self, ctx: &mut Context, sender: u16, state: InputState, world: &mut World)
        -> tetra::Result {
        if let Some(player) = self.players.get(&sender) {
            let disconnect = state.disconnect;
            player.borrow_mut().apply_state(ctx, state, world)?;
            if disconnect {
                self.remove_player(sender);
            }
        } else {
            println!("Failed to apply state. Reason: Player ^{} does not exist", sender);
        }
        Ok(())
    }

    fn send_curr_state(&mut self) -> BbResult {
        self.game.borrow_mut().network.as_mut().unwrap().send_input(
            self.curr_input_state.clone())?;
        self.curr_input_state = InputState::default();
        Ok(())
    }

    fn check_sync_state(&mut self) -> BbResult {
        if self.curr_gen % SYNC_STATE_GEN_INTERVAL == 0 && self.curr_gen > 0 {
            let player_ships = self.players
                .values()
                .map(|p| p.borrow().possessed_ship.clone())
                .collect::<Vec<_>>();
            let state = SyncState::gen_from_ships(self.curr_gen, player_ships.clone());
            {
                let ship_data = self.players.values().map(|p| {
                    let p_ref = p.borrow();
                    let ship_ref = p_ref.possessed_ship.borrow_mut();
                    let translation = ship_ref.transform.get_translation();
                    SyncStateShipData::new(p_ref.id.n,
                        translation.0, translation.1, ship_ref.data.curr_health, ship_ref.treasury.balance)
                }).collect();

                let mut game_ref = self.game.borrow_mut();
                game_ref.diagnostics.add_state(DiagnosticState::new_sync_state(
                    self.curr_gen, self.input_buffer.curr_frames, state, ship_data));
                game_ref.network.as_mut().unwrap().send_packet(Packet::Sync {
                    state
                })
            }
        } else {
            Ok(())
        }
    }
}

impl GameState for Controller {
    fn update(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result {
        // First update step, to start checking at frame zero?
        self.update_step(ctx, world)?;
        self.input_buffer.update(ctx)?;
        self.game.borrow_mut().simulation_settings.update(
            self.curr_gen, self.input_buffer.curr_frames);
        Ok(())

    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(if let Some(mouse_pos) = self.curr_target_pos.as_ref() {
            self.target_x.draw(ctx, mouse_pos.clone(), 0.0);
        })
    }

    fn event(&mut self, ctx: &mut Context, event: Event, _world: &mut World)
        -> tetra::Result {
        if !self.catch_input {
            return Ok(())
        }
        
        if let Some(local_player) = self.local_player.as_ref() {
            match event {
                Event::MouseButtonPressed { button } if button == MouseButton::Right => {
                    let mouse_pos = Some({
                        let game_ref = self.game.borrow();
                        game_ref.cam.get_mouse_pos(ctx)
                    });
                    self.curr_target_pos = mouse_pos.clone();
                    self.curr_input_state.mouse_pos = mouse_pos;
                    self.curr_input_state.rmb = true;
                    self.curr_input_state.r = false;
                },
                Event::KeyPressed { key } => {
                    match key {
                        Key::R => {
                            let mouse_pos = Some({
                                let game_ref = self.game.borrow();
                                game_ref.cam.get_mouse_pos(ctx)
                            });
                            self.curr_target_pos = mouse_pos.clone();
                            self.curr_input_state.mouse_pos = mouse_pos;
                            self.curr_input_state.r = true;
                            self.curr_input_state.rmb = false;
                        },
                        Key::Space => {
                            self.curr_input_state.q = true;
                            self.curr_input_state.e = true;
                        },
                        Key::Q => self.curr_input_state.q = true,
                        Key::E => self.curr_input_state.e = true,
                        Key::Tab => {
                            let curr_pos = local_player.borrow()
                                .possessed_ship.borrow().transform.get_translation().0;
                            self.game.borrow_mut().cam.centre_on(curr_pos);
                        }
                        _ => ()
                    }
                },
                _ => ()
            }
        }
        Ok(())
    }
}
