use std::{net::SocketAddr, thread::{self, JoinHandle}, time::Duration};
use binary_stream::{BinaryStream, Serializable};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use laminar::{Config, Packet as LaminarPacket, Socket, SocketEvent};
use crate::{BbError, BbErrorType, BbResult};

const HEARTBEAT_INTERVAL: f32 = 5.0;

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
        let mut socket = Socket::bind_with_config(format!("0.0.0.0:{}", port),
            Config {
                idle_connection_timeout: Duration::from_secs_f32(150.0),
                heartbeat_interval: Some(Duration::from_secs_f32(HEARTBEAT_INTERVAL)),
                ..Default::default()
            })
            .or_else(|err| Err(BbError::Laminar(err)))?;
        let sender = socket.get_packet_sender();
        let receiver = socket.get_event_receiver();
        let poll_thread = thread::spawn(move || loop {
            socket.start_polling();
        });
        Ok(Peer {
            sender, receiver, poll_thread
        })
    }
    
    pub fn send_raw_packet(&mut self, packet_bytes: Vec<u8>, target_addr: SocketAddr) -> BbResult {
        let packet = LaminarPacket::reliable_ordered(target_addr, packet_bytes, None);
        self.sender.send(packet).or_else(|err| Err(BbError::CrossbeamSender(err)))
    }

    pub fn poll_received_packets(&mut self) -> BbResult<Option<(LaminarPacket, SocketAddr)>> {
        let event = match self.receiver.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(e) if e == TryRecvError::Empty => {
                Ok(None)
            },
            Err(..) => Err(BbError::CrossbeamReceiver) // Disconnect
        }?;
        if let Some(event) = event {
            match event {
                SocketEvent::Packet(packet) => {
                    let addr = packet.addr();
                    Ok(Some((packet, addr)))
                },
                SocketEvent::Timeout(addr) => {
                    println!("{} timed out.", addr);
                    Err(BbError::Bb(BbErrorType::NetTimeout(addr)))
                },
                SocketEvent::Connect(addr) => {
                    Ok(None)
                },
                SocketEvent::Disconnect(addr) => {
                    println!("{} disconnected.", addr);
                    Err(BbError::Bb(BbErrorType::NetDisconnect(addr)))
                }
            }
        } else {
            Ok(None)
        }
    }

    pub fn shutdown(&mut self) -> BbResult { // Stop polling loop
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
            n @ _ => panic!("Index {} not assigned to any disconnect reason", n)
        }
    }
}

pub fn is_auth_client(id: u16) -> bool {
    id == 0
}
