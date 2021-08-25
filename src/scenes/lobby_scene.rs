use tetra::{Context, State};
use crate::{GC, Rcc, V2, button::{Button, DefaultButton}, grid::{Grid, UIAlignment}, label::Label, ui_element::DefaultUIReactor};
use super::scenes::{Scene, SceneType};

pub struct LobbyScene {
    pub grid: Grid,
    start_game_button: Rcc<DefaultButton>,
    disconnect_button: Rcc<DefaultButton>,
    player_list_grid: Rcc<Grid>,
    game: GC
}

impl LobbyScene {
    pub fn create(ctx: &mut Context, port: u16, game: GC) -> tetra::Result<LobbyScene> {

        Self::new(ctx, game)
    }

    pub fn join(ctx: &mut Context, endpoint: String, game: GC)
        -> tetra::Result<LobbyScene> {
        Self::new(ctx, game)
    }

    fn new(ctx: &mut Context, game: GC) -> tetra::Result<LobbyScene> {
        let mut grid = Grid::new(ctx, UIAlignment::Vertical, V2::zero(),
            V2::one() * 500.0, 5.0)?;
        grid.add_element(Label::new(ctx, "Setting up network...", false,
            5.0, game.clone())?);
        let start_game_button = grid.add_element(Button::new(ctx, "Start Game",
            V2::new(90.0, 30.0), 5.0, DefaultUIReactor::new(), game.clone())?);
        let disconnect_button = grid.add_element(Button::new(ctx, "Disconnect", 
            V2::new(85.0, 30.0), 5.0, DefaultUIReactor::new(), game.clone())?);
        
        grid.add_element(Label::new(ctx, "Connected Players", true, 5.0, game.clone())?);
        let player_list_grid = grid.add_element(Grid::new(ctx, UIAlignment::Vertical,
            V2::zero(), V2::one() * 300.0, 5.0)?);
        
        Ok(LobbyScene {
            grid, start_game_button, disconnect_button, player_list_grid, game
        })
    }
}

impl Scene for LobbyScene {
    fn get_type(&self) -> SceneType {
        SceneType::Lobby
    }

    fn get_grid(&self) -> &Grid {
        &self.grid
    }

    fn get_grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    fn poll(&self, ctx: &mut Context) -> tetra::Result<Option<Box<dyn Scene + 'static>>> {
        Ok(None)
    }
}

impl State for LobbyScene {
}
