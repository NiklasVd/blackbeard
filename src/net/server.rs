use std::{collections::{HashMap, HashSet}, iter::FromIterator, net::SocketAddr, thread, time::Duration};
use tetra::State;

use crate::{BbError, BbErrorType, BbResult, ID, TransformResult, net_settings::NetSettings, packet::{InputState, InputStep, Packet, deserialize_packet_unsigned, serialize_packet}, peer::{DisconnectReason, Peer, is_auth_client}};

pub const STEP_PHASE_FRAME_LENGTH: u32 = 15;
pub const MAX_CLIENT_STATE_SEND_DELAY: u32 = STEP_PHASE_FRAME_LENGTH * 10;

// Auth Client: First player to connect to the server
pub struct Server {
    settings: NetSettings,
    peer: Peer,
    connections: HashMap<u16, (ID, SocketAddr)>,
    curr_id: u16,
    input_pool: Option<InputPool>
}

impl Server {
    pub fn host(port: u16, settings: NetSettings) -> BbResult<Server> {
        println!("Hosting server at {}.", port);
        Ok(Server {
            settings, peer: Peer::setup(Some(port))?, connections: HashMap::new(),
            curr_id: 0, input_pool: None
        })
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

    pub fn shutdown(&mut self) -> BbResult {
        self.send_multicast(Packet::PlayerDisconnect {
            reason: DisconnectReason::HostShutdown
        }, 0)?;
        thread::sleep(Duration::from_secs_f32(2.0));
        self.peer.shutdown()
    }

    pub fn poll_received_packets(&mut self) -> BbResult {
        match self.peer.poll_received_packets() {
            Ok(Some((packet, sender_addr))) => {
                let packet = deserialize_packet_unsigned(packet.payload().to_vec());
                if let Some(id) = self.get_connection_by_addr(sender_addr) {
                    let id = id.clone();
                    self.handle_internal_packet(packet, id)
                }
                else {
                    self.handle_external_packet(packet, sender_addr)
                }
            },
            Ok(None) => Ok(()),
            Err(BbError::Bb(BbErrorType::NetDisconnect(addr)))
                | Err(BbError::Bb(BbErrorType::NetTimeout(addr)))  => {
                if let Some(id) = self.get_connection_by_addr(addr) {
                    let id = id.to_owned();
                    self.on_receive_disconnect(id.n, DisconnectReason::Timeout)
                } else {
                    println!("Received packet from unknown peer {}. Dropping...", addr);
                    Ok(())
                }
            },
            Err(e) => Err(e)
        }
    }

    fn handle_internal_packet(&mut self, packet: Packet, sender: ID) -> BbResult {
        //println!("Server: Received packet {:?} from {:?}", &packet, sender);
        match &packet {
            Packet::PlayerDisconnect { reason } => self.on_receive_disconnect(sender.n, *reason)?,
            Packet::Input { state } => {
                self.on_receive_input(sender.n, state.clone());
                return Ok(())
            },
            Packet::Game { phase } => {
                if !is_auth_client(sender.n) {
                    println!("Server: {:?} tried and failed to set the game phase: insufficient authority.", sender);
                    return Ok(())
                } else {
                    self.on_start_game();
                }
            }
            _ => ()
        }
        self.send_multicast(packet, sender.n)
    }

    fn handle_external_packet(&mut self, packet: Packet, sender_addr: SocketAddr) -> BbResult {
        match &packet {
            Packet::Handshake { name } => {
                if self.input_pool.is_some() {
                    println!("Server: Blocked connection attempt by {} ({}). Reason: Game is already running.",
                        name, sender_addr)
                } else if self.connections.len() >= self.settings.max_players {
                    println!("Server: Blocked connection attempt by {} ({}). Reason: Server is full.",
                        name, sender_addr)
                } else {
                    self.on_receive_handshake(name.clone(), sender_addr)?;
                }
                Ok(())
            },
            _ => {
                println!("Received packet {:?} from unknown peer {}. Dropping...", packet, sender_addr);
                Ok(())
            }
        }
    }

    fn on_receive_handshake(&mut self, name: String, remote_addr: SocketAddr) -> BbResult {
        let new_player_id = self.curr_id;
        self.curr_id += 1;
        let id = ID::new(name.to_owned(), new_player_id);
        println!("Server: {:?} ({}) joined the server.", &id, remote_addr);
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
            println!("Server: {:?} disconnected. Reason: {:?}", conn.0, reason);
            if let Some(pool) = self.input_pool.as_mut() {
                pool.remove_player(sender);
            }
            Ok(())
        } else {
            Err(BbError::Bb(BbErrorType::InvalidPlayerID(sender)))
        }
    }

    fn on_start_game(&mut self) {
        self.input_pool = Some(InputPool::new(self.connections.keys().map(|id| *id).collect()));
    }

    fn on_receive_input(&mut self, sender: u16, state: InputState) {
        if let Some(pool) = self.input_pool.as_mut() {
            pool.add_state(sender, state);
        } else {
            println!("Server: Dropping received input state of player {}. Game hasn't started yet.", sender);
        }
    }

    fn update_input_pool(&mut self) -> BbResult {
        if let Some(pool) = self.input_pool.as_mut() {
            pool.update_states();
            if pool.is_step_phase_over() { // By now clients should have sent all states, so server can bundle and send them back to all
                let delayed_players = pool.check_delayed_players();
                // In the first generation every player has to send their state, to signal they're ready
                if pool.curr_gen > 0 || delayed_players.len() == 0 {
                    for delayed_player in delayed_players.into_iter() {
                        println!("Server: Player {} failed to send input packet in time. Ignoring...", delayed_player);
                    }

                    let step = pool.flush_states();
                    println!("Server: Advancing step to gen {}.", pool.curr_gen);
                    std::mem::drop(pool);
                    self.send_multicast(Packet::InputStep {
                            step
                    }, 0)?;
                }
            }
        }
        Ok(())
    }
}

impl State for Server {
    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result {
        self.update_input_pool().convert()
    }
}

struct InputPool {
    players: HashSet<u16>,
    player_states: HashSet<u16>,
    input_states: HashMap<u16, InputState>,
    pub curr_gen: u64,
    pub curr_frame_index: u32
}

impl InputPool {
    fn new(players: Vec<u16>) -> Self {
        Self {
            players: HashSet::from_iter(players.into_iter()),
            player_states: HashSet::new(),
            input_states: HashMap::new(),
            curr_gen: 0, curr_frame_index: 0
        }
    }

    pub fn add_state(&mut self, sender: u16, state: InputState) {
        self.player_states.insert(sender);
        self.input_states.insert(sender, state); // If client sends state more than once during step, overwrite
    }

    pub fn remove_player(&mut self, id: u16) {
        self.players.remove(&id);
        self.player_states.remove(&id);
        self.input_states.remove(&id);
    }

    pub fn is_step_phase_over(&self) -> bool {
        self.curr_frame_index >= STEP_PHASE_FRAME_LENGTH
    }

    pub fn is_max_delay_over(&self) -> bool {
        self.curr_frame_index >= MAX_CLIENT_STATE_SEND_DELAY
    }

    pub fn check_delayed_players(&mut self) -> Vec<u16> {
        self.players.iter().filter(|id| !self.player_states.contains(id)).map(|id| *id).collect()
    }

    pub fn update_states(&mut self) {
        self.curr_frame_index += 1;
    }

    pub fn flush_states(&mut self) -> InputStep {
        self.player_states.clear();
        self.curr_frame_index = 0;
        self.curr_gen += 1;

        let states = self.input_states.drain().collect::<Vec<_>>();
        InputStep::new(states, self.curr_gen)
    }
}
