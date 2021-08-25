use std::{net::SocketAddr, thread::{self, JoinHandle}};
use binary_stream::{BinaryStream, Serializable};
use crossbeam_channel::{Receiver, Sender};
use laminar::{Packet as LaminarPacket, Socket, SocketEvent};
use crate::{BbError, BbErrorType, BbResult};

pub trait NetPeer {
    fn get_peer(&self) -> &Peer;
}

pub struct Peer {
    sender: Sender<LaminarPacket>,
    receiver: Receiver<SocketEvent>,
    poll_thread: JoinHandle<()>
}

impl Peer {
    pub fn setup(port: u16) -> BbResult<Self> {
        let mut socket = Socket::bind(format!("0.0.0.0:{}", port))
            .or_else(|err| Err(BbError::Laminar(err)))?;
        let sender = socket.get_packet_sender();
        let receiver = socket.get_event_receiver();
        let poll_thread = thread::spawn(move || socket.start_polling());
        Ok(Peer {
            sender, receiver, poll_thread
        })
    }

    pub fn send_raw_packet(&mut self, packet_bytes: Vec<u8>, target_addr: SocketAddr) -> BbResult<()> {
        let packet = LaminarPacket::reliable_ordered(target_addr, packet_bytes, None);
        self.sender.send(packet).or_else(|err| Err(BbError::CrossbeamSender(err)))
    }

    pub fn poll_received_packets(&mut self) -> BbResult<(LaminarPacket, SocketAddr)> {
        let event = self.receiver.try_recv()
            .or_else(|err| Err(BbError::CrossbeamReceiver(err)))?;
        match event {
            SocketEvent::Packet(packet) => {
                let addr = packet.addr();
                Ok((packet, addr))
            },
            SocketEvent::Timeout(addr) => {
                Err(BbError::Bb(BbErrorType::NetTimeout(addr)))
            },
            // Handle connect/disconnect
            _ => Err(BbError::Bb(BbErrorType::NetPollEmpty))
        }
    }

    pub fn shutdown(&mut self) -> BbResult<()> { // Stop polling loop
        // self.poll_thread.join()
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DisconnectReason {
    Manual,
    Timeout
}

impl Serializable for DisconnectReason {
    fn to_stream(&self, stream: &mut BinaryStream) {
        stream.write_buffer_single(match self {
            DisconnectReason::Manual => 0,
            DisconnectReason::Timeout => 1,
        }).unwrap();
    }

    fn from_stream(stream: &mut BinaryStream) -> Self {
        match stream.read_buffer_single().unwrap() {
            0 => DisconnectReason::Manual,
            1 => DisconnectReason::Timeout,
            _ => panic!("Index not assigned to any disconnect reason")
        }
    }
}
