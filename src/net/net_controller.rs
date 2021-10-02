use std::net::SocketAddr;

use tetra::Context;
use crate::{BbResult, ID, client::ClientEvent, game_settings::GameSettings, packet::{GamePhase, InputState, InputStep, Packet}, peer::{DisconnectReason, is_auth_client}, server::ServerEvent, ship::ShipType, sync_checker::SyncState};

pub trait NetController {
    fn poll_received_server_packets(&mut self, ctx: &mut Context) -> BbResult<ServerEvent>;
    fn handle_received_server_packets(&mut self, ctx: &mut Context) -> BbResult {
        match self.poll_received_server_packets(ctx)? {
            ServerEvent::ReceivePacket(sender, packet) => {
                match packet {
                    Packet::ChatMessage { message } => self.on_server_receive_chat_message(sender, message),
                    Packet::Input { state } => self.on_server_receive_input(ctx, sender, state),
                    Packet::Game { phase } => self.on_server_set_game_phase(sender, phase),
                    Packet::Sync { state } => self.on_server_receive_sync_state(ctx, sender, state),
                    Packet::Selection { mode, ship, .. } if mode => self.on_server_receive_ship_selection(ctx, sender, ship.unwrap()),
                    Packet::Selection { settings, .. } => self.on_server_receive_settings(ctx, sender, settings.unwrap()),
                    _=> Ok(())
                }
            },
            ServerEvent::PlayerConnect(id, addr) => self.on_server_receive_handshake(id, addr),
            ServerEvent::PlayerDisconnect(sender, reason) => self.on_server_receive_disconnect(sender, reason),
            ServerEvent::Empty => Ok(()),
        }
    }

    fn poll_received_client_packets(&mut self, ctx: &mut Context) -> BbResult<ClientEvent>;
    fn handle_received_client_packets(&mut self, ctx: &mut Context) -> BbResult {
        match self.poll_received_client_packets(ctx)? {
            ClientEvent::ReceivePacket(sender, packet) => {
                match packet {
                    Packet::PlayerConnect { name } => self.on_player_connect(ctx, ID::new(name, sender)),
                    Packet::PlayerDisconnect { reason } => {
                        if is_auth_client(sender) {
                            self.on_connection_lost(ctx, DisconnectReason::HostShutdown)
                        } else {
                            self.on_player_disconnect(ctx, sender, reason)
                        }
                    },
                    Packet::ChatMessage { message } => self.on_chat_message(ctx, message, sender),
                    Packet::InputStep { step } => self.on_input_step(ctx, step),
                    Packet::Game { phase } => self.on_game_phase_changed(ctx, phase),
                    Packet::Selection { mode, ship, .. } if mode => self.on_select_ship(ctx, sender, ship.unwrap()),
                    Packet::Selection { settings, .. } => self.on_change_settings(ctx, settings.unwrap()),
                    _ => Ok(())
                }
            },
            ClientEvent::Connect(_) => self.on_establish_connection(ctx), 
            ClientEvent::Disconnect(reason) => self.on_connection_lost(ctx, reason),
            _ => Ok(())
        }
    }

    fn handle_received_packets(&mut self, ctx: &mut Context) -> BbResult {
        self.handle_received_server_packets(ctx)?;
        self.handle_received_client_packets(ctx)
    }

    fn on_server_receive_handshake(&mut self, id: ID, remote_addr: SocketAddr) -> BbResult {
        Ok(())
    }

    fn on_server_receive_disconnect(&mut self, sender: u16, reason: DisconnectReason) -> BbResult {
        Ok(())
    }

    fn on_server_set_game_phase(&mut self, sender: u16, phase: GamePhase) -> BbResult {
        // Check authority and current scene
        Ok(())
    }

    fn on_server_receive_chat_message(&mut self, sender: u16, message: String) -> BbResult {
        // Check for size and profanity
        Ok(())
    }

    fn on_server_receive_input(&mut self, ctx: &mut Context, sender: u16, input: InputState) -> BbResult {
        Ok(())
    }

    fn on_server_receive_sync_state(&mut self, ctx: &mut Context, sender: u16,
        state: SyncState) -> BbResult {
        Ok(())
    }

    fn on_server_receive_ship_selection(&mut self, ctx: &mut Context, sender: u16,
        ship_type: ShipType) -> BbResult {
        Ok(())
    }

    fn on_server_receive_settings(&mut self, ctx: &mut Context, sender: u16,
        settings: GameSettings) -> BbResult {
        Ok(())
    }

    fn on_establish_connection(&mut self, ctx: &mut Context) -> BbResult {
        Ok(())
    }
    fn on_connection_lost(&mut self, ctx: &mut Context, reason: DisconnectReason)-> BbResult {
        Ok(())
    }

    fn on_player_connect(&mut self, ctx: &mut Context, id: ID) -> BbResult {
        Ok(())
    }
    fn on_player_disconnect(&mut self, ctx: &mut Context, id: u16,
        reason: DisconnectReason) -> BbResult {
        Ok(())
    }
    fn on_chat_message(&mut self, ctx: &mut Context, text: String, sender: u16) -> BbResult {
        Ok(())
    }
    fn on_input_step(&mut self, ctx: &mut Context, step: InputStep) -> BbResult {
        Ok(())
    }
    fn on_game_phase_changed(&mut self, ctx: &mut Context, phase: GamePhase) -> BbResult {
        Ok(())
    }
    fn on_select_ship(&mut self, ctx: &mut Context, sender: u16, ship: ShipType) -> BbResult {
        Ok(())
    }
    fn on_change_settings(&mut self, ctx: &mut Context, settings: GameSettings) -> BbResult {
        Ok(())
    }
}