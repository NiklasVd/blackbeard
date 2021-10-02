use std::{collections::HashMap, net::{SocketAddr}, thread, time::Duration};
use laminar::SocketEvent;

use crate::{BbResult, ID, packet::{Packet, deserialize_packet, serialize_packet_unsigned}, peer::{DisconnectReason, Peer, is_auth_client}};

pub enum ClientEvent {
    ReceivePacket(u16, Packet),
    Connect(u16),
    Disconnect(DisconnectReason),
    Empty
}

pub struct Client {
    peer: Peer,
    server_addr: SocketAddr,
    connections: HashMap<u16, ID>,
    local_id: Option<ID>,
    connected: bool,
    name: String
}

impl Client {
    pub fn connect(server_addr: &str, name: String) -> BbResult<Client> {
        let mut client = Client {
            peer: Peer::setup(None)?, server_addr: server_addr.parse().unwrap(),
            connections: HashMap::new(), local_id: None, connected: false, name: name.to_owned()
        };
        println!("Connecting to {}", server_addr);
        client.send_packet(Packet::Handshake {
            name: name.clone()
        })?;
        Ok(client)
    }

    pub fn get_local_id(&self) -> Option<ID> {
        self.local_id.as_ref().and_then(|id| Some(id.clone()))
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn get_connection(&self, id: u16) -> Option<&ID> {
        self.connections.get(&id)
    }

    pub fn get_connections(&self) -> Vec<ID> {
        self.connections.values().map(|id| id.clone()).collect::<Vec<_>>() // Performance?
    }

    pub fn send_packet(&mut self, packet: Packet) -> BbResult {
        self.peer.send_raw_packet(serialize_packet_unsigned(packet), self.server_addr)
    }

    pub fn disconnect(&mut self, reason: DisconnectReason) -> BbResult {
        println!("Disconnecting connection to server. Reason: {:?}", reason);
        self.send_packet(Packet::PlayerDisconnect {
            reason
        })?;
        thread::sleep(Duration::from_secs_f32(1.0));
        self.peer.shutdown()
    }

    pub fn poll_received_packets(&mut self) -> BbResult<ClientEvent> {
        if let Some(event) = self.peer.poll_received_packets()? {
            Ok(match event {
                laminar::SocketEvent::Packet(packet) =>  {
                    let sender_addr = packet.addr();
                    if sender_addr != self.server_addr {
                        println!("Received packet {:?} from unknown endpoint: {}. Dropping...", packet, sender_addr);
                        ClientEvent::Empty
                    } else {
                        let (packet, sender) = deserialize_packet(packet.payload().to_vec());
                        self.handle_server_packet(packet, sender)?
                    }
                },
                SocketEvent::Connect(_) => {
                    println!("Established connection to server.");
                    ClientEvent::Empty
                },
                SocketEvent::Timeout(_) => {
                    println!("Connection to server timed out.");
                    ClientEvent::Disconnect(DisconnectReason::Timeout)
                },
                SocketEvent::Disconnect(_) => {
                    println!("Disconnected from server.");
                    //ClientEvent::Disconnect()
                    ClientEvent::Empty
                },
            })
        } else {
            Ok(ClientEvent::Empty)
        }
    }

    fn handle_server_packet(&mut self, packet: Packet, sender: u16) -> BbResult<ClientEvent> {
        Ok(match &packet {
            Packet::HandshakeReply { players } => {
                println!("Server accepted connection attempt!");
                players.iter().for_each(|id| {
                    println!("Updating player: {}^{}", id.name, id.n);
                    self.connections.insert(id.n, id.clone());
                    if id.n == sender {
                        self.local_id = Some(id.clone());
                    }
                });
                self.connected = true;
                ClientEvent::Connect(sender)
            },
            Packet::PlayerConnect { name } => {
                self.connections.insert(sender, ID::new(name.to_owned(), sender));
                ClientEvent::ReceivePacket(sender, packet)
            },
            Packet::PlayerDisconnect { reason } => {
                if let Some(player) = self.connections.remove(&sender) {
                    println!("{}^{} disconnected. Reason: {:?}", player.name, sender, reason);
                    if is_auth_client(sender) {
                        println!("Host disconnected - connection terminated.");
                        self.connected = false;
                        ClientEvent::ReceivePacket(sender, packet)
                    } else if player.n == self.local_id.as_ref().unwrap().n {
                        println!("Server terminated this connection, potentially due to latency issues.");
                        self.connected = false;
                        ClientEvent::Disconnect(*reason)
                    } else {
                        ClientEvent::ReceivePacket(sender, packet)
                    }
                } else {
                    ClientEvent::Empty
                }
            },
            _ => ClientEvent::ReceivePacket(sender, packet)
        })
    }
}