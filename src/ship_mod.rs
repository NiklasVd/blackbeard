use std::ops::{Add, AddAssign, DivAssign, MulAssign, SubAssign};

use tetra::{Context, graphics::Texture};
use crate::{BbResult, GC, Rcc, entity::GameState, ship::Ship};

pub struct Attribute<T>
    where T:
        Clone + Copy+ PartialEq + PartialOrd + Add<Output = T> + AddAssign + SubAssign + MulAssign + DivAssign + From<u8> {
    pub base: T,
    pub surplus: T
}

impl<T: Clone + Copy + PartialEq + PartialOrd + Add<Output = T> + AddAssign + SubAssign + MulAssign + DivAssign + From<u8>> Attribute<T> {
    pub fn new(base: T, surplus: T) -> Attribute<T> {
        Attribute {
            base, surplus
        }
    }

    pub fn setup(base: T) -> Attribute<T> {
        Self::new(base, 0u8.into())
    }

    pub fn add(&mut self, val: T) {
        self.surplus += val;
    }

    pub fn sub(&mut self, val: T) {
        self.surplus -= val;
    }

    pub fn total(&self) -> T {
        self.base + self.surplus
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShipModType {
    AmmoUpgrade,
    CannonUpgrade
}

pub trait ShipMod : GameState {
    fn get_name(&self) -> String;
    fn get_description(&self) -> String;
    fn get_type(&self) -> ShipModType;
    fn get_icon(&self) -> Texture;
    fn get_applied_ship(&self) -> Rcc<Ship>;
    fn on_remove(&mut self) -> BbResult;
}

pub struct AmmoUpgradeMod {
    icon: Texture,
    ship: Rcc<Ship>
}

impl AmmoUpgradeMod {
    pub fn new(ctx: &mut Context, ship: Rcc<Ship>, game: GC)
        -> tetra::Result<AmmoUpgradeMod> {
        let icon = game.borrow_mut().assets.load_texture(ctx,
            "UI/Ammo Upgrade Mod.png".to_owned(), true)?;
        {
            let mut ship_ref = ship.borrow_mut();
            for cannon in ship_ref.cannons.iter_mut() {
                cannon.dmg.add(Self::get_surplus_dmg()) // Make surplus dmg dynamic?
            }
        }
        Ok(AmmoUpgradeMod {
            icon, ship
        })
    }

    pub const fn get_surplus_dmg() -> u16 {
        5
    }
}

impl ShipMod for AmmoUpgradeMod {
    fn get_name(&self) -> String {
        "Cannon Ammo Upgrade".to_owned()
    }

    fn get_description(&self) -> String {
        format!("Upgrades damage of all cannons by {}", Self::get_surplus_dmg())
    }

    fn get_type(&self) -> ShipModType {
        ShipModType::AmmoUpgrade
    }

    fn get_icon(&self) -> Texture {
        self.icon.clone()
    }

    fn get_applied_ship(&self) -> Rcc<Ship> {
        self.ship.clone()
    }

    fn on_remove(&mut self) -> BbResult {
        for cannon in self.ship.borrow_mut().cannons.iter_mut() {
            cannon.dmg.sub(Self::get_surplus_dmg());
        }
        Ok(())
    }
}

impl GameState for AmmoUpgradeMod {
}

// pub struct CannonUpgradeMod {
//     icon: Texture,
//     ship: Rcc<Ship>
// }

// impl CannonUpgradeMod {
//     pub fn new(ctx: &mut Context, ) -> CannonUpgradeMod {
//         CannonUpgradeMod{}
//     }
// }
