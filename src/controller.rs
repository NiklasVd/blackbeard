use std::{collections::HashMap, time::Instant};
use tetra::{Context, Event, State, input::{Key, MouseButton}, time::{Timestep, set_timestep}};
use crate::{BbResult, CannonSide, GC, Player, Rcc, Sprite, SpriteOrigin, TransformResult, V2, entity::GameState, input_pool::STEP_PHASE_TIME_SECS, packet::{InputState, InputStep, Packet}, playback_buffer::PlaybackBuffer, ship_mod::{AMMO_UPGRADE_MOD_COST, AmmoUpgradeMod, HARBOUR_REPAIR_COST, ShipModType}, sync_checker::{SYNC_STATE_GEN_INTERVAL, SyncState}, world::World, wrap_rcc};

pub const MAX_INPUT_STEP_BLOCK_TIME: f32 = 15.0;
pub const DEFAULT_SIMULATION_TIMESTEP: f64 = 60.0;
pub const ACCELERATED_SIMULATION_TIMESTEP: f64 = DEFAULT_SIMULATION_TIMESTEP * 3.0;

pub struct Controller {
    pub players: HashMap<u16, Rcc<Player>>,
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
            players: HashMap::new(), local_player: None, catch_input: true,
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
        println!("Attempting to buy {:?} mod", &mod_type);
        self.curr_input_state.buy_mod = true;
        self.curr_input_state.mod_type = Some(mod_type);
    }

    pub fn add_step(&mut self, step: InputStep) {
        self.input_buffer.add_step(step);
    }

    pub fn is_next_step_ready(&self) -> bool {
        !(self.input_buffer.is_phase_over() &&
            (self.input_buffer.get_buffer_size() == 0))
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
        // After multiple tests, it seems that the simulation does not experience desync
        // when ships are simply travelling forward. Yet as soon as they collide,
        // the local positions quickly drift apart. There could be one solution:
        // The timers used for ship stuns ignore timestep changes.
        // A timer with duration 3s will take the same time to complete on 60
        // or 180 FPS. While the frame rate rises, the delta time falls, balancing out
        // the increment on the curr. timer duration. That will obviously
        // throw the synchronisation into chaos.
        // This, combined with the physics engine delta time adjustment, may yet
        // prevent desyncs from occuring.
        let buffered_steps = self.input_buffer.get_buffer_size();
        let timestep = {
            if buffered_steps > 1 {
                println!("Input feedback delayed by {} steps. Accelerate simulation to {} frames/s",
                    buffered_steps - 1, ACCELERATED_SIMULATION_TIMESTEP);
                ACCELERATED_SIMULATION_TIMESTEP
            } else {
                DEFAULT_SIMULATION_TIMESTEP
            }
        };
        // Physics engine needs no adjustment. This would lead to same effect as for timers.
        // When adjusting physics integration params delta time to the accelerated value,
        // the simulation would run at the exact same speed, merely with higher frame rate.
        set_timestep(ctx, Timestep::Fixed(timestep));
    }

    fn update_step(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result {
        if self.input_buffer.is_phase_over() {
            self.adjust_simulation(ctx);
            if let Some(next_step) = self.input_buffer.get_next_step() {
                self.apply_step(ctx, next_step, world)?;
                self.send_curr_state().convert()?;
            }
        }
        Ok(())
    }

    fn apply_step(&mut self, ctx: &mut Context, step: InputStep, world: &mut World) -> tetra::Result {
        self.curr_gen += 1;
        self.check_sync_state().convert()?;

        self.blocking_time = Instant::now();
        for (sender, state) in step.states.into_iter() {
            self.apply_state(ctx, sender, state, world)?;
        }
        Ok(())
    }

    fn apply_state(&mut self, ctx: &mut Context, sender: u16, state: InputState, world: &mut World)
        -> tetra::Result {
        if let Some(player) = self.players.get(&sender) {
            let player_ref = player.borrow();
            // Handle buy_mod before borrowing ship ref
            if state.buy_mod && player_ref.possessed_ship.borrow().is_in_harbour {
                println!("Player {:?} is purchasing a mod at the harbour.",
                    player_ref.id);
                if let Some(mod_type) = state.mod_type {
                    let ship_mod = match &mod_type {
                        ShipModType::Repair => {
                            let mut ship_ref = player_ref.possessed_ship.borrow_mut();
                            if ship_ref.treasury.balance < HARBOUR_REPAIR_COST {
                                println!("Not enough escudos to repair ship!");
                            } else {
                                ship_ref.repair();
                                ship_ref.treasury.spend(HARBOUR_REPAIR_COST);
                                println!("Player {:?} repaired their ship.", player_ref.id);
                            }
                            None
                        },
                        ShipModType::AmmoUpgrade => {
                            let mut ship_ref = player_ref.possessed_ship.borrow_mut();
                            if ship_ref.treasury.balance < AMMO_UPGRADE_MOD_COST {
                                println!("Not enough escudos to buy ammo upgrade!");
                                None
                            } else {
                                ship_ref.treasury.spend(AMMO_UPGRADE_MOD_COST);
                                std::mem::drop(ship_ref);
                                let ship_mod = AmmoUpgradeMod::new(ctx,
                                    player_ref.possessed_ship.clone(), self.game.clone())?;
                                Some(ship_mod)
                            }
                        },
                        ShipModType::CannonUpgrade => todo!(),
                    };
                    if let Some(ship_mod) = ship_mod {
                        player_ref.possessed_ship.borrow_mut().apply_mod(ship_mod);
                        println!("Player {:?} purchased and applied {:?} mod at harbour.",
                            player_ref.id, mod_type);
                    }
                }
            }

            let mut ship_ref = player_ref.possessed_ship.borrow_mut();
            if state.q {
                ship_ref.shoot_cannons_on_side(ctx, CannonSide::Bowside, world)?;
            }
            if state.e {
                ship_ref.shoot_cannons_on_side(ctx, CannonSide::Portside, world)?;
            }

            if let Some(mouse_pos) = state.mouse_pos {
                ship_ref.set_target_pos(mouse_pos, state.r);
            }
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
        if self.curr_gen > 0 && self.curr_gen % SYNC_STATE_GEN_INTERVAL == 0 {
            let mut player_ships = self.players.values().collect::<Vec<_>>();
            player_ships.sort_unstable_by(|a, b| a.borrow().id.n.cmp(&b.borrow().id.n));
            let player_ships = player_ships.into_iter()
                .map(|p| p.borrow().possessed_ship.clone()).collect();
            let state = SyncState::gen_from_ships(self.curr_gen, player_ships);
            println!("Generated sync state. Hash = {}", state.hash);
            self.game.borrow_mut().network.as_mut().unwrap().send_packet(Packet::Sync {
                state
            })
        } else {
            Ok(())
        }
    }
}

impl GameState for Controller {
    fn update(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result {
        self.input_buffer.update(ctx)?;
        self.update_step(ctx, world)
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        if let Some(mouse_pos) = self.curr_target_pos.as_ref() {
            self.target_x.draw(ctx, mouse_pos.clone(), 0.0);
        }
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event, world: &mut World)
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
                        _ => ()
                    }
                },
                _ => ()
            }
        }

        Ok(())
    }
}
