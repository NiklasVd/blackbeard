use tetra::State;
use crate::{BbError, BbErrorType, BbResult, client::Client, net_settings::NetSettings, packet::{GamePhase, InputState, Packet}, peer::{DisconnectReason, is_auth_client}, server::Server};

pub struct Network {
    pub client: Client,
    pub server: Option<Server>
}

impl Network {
    pub fn create(port: u16, name: String, settings: NetSettings) -> BbResult<Network> {
        let server = Server::host(port, settings)?;
        let client = Client::connect(format!("127.0.0.1:{}", port).as_str(), name)?;
        Ok(Network {
            client, server: Some(server)
        })
    }

    pub fn join(server_addr: &str, name: String) -> BbResult<Network> {
        let client = Client::connect(server_addr, name)?;
        Ok(Network {
            client, server: None
        })
    }

    pub fn has_authority(&self) -> bool {
        self.server.is_some()
    }

    pub fn poll_received_packets(&mut self) -> BbResult<Option<(Packet, u16)>> {
        if let Some(server) = self.server.as_mut() {
            server.poll_received_packets()?;
        }
        self.client_poll_received_packets()
    }

    pub fn get_connection_name(&self, id: u16) -> String {
        self.client.get_connection(id)
            .map(|id| id.name.clone())
            .unwrap_or(format!("Unknown player (ID: {})", id))
    }

    pub fn send_packet(&mut self, packet: Packet) -> BbResult {
        self.client.send_packet(packet)
    }

    pub fn send_input(&mut self, state: InputState) -> BbResult {
        self.send_packet(Packet::Input {
            state
        })
    }

    pub fn set_game_phase(&mut self, phase: GamePhase) -> BbResult {
        if !self.has_authority() {
            return Err(BbError::Bb(BbErrorType::NetInsufficientAuthority))
        } else {
            self.send_packet(Packet::Game {
                phase
            })
        }
    }

    pub fn disconnect(&mut self, reason: DisconnectReason) -> BbResult {
        self.client.disconnect(reason)?;
        if let Some(mut server) = self.server.take() {
            server.shutdown()?;
        }
        Ok(())
    }

    fn client_poll_received_packets(&mut self) -> BbResult<Option<(Packet, u16)>> {
        if let Some((packet, sender)) = self.client.poll_received_packets()? {
            match &packet {
                &Packet::PlayerDisconnect { reason } if is_auth_client(sender) =>
                    self.disconnect(reason)?, // If host shut down, disconnect from server
                _ => ()
            }
            Ok(Some((packet, sender)))
        } else {
            Ok(None)
        }
    }
}

impl State for Network {
    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result {
        if let Some(server) = self.server.as_mut() {
            server.update(ctx)?;
        }
        Ok(())
    }
}
