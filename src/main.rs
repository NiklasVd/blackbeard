#![allow(unused_variables)]

mod game;
mod physics;
mod assets;
mod entities {
    pub mod entity;
    pub mod object;
    pub mod ship;
    pub mod cannon;
    pub mod harbour;
}
mod ship_mod;
mod controller;
mod scenes {
    pub mod scenes;
    pub mod startup_scene;
    pub mod menu_scene;
    pub mod world_scene;
    pub mod loading_scene;
    pub mod lobby_scene;
    pub mod connection_scene;
    pub mod login_scene;
}
mod player;
mod util;
mod settings;
mod world_settings;
mod cam;
mod transform;
mod id;
mod sprite;
mod animated_sprite;
mod ui {
    pub mod grid;
    pub mod ui_element;
    pub mod ui_transform;
    pub mod label;
    pub mod button;
    pub mod spritesheet;
    pub mod image;
    pub mod textbox;
    pub mod chat;
}
mod net {
    pub mod network;
    pub mod packet;
    pub mod peer;
    pub mod client;
    pub mod server;
    pub mod playback_buffer;
    pub mod net_controller;
    pub mod net_settings;
}
mod err;
mod diagnostics;
mod world;

pub use game::*;
pub use physics::*;
pub use assets::*;
pub use controller::*;
pub use scenes::*;
pub use player::*;
pub use util::*;
pub use settings::*;
pub use world_settings::*;
pub use cam::*;
pub use transform::*;
pub use id::*;
pub use sprite::*;
pub use animated_sprite::*;
pub use cannon::*;
pub use ui::*;
pub use net::*;
pub use err::*;
pub use diagnostics::*;
pub use world::*;
pub use entities::*;

use tetra::{ContextBuilder};
use std::io::{Read, stdin};

pub const PRIMARY_VERSION: u32 = 0;
pub const SECONDARY_VERSION: u32 = 1;

fn get_version() -> String {
    format!("v{}.{}", PRIMARY_VERSION, SECONDARY_VERSION)
}

fn main() -> tetra::Result {
    println!("Blackbeard {} - (c) 2021, Niklas Vaudt", get_version());
    let args: Vec<String> = std::env::args().collect();
    println!("Startup params: {:?}", args);
    if let Err(e) = ContextBuilder::new("Blackbeard", 1200, 650)
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
