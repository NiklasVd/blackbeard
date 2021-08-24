#![allow(unused_variables)]

mod game;
mod ship;
mod physics;
mod assets;
mod controller;
mod entity;
mod scenes {
    pub mod scenes;
    pub mod startup_scene;
    pub mod menu_scene;
    pub mod world_scene;
    pub mod loading_scene;
    pub mod lobby_scene;
    pub mod connection_scene;
}
mod player;
mod util;
mod settings;
mod world;
mod cam;
mod object;
mod transform;
mod id;
mod sprite;
mod animated_sprite;
mod cannon;
mod ui {
    pub mod grid;
    pub mod ui_element;
    pub mod ui_transform;
    pub mod label;
    pub mod button;
    pub mod spritesheet;
    pub mod image;
    pub mod textbox;
}
mod net {
    pub mod network;
}

pub use game::*;
pub use ship::*;
pub use physics::*;
pub use assets::*;
pub use controller::*;
pub use entity::*;
pub use scenes::*;
pub use player::*;
pub use util::*;
pub use settings::*;
pub use world::*;
pub use cam::*;
pub use object::*;
pub use transform::*;
pub use id::*;
pub use sprite::*;
pub use animated_sprite::*;
pub use cannon::*;
pub use ui::*;

use tetra::ContextBuilder;
use std::io::{Read, stdin};

pub const WINDOW_WIDTH: f32 = 900.0;
pub const WINDOW_HEIGHT: f32 = 600.0;

pub const PRIMARY_VERSION: u32 = 0;
pub const SECONDARY_VERSION: u32 = 1;

fn get_version() -> String {
    format!("v{}.{}", PRIMARY_VERSION, SECONDARY_VERSION)
}

fn main() -> tetra::Result {
    println!("Blackbeard {} - (c) 2021, Niklas Vaudt", get_version());
    if let Err(e) = ContextBuilder::new("Blackbeard", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .quit_on_escape(true)
        .debug_info(true)
        .high_dpi(true)
        .show_mouse(true)
        .build()?
        .run(Game::new)
    {
        println!("Game loop encountered an error: {}", e);
        stdin().read(&mut Vec::new()).unwrap();
        return Err(e)
    }
    Ok(())
}
