use tetra::State;
use crate::{BbResult, client::Client, packet::{InputState, Packet}, peer::DisconnectReason, server::Server};

const POLL_FRAME_INTERVAL: u32 = 15;

pub struct Network {
    pub client: Client,
    pub server: Option<Server>,
    curr_poll_frame: u32
}

impl Network {
    pub fn create(port: u16, name: String) -> BbResult<Network> {
        let server = Server::host(port)?;
        let client = Client::connect(port + 1, format!("127.0.0.1:{}", port).as_str(), name)?;
        Ok(Network {
            client, server: Some(server), curr_poll_frame: 0
        })
    }

    pub fn join(port: u16, server_addr: &str, name: String) -> BbResult<Network> {
        let client = Client::connect(port, server_addr, name)?;
        Ok(Network {
            client, server: None, curr_poll_frame: 0
        })
    }

    pub fn poll_received_packets(&mut self) -> BbResult<Option<(Packet, u16)>> {
        if let Some(server) = self.server.as_mut() {
            server.poll_received_packets()?;
        }
        self.client.poll_received_packets()
    }

    pub fn send_packet(&mut self, packet: Packet) -> BbResult {
        self.client.send_packet(packet)
    }

    pub fn send_input(&mut self, state: InputState) -> BbResult {
        self.send_packet(Packet::Input {
            state
        })
    }

    pub fn disconnect(&mut self, reason: DisconnectReason) -> BbResult {
        self.send_packet(Packet::PlayerDisconnect {
            reason
        })
    }
}

impl State for Network {
    
}
