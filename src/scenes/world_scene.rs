use tetra::{Context, Event, State, input::Key};
use crate::{BbError, BbErrorType, BbResult, Controller, GC, ID, Player, Rcc, TransformResult, V2, WorldEvent, button::{Button, DefaultButton}, chat::Chat, entity::{Entity, GameState}, grid::{Grid, UIAlignment, UILayout}, image::Image, label::{FontSize, Label}, menu_scene::MenuScene, net_controller::NetController, packet::{InputStep, Packet}, peer::DisconnectReason, ship::ShipType, ship_mod::{HARBOUR_REPAIR_COST, ShipModType}, ui_element::{DefaultUIReactor, UIElement}, world::World};
use super::scenes::{Scene, SceneType};

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
        
        world_scene.world.add_island(ctx, V2::new(800.0, 500.0), 0.6, 1).convert()?;
        world_scene.world.add_island(ctx, V2::new(150.0, 1000.0), 4.0, 2).convert()?;
        world_scene.world.add_island(ctx, V2::new(-200.0, -800.0), 0.0, 1).convert()?;
        world_scene.world.add_island(ctx, V2::new(-900.0, 200.0), 0.0, 3).convert()?;
        world_scene.world.add_island(ctx, V2::new(1200.0, -500.0), 0.0, 3).convert()?;
        world_scene.world.add_island(ctx, V2::new(-600.0, -300.0), 1.0, 2).convert()?;
        world_scene.world.add_island(ctx, V2::new(2000.0, 0.0), 2.0, 1).convert()?;
        world_scene.world.add_island(ctx, V2::new(2300.0, 850.0), 3.5, 2).convert()?;
        world_scene.world.add_island(ctx, V2::new(1800.0, 1200.0), 0.0, 3).convert()?;

        world_scene.world.add_harbour(ctx, "Port Elisabeth",
            V2::new(1200.0, 780.0), 5.0).convert()?;

        world_scene.init_players(ctx, players)?;
        world_scene.ui.set_local_player(
            world_scene.controller.local_player.as_ref().unwrap().clone());
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

    fn update_world(&mut self, ctx: &mut Context) -> tetra::Result {
        let is_next_step_ready = self.controller.is_next_step_ready();
        self.game.borrow_mut().simulation_state = is_next_step_ready;
        
        if is_next_step_ready {
            self.controller.update(ctx, &mut self.world)?;
            self.world.update(ctx)
        } else if self.controller.is_block_timed_out() {
            println!("Failed to procure next input step in time. Leaving match...");
            self.leave_match().convert()
        } else {
            Ok(())
        }
    }

    fn event_world(&mut self, ctx: &mut Context, event: Event) -> tetra::Result {
        if self.controller.is_next_step_ready() {
            self.controller.event(ctx, event.clone(), &mut self.world)?;
            self.world.event(ctx, event.clone())
        } else {
            Ok(())
        }
    }

    fn update_menu_ui(&mut self) -> BbResult {
        let curr_gen = self.controller.get_curr_gen();
        if curr_gen % 25 == 0 && curr_gen != self.ui.last_info_update_gen {
            let (_, _, avg) = self.controller.input_buffer.calc_latency();
            // FIX: Feedback latency is calculated right after latest step is applied, leading to zero
            let feedback_lat = self.controller.calc_input_feedback_latency();
            self.ui.update_match_info(&format!("Latency: Step ~ {:.2}s, Feedback ~ {:.2}s",
                avg, feedback_lat));
            self.ui.last_info_update_gen = curr_gen;
        }

        if self.ui.leave_button.borrow().is_pressed() {
            self.leave_match()
        } else {
            Ok(())
        }
    }

    fn update_harbour_ui(&mut self) -> BbResult {
        if !self.controller.local_player.as_ref().unwrap().borrow()
            .possessed_ship.borrow().is_in_harbour {
            return Ok(())
        }
        
        if self.ui.harbour_ui.repair_ship_button.borrow().is_pressed() {
            self.controller.buy_ship_mod(ShipModType::Repair)
        }
        if self.ui.harbour_ui.buy_ammo_upgrade_button.borrow().is_pressed() {
            self.controller.buy_ship_mod(ShipModType::AmmoUpgrade);
        }
        Ok(())
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
        self.update_harbour_ui().convert()?;
        // Don't react to input if player is writing in chat
        self.controller.catch_input = !self.ui.is_chat_focused();
        self.handle_received_packets(ctx).convert()?;
        self.update_world(ctx)
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.controller.draw(ctx)?;
        self.world.draw(ctx)
    }

    fn event(&mut self, ctx: &mut Context, event: Event)
        -> tetra::Result {
        self.event_world(ctx, event.clone())?;
        self.ui.event(ctx, event)
    }
}

impl NetController for WorldScene {
    fn poll_received_packets(&mut self, ctx: &mut Context) -> BbResult<Option<(Packet, u16)>> {
        self.game.borrow_mut().network.as_mut().unwrap().poll_received_packets()
    }

    fn on_connection_lost(&mut self, ctx: &mut Context, reason: DisconnectReason) -> BbResult {
        println!("Lost connection to server! Aborting game...");
        self.leave_match() // Previously only set self.back_to_menu to true. Problem if connection is already terminated when calling network.disconnect()? 
    }
    
    fn on_player_disconnect(&mut self, ctx: &mut Context, id: u16, reason: DisconnectReason)
        -> BbResult {
        if let Some(player) = self.controller.remove_player(id) {
            let player_ref = player.borrow_mut();
            player_ref.possessed_ship.borrow_mut().destroy();
            self.ui.update_players(ctx, self.game.borrow().network.as_ref().unwrap()
                .client.get_connections()).convert()?;
            //println!("Player with ID {} disconnected. Reason: {:?}", id, reason);
            self.ui.chat.add_line(ctx,
                &format!("{:?} left the game. Reason: {:?}.", player_ref.id, reason)).convert()?;
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
        self.ui.chat.add_message(ctx, sender.as_str(), text.as_str()).convert()
    }
}

struct WorldSceneUI {
    // Add event log/chat (combined?)
    pub chat: Chat,
    menu_button: Rcc<DefaultButton>,
    menu_grid: Rcc<Grid>,
    leave_button: Rcc<DefaultButton>,
    match_info_label: Rcc<Label>,
    last_info_update_gen: u64,
    players_grid: Rcc<Grid>,
    health_label: Rcc<Label>,
    escudos_label: Rcc<Label>,
    harbour_ui: HarbourUI,
    ship_stats_panel: Rcc<Grid>,
    local_player: Option<Rcc<Player>>,
    game: GC
}

impl WorldSceneUI {
    fn new(ctx: &mut Context, game: GC, grid: &mut Grid)
        -> tetra::Result<WorldSceneUI> {
        let menu_button = grid.add_element(Button::new(ctx, "-", V2::new(20.0, 20.0),
            1.0, DefaultUIReactor::new(), game.clone())?);

        let mut menu_grid = Grid::default(ctx, UIAlignment::Vertical, V2::zero(),
            V2::new(100.0, 20.0), 0.0)?;
        menu_grid.set_visibility(false);
        let leave_button = menu_grid.add_element(Button::new(ctx, "Leave Match",
            V2::new(120.0, 35.0), 1.0, DefaultUIReactor::new(), game.clone())?);
        let match_info_label = menu_grid.add_element(Label::new(ctx, "Latency: Step ~ 0.00s, Feedback ~ 0.00s",
            FontSize::Small, 4.0, game.clone())?);
        menu_grid.add_element(Label::new(ctx, "Connected Players", FontSize::Normal,
            4.0, game.clone())?);
        let players_grid = menu_grid.add_element(Grid::default(ctx, UIAlignment::Vertical,
            V2::zero(), V2::new(120.0, 300.0), 2.0)?);
        let menu_grid = grid.add_element(menu_grid);

        let mut player_info_grid = Grid::new(ctx, UIAlignment::Horizontal,
            UILayout::TopRight, V2::new(330.0, 35.0), 0.0)?;
        let health_label = player_info_grid.add_element(Label::new(ctx,
            "1000/1000 Health", FontSize::Normal, 1.0, game.clone())?);
        let escudos_label = player_info_grid.add_element(Label::new(ctx,
            "1000 Escudos", FontSize::Normal, 1.0, game.clone())?);
        grid.add_element(player_info_grid);

        let chat = Chat::new(ctx, UILayout::BottomLeft, grid, game.clone())?;
        let harbour_ui = HarbourUI::new(ctx, grid, game.clone())?;

        let ship_stats_grid = grid.add_element(Grid::new(ctx, UIAlignment::Horizontal,
            UILayout::BottomRight, V2::new(350.0, 80.0), 0.0)?);

        Ok(WorldSceneUI {
            chat, menu_button, menu_grid, leave_button, match_info_label,
            last_info_update_gen: 0, players_grid,
            health_label, escudos_label, harbour_ui, ship_stats_panel: ship_stats_grid,
            local_player: None, game
        })
    }

    pub fn set_local_player(&mut self, player: Rcc<Player>) {
        self.local_player = Some(player);
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
            players_grid_ref.add_element(Label::new(ctx, format!(" {:?} {}", player,
                match player.n {
                    0 => "(Host)",
                    _ => ""
                }).as_str(), FontSize::Normal, 2.0, self.game.clone())?);
        }
        Ok(())
    }

    pub fn is_chat_focused(&self) -> bool {
        self.chat.is_focused()
    }

    fn update_world_events(&mut self, ctx: &mut Context) -> tetra::Result {
        let events = {
            let mut game_ref = self.game.borrow_mut();
            game_ref.world.flush_events().into_iter()
        };
        for event in events {
            self.chat.add_line(ctx, &match event {
                WorldEvent::PlayerSunkByCannon(a, b) =>
                    format!("{} sunk {} with a cannon shot!", a, b),
                WorldEvent::PlayerSunkByRamming(a, b) =>
                    format!("{} sunk {} by ramming!", a, b),
                WorldEvent::PlayerSunkByAccident(a) =>
                    format!("{} sunk their own ship by accident!", a)
            })?;
        }
        Ok(())
    }

    fn update_ship_stats_panel(&mut self, ctx: &mut Context) -> tetra::Result {
        let player_ref = self.local_player.as_ref().unwrap().borrow();
        let ship_ref = player_ref.possessed_ship.borrow();
        let mut panel_ref = self.ship_stats_panel.borrow_mut();
        if ship_ref.mods.len() != panel_ref.elements.len() {
            panel_ref.clear_elements();
            for ship_mod in ship_ref.mods.iter() {
                panel_ref.add_element(Image::from(ctx, V2::new(50.0, 50.0), 5.0,
                    ship_mod.get_icon())?);
            }
        }
        Ok(())
    }
}

impl State for WorldSceneUI {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if self.menu_button.borrow().is_pressed() {
            self.toggle_menu_visibility();
        }
        if let Some(message) = self.chat.check_messages(ctx) {
            self.game.borrow_mut().network.as_mut().unwrap().send_packet(Packet::ChatMessage {
                message
            }).convert()?;
        }
        if let Some(local_player) = self.local_player.as_ref() {
            let player_ref = local_player.borrow();
            let ship_ref = player_ref.possessed_ship.borrow();
            self.health_label.borrow_mut().set_text(
                &format!("{}/{} Health", ship_ref.curr_health, ship_ref.attr.health));
            self.escudos_label.borrow_mut().set_text(
                &format!("{} Escudos", ship_ref.treasury.balance));
        }

        self.update_world_events(ctx)?;
        self.update_ship_stats_panel(ctx)
    }

    fn event(&mut self, ctx: &mut Context, event: Event) -> tetra::Result {
        let is_in_harbour = self.local_player.as_ref().unwrap().borrow()
            .possessed_ship.borrow().is_in_harbour;
        if !is_in_harbour {
            self.harbour_ui.set_visibility(false);
        }
        match event {
            Event::KeyPressed { key } => {
                match key {
                    Key::Escape => self.toggle_menu_visibility(),
                    Key::T => {
                        if is_in_harbour {
                            let invisible = self.harbour_ui.grid.borrow().is_invisible();
                            self.harbour_ui.set_visibility(invisible);
                        }
                    },
                    _ => ()
                };
            },
            _ => ()
        }
        Ok(())
    }
}

struct HarbourUI {
    grid: Rcc<Grid>,
    repair_ship_button: Rcc<DefaultButton>,
    buy_ammo_upgrade_button: Rcc<DefaultButton>
}

impl HarbourUI {
    pub fn new(ctx: &mut Context, grid: &mut Grid, game: GC) -> tetra::Result<HarbourUI> {
        let mut harbour_grid = Grid::new_bg(ctx, UIAlignment::Vertical,
            UILayout::Centre, V2::new(225.0, 385.0), 0.0,
            Some("UI/Background.png".to_owned()), Some(game.clone()))?;
        harbour_grid.set_visibility(false);
        harbour_grid.add_element(Label::new(ctx, "Harbour", FontSize::Header, 2.0,
            game.clone())?);
        let repair_ship_button = harbour_grid.add_element(Button::new(ctx,
            &format!("Repair Ship ({})", HARBOUR_REPAIR_COST), V2::new(140.0, 35.0),
            2.0, DefaultUIReactor::new(),
            game.clone())?);
        harbour_grid.add_element(Label::new(ctx,
            "Ammo Upgrade: Increases cannon ball damage +5.", FontSize::Small, 2.0, game.clone())?);
        let buy_ammo_upgrade_button = harbour_grid.add_element(Button::new(ctx,
            "Ammo Upgrade (100)", V2::new(180.0, 35.0), 2.0, DefaultUIReactor::new(),
            game.clone())?);
        let harbour_grid = grid.add_element(harbour_grid);
        Ok(HarbourUI {
            grid: harbour_grid, repair_ship_button, buy_ammo_upgrade_button
        })
    }

    pub fn set_visibility(&mut self, state: bool) {
        self.grid.borrow_mut().set_visibility(state);
    }
}
