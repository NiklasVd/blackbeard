use std::ops::{Add, AddAssign, DivAssign, MulAssign, SubAssign};

use binary_stream::{BinaryStream, Serializable};
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
    Repair,
    CannonAmmoUpgrade,
    CannonReloadUpgrade,
    CannonRangeUpgrade
}

impl Serializable for ShipModType {
    fn to_stream(&self, stream: &mut BinaryStream) {
        stream.write_buffer_single(match self {
            ShipModType::Repair => 0,
            ShipModType::CannonAmmoUpgrade => 1,
            ShipModType::CannonReloadUpgrade => 2,
            ShipModType::CannonRangeUpgrade => 3
        }).unwrap();
    }

    fn from_stream(stream: &mut BinaryStream) -> Self {
        match stream.read_buffer_single().unwrap() {
            0 => ShipModType::Repair,
            1 => ShipModType::CannonAmmoUpgrade,
            2 => ShipModType::CannonReloadUpgrade,
            3 => ShipModType::CannonRangeUpgrade,
            n @ _ => panic!("Index {} not assigned to any ship mod type", n)
        }
    }
}

pub trait ShipMod : GameState {
    fn get_name(&self) -> String;
    fn get_description(&self) -> String;
    fn get_type(&self) -> ShipModType;
    fn get_icon(&self) -> Texture;
    fn get_applied_ship(&self) -> Rcc<Ship>;
    fn on_apply(&mut self) -> BbResult;
    fn on_remove(&mut self) -> BbResult;
}

pub const HARBOUR_REPAIR_COST: u32 = 25;
pub const CANNON_AMMO_UPGRADE_MOD_COST: u32 = 120;
pub const CANNON_RELOAD_UPGRADE_MOD_COST: u32 = 110;
pub const CANNON_RANGE_UPGRADE_MOD_COST: u32 = 100;

pub fn get_ship_mod_cost(ship_mod: ShipModType) -> u32 {
    match ship_mod {
        ShipModType::Repair => HARBOUR_REPAIR_COST,
        ShipModType::CannonAmmoUpgrade => CANNON_AMMO_UPGRADE_MOD_COST,
        ShipModType::CannonReloadUpgrade => CANNON_RELOAD_UPGRADE_MOD_COST,
        ShipModType::CannonRangeUpgrade => CANNON_RANGE_UPGRADE_MOD_COST
    }
}

pub struct CannonAmmoUpgradeMod {
    icon: Texture,
    ship: Rcc<Ship>
}

impl CannonAmmoUpgradeMod {
    pub fn new(ctx: &mut Context, ship: Rcc<Ship>, game: GC) // Separate apply method
        -> tetra::Result<CannonAmmoUpgradeMod> {
        let icon = game.borrow_mut().assets.load_texture(ctx,
            "UI/Ammo Upgrade Mod.png".to_owned(), true)?;
        Ok(CannonAmmoUpgradeMod {
            icon, ship
        })
    }

    pub const fn get_surplus_dmg() -> u16 {
        5
    }
}

impl ShipMod for CannonAmmoUpgradeMod {
    fn get_name(&self) -> String {
        "Cannon Ammo Upgrade".to_owned()
    }

    fn get_description(&self) -> String {
        format!("Upgrades damage of all cannons by {}", Self::get_surplus_dmg())
    }

    fn get_type(&self) -> ShipModType {
        ShipModType::CannonAmmoUpgrade
    }

    fn get_icon(&self) -> Texture {
        self.icon.clone()
    }

    fn get_applied_ship(&self) -> Rcc<Ship> {
        self.ship.clone()
    }

    fn on_apply(&mut self) -> BbResult {
        for cannon in self.ship.borrow_mut().cannons.iter_mut() {
            cannon.dmg.add(Self::get_surplus_dmg()) // Make surplus dmg dynamic?
        }
        Ok(())
    }

    fn on_remove(&mut self) -> BbResult {
        for cannon in self.ship.borrow_mut().cannons.iter_mut() {
            cannon.dmg.sub(Self::get_surplus_dmg());
        }
        Ok(())
    }
}

impl GameState for CannonAmmoUpgradeMod {
}

pub struct CannonReloadUpgradeMod {
    icon: Texture,
    ship: Rcc<Ship>
}

impl CannonReloadUpgradeMod {
    pub fn new(ctx: &mut Context, ship: Rcc<Ship>, game: GC)
        -> tetra::Result<CannonReloadUpgradeMod> {
        Ok(CannonReloadUpgradeMod {
            icon: game.borrow_mut().assets.load_texture(ctx,
                "UI/Cannon Reload Upgrade Mod.png".to_owned(), true)?,
            ship
        })
    }

    pub const fn get_reload_decrease() -> f32 {
        1.5
    }
}

impl ShipMod for CannonReloadUpgradeMod {
    fn get_name(&self) -> String {
        "Cannon Reload Upgrade".to_owned()
    }

    fn get_description(&self) -> String {
        format!("Improves cannon reload mechanism to reload {}s faster.", Self::get_reload_decrease())
    }

    fn get_type(&self) -> ShipModType {
        ShipModType::CannonReloadUpgrade
    }

    fn get_icon(&self) -> Texture {
        self.icon.clone()
    }

    fn get_applied_ship(&self) -> Rcc<Ship> {
        self.ship.clone()
    }

    fn on_apply(&mut self) -> BbResult {
        for cannon in self.ship.borrow_mut().cannons.iter_mut() {
            cannon.change_reload_time(-Self::get_reload_decrease());
        }
        Ok(())
    }

    fn on_remove(&mut self) -> BbResult {
        for cannon in self.ship.borrow_mut().cannons.iter_mut() {
            cannon.change_reload_time(Self::get_reload_decrease());
        }
        Ok(())
    }
}

impl GameState for CannonReloadUpgradeMod {
}

pub struct CannonRangeUpgradeMod {
    icon: Texture,
    ship: Rcc<Ship>
}

impl CannonRangeUpgradeMod {
    pub fn new(ctx: &mut Context, ship: Rcc<Ship>, game: GC)
        -> tetra::Result<CannonRangeUpgradeMod> {
        Ok(CannonRangeUpgradeMod {
            icon: game.borrow_mut().assets.load_texture(ctx,
                "UI/Cannon Range Upgrade Mod.png".to_owned(), true)?,
            ship
        })
    }

    pub const fn get_surplus_range_percentage() -> f32 {
        0.4
    }
}

impl ShipMod for CannonRangeUpgradeMod {
    fn get_name(&self) -> String {
        "Cannon Range Upgrade".to_owned()
    }

    fn get_description(&self) -> String {
        format!("Increases cannon power by {}%, extending their range.",
            Self::get_surplus_range_percentage() * 100.0)
    }

    fn get_type(&self) -> ShipModType {
        ShipModType::CannonRangeUpgrade
    }

    fn get_icon(&self) -> Texture {
        self.icon.clone()
    }

    fn get_applied_ship(&self) -> Rcc<Ship> {
        self.ship.clone()
    }

    fn on_apply(&mut self) -> BbResult {
        for cannon in self.ship.borrow_mut().cannons.iter_mut() {
            cannon.shooting_power.add(Self::get_surplus_range_percentage());
        }
        Ok(())
    }

    fn on_remove(&mut self) -> BbResult {
        for cannon in self.ship.borrow_mut().cannons.iter_mut() {
            cannon.shooting_power.sub(Self::get_surplus_range_percentage());
        }
        Ok(())
    }
}

impl GameState for CannonRangeUpgradeMod {
}
