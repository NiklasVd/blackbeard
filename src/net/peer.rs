use std::{net::SocketAddr, sync::{Arc, atomic::{AtomicBool, Ordering}}, thread::{self, JoinHandle}, time::{Duration, Instant}};
use binary_stream::{BinaryStream, Serializable};
use crossbeam_channel::{Receiver, Sender};
use laminar::{Config, Packet as LaminarPacket, Socket, SocketEvent};
use crate::{BbError, BbErrorType, BbResult};

const IDLE_TIMEOUT_DURATION: f32 = 15.0;
const HEARTBEAT_INTERVAL: f32 = 5.0;

pub trait NetPeer {
    fn get_peer(&self) -> &Peer;
}

pub struct Peer {
    sender: Sender<LaminarPacket>,
    receiver: Receiver<SocketEvent>,
    poll_thread: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>
}

impl Peer {
    pub fn setup(port: Option<u16>) -> BbResult<Self> {
        let config = Config {
            idle_connection_timeout: Duration::from_secs_f32(IDLE_TIMEOUT_DURATION),
            heartbeat_interval: Some(Duration::from_secs_f32(HEARTBEAT_INTERVAL)),
            socket_event_buffer_size: 1024 * 50,
            ..Default::default()
        };
        let mut socket = match port {
            Some(port) => Socket::bind_with_config(format!("0.0.0.0:{}", port), config),
            None => Socket::bind_with_config("0.0.0.0:0", config)
        }.or_else(|err| Err(BbError::Laminar(err)))?;
        let sender = socket.get_packet_sender();
        let receiver = socket.get_event_receiver();

        let running = Arc::new(AtomicBool::new(true));
        let running_ref = running.clone();
        let poll_thread = Some(thread::spawn(move || {
            while running_ref.load(Ordering::Relaxed) {
                socket.manual_poll(Instant::now());
                thread::sleep(Duration::from_millis(1));
            };
        }));
        Ok(Peer {
            sender, receiver, poll_thread, running
        })
    }
    
    pub fn send_raw_packet(&mut self, packet_bytes: Vec<u8>, target_addr: SocketAddr) -> BbResult {
        let packet = LaminarPacket::reliable_ordered(target_addr, packet_bytes, None);
        if let Err(e) = self.sender.send(packet) {
            println!("Failed to send packet: {:?}", e);
        }
        Ok(())
    }

    pub fn poll_received_packets(&mut self) -> BbResult<Option<(LaminarPacket, SocketAddr)>> {
        if !self.running.load(Ordering::Relaxed) {
            return Ok(None)
        }

        let event = match self.receiver.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(e) => {
                match e {
                    crossbeam_channel::TryRecvError::Disconnected =>
                        println!("Poll loop error: {}", e),
                    _ => ()
                }                
                Ok(None)
            }
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
        if let Some(poll_thread) = self.poll_thread.take() {
            self.running.store(false, Ordering::Relaxed);
            poll_thread.join().or_else(
                |e| Err(BbError::Bb(BbErrorType::NetShutdownFailure(e))))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisconnectReason {
    Manual,
    Timeout,
    HostShutdown,
    Desync
}

impl Serializable for DisconnectReason {
    fn to_stream(&self, stream: &mut BinaryStream) {
        stream.write_buffer_single(match self {
            DisconnectReason::Manual => 0,
            DisconnectReason::Timeout => 1,
            DisconnectReason::HostShutdown => 2,
            DisconnectReason::Desync => 3
        }).unwrap();
    }

    fn from_stream(stream: &mut BinaryStream) -> Self {
        match stream.read_buffer_single().unwrap() {
            0 => DisconnectReason::Manual,
            1 => DisconnectReason::Timeout,
            2 => DisconnectReason::HostShutdown,
            3 => DisconnectReason::Desync,
            n @ _ => panic!("Index {} not assigned to any disconnect reason", n)
        }
    }
}

pub fn is_auth_client(id: u16) -> bool {
    id == 0
}
