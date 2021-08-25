use std::net::SocketAddr;
use crate::{BbError, BbErrorType, BbResult, packet::{Packet, deserialize_packet, serialize_packet_unsigned}, peer::Peer};

pub struct Client {
    peer: Peer,
    server_addr: SocketAddr
}

impl Client {
    pub fn connect(port: u16, server_addr: &str, name: String) -> BbResult<Client> {
        let mut client = Client {
            peer: Peer::setup(port)?, server_addr: server_addr.parse().unwrap()
        };
        client.send_packet(Packet::Handshake {
            name
        })?;
        Ok(client)
    }

    pub fn send_packet(&mut self, packet: Packet) -> BbResult<()> {
        self.peer.send_raw_packet(serialize_packet_unsigned(packet), self.server_addr)
    }

    pub fn poll_received_packets(&mut self) -> BbResult<(Packet, u16)> {
        match self.peer.poll_received_packets() {
            Ok((packet, sender)) => {
                if sender != self.server_addr {
                    Err(BbError::Bb(BbErrorType::NetInvalidSender(sender)))
                } else {
                    Ok(deserialize_packet(packet.payload().to_vec()))
                }
            },
            Err(e) => Err(e)
        }
    }
}