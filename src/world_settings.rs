use std::f32::consts::PI;

use rand::{SeedableRng};
use rand_xoshiro::{Xoshiro128Plus};
use tetra::Context;
use worldgen::constraint;
use worldgen::noisemap::{NoiseMapGenerator, Seed, Size, Step};
use worldgen::world::tile::{Constraint, ConstraintType};
use worldgen::{noise::perlin::PerlinNoise, noisemap::NoiseMap};
use worldgen::world::{Tile, World as NoiseWorld};
use crate::{V2, World, rand_f32};

pub struct WorldSettings {
    events: Vec<WorldEvent>
}

impl WorldSettings {
    pub fn new() -> WorldSettings {
        WorldSettings {
            events: Vec::new()
        }
    }

    pub fn add_event(&mut self, event: WorldEvent) {
        self.events.push(event);
    }

    pub fn flush_events(&mut self) -> Vec<WorldEvent> {
        self.events.drain(0..).collect()
    }
}

pub enum WorldEvent {
    PlayerSunkByCannon(String, String),
    PlayerSunkByRamming(String, String),
    PlayerSunkByAccident(String)
}

#[derive(Debug, Clone, Copy)]
enum WorldTile {
    Island(u32),
    Harbour,
    Reef,
    Empty,
}

pub fn gen_world(ctx: &mut Context, width: i64, height: i64, base_tile_size: f32,
    tile_padding: f32, seed: u64, scale: i64, world: &mut World) -> tetra::Result {
    let noise = PerlinNoise::new();
    let nm1 = NoiseMap::new(noise)
        .set(Seed::of(seed))
        .set(Step::of(250.0, 250.0));
    let nm = Box::new(nm1 * scale);

    let noise_world = NoiseWorld::new()
        .set(Size::of(width, height))
        .add(Tile::new(WorldTile::Harbour)
            .when(constraint!(nm.clone(), > 1.8)))
        .add(Tile::new(WorldTile::Island(1))
            .when(constraint!(nm.clone(), > 1.45)).when(constraint!(nm.clone(), < 1.65)))
        .add(Tile::new(WorldTile::Island(2))
            .when(constraint!(nm.clone(), > 1.0)).when(constraint!(nm.clone(), < 1.45)))
        .add(Tile::new(WorldTile::Island(3))
            .when(constraint!(nm.clone(), > 0.6)).when(constraint!(nm.clone(), < 1.0)))
        .add(Tile::new(WorldTile::Reef)
            .when(constraint!(nm.clone(), > 0.485)).when(constraint!(nm.clone(), < 0.6)))
        .add(Tile::new(WorldTile::Empty));
    
    let mut rng = Xoshiro128Plus::seed_from_u64(seed);
    let base_pos = V2::zero(); // V2::new(0.0 - (width as f32 * base_tile_size) * 0.5,
    //     0.0 - (height as f32 * base_tile_size) * 0.5);
    let mut curr_pos = base_pos;
    let mut harbours = 0;
    let mut harbour_col = 0;
    for row in noise_world.generate(0, 0).into_iter() {
        for (curr_col, col) in row.iter().enumerate() {
            for tile in col.iter() {
                let y = add_procedural_element(ctx, &mut rng, curr_pos, base_tile_size,
                    tile_padding, *tile, &mut harbours, &mut harbour_col,
                    col.len() as u32, curr_col as u32, world)?;
                curr_pos.y += y;
            }
            curr_pos.x += base_tile_size * tile_padding;
            curr_pos.y = base_pos.y;
        }
    }
    Ok(())
}

fn add_procedural_element(ctx: &mut Context, rng: &mut Xoshiro128Plus, pos: V2,
    base_tile_size: f32, tile_padding: f32, tile: WorldTile, harbours: &mut u32,
    harbour_col: &mut u32, total_cols: u32, curr_col: u32, world: &mut World) -> tetra::Result<f32> {
    let noise_pos = pos + V2::new(rand_f32(rng) - 0.5,
        rand_f32(rng) - 0.5) * 100.0;
    let noise_rot = rand_f32(rng) * PI;
    Ok(match tile {
        WorldTile::Island(island_type) => world.add_island(ctx, noise_pos,
            noise_rot, island_type)?.borrow().sprite.get_size().y * tile_padding,
        WorldTile::Harbour => {
            if *harbours >= 2 || (curr_col - *harbour_col) < (total_cols / 4) {
                return Ok(base_tile_size * 0.25 * tile_padding)
            }
            let harbour = world.add_harbour(ctx, "Port Royal", noise_pos,
                noise_rot)?;
            let size_y = harbour.borrow().sprite.get_size().y;
            *harbours += 1;
            *harbour_col = curr_col;
            size_y * tile_padding
        },
        WorldTile::Reef => world.add_reef(ctx, noise_pos, noise_rot)?
            .borrow().sprite.get_size().y * tile_padding,
        WorldTile::Empty => base_tile_size * 0.3 * tile_padding,
    })
}
