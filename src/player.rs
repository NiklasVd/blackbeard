use binary_stream::{BinaryStream, Serializable};
use rapier2d::data::Index;
use tetra::Context;
use crate::{CannonSide, GC, ID, Rcc, TransformResult, World, entity::Entity, packet::InputState, ship::{Ship}, ship_data::ShipType, ship_mod::{CannonAmmoUpgradeMod, CannonRangeUpgradeMod, CannonReloadUpgradeMod, ShipMod, ShipModType, get_ship_mod_cost}};

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
        
        if state.q && state.e {
            ship_ref.shoot_cannons(ctx, None, world)?;
        } else if state.q {
            ship_ref.shoot_cannons(ctx, Some(CannonSide::Bowside), world)?;
        } else if state.e {
            ship_ref.shoot_cannons(ctx, Some(CannonSide::Portside), world)?;
        }

        if let Some(mouse_pos) = state.mouse_pos {
            ship_ref.set_target_pos(mouse_pos, state.r);
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
    pub fn new(id: ID) -> PlayerParams {
        PlayerParams {
            id, ship_type: ShipType::Caravel
        }
    }
}

impl Serializable for PlayerParams {
    fn to_stream(&self, stream: &mut BinaryStream) {
        self.id.to_stream(stream);
        self.ship_type.to_stream(stream);
    }

    fn from_stream(stream: &mut BinaryStream) -> Self {
        let id = ID::from_stream(stream);
        let ship_type = ShipType::from_stream(stream);
        PlayerParams {
            id, ship_type
        }
    }
}
