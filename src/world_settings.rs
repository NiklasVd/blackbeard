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
            .when(constraint!(nm.clone(), > 1.7)).when(constraint!(nm.clone(), < 1.785)))
        .add(Tile::new(WorldTile::Island(1))
            .when(constraint!(nm.clone(), > 1.3)).when(constraint!(nm.clone(), < 1.7)))
        .add(Tile::new(WorldTile::Island(2))
            .when(constraint!(nm.clone(), > 1.2)).when(constraint!(nm.clone(), < 1.3)))
        .add(Tile::new(WorldTile::Island(3))
            .when(constraint!(nm.clone(), > 1.0)).when(constraint!(nm.clone(), < 1.2)))
        .add(Tile::new(WorldTile::Reef)
            .when(constraint!(nm.clone(), > 0.8)).when(constraint!(nm.clone(), < 0.8125)))
        .add(Tile::new(WorldTile::Empty));
    
    let mut rng = Xoshiro128Plus::seed_from_u64(seed);
    let mut curr_pos = V2::new(0.0 - (width as f32 * base_tile_size * tile_padding) * 0.5,
        0.0 - (height as f32 * base_tile_size * tile_padding) * 0.5);
    for row in noise_world.generate(0, 0).into_iter() {
        for col in row.into_iter() {
            for tile in col.into_iter() {
                let y = add_procedural_element(ctx, &mut rng, curr_pos, base_tile_size,
                    tile_padding, tile, world)?;
                curr_pos.y += y;
            }
            curr_pos.x += base_tile_size * tile_padding;
            curr_pos.y = 0.0;
        }
    }
    Ok(())
}

fn add_procedural_element(ctx: &mut Context, rng: &mut Xoshiro128Plus, pos: V2,
    base_tile_size: f32, tile_padding: f32, tile: WorldTile, world: &mut World) -> tetra::Result<f32> {
    let noise_pos = pos + V2::new(rand_f32(rng) - 0.5,
        rand_f32(rng) - 0.5) * 100.0;
    let noise_rot = rand_f32(rng) * PI;

    Ok(match tile {
        WorldTile::Island(island_type) => world.add_island(ctx, noise_pos,
            noise_rot, island_type)?,
        WorldTile::Harbour => {
            let offset_pos = V2::new(20.0, -275.0);
            let offset_rot = -0.7;
            let island = world.add_island(ctx, noise_pos, noise_rot, 4)?;
            world.add_harbour(ctx, "Port Royal", noise_pos + offset_pos,
                noise_rot + offset_rot)?;
            island
        },
        WorldTile::Reef => world.add_reef(ctx, noise_pos, noise_rot)?,
        WorldTile::Empty => return Ok(base_tile_size * 0.2 * tile_padding),
    }.borrow().sprite.get_size().y * tile_padding)
}
