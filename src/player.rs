use rapier2d::data::Index;
use tetra::Context;
use crate::{CannonSide, GC, ID, Rcc, StateEvent, TransformResult, World, entity::Entity, log_state_event, packet::InputState, ship::{Ship, ShipType}, ship_mod::{CannonAmmoUpgradeMod, CannonRangeUpgradeMod, CannonReloadUpgradeMod, ShipMod, ShipModType, get_ship_mod_cost}};

pub struct Player {
    pub id: ID,
    pub possessed_ship_index: Index,
    pub possessed_ship: Rcc<Ship>,
    game: GC
}

impl Player {
    pub fn new(id: ID, possessed_ship: Rcc<Ship>, game: GC) -> Player {
        let possessed_ship_index = possessed_ship.borrow().get_index();
        Player {
            id, possessed_ship_index, possessed_ship, game
        }
    }

    pub fn possess_ship(&mut self, possessed_ship: Rcc<Ship>) {
        self.possessed_ship_index = possessed_ship.borrow().get_index();
        self.possessed_ship = possessed_ship;
    }

    pub fn apply_state(&mut self, ctx: &mut Context, state: InputState,
        world: &mut World) -> tetra::Result {
        let mut ship_ref = self.possessed_ship.borrow_mut();
        if state.disconnect {
            ship_ref.destroy();
            return Ok(())
        }
        
        if state.q {
            ship_ref.shoot_cannons_on_side(ctx, CannonSide::Bowside, world)?;
        }
        if state.e {
            ship_ref.shoot_cannons_on_side(ctx, CannonSide::Portside, world)?;
        }
        if state.q || state.e {
            log_state_event(self.game.clone(), StateEvent::ShipShoot(
                self.id.n, state.q, state.e));
        }

        if let Some(mouse_pos) = state.mouse_pos {
            ship_ref.set_target_pos(mouse_pos, state.r);
            log_state_event(self.game.clone(), StateEvent::ShipMotion(
                self.id.n, state.r, mouse_pos));
        }

        if state.buy_mod && ship_ref.is_in_harbour {
            if let Some(mod_type) = state.mod_type {
                let cost = get_ship_mod_cost(mod_type);
                if ship_ref.treasury.balance < cost {
                    println!("{:?} does not have enough escudos to buy {:?}",
                        self.id, mod_type);
                } else {
                    ship_ref.treasury.spend(cost);
                    std::mem::drop(ship_ref);

                    match &mod_type {
                        ShipModType::Repair => {
                            self.possessed_ship.borrow_mut().repair();
                        },
                        ShipModType::CannonAmmoUpgrade => {
                            let mut ship_mod = CannonAmmoUpgradeMod::new(ctx,
                                self.possessed_ship.clone(), self.game.clone())?;
                            ship_mod.on_apply().convert()?;
                            self.possessed_ship.borrow_mut().apply_mod(ship_mod);
                        },
                        ShipModType::CannonReloadUpgrade => {
                            let mut ship_mod = CannonReloadUpgradeMod::new(ctx,
                                self.possessed_ship.clone(), self.game.clone())?;
                            ship_mod.on_apply().convert()?;
                            self.possessed_ship.borrow_mut().apply_mod(ship_mod);
                        },
                        ShipModType::CannonRangeUpgrade => {
                            let mut ship_mod = CannonRangeUpgradeMod::new(ctx, 
                                self.possessed_ship.clone(), self.game.clone())?;
                            ship_mod.on_apply().convert()?;
                            self.possessed_ship.borrow_mut().apply_mod(ship_mod);
                        },
                    };
                    println!("Player {:?} purchased and applied {:?} mod at harbour.",
                        self.id, mod_type);
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PlayerParams {
    pub id: ID,
    pub ship_type: ShipType
}

impl PlayerParams {
    pub fn new(id: ID, ship_type: ShipType) -> PlayerParams {
        PlayerParams {
            id, ship_type
        }
    }
}
