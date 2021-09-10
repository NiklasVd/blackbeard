use std::{collections::HashMap};
use tetra::{Context, Event, State, input::{Key, MouseButton}};
use crate::{BbResult, CannonSide, GC, GameState, Player, Rcc, Sprite, SpriteOrigin, TransformResult, V2, packet::{InputState, InputStep}, playback_buffer::PlaybackBuffer, world::World, wrap_rcc};

pub struct Controller {
    pub players: HashMap<u16, Rcc<Player>>,
    pub local_player: Option<Rcc<Player>>,
    pub catch_input: bool,
    input_buffer: PlaybackBuffer,
    curr_input_state: InputState,
    curr_gen: u64,
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
            curr_gen: 0, target_x: Sprite::new(target_x, SpriteOrigin::Centre, None),
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

    pub fn add_step(&mut self, step: InputStep) {
        self.input_buffer.add_step(step);
    }

    pub fn wait_next_step(&self) -> bool {
        self.input_buffer.is_phase_over() & (self.input_buffer.get_buffered_step_count() == 0)
    }

    fn update_step(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result {
        if self.input_buffer.is_phase_over() {
            if let Some(next_step) = self.input_buffer.get_next_step() {
                self.apply_step(ctx, next_step, world)?;
                self.send_curr_state().convert()?;
            }
        }
        Ok(())
    }

    fn apply_step(&mut self, ctx: &mut Context, step: InputStep, world: &mut World) -> tetra::Result {
        self.curr_gen += 1;
        for (sender, state) in step.states.into_iter() {
            self.apply_state(ctx, sender, state, world)?;
        }
        Ok(())
    }

    fn apply_state(&mut self, ctx: &mut Context, sender: u16, state: InputState, world: &mut World)
        -> tetra::Result {
        if let Some(player) = self.players.get(&sender) {
            let player_ref = player.borrow();
            let mut ship_ref = player_ref.possessed_ship.borrow_mut();
            if state.q {
                ship_ref.shoot_cannons_on_side(ctx, CannonSide::Bowside, world)?;
            }
            if state.e {
                ship_ref.shoot_cannons_on_side(ctx, CannonSide::Portside, world)?;
            }

            if state.rmb {
                if let Some(mouse_pos) = state.mouse_pos {
                    ship_ref.set_target_pos(mouse_pos);
                }
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
                },
                Event::KeyPressed { key } => {
                    match key {
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
