use std::collections::HashMap;

use tetra::{Context, State};
use crate::{BbError, BbErrorType, BbResult, GC, Rcc, TransformResult, V2, button::{Button, DefaultButton}, grid::{Grid, UIAlignment}, label::Label, menu_scene::MenuScene, net_controller::NetController, net_settings::NetSettings, network::Network, packet::Packet, peer::DisconnectReason, ui_element::DefaultUIReactor};
use super::scenes::{Scene, SceneType};

pub struct LobbyScene {
    pub grid: Grid,
    start_game_button: Rcc<DefaultButton>,
    disconnect_button: Rcc<DefaultButton>,
    player_list_grid: Rcc<Grid>,
    player_index: HashMap<u16, usize>,
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
        let mut grid = Grid::new(ctx, UIAlignment::Vertical, V2::zero(),
            V2::one() * 500.0, 5.0).convert()?;
        grid.add_element(Label::new(ctx, "Setting up network...", false,
            5.0, game.clone()).convert()?);
        let start_game_button = grid.add_element(Button::new(ctx, "Start Game",
            V2::new(90.0, 30.0), 5.0, DefaultUIReactor::new(), game.clone()).convert()?);
        let disconnect_button = grid.add_element(Button::new(ctx, "Disconnect", 
            V2::new(85.0, 30.0), 5.0, DefaultUIReactor::new(), game.clone()).convert()?);
        
        grid.add_element(Label::new(ctx, "Connected Players", true, 5.0, game.clone()).convert()?);
        let player_list_grid = grid.add_element(Grid::new(ctx, UIAlignment::Vertical,
            V2::zero(), V2::one() * 300.0, 5.0).convert()?);
        
        Ok(LobbyScene {
            grid, start_game_button, disconnect_button, player_list_grid, player_index: HashMap::new(),
            disconnected: false, game
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
        if self.disconnect_button.borrow().is_pressed() {
            self.game.borrow_mut().network.as_mut().unwrap().disconnect(DisconnectReason::Manual)?;
            Ok(Some(Box::new(MenuScene::new(ctx, self.game.clone()).convert()?)))
        } else if self.disconnected {
            Ok(Some(Box::new(MenuScene::new(ctx, self.game.clone()).convert()?)))
        } else {
            Ok(None)
        }
        
    }
}

impl State for LobbyScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.handle_received_packets(ctx).convert()
    }
}

impl NetController for LobbyScene {
    fn poll_received_packets(&mut self) -> BbResult<Option<(Packet, u16)>> {
        self.game.borrow_mut().network.as_mut().unwrap().poll_received_packets()
    }

    fn on_establish_connection(&mut self, ctx: &mut Context) -> BbResult {
        let players = {
            let game_ref = self.game.borrow();
            game_ref.network.as_ref().unwrap().client.get_connections()
                .map(|(id, name)| (*id, name.to_owned())).collect::<Vec<(u16, String)>>()
        };
        for (id, name) in players.into_iter() {
            self.on_player_connect(ctx, name, id)?;
        }
        self.grid.remove_element_at(0);
        Ok(())
    }

    fn on_connection_lost(&mut self, ctx: &mut Context, reason: DisconnectReason) -> BbResult {
        self.disconnected = true;
        println!("Host shut down server. Returning to menu...");
        Ok(())
    }

    fn on_player_connect(&mut self, ctx: &mut Context, name: String, id: u16) -> BbResult {
        let mut player_list_grid_ref = self.player_list_grid.borrow_mut();
        self.player_index.insert(id, player_list_grid_ref.elements.len());
        let name = match id {
            0 => format!("{}^0 (Host)", name),
            _ => format!("{}^{}", name, id)
        };
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
        let sender_name = {
            let game_ref = self.game.as_ref().borrow();
            game_ref.network.as_ref().unwrap().client.get_connection(sender)
                .unwrap_or(&format!("Unknown player (ID: {})", sender)).to_owned()
        };
        println!("{} in chat: {}", sender_name, text);
        Ok(())
    }
}
