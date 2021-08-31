use std::{collections::HashMap, net::SocketAddr, thread, time::Duration};
use crate::{BbError, BbErrorType, BbResult, packet::{Packet, deserialize_packet, serialize_packet_unsigned}, peer::{DisconnectReason, Peer, is_auth_client}};

pub struct Client {
    peer: Peer,
    server_addr: SocketAddr,
    connections: HashMap<u16, String>,
    connected: bool,
    name: String
}

impl Client {
    pub fn connect(server_addr: &str, name: String) -> BbResult<Client> {
        let mut client = Client {
            peer: Peer::setup(None)?, server_addr: server_addr.parse().unwrap(),
            connections: HashMap::new(), connected: false, name: name.to_owned()
        };
        client.send_packet(Packet::Handshake {
            name
        })?;
        println!("Sending handshake to {}", server_addr);
        Ok(client)
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn get_connection(&self, id: u16) -> Option<&String> {
        self.connections.get(&id)
    }

    pub fn get_connections(&self) -> std::collections::hash_map::Iter<u16, String> {
        self.connections.iter()
    }

    pub fn send_packet(&mut self, packet: Packet) -> BbResult {
        self.peer.send_raw_packet(serialize_packet_unsigned(packet), self.server_addr)
    }

    pub fn disconnect(&mut self, reason: DisconnectReason) -> BbResult {
        println!("Disconnecting connection to server. Reason: {:?}", reason);
        self.send_packet(Packet::PlayerDisconnect {
            reason
        })?;
        thread::sleep(Duration::from_secs_f32(2.0));
        self.peer.shutdown()
    }

    pub fn poll_received_packets(&mut self) -> BbResult<Option<(Packet, u16)>> {
        match self.peer.poll_received_packets() {
            Ok(Some((packet, sender))) => {
                if sender != self.server_addr {
                    Err(BbError::Bb(BbErrorType::NetInvalidSender(sender)))
                } else {
                    let (packet, sender) = deserialize_packet(packet.payload().to_vec());
                    self.handle_server_packet(&packet, sender);
                    Ok(Some((packet, sender)))
                }
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e)
        }
    }

    fn handle_server_packet(&mut self, packet: &Packet, sender: u16) {
        match packet {
            Packet::HandshakeReply { players } => {
                println!("Server accepted connection attempt!");
                self.connections.insert(sender, self.name.to_owned());
                players.iter().for_each(|id| {
                    println!("Updating player: {}^{}", id.name, id.n);
                    self.connections.insert(id.n, id.name.to_owned());
                });
                self.connected = true;
            },
            Packet::PlayerConnect { name } => {
                self.connections.insert(sender, name.to_owned());
                println!("{}^{} connected.", name, sender);
            },
            Packet::PlayerDisconnect { reason } => {
                if let Some(player) = self.connections.remove(&sender) {
                    println!("{}^{} disconnected. Reason: {:?}", player, sender, reason);
                    if is_auth_client(sender) {
                        println!("Host disconnected - connection terminated.");
                        self.connected = false;
                        // Host migration?
                    }
                }
            },
            _ => ()
        }
    }
}