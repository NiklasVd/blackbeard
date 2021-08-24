use tetra::Context;

use crate::{GC, V2, grid::{Grid, UIAlignment}};

pub struct LobbyScene {
    pub grid: Grid,
    game: GC
}

impl LobbyScene {
    pub fn create(ctx: &mut Context, port: u16, game: GC) -> tetra::Result<LobbyScene> {

        Self::new(ctx, game)
    }

    pub fn connect(ctx: &mut Context, endpoint: String, game: GC)
        -> tetra::Result<LobbyScene> {
        Self::new(ctx, game)
    }

    fn new(ctx: &mut Context, game: GC) -> tetra::Result<LobbyScene> {
        let mut grid = Grid::new(ctx, UIAlignment::Vertical, V2::zero(),
            V2::one() * 250.0, 5.0);


        Ok(LobbyScene {
            grid, game
        })
    }
}
