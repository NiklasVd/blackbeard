use std::collections::HashMap;
use tetra::{Context, State};
use crate::{BbResult, GC, ID, PlayerParams, Rcc, TransformResult, V2, button::{Button, DefaultButton}, chat::Chat, game_settings::GameSettings, grid::{Grid, UIAlignment, UILayout}, label::{FontSize, Label}, loading_scene::LoadingScene, menu_scene::MenuScene, net_controller::NetController, net_settings::NetSettings, network::Network, packet::{GamePhase, Packet}, peer::DisconnectReason, rand_u64, ship::ShipType, ui_element::{DefaultUIReactor, UIElement}};
use super::scenes::{Scene, SceneType};

pub struct LobbyScene {
    pub grid: Grid,
    ui: LobbySceneUI,
    players: HashMap<u16, PlayerParams>,
    world_seed: u64,
    game_started: bool,
    disconnected: bool,
    game_settings: GameSettings,
    game: GC
}

impl LobbyScene {
    pub fn create(ctx: &mut Context, port: u16, settings: NetSettings, game: GC) -> BbResult<LobbyScene> {
        {
            let mut game_ref = game.borrow_mut();
            let name = game_ref.settings.name.to_owned();
            game_ref.network = Some(Network::create(port, name, settings)?);
        }
        Self::new(ctx, game)
    }

    pub fn join(ctx: &mut Context, endpoint: &str, game: GC)
        -> BbResult<LobbyScene> {
        {
            let mut game_ref = game.borrow_mut();
            let name = game_ref.settings.name.to_owned();
            game_ref.network = Some(Network::join(endpoint, name)?);
        }
        Self::new(ctx, game)
    }

    fn new(ctx: &mut Context, game: GC) -> BbResult<LobbyScene> {
        let mut grid = Grid::default(ctx, UIAlignment::Horizontal,
            V2::zero(), V2::one() * 500.0, 5.0).convert()?;
        let ui = LobbySceneUI::new(ctx, &mut grid, game.clone())?;

        Ok(LobbyScene {
            grid, ui, players: HashMap::new(), world_seed: 0, game_started: false,
            disconnected: false, game_settings: GameSettings::default(), game
        })
    }

    fn update_ship_selection(&mut self) -> BbResult {
        if let Some(selected_ship_type) = self.ui.selected_ship_type.take() {
            self.game.borrow_mut().network.as_mut().unwrap().send_packet(Packet::Selection {
                mode: true, ship: Some(selected_ship_type), settings: None
            })?;
            println!("Selected ship type: {:?}", selected_ship_type);
            Ok(())
        } else {
            Ok(())
        }
    }
}

impl Scene for LobbyScene {
    fn get_type(&self) -> SceneType {
        SceneType::Lobby
    }

    fn get_grid(&self) -> &Grid {
        &self.grid
    }

    fn get_grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    fn poll(&self, ctx: &mut Context) -> BbResult<Option<Box<dyn Scene + 'static>>> {
        Ok(if self.game_started {
            Some(Box::new(LoadingScene::new(ctx, self.players.values().map(|p| p.clone())
                .collect(), self.world_seed, self.game.clone()).convert()?))
        } else if self.ui.disconnect_button.borrow().is_pressed() {
            self.game.borrow_mut().network.as_mut().unwrap().disconnect(DisconnectReason::Manual)?;
            Some(Box::new(MenuScene::new(ctx, self.game.clone()).convert()?))
        } else if self.disconnected {
            Some(Box::new(MenuScene::new(ctx, self.game.clone()).convert()?))
        } else {
            None
        })
    }
}

impl State for LobbyScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.ui.update(ctx).convert()?;
        self.update_ship_selection().convert()?;
        
        self.handle_received_packets(ctx).convert()
    }
}

impl NetController for LobbyScene {
    fn poll_received_packets(&mut self, ctx: &mut Context) -> BbResult<Option<(Packet, u16)>> {
        self.game.borrow_mut().network.as_mut().unwrap().poll_received_packets()
    }

    fn on_establish_connection(&mut self, ctx: &mut Context) -> BbResult {
        let players = {
            let game_ref = self.game.borrow();
            game_ref.network.as_ref().unwrap().client.get_connections()
        };
        for id in players.into_iter() {
            self.on_player_connect(ctx, id)?;
        }
        self.ui.match_grid.borrow_mut().remove_element_at(0);
        Ok(())
    }

    fn on_connection_lost(&mut self, ctx: &mut Context, reason: DisconnectReason) -> BbResult {
        self.disconnected = true;
        println!("Connection to server was lost. Reason: {:?}. Returning to menu...", reason);
        Ok(())
    }

    fn on_player_connect(&mut self, ctx: &mut Context, id: ID) -> BbResult {
        self.players.insert(id.n, PlayerParams::new(id.clone(), ShipType::Caravel));
        self.ui.add_player(ctx, id.clone(), ShipType::Caravel)?;
        self.ui.chat.add_line(ctx, &format!("{:?} connected to the game!", id)).convert()
    }

    fn on_player_disconnect(&mut self, ctx: &mut Context, id: u16, reason: DisconnectReason)
        -> BbResult {
        if let Some(player) = self.players.remove(&id) {
            self.ui.update_player_list(ctx, self.players.values()
                .map(|p| p.clone()).collect())?;
            self.ui.chat.add_line(ctx,
                &format!("{:?} left the game. Reason: {:?}.", id, reason)).convert()
        } else {
            println!("Unknown player with id {} left the game. Reason: {:?}", id, reason);
            Ok(())
        }
    }

    fn on_chat_message(&mut self, ctx: &mut Context, text: String, sender: u16) -> BbResult {
        let sender = {
            self.game.borrow().network.as_ref().unwrap().get_connection_name(sender)
        };
        self.ui.chat.add_message(ctx, sender.as_str(), text.as_str()).convert()
    }

    fn on_game_phase_changed(&mut self, ctx: &mut Context, phase: GamePhase) -> BbResult {
        match phase {
            GamePhase::World(world_seed) => {
                self.world_seed = world_seed;
                self.game_started = true;
            },
            _ => ()
        }
        Ok(())
    }

    fn on_select_ship(&mut self, ctx: &mut Context, sender: u16, ship: ShipType) -> BbResult {
        if let Some(player) = self.players.get_mut(&sender) {
            println!("{:?} selected the {:?} ship.", player.id, ship);
            player.ship_type = ship;
            self.ui.update_player_list(ctx, self.players.values()
                .map(|p| p.clone()).collect())?;
        } else {
            println!("Unknown player with ID {} attempted to change ship type to {:?}", sender, ship)
        }
        Ok(())
    }

    fn on_change_settings(&mut self, ctx: &mut Context, settings: GameSettings) -> BbResult {
        println!("Updated settings: {:?}", &settings);
        self.game_settings = settings;
        Ok(())
    }
}

struct LobbySceneUI {
    match_grid: Rcc<Grid>,
    chat: Chat,
    start_game_button: Rcc<DefaultButton>,
    disconnect_button: Rcc<DefaultButton>,
    player_list_grid: Rcc<Grid>,
    caravel_ship_button: Rcc<DefaultButton>,
    galleon_ship_button: Rcc<DefaultButton>,
    schooner_ship_button: Rcc<DefaultButton>,
    selected_ship_type: Option<ShipType>,
    game_started: bool,
    game: GC
}

impl LobbySceneUI {
    pub fn new(ctx: &mut Context, grid: &mut Grid, game: GC) -> BbResult<LobbySceneUI> {
        let mut match_grid = Grid::default(ctx, UIAlignment::Vertical,
            V2::zero(), V2::new(330.0, 500.0), 5.0).convert()?;
        match_grid.add_element(Label::new(ctx, "Setting up network...", FontSize::Header,
            5.0, game.clone()).convert()?);

        let mut start_game_button = Button::new(ctx, "Start Game",
            V2::new(110.0, 30.0), 2.0, DefaultUIReactor::new(), game.clone()).convert()?;
        if !game.borrow().network.as_ref().unwrap().has_authority() {
            start_game_button.set_disabled(true);
        }
        let start_game_button = match_grid.add_element(start_game_button);
        let disconnect_button = match_grid.add_element(Button::new(ctx, "Disconnect", 
            V2::new(105.0, 30.0), 2.0, DefaultUIReactor::new(), game.clone()).convert()?);
        match_grid.add_element(Label::new(ctx, "Connected Players", FontSize::Header,
            5.0, game.clone()).convert()?);
        let player_list_grid = match_grid.add_element(Grid::default(ctx, UIAlignment::Vertical,
            V2::zero(), V2::one() * 300.0, 5.0).convert()?);
        let match_grid = grid.add_element(match_grid);

        let mut game_grid = Grid::default(ctx, UIAlignment::Vertical,
            V2::zero(), V2::new(250.0, 500.0), 5.0).convert()?;
        let mut game_settings_grid = Grid::default(ctx, UIAlignment::Vertical,
            V2::zero(), V2::new(120.0, 230.0), 2.0).convert()?;
        game_settings_grid.add_element(Label::new(ctx,
            "Select Ship", FontSize::Normal, 1.0, game.clone()).convert()?);
        game_settings_grid.add_element(Label::new(ctx,
            "Caravel: Medium sized two-master. 4 cannons/side. 140 HP. 60 Defence. Jack of all trades.", FontSize::Small, 2.0, game.clone()).convert()?);
        let caravel_ship_button = game_settings_grid.add_element(Button::new(ctx,
            "Caravel", V2::new(90.0, 35.0), 2.0, DefaultUIReactor::new(), game.clone()).convert()?);
        caravel_ship_button.borrow_mut().set_disabled(true);
        game_settings_grid.add_element(Label::new(ctx,
            "Galleon: Heavy square rig. 5 cannons/side. 160 HP. 80 Defence. Slow but destructive.", FontSize::Small, 2.0, game.clone()).convert()?);
        let galleon_ship_button = game_settings_grid.add_element(Button::new(ctx,
            "Galleon", V2::new(90.0, 35.0), 2.0, DefaultUIReactor::new(), game.clone()).convert()?);
        game_settings_grid.add_element(Label::new(ctx,
            "Schooner: Light fore-and-aft rig. 3 cannons/side. 120 HP. 35 Defence. Quick and mobile.", FontSize::Small, 2.0, game.clone()).convert()?);
        let schooner_ship_button = game_settings_grid.add_element(Button::new(ctx,
            "Schooner", V2::new(90.0, 35.0), 2.0, DefaultUIReactor::new(), game.clone()).convert()?);
        game_grid.add_element(game_settings_grid);

        let chat = Chat::new(ctx, UILayout::Default, &mut game_grid, game.clone()).convert()?;
        grid.add_element(game_grid);
        
        Ok(LobbySceneUI {
            match_grid, chat, start_game_button, disconnect_button, player_list_grid,
            caravel_ship_button, galleon_ship_button, schooner_ship_button,
            selected_ship_type: Some(ShipType::Caravel),
            game_started: false, game
        })
    }

    fn add_player(&mut self, ctx: &mut Context, id: ID, ship_type: ShipType) -> BbResult {
        let mut player_list_grid_ref = self.player_list_grid.borrow_mut();
        let name = format!("  {:?} {} - {:?}", &id, {
            if id.n == 0 {
                "(Host)"
            } else {
                ""
            }
        }, ship_type);
        player_list_grid_ref.add_element(
            Label::new(ctx, name.as_str(), FontSize::Normal, 2.0, self.game.clone()).convert()?);
        Ok(())
    }

    fn update_player_list(&mut self, ctx: &mut Context, players: Vec<PlayerParams>)
        -> BbResult {
        {
            let mut player_list_grid_ref = self.player_list_grid.borrow_mut();
            player_list_grid_ref.clear_elements();
        }
        for player in players.into_iter() {
            self.add_player(ctx, player.id.clone(), player.ship_type)?;
        }
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> BbResult {
        {
            let mut start_game_button_ref = self.start_game_button.borrow_mut();
            if start_game_button_ref.is_pressed() && self.game.borrow().network.as_ref().unwrap()
                .has_authority() && !self.game_started {
                self.game.borrow_mut().network.as_mut().unwrap()
                    .load_world_phase(rand_u64())?;
                start_game_button_ref.set_disabled(true);
                self.game_started = true;
            }
        }
        if let Some(message) = self.chat.check_messages(ctx) {
            self.game.borrow_mut().network.as_mut().unwrap().send_packet(Packet::ChatMessage {
                message
            })?;
        }

        {
            let mut caravel_ship_button_ref = self.caravel_ship_button.borrow_mut();
            if caravel_ship_button_ref.is_pressed() {
                self.selected_ship_type = Some(ShipType::Caravel);
                caravel_ship_button_ref.set_disabled(true);
                self.galleon_ship_button.borrow_mut().set_disabled(false);
                self.schooner_ship_button.borrow_mut().set_disabled(false);
            }
        }
        {
            let mut galleon_ship_button_ref = self.galleon_ship_button.borrow_mut();
            if galleon_ship_button_ref.is_pressed() {
                self.selected_ship_type = Some(ShipType::Galleon);
                galleon_ship_button_ref.set_disabled(true);
                self.caravel_ship_button.borrow_mut().set_disabled(false);
                self.schooner_ship_button.borrow_mut().set_disabled(false);
            }
        }
        {
            let mut schooner_ship_button_ref = self.schooner_ship_button.borrow_mut();
            if schooner_ship_button_ref.is_pressed() {
                self.selected_ship_type = Some(ShipType::Schooner);
                schooner_ship_button_ref.set_disabled(true);
                self.caravel_ship_button.borrow_mut().set_disabled(false);
                self.galleon_ship_button.borrow_mut().set_disabled(false);
            }
        }
        Ok(())
    }
}
