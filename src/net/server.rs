use std::{collections::HashMap, net::SocketAddr};
use crate::{BbError, BbErrorType, BbResult, ID, packet::{Packet, deserialize_packet_unsigned, serialize_packet}, peer::{DisconnectReason, Peer}};

// Auth Client: First player to connect to the server
pub struct Server {
    peer: Peer,
    connections: HashMap<u16, (ID, SocketAddr)>,
    curr_id: u16,
    accept_connections: bool
}

impl Server {
    pub fn host(port: u16) -> BbResult<Server> {
        println!("Hosting server at {}.", port);
        Ok(Server {
            peer: Peer::setup(port)?, connections: HashMap::new(), curr_id: 0, accept_connections: true
        })
    }

    pub fn accept_connections(&mut self, state: bool) {
        self.accept_connections = state;
    }

    pub fn get_connection_by_addr(&self, addr: SocketAddr) -> Option<&ID> {
        self.connections.values().find(|(id, remote_addr)| addr == *remote_addr).map(|val| &val.0)
    }

    pub fn send_unicast(&mut self, packet: Packet, target_id: u16) -> BbResult {
        let connections = &self.connections;
        if let Some(conn) = connections.get(&target_id) {
            self.peer.send_raw_packet(serialize_packet(packet, target_id), conn.1)
        } else {
            Err(BbError::Bb(BbErrorType::InvalidPlayerID(target_id)))
        }
    }

    pub fn send_multicast(&mut self, packet: Packet, sender: u16) -> BbResult {
        let packet_bytes = serialize_packet(packet, sender);
        let peer = &mut self.peer;
        let m = self.connections.values().try_for_each(
            |conn| peer.send_raw_packet(packet_bytes.clone(), conn.1));
        Ok(())
    }

    pub fn send_multicast_group(&mut self, packet: Packet, sender: u16, targets: &[u16]) -> BbResult {
        let packet_bytes = serialize_packet(packet, sender);
        targets.into_iter().try_for_each(|id| {
            if let Some(conn) = self.connections.get(id) {
                self.peer.send_raw_packet(packet_bytes.clone(), conn.1)
            } else {
                Err(BbError::Bb(BbErrorType::InvalidPlayerID(*id)))
            }
        })
    }

    pub fn poll_received_packets(&mut self) -> BbResult {
        match self.peer.poll_received_packets() {
            Ok(Some((packet, sender_addr))) => {
                let packet = deserialize_packet_unsigned(packet.payload().to_vec());
                if let Some((id, ..)) = self.connections.values().find(|conn| conn.1 == sender_addr) {
                    println!("Server: {:?} sent {:?}", id, &packet);
                    let id = id.n;
                    match &packet {
                        Packet::PlayerDisconnect { reason } => self.on_receive_disconnect(id, *reason)?,
                        _ => ()
                    }
                    self.send_multicast(packet, id)?; // Echo packet back to all players
                    Ok(())
                }
                else {
                    match &packet {
                        Packet::Handshake { name } => {
                            if !self.accept_connections {
                                println!("Blocked connection attempt by '{}'. Reason: New connections disallowed.", name)
                            } else {
                                self.on_receive_handshake(name.clone(), sender_addr)?;
                            }
                            Ok(())
                        },
                        _ => Err(BbError::Bb(BbErrorType::NetInvalidSender(sender_addr)))
                    }
                }
            },
            Ok(None) => Ok(()),
            Err(BbError::Bb(BbErrorType::NetDisconnect(addr)))
                | Err(BbError::Bb(BbErrorType::NetTimeout(addr)))  => {
                if let Some(id) = self.get_connection_by_addr(addr) {
                    let id = id.to_owned();
                    self.on_receive_disconnect(id.n, DisconnectReason::Timeout)
                } else {
                    Err(BbError::Bb(BbErrorType::NetInvalidSender(addr)))
                }
            },
            Err(e) => Err(e)
        }
    }

    fn on_receive_handshake(&mut self, name: String, remote_addr: SocketAddr) -> BbResult {
        let new_player_id = self.curr_id;
        self.curr_id += 1;
        let id = ID::new(name.to_owned(), new_player_id);
        println!("{:?} ({}) joined the server.", &id, remote_addr);
        self.peer.send_raw_packet(serialize_packet(Packet::HandshakeReply {
            players: self.connections.values()
            .map(|p| p.0.clone())
            .collect() // Send list of all players to new connection
        }, new_player_id), remote_addr)?;

        self.connections.insert(new_player_id, (id, remote_addr));
        if self.connections.len() > 1 {
            self.send_multicast_group(Packet::PlayerConnect {
                name
            }, new_player_id, self.connections.keys()
                .filter(|&&id| id != new_player_id)
                .map(|id| *id)
                .collect::<Vec<u16>>().as_slice())?; // Send new player connection to remaining players
        }
        Ok(())
    }

    fn on_receive_disconnect(&mut self, sender: u16, reason: DisconnectReason) -> BbResult {
        if let Some(conn) = self.connections.remove(&sender) {
            println!("{:?} disconnected. Reason: {:?}", conn.0, reason);
            self.send_multicast(Packet::PlayerDisconnect { reason }, sender)
        } else {
            Err(BbError::Bb(BbErrorType::InvalidPlayerID(sender)))
        }
    }
}