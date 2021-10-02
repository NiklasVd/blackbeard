use std::{collections::{HashMap, hash_map::Values}, net::SocketAddr, thread, time::{Duration}};
use laminar::SocketEvent;
use crate::{BbError, BbErrorType, BbResult, ID, net_settings::NetSettings, packet::{Packet, deserialize_packet_unsigned, serialize_packet}, peer::{DisconnectReason, Peer}};

pub enum ServerEvent {
    ReceivePacket(u16, Packet),
    PlayerConnect(ID, SocketAddr),
    PlayerDisconnect(u16, DisconnectReason),
    Empty
}

#[derive(Clone)]
pub struct ClientConnection(pub ID, pub SocketAddr);

pub struct Server {
    settings: NetSettings,
    peer: Peer,
    connections: HashMap<u16, ClientConnection>,
    connections_addr: HashMap<SocketAddr, ID>,
    curr_id: u16
}

impl Server {
    pub fn host(port: u16, settings: NetSettings) -> BbResult<Server> {
        println!("Server: Hosting at {}.", port);
        Ok(Server {
            settings, peer: Peer::setup(Some(port))?,
            connections: HashMap::new(), connections_addr: HashMap::new(),
            curr_id: 0
        })
    }

    pub fn get_connections(&self) -> Values<u16, ClientConnection> {
        self.connections.values()
    }

    pub fn get_connection_count(&self) -> usize {
        self.connections.len()
    }

    pub fn get_conn_by_id(&self, id: u16) -> Option<&ClientConnection> {
        self.connections.get(&id)
    }

    pub fn get_connection_by_addr(&self, addr: SocketAddr) -> Option<&ID> {
        self.connections_addr.get(&addr)
    }

    pub fn disconnect_player(&mut self, player_id: u16, reason: DisconnectReason) -> BbResult {
        if let Some(conn) = self.connections.remove(&player_id) {
            println!("Server: {:?} disconnected. Reason: {:?}", conn.0, reason);
            // As this method is also called upon timeouts (which don't simply echo back
            // all client packets), this is done manually here.
            self.send_multicast(Packet::PlayerDisconnect {
                reason
            }, player_id)
        } else {
            println!("Server: Player with ID {} is already disconnected.", player_id);
            Ok(())
        }
    }

    pub fn send_unicast(&mut self, packet: Packet, target_id: u16) -> BbResult {
        let connections = &self.connections;
        if let Some(conn) = connections.get(&target_id) {
        //                          Is target_id right? Implies sender is always receiver
        //                                                    \/
            self.peer.send_raw_packet(serialize_packet(packet, target_id), conn.1)
        } else {
            Err(BbError::Bb(BbErrorType::InvalidPlayerID(target_id)))
        }
    }

    pub fn send_raw_unicast(&mut self, packet_bytes: Vec<u8>, addr: SocketAddr) -> BbResult {
        self.peer.send_raw_packet(packet_bytes, addr)
    }

    pub fn send_multicast(&mut self, packet: Packet, sender: u16) -> BbResult {
        let packet_bytes = serialize_packet(packet, sender);
        let peer = &mut self.peer;
        self.connections.values().try_for_each(
            |conn| peer.send_raw_packet(packet_bytes.clone(), conn.1))
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

    pub fn shutdown(&mut self) -> BbResult {
        self.send_multicast(Packet::PlayerDisconnect {
            reason: DisconnectReason::HostShutdown
        }, 0)?;
        thread::sleep(Duration::from_secs_f32(1.5));
        self.peer.shutdown()
    }

    pub fn poll_received_packets(&mut self) -> BbResult<ServerEvent> {
        if let Some(event) = self.peer.poll_received_packets()? {
            Ok(match event {
                SocketEvent::Packet(packet) =>  {
                    let sender_addr = packet.addr();
                    let packet = deserialize_packet_unsigned(packet.payload().to_vec());
                    if let Some(id) = self.get_connection_by_addr(sender_addr) {
                        let id = id.clone();
                        self.handle_internal_packet(packet, id)?
                    }
                    else {
                        self.handle_external_packet(packet, sender_addr)?
                    }
                },
                SocketEvent::Timeout(addr) => {
                    if let Some(id) = self.get_connection_by_addr(addr) {
                        let id = id.clone();
                        self.disconnect_player(id.n, DisconnectReason::Timeout)?;
                        ServerEvent::PlayerDisconnect(id.n, DisconnectReason::Timeout)
                    } else {
                        ServerEvent::Empty
                    }
                },
                // Unfortunately it seems that the Disconnect event occurs even
                // when the client timed out, hence, triggering two socket events in a row.
                // SocketEvent::Disconnect(addr) => {
                //     if let Some(id) = self.get_connection_by_addr(addr) {
                //         println!("Server: Player with ID {} disconnected.", id.n);
                //         ServerEvent::PlayerDisconnect(id.n, DisconnectReason::Manual)
                //     } else {
                //         ServerEvent::Empty
                //     }
                // },
                _ => ServerEvent::Empty
            })
        } else {
            Ok(ServerEvent::Empty)
        }
    }

    fn handle_internal_packet(&mut self, packet: Packet, sender: ID) -> BbResult<ServerEvent> {
        match &packet {
            Packet::PlayerDisconnect { reason } => {
                self.disconnect_player(sender.n, *reason)?;
                return Ok(ServerEvent::PlayerDisconnect(sender.n, *reason))
            },
            Packet::Input { .. } => {
                return Ok(ServerEvent::ReceivePacket(sender.n, packet))
            },
            Packet::Sync { .. } => {
                return Ok(ServerEvent::ReceivePacket(sender.n, packet))
            },
            _ => () // Should filter invalid packets here
        }
        // Echo is done further down the hierarchy
        //self.send_multicast(packet.clone(), sender.n)?;
        Ok(ServerEvent::ReceivePacket(sender.n, packet))
    }

    fn handle_external_packet(&mut self, packet: Packet, sender_addr: SocketAddr)
        -> BbResult<ServerEvent> {
        Ok(match &packet {
            Packet::Handshake { name } => {
                if self.connections.len() >= self.settings.max_players {
                    println!("Server: Blocked connection attempt by {} ({}). Reason: Server is full.",
                        name, sender_addr);
                    ServerEvent::Empty
                } else {
                    self.on_receive_handshake(name.to_owned(), sender_addr)
                }
            },
            _ => {
                println!("Received packet {:?} from unknown peer {}. Dropping...", packet, sender_addr);
                ServerEvent::Empty
            }
        })
    }

    fn on_receive_handshake(&mut self, name: String, remote_addr: SocketAddr) -> ServerEvent {
        let new_idn = self.curr_id;
        let new_id = ID::new(name, new_idn);
        println!("Server: {:?} ({:?}) joined the server.", new_id, remote_addr);
        self.add_connection(ClientConnection(new_id.clone(), remote_addr));

        self.curr_id += 1;
        ServerEvent::PlayerConnect(new_id, remote_addr)
    }

    fn add_connection(&mut self, conn: ClientConnection) {
        self.connections.insert(conn.0.n, conn.clone());
        self.connections_addr.insert(conn.1, conn.0);
    }
}
