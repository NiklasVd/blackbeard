use tetra::Context;
use crate::{BbResult, packet::{InputState, Packet}, peer::{DisconnectReason, is_auth_client}};

pub trait NetController {
    fn poll_received_packets(&mut self) -> BbResult<Option<(Packet, u16)>>;
    fn handle_received_packets(&mut self, ctx: &mut Context) -> BbResult {
        match self.poll_received_packets() {
            Ok(Some((packet, sender))) => {
                match packet {
                    Packet::HandshakeReply { .. } => self.on_establish_connection(ctx),
                    Packet::PlayerConnect { name } => self.on_player_connect(ctx, name, sender),
                    Packet::PlayerDisconnect { reason } => {
                        if is_auth_client(sender) {
                            self.on_connection_lost(ctx, DisconnectReason::HostShutdown)
                        } else {
                            self.on_player_disconnect(ctx, sender, reason)
                        }
                    },
                    Packet::ChatMessage { message } => self.on_chat_message(ctx, message, sender),
                    Packet::Input { state } => self.on_input_state(ctx, state),
                    _ => Ok(())
                }
            },
            Ok(None) => Ok(()),
            Err(e) => Err(e)
        }
    }

    fn on_establish_connection(&mut self, ctx: &mut Context) -> BbResult {
        Ok(())
    }
    fn on_connection_lost(&mut self, ctx: &mut Context, reason: DisconnectReason) -> BbResult {
        Ok(())
    }

    fn on_player_connect(&mut self, ctx: &mut Context, name: String, id: u16) -> BbResult {
        Ok(())
    }
    fn on_player_disconnect(&mut self, ctx: &mut Context, id: u16, reason: DisconnectReason) -> BbResult {
        Ok(())
    }
    fn on_chat_message(&mut self, ctx: &mut Context, text: String, sender: u16) -> BbResult {
        Ok(())
    }
    fn on_input_state(&mut self, ctx: &mut Context, state: InputState) -> BbResult {
        Ok(())
    }
}