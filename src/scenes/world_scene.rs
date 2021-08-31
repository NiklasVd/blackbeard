use tetra::{Context, Event, State};
use crate::{BbResult, Controller, GC, GameState, ID, Player, Rcc, ShipType, TransformResult, V2, grid::{Grid, UIAlignment}, label::Label, world::World};
use super::scenes::{Scene, SceneType};

pub struct WorldScene {
    pub controller: Controller,
    pub world: World,
    grid: Grid,
    game: GC
}

impl WorldScene {
    pub fn new(ctx: &mut Context, game: GC) -> BbResult<WorldScene> {
        let mut grid = Grid::new(ctx, UIAlignment::Vertical,
            V2::zero(), V2::one() * 200.0, 0.0).convert()?;
        grid.add_element(Label::new(ctx, "Pre-Alpha WIP", false, 2.0, game.clone()).convert()?);
        let mut world_scene = WorldScene {
            controller: Controller::new(ctx, game.clone()).convert()?,
            world: World::new(ctx, game.clone()),
            grid, game
        };
        
        let player = world_scene.add_player(ctx, ID::new("Francis Drake".to_owned(), 0), ShipType::Caravel)?;
        world_scene.controller.set_local_player(player);
        world_scene.add_player(ctx, ID::new("Jack Sparrow".to_owned(), 1), ShipType::Caravel)?;
        world_scene.world.add_island(ctx, V2::new(800.0, 500.0), 0.1).convert()?;
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
        Ok(None)
    }

    fn get_type(&self) -> SceneType {
        SceneType::World
    }
}

impl State for WorldScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
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
