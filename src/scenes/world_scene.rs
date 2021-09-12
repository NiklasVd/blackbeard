use std::time::{Duration, Instant};
use tetra::{Context, Event, State, input::Key};
use crate::{BbError, BbErrorType, BbResult, Controller, Entity, GC, GameState, ID, Player, Rcc, ShipType, TransformResult, V2, button::{Button, DefaultButton}, chat::Chat, grid::{Grid, UIAlignment, UILayout}, label::Label, menu_scene::MenuScene, net_controller::NetController, packet::{InputStep, Packet}, peer::DisconnectReason, ui_element::{DefaultUIReactor, UIElement}, world::World};
use super::scenes::{Scene, SceneType};

pub const MAX_INPUT_STEP_BLOCK_DURATION: u64 = 60 * 10;

pub struct WorldScene {
    pub controller: Controller,
    pub world: World,
    grid: Grid,
    ui: WorldSceneUI,
    back_to_menu: bool,
    game: GC
}

impl WorldScene {
    pub fn new(ctx: &mut Context, players: Vec<(ID, ShipType)>, game: GC) -> BbResult<WorldScene> {
        let mut grid = Grid::default(ctx, UIAlignment::Horizontal,
            V2::zero(), V2::one() * 200.0, 0.0).convert()?;
        let mut ui = WorldSceneUI::new(ctx, game.clone(), &mut grid).convert()?;
        ui.update_players(ctx, players.iter().map(|(id, ..)| id.clone()).collect()).convert()?;

        let mut world_scene = WorldScene {
            controller: Controller::new(ctx, game.clone()).convert()?,
            world: World::new(ctx, game.clone()),
            grid, ui, back_to_menu: false, game: game.clone()
        };
        world_scene.init_players(ctx, players)?;
        
        world_scene.world.add_island(ctx, V2::new(800.0, 500.0), 0.6, 1).convert()?;
        world_scene.world.add_island(ctx, V2::new(150.0, 1000.0), 4.0, 2).convert()?;
        world_scene.world.add_island(ctx, V2::new(-200.0, -800.0), 0.0, 1).convert()?;
        world_scene.world.add_island(ctx, V2::new(-900.0, 200.0), 0.0, 3).convert()?;
        world_scene.world.add_island(ctx, V2::new(1200.0, -500.0), 0.0, 3).convert()?;
        world_scene.world.add_island(ctx, V2::new(-600.0, -300.0), 1.0, 2).convert()?;

        Ok(world_scene)
    }

    pub fn add_player(&mut self, ctx: &mut Context, id: ID, ship_type: ShipType) -> BbResult<Rcc<Player>> {
        let ship = self.world.add_player_ship(ctx, id.clone(), ship_type).convert()?;
        Ok(self.controller.add_player(Player::new(id, ship)))
    }

    pub fn leave_match(&mut self) -> BbResult {
        self.game.borrow_mut().network.as_mut().unwrap().disconnect(
            DisconnectReason::Timeout)?;
        self.back_to_menu = true;
        Ok(())
    }

    fn init_players(&mut self, ctx: &mut Context, players: Vec<(ID, ShipType)>) -> BbResult {
        let local_id = {
            if let Some(network) = self.game.borrow().network.as_ref() {
                network.client.get_local_id()
            } else {
                None
            }
        };
        for (id, ship_type) in players.into_iter() {
            let player_instance = self.add_player(ctx, id.clone(), ship_type)?;
            if let Some(local_id) = local_id.as_ref() {
                if &id != local_id {
                    continue
                }
            }
            self.controller.set_local_player(player_instance);
        }
        Ok(())
    }

    fn update_menu_ui(&mut self) -> BbResult {
        if self.ui.leave_button.borrow().is_pressed() {
            self.leave_match()
        } else {
            Ok(())
        }
    }

    fn event_menu_ui(&mut self, event: Event) {
        match event {
            Event::KeyPressed { key } if key == Key::Escape =>
                self.ui.toggle_menu_visibility(),
            _ => ()
        }
    }
}

impl Scene for WorldScene {
    fn get_grid(&self) -> &Grid {
        &self.grid
    }

    fn get_grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    fn poll(&self, ctx: &mut Context) -> BbResult<Option<Box<dyn Scene>>> {
        Ok(if self.back_to_menu {
            Some(Box::new(MenuScene::new(ctx, self.game.clone()).convert()?))
        }
        else {
            None
        })
    }

    fn get_type(&self) -> SceneType {
        SceneType::World
    }
}

impl State for WorldScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.ui.update(ctx)?;
        self.update_menu_ui().convert()?;
        // Don't react to input if player is writing in chat
        self.controller.catch_input = !self.ui.is_chat_focused();

        self.handle_received_packets(ctx).convert()?;
        self.controller.update(ctx, &mut self.world)?;
        self.world.update(ctx)
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.controller.draw(ctx)?;
        self.world.draw(ctx)
    }

    fn event(&mut self, ctx: &mut Context, event: Event)
        -> tetra::Result {
        self.controller.event(ctx, event.clone(), &mut self.world)?;
        self.world.event(ctx, event.clone())?;
        self.event_menu_ui(event.clone());
        Ok(())
    }
}

impl NetController for WorldScene {
    fn poll_received_packets(&mut self, ctx: &mut Context) -> BbResult<Option<(Packet, u16)>> {
        let time = Instant::now();
        loop {
            let packet = {
                self.game.borrow_mut().network.as_mut().unwrap().poll_received_packets()?
            };
            if self.controller.wait_next_step() {
                match packet {
                    Some((packet, sender)) => {
                        match &packet {
                                &Packet::InputStep { .. } => return Ok(Some((packet, sender))),
                            _ => self.handle_packets(ctx, (packet, sender))?
                        }
                    },
                    None => ()
                }

                std::thread::sleep(Duration::from_millis(1));
                let elapsed_secs = time.elapsed().as_secs();
                if elapsed_secs % 2 == 0 {
                    println!("Blocking until next input step arrives from server...");
                }
                if elapsed_secs >= MAX_INPUT_STEP_BLOCK_DURATION {
                    println!("Failed to procure next input step in time. Terminating connection to server...");
                    self.leave_match()?;
                    return Ok(None)
                }
                continue
            } else {
                return Ok(packet)
            }
        }
    }

    fn on_connection_lost(&mut self, ctx: &mut Context, reason: DisconnectReason) -> BbResult {
        println!("Lost connection to server! Aborting game...");
        self.leave_match() // Previously only set self.back_to_menu to true. Problem if connection is already terminated when calling network.disconnect()? 
    }
    
    fn on_player_disconnect(&mut self, ctx: &mut Context, id: u16, reason: DisconnectReason) -> BbResult {
        if let Some(player) = self.controller.remove_player(id) {
            player.borrow_mut().possessed_ship.borrow_mut().destroy();
            self.ui.update_players(ctx, self.game.borrow().network.as_ref().unwrap()
                .client.get_connections()).convert()?;
            println!("Player with ID {} disconnected. Reason: {:?}", id, reason);
            Ok(())
        } else {
            Err(BbError::Bb(BbErrorType::InvalidPlayerID(id)))
        }
    }

    fn on_input_step(&mut self, ctx: &mut Context, step: InputStep) -> BbResult {
        self.controller.add_step(step);
        Ok(())
    }

    fn on_chat_message(&mut self, ctx: &mut Context, text: String, sender: u16) -> BbResult {
        let sender = {
            self.game.borrow().network.as_ref().unwrap().get_connection_name(sender)
        };
        self.ui.add_chat_message(ctx, sender.as_str(), text.as_str()).convert()
    }
}

struct WorldSceneUI {
    // Add event log/chat (combined?)
    chat: Chat,
    menu_button: Rcc<DefaultButton>,
    menu_grid: Rcc<Grid>,
    leave_button: Rcc<DefaultButton>,
    match_info_label: Rcc<Label>,
    players_grid: Rcc<Grid>,
    game: GC
}

impl WorldSceneUI {
    fn new(ctx: &mut Context, game: GC, grid: &mut Grid) -> tetra::Result<WorldSceneUI> {
        let menu_button = grid.add_element(Button::new(ctx, "-", V2::new(20.0, 20.0),
            1.0, DefaultUIReactor::new(), game.clone())?);

        let mut menu_grid = Grid::default(ctx, UIAlignment::Vertical, V2::zero(),
            V2::new(100.0, 20.0), 0.0)?;
        menu_grid.set_visibility(false);
        let leave_button = menu_grid.add_element(Button::new(ctx, "Leave Match",
            V2::new(120.0, 35.0), 1.0, DefaultUIReactor::new(), game.clone())?);
        let match_info_label = menu_grid.add_element(Label::new(ctx, "Connected to Server",
            false, 2.0, game.clone())?);
        menu_grid.add_element(Label::new(ctx, "Connected Players", false, 2.0, game.clone())?);
        let players_grid = menu_grid.add_element(Grid::default(ctx, UIAlignment::Vertical,
            V2::zero(), V2::new(120.0, 300.0), 2.0)?);
        let menu_grid = grid.add_element(menu_grid);

        let chat = Chat::new(ctx, UILayout::BottomLeft, grid, game.clone())?;
        Ok(WorldSceneUI {
            chat, menu_button, menu_grid, leave_button, match_info_label, players_grid, game
        })
    }

    pub fn add_chat_message(&mut self, ctx: &mut Context, sender: &str, msg: &str)
        -> tetra::Result {
        self.chat.add_message(ctx, sender, msg)
    }

    pub fn toggle_menu_visibility(&mut self) {
        let mut menu_grid_ref = self.menu_grid.borrow_mut();
        let state = menu_grid_ref.is_invisible();
        menu_grid_ref.set_visibility(state);
    }

    pub fn update_match_info(&mut self, text: &str) {
        self.match_info_label.borrow_mut().set_text(text);
    }

    pub fn update_players(&mut self, ctx: &mut Context, players: Vec<ID>) -> tetra::Result {
        let mut players_grid_ref = self.players_grid.borrow_mut();
        players_grid_ref.clear_elements();
        for player in players.into_iter() {
            players_grid_ref.add_element(Label::new(ctx, format!("{:?} {}", player,
                match player.n {
                    0 => "(Host)",
                    _ => ""
                }).as_str(), false, 2.0, self.game.clone())?);
        }
        Ok(())
    }

    pub fn is_chat_focused(&self) -> bool {
        self.chat.is_focused()
    }

    pub fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if self.menu_button.borrow().is_pressed() {
            self.toggle_menu_visibility();
        }
        if let Some(message) = self.chat.check_messages(ctx) {
            self.game.borrow_mut().network.as_mut().unwrap().send_packet(Packet::ChatMessage {
                message
            }).convert()?;
        }
        Ok(())
    }
}
