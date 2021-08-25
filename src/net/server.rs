use std::{collections::HashMap, net::SocketAddr};
use crate::{BbError, BbErrorType, BbResult, ID, log_warning, packet::{Packet, deserialize_packet_unsigned, serialize_packet}, peer::{DisconnectReason, Peer}};

pub struct Server {
    peer: Peer,
    connections: HashMap<u16, (ID, SocketAddr)>,
    curr_id: u16,
    accept_connections: bool
}

impl Server {
    pub fn host(port: u16) -> BbResult<Server> {
        Ok(Server {
            peer: Peer::setup(port)?, connections: HashMap::new(), curr_id: 0, accept_connections: true
        })
    }

    pub fn accept_connections(&mut self, state: bool) {
        self.accept_connections = state;
    }

    pub fn send_unicast(&mut self, packet: Packet, target_id: u16) -> BbResult<()> {
        let connections = &self.connections;
        if let Some(conn) = connections.get(&target_id) {
            self.peer.send_raw_packet(serialize_packet(packet, target_id), conn.1)
        } else {
            Err(BbError::Bb(BbErrorType::InvalidPlayerID(target_id)))
        }
    }

    pub fn send_multicast(&mut self, packet: Packet, sender: u16) -> BbResult<()> {
        let packet_bytes = serialize_packet(packet, sender);
        let peer = &mut self.peer;
        let m = self.connections.values().try_for_each(
            |conn| peer.send_raw_packet(packet_bytes.clone(), conn.1));
        Ok(())
    }

    pub fn poll_received_packets(&mut self) -> BbResult<(Packet, SocketAddr)> {
        match self.peer.poll_received_packets() {
            Ok((packet, sender_addr)) => {
                let packet = deserialize_packet_unsigned(packet.payload().to_vec());
                let connections = &self.connections;
                if let Some((id, ..)) = connections.values().find(|conn| conn.1 == sender_addr) {
                    // Normal packet by registered player
                    match &packet {
                        Packet::Disconnect { reason } => self.on_receive_disconnect(id.n, *reason)?,
                        _ => ()
                    }
                }
                else {
                    match &packet {
                        Packet::Handshake { name } => {
                            if !self.accept_connections {
                                log_warning("Blocked connection attempt by '{}'. Reason: New connections disallowed.")
                            }
                            self.on_receive_handshake(name.clone(), sender_addr);
                        },
                        _ => ()
                    }
                }
                Ok((packet, sender_addr))
            },
            Err(e) => Err(e)
        }
    }

    fn on_receive_handshake(&mut self, name: String, remote_addr: SocketAddr) {
        self.connections.insert(self.curr_id, (ID::new(name, self.curr_id), remote_addr));
        self.curr_id += 1;
    }

    fn on_receive_disconnect(&mut self, sender: u16, reason: DisconnectReason) -> BbResult<()> {
        if let Some(conn) = self.connections.remove(&sender) {
            println!("{:?} disconnected. Reason: {:?}", conn.0, reason);
            self.send_multicast(Packet::Disconnect { reason }, sender)
        } else {
            Err(BbError::Bb(BbErrorType::InvalidPlayerID(sender)))
        }
    }
}