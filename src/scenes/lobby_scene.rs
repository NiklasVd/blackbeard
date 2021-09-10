use std::collections::HashMap;
use tetra::{Context, State};
use crate::{BbError, BbErrorType, BbResult, GC, ID, Rcc, ShipType, TransformResult, V2, button::{Button, DefaultButton}, chat::Chat, grid::{Grid, UIAlignment, UILayout}, label::Label, loading_scene::LoadingScene, menu_scene::MenuScene, net_controller::NetController, net_settings::NetSettings, network::Network, packet::{GamePhase, Packet}, peer::DisconnectReason, ui_element::{DefaultUIReactor, UIElement}};
use super::scenes::{Scene, SceneType};

pub struct LobbyScene {
    pub grid: Grid,
    match_grid: Rcc<Grid>,
    chat: Chat,
    start_game_button: Rcc<DefaultButton>,
    disconnect_button: Rcc<DefaultButton>,
    player_list_grid: Rcc<Grid>,
    player_index: HashMap<u16, usize>,
    player_world_params: Vec<(ID, ShipType)>,
    disconnected: bool,
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
        let mut super_grid = Grid::default(ctx, UIAlignment::Horizontal,
            V2::zero(), V2::one() * 500.0, 5.0).convert()?;
        let mut match_grid = Grid::default(ctx, UIAlignment::Vertical,
            V2::zero(), V2::new(330.0, 500.0), 5.0).convert()?;
        match_grid.add_element(Label::new(ctx, "Setting up network...", false,
            5.0, game.clone()).convert()?);

        let mut start_game_button = Button::new(ctx, "Start Game",
            V2::new(110.0, 30.0), 5.0, DefaultUIReactor::new(), game.clone()).convert()?;
        if !game.borrow().network.as_ref().unwrap().has_authority() {
            start_game_button.set_disabled(true);
        }
        let start_game_button = match_grid.add_element(start_game_button);
        let disconnect_button = match_grid.add_element(Button::new(ctx, "Disconnect", 
            V2::new(105.0, 30.0), 5.0, DefaultUIReactor::new(), game.clone()).convert()?);
        match_grid.add_element(Label::new(ctx, "Connected Players", true, 5.0, game.clone()).convert()?);
        let player_list_grid = match_grid.add_element(Grid::default(ctx, UIAlignment::Vertical,
            V2::zero(), V2::one() * 300.0, 5.0).convert()?);
        let match_grid = super_grid.add_element(match_grid);

        let mut game_grid = Grid::default(ctx, UIAlignment::Vertical,
            V2::zero(), V2::new(250.0, 500.0), 5.0).convert()?;
        let chat = Chat::new(ctx, UILayout::Default, &mut game_grid, game.clone()).convert()?;
        super_grid.add_element(game_grid);

        Ok(LobbyScene {
            grid: super_grid, match_grid, chat, start_game_button, disconnect_button, player_list_grid, player_index: HashMap::new(),
            player_world_params: Vec::new(), disconnected: false, game
        })
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
        Ok(if self.player_world_params.len() > 0 {
            Some(Box::new(LoadingScene::new(ctx, self.player_world_params.clone(),
                self.game.clone()).convert()?))
        } else if self.disconnect_button.borrow().is_pressed() {
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
        {
            let mut start_game_button_ref = self.start_game_button.borrow_mut();
            if start_game_button_ref.is_pressed() && self.game.borrow().network.as_ref().unwrap()
                .has_authority() && self.player_world_params.len() == 0 {
                self.game.borrow_mut().network.as_mut().unwrap().set_game_phase(GamePhase::World).convert()?;
                start_game_button_ref.set_disabled(true);
            }
        }
        if let Some(message) = self.chat.check_messages(ctx) {
            self.game.borrow_mut().network.as_mut().unwrap().send_packet(Packet::ChatMessage {
                message
            }).convert()?
        }

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
        self.match_grid.borrow_mut().remove_element_at(0);
        Ok(())
    }

    fn on_connection_lost(&mut self, ctx: &mut Context, reason: DisconnectReason) -> BbResult {
        self.disconnected = true;
        println!("Host shut down server. Returning to menu...");
        Ok(())
    }

    fn on_player_connect(&mut self, ctx: &mut Context, id: ID) -> BbResult {
        let mut player_list_grid_ref = self.player_list_grid.borrow_mut();
        self.player_index.insert(id.n, player_list_grid_ref.elements.len());
        let name = format!("{:?} {}", &id, {
            if id.n == 0 {
                "(Host)"
            } else {
                ""
            }
        });
        player_list_grid_ref.add_element(
            Label::new(ctx, name.as_str(), false, 2.0, self.game.clone()).convert()?);
        Ok(())
    }

    fn on_player_disconnect(&mut self, ctx: &mut Context, id: u16, reason: DisconnectReason) -> BbResult {
        if let Some(&index) = self.player_index.get(&id) {
            self.player_list_grid.borrow_mut().remove_element_at(index);
            Ok(())
        } else {
            Err(BbError::Bb(BbErrorType::InvalidPlayerID(id)))
        }
    }

    fn on_chat_message(&mut self, ctx: &mut Context, text: String, sender: u16) -> BbResult {
        let sender = {
            self.game.borrow().network.as_ref().unwrap().get_connection_name(sender)
        };
        self.chat.add_message(ctx, sender.as_str(), text.as_str()).convert()
    }

    fn on_game_phase_changed(&mut self, ctx: &mut Context, phase: GamePhase) -> BbResult {
        self.player_world_params = self.game.borrow_mut().network.as_ref().unwrap()
            .client.get_connections()
            .into_iter()
            .map(|id| (id.clone(), ShipType::Caravel))
            .collect::<Vec<_>>();
        Ok(())
    }
}
