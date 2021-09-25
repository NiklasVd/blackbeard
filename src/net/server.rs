use std::{collections::{HashMap}, net::SocketAddr, thread, time::{Duration}};
use tetra::State;
use crate::{BbError, BbErrorType, BbResult, ID, TransformResult, input_pool::InputPool, net_settings::NetSettings, packet::{InputState, Packet, deserialize_packet_unsigned, serialize_packet}, peer::{DisconnectReason, Peer, is_auth_client}, sync_checker::{SyncChecker, SyncState}};

// Auth Client: First player to connect to the server, i.e., with ID of zero
pub struct Server {
    settings: NetSettings,
    peer: Peer,
    connections: HashMap<u16, (ID, SocketAddr)>,
    curr_id: u16,
    input_pool: Option<InputPool>,
    sync_checker: Option<SyncChecker>
}

impl Server {
    pub fn host(port: u16, settings: NetSettings) -> BbResult<Server> {
        println!("Hosting server at {}.", port);
        Ok(Server {
            settings, peer: Peer::setup(Some(port))?, connections: HashMap::new(),
            curr_id: 0, input_pool: None, sync_checker: None
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
        thread::sleep(Duration::from_secs_f32(1.5));
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
                    self.disconnect_player(id.n, DisconnectReason::Timeout)
                } else {
                    println!("Received packet from unknown peer {}. Dropping...", addr);
                    Ok(())
                }
            },
            Err(e) => Err(e)
        }
    }

    fn handle_internal_packet(&mut self, packet: Packet, sender: ID) -> BbResult {
        match &packet {
            Packet::PlayerDisconnect { reason } =>
                return self.disconnect_player(sender.n, *reason),
            Packet::Input { state } =>
                return self.on_receive_input(sender.n, state.clone()),
            Packet::Game { phase } => {
                if !is_auth_client(sender.n) {
                    println!("Server: {:?} tried and failed to set the game phase: insufficient authority.", sender);
                    return Ok(())
                } else {
                    self.on_start_game();
                }
            },
            Packet::Sync { state } => return self.on_receive_sync(sender.n, *state),
            Packet::Handshake { .. } => {
                // ?!
                return Ok(())
            },
            _ => ()
        }
        // If not returned before, echo back the packet to all clients
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

    fn disconnect_player(&mut self, sender: u16, reason: DisconnectReason) -> BbResult {
        if let Some(conn) = self.connections.remove(&sender) {
            println!("Server: {:?} disconnected. Reason: {:?}", conn.0, reason);
            if let Some(pool) = self.input_pool.as_mut() {
                pool.remove_player(sender);
            }
            // As this method is also called upon timeouts (which don't simply echo back
            // all client packets), this is done manually here.
            self.send_multicast(Packet::PlayerDisconnect {
                reason
            }, sender)
        } else {
            println!("Server: Player with ID {} already disconnected.", sender);
            Ok(())
        }
    }

    fn on_start_game(&mut self) {
        self.input_pool = Some(InputPool::new(
            self.connections.keys().map(|id| *id).collect()));
        self.sync_checker = Some(SyncChecker::new());
    }

    fn on_receive_sync(&mut self, sender: u16, state: SyncState) -> BbResult {
        if let Some(sync_checker) = self.sync_checker.as_mut() {
            let state_gen = state.gen;
            sync_checker.add_state(sender, state);
            for id in sync_checker.get_desynced_players() {
                println!("Player with ID {} is out of sync. Terminating connection...", id);
                //self.disconnect_player(id, DisconnectReason::Desync)?;
            }
        } else {
            println!("Server: Dropping sync state of player {}. Game hasn't started yet.", sender);
        }
        Ok(())
    }

    fn on_receive_input(&mut self, sender: u16, state: InputState) -> BbResult {
        if let Some(pool) = self.input_pool.as_mut() {            
            pool.add_state(sender, state);
            self.check_input_pool()
        } else {
            println!("Server: Dropping input state of player {}. Game hasn't started yet.", sender);
            Ok(())
        }
    }

    fn update_input_pool(&mut self) -> BbResult {
        if let Some(pool) = self.input_pool.as_mut() {
            pool.update_states();
            self.check_input_pool()
        } else {
            Ok(())
        }
    }

    fn check_input_pool(&mut self) -> BbResult {
        if let Some(pool) = self.input_pool.as_mut() {
            if pool.is_step_phase_over() { // By now clients should have sent all states, so server can bundle and send them back to all
                let delayed_players = pool.check_delayed_players();
                // In the first generation every player has to send their state, to signal they're ready
                if pool.curr_gen > 0 || delayed_players.len() == 0 {
                    let step = pool.flush_states();
                    self.send_multicast(Packet::InputStep {
                            step
                    }, 0)?;
                } else if pool.curr_gen == 0 && delayed_players.len() > 0
                    && pool.is_max_delay_exceeded() {
                    for id in delayed_players.iter() {
                        println!("Player with ID {} failed to send first input state in time. Terminating connection...", id);
                        self.disconnect_player(*id, DisconnectReason::Timeout)?;
                    }
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

