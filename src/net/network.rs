use crate::{BbError, BbErrorType, BbResult, client::{Client, ClientEvent}, net_settings::NetSettings, packet::{GamePhase, InputState, Packet}, peer::{DisconnectReason}, server::{Server, ServerEvent}};

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

    pub fn poll_received_client_packets(&mut self) -> BbResult<ClientEvent> {
        self.client.poll_received_packets()
    }

    pub fn poll_received_server_packets(&mut self) -> BbResult<ServerEvent> {
        if let Some(server) = self.server.as_mut() {
            server.poll_received_packets()
        } else {
            Ok(ServerEvent::Empty)
        }
    }

    // pub fn poll_received_packets(&mut self) -> BbResult<ClientEvent> {
    //     if let Some(server) = self.server.as_mut() {
    //         server.poll_received_packets()?;
    //     }
    //     self.client.poll_received_packets()
    // }

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

    pub fn load_world_phase(&mut self, world_seed: u64) -> BbResult {
        if !self.has_authority() {
            return Err(BbError::Bb(BbErrorType::NetInsufficientAuthority))
        } else {
            self.send_packet(Packet::Game {
                phase: GamePhase::World(world_seed)
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
}
