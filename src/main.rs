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
    pub mod input_pool;
    pub mod sync_checker;
}
mod err;
mod diagnostics;
mod world;
mod economy;

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

pub const DEFAULT_WINDOW_SIZE_WIDTH: i32 = 1200;
pub const DEFAULT_WINDOW_SIZE_HEIGHT: i32 = 640;

pub const PRIMARY_VERSION: u32 = 0;
pub const SECONDARY_VERSION: u32 = 1;

fn get_version() -> String {
    format!("v{}.{}", PRIMARY_VERSION, SECONDARY_VERSION)
}

fn main() -> tetra::Result {
    println!("Blackbeard {} - (c) 2021, Niklas Vaudt", get_version());
    let startup_params = process_params();

    if let Err(e) = ContextBuilder::new("Blackbeard", startup_params.1, startup_params.2)
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

struct StartupParams(String, i32, i32);

fn process_params() -> StartupParams {
    let mut args: Vec<String> = std::env::args().collect();
    let startup_path = args[0].to_owned();
    if args.len() >= 3 {
        let window_size_params = args.drain(1..3);
        let mut x: i32 = DEFAULT_WINDOW_SIZE_WIDTH;
        let mut y: i32 = DEFAULT_WINDOW_SIZE_HEIGHT;
        for (i, n) in window_size_params.map(|s| s.parse::<i32>().unwrap()).enumerate() {
            if i == 0 {
                x = n;
            } else {
                y = n;
            }
        }
        StartupParams(startup_path, x, y)
    } else {
        StartupParams(startup_path, DEFAULT_WINDOW_SIZE_WIDTH, DEFAULT_WINDOW_SIZE_HEIGHT)
    }
}
