use std::time::Duration;

use tetra::{Context, Event, State};
use crate::{BbError, BbErrorType, BbResult, Controller, Entity, GC, GameState, ID, Player, Rcc, ShipType, TransformResult, V2, grid::{Grid, UIAlignment}, label::Label, menu_scene::MenuScene, net_controller::NetController, packet::{InputStep, Packet}, peer::DisconnectReason, world::World};
use super::scenes::{Scene, SceneType};

pub const MAX_INPUT_STEP_BLOCK_FRAMES: u32 = 60 * 15;

pub struct WorldScene {
    pub controller: Controller,
    pub world: World,
    grid: Grid,
    back_to_menu: bool,
    game: GC
}

impl WorldScene {
    pub fn new(ctx: &mut Context, players: Vec<(ID, ShipType)>, game: GC) -> BbResult<WorldScene> {
        let mut grid = Grid::new(ctx, UIAlignment::Vertical,
            V2::zero(), V2::one() * 200.0, 0.0).convert()?;
        grid.add_element(Label::new(ctx, "Pre-Alpha WIP", false, 2.0, game.clone()).convert()?);
        let mut world_scene = WorldScene {
            controller: Controller::new(ctx, game.clone()).convert()?,
            world: World::new(ctx, game.clone()),
            grid, back_to_menu: false, game: game.clone()
        };
        
        let local_id = {
            if let Some(network) = game.borrow().network.as_ref() {
                network.client.get_local_id()
            } else {
                None
            }
        };
        for (id, ship_type) in players.into_iter() {
            let player = world_scene.add_player(ctx, id.clone(), ship_type)?;
            if let Some(local_id) = local_id.as_ref() {
                if &id != local_id {
                    continue
                }
            }
            world_scene.controller.set_local_player(player);
        }
        world_scene.world.add_island(ctx, V2::new(800.0, 500.0), 0.6, 1).convert()?;
        world_scene.world.add_island(ctx, V2::new(150.0, 1000.0), 4.0, 2).convert()?;
        world_scene.world.add_island(ctx, V2::new(-200.0, -800.0), 0.0, 1).convert()?;
        world_scene.world.add_island(ctx, V2::new(-900.0, 200.0), 0.0, 3).convert()?;
        world_scene.world.add_island(ctx, V2::new(1200.0, -500.0), 0.0, 3).convert()?;
        world_scene.world.add_island(ctx, V2::new(-600.0, -300.0), 1.0, 2).convert()?;

        Ok(world_scene)
    }

    pub fn add_player(&mut self, ctx: &mut Context, id: ID, ship_type: ShipType) -> BbResult<Rcc<Player>> {
        let ship = self.world.add_player_ship(ctx, id.clone(), ship_type).convert()?;
        Ok(self.controller.add_player(Player::new(id, ship)))
    }
}

impl Scene for WorldScene {
    fn get_grid(&self) -> &Grid {
        &self.grid
    }

    fn get_grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    fn poll(&self, ctx: &mut Context) -> BbResult<Option<Box<dyn Scene>>> {
        Ok(if self.back_to_menu {
            Some(Box::new(MenuScene::new(ctx, self.game.clone()).convert()?))
        } else {
            None
        })
    }

    fn get_type(&self) -> SceneType {
        SceneType::World
    }
}

impl State for WorldScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.handle_received_packets(ctx).convert()?;
        self.controller.update(ctx, &mut self.world)?;
        self.world.update(ctx)
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.controller.draw(ctx)?;
        self.world.draw(ctx)
    }

    fn event(&mut self, ctx: &mut Context, event: Event)
        -> tetra::Result {
        self.controller.event(ctx, event.clone(), &mut self.world)?;
        self.world.draw(ctx)
    }
}

impl NetController for WorldScene {
    fn poll_received_packets(&mut self, ctx: &mut Context) -> BbResult<Option<(Packet, u16)>> {
        let mut poll_iterations = 0;
        loop {
            let packet = {
                self.game.borrow_mut().network.as_mut().unwrap().poll_received_packets()?
            };
            if self.controller.wait_next_step() {
                match packet {
                    Some((packet, sender)) => {
                        match &packet {
                                &Packet::InputStep { .. } => return Ok(Some((packet, sender))),
                            _ => self.handle_packets(ctx, (packet, sender))?
                        }
                    },
                    None => ()
                }

                std::thread::sleep(Duration::from_millis(1));
                poll_iterations += 1;
                if poll_iterations % 120 == 0 {
                    println!("Blocking until next input step arrives...");
                } else if poll_iterations % MAX_INPUT_STEP_BLOCK_FRAMES == 0 {
                    println!("Failed to procure next input step from server. Terminating connection to server...");
                    self.game.borrow_mut().network.as_mut().unwrap().disconnect(DisconnectReason::Timeout)?;
                    self.back_to_menu = true;
                    return Ok(None)
                }
                continue
            } else {
                return Ok(packet)
            }
        }
    }

    fn on_connection_lost(&mut self, ctx: &mut Context, reason: crate::peer::DisconnectReason) -> BbResult {
        println!("Lost connection to server! Aborting game...");
        self.back_to_menu = true;
        Ok(())
    }
    
    fn on_player_disconnect(&mut self, ctx: &mut Context, id: u16, reason: DisconnectReason) -> BbResult {
        if let Some(player) = self.controller.remove_player(id) {
            player.borrow_mut().possessed_ship.borrow_mut().destroy();
            println!("Player with ID {} disconnected. Reason: {:?}", id, reason);
            Ok(())
        } else {
            Err(BbError::Bb(BbErrorType::InvalidPlayerID(id)))
        }
    }

    fn on_input_step(&mut self, ctx: &mut Context, step: InputStep) -> BbResult {
        self.controller.add_step(step);
        Ok(())
    }
}
