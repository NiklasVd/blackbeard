use tetra::Context;
use crate::{BbResult, ID, game_settings::GameSettings, packet::{GamePhase, InputStep, Packet}, peer::{DisconnectReason, is_auth_client}, ship::ShipType};

pub trait NetController {
    fn poll_received_packets(&mut self, ctx: &mut Context) -> BbResult<Option<(Packet, u16)>>;
    fn handle_received_packets(&mut self, ctx: &mut Context) -> BbResult {
        match self.poll_received_packets(ctx) {
            Ok(Some((packet, sender))) => {
                self.handle_packets(ctx, (packet, sender))
            },
            Ok(None) => Ok(()),
            Err(e) => {
                println!("Client poll returned error: {:?}. Shutting down peer...", e);
                self.on_connection_lost(ctx, DisconnectReason::Timeout /* Add more specific reasons */)
            }
        }
    }

    fn handle_packets(&mut self, ctx: &mut Context, packet: (Packet, u16)) -> BbResult {
        let (packet, sender) = packet;
        match packet {
            Packet::HandshakeReply { .. } => self.on_establish_connection(ctx),
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
            Packet::Selection { mode, ship: _, settings } => self.on_change_settings(ctx, settings.unwrap()),
            _ => Ok(())
        }
    }

    fn on_establish_connection(&mut self, ctx: &mut Context) -> BbResult {
        Ok(())
    }
    fn on_connection_lost(&mut self, ctx: &mut Context, reason: DisconnectReason) -> BbResult {
        Ok(())
    }

    fn on_player_connect(&mut self, ctx: &mut Context, id: ID) -> BbResult {
        Ok(())
    }
    fn on_player_disconnect(&mut self, ctx: &mut Context, id: u16, reason: DisconnectReason) -> BbResult {
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