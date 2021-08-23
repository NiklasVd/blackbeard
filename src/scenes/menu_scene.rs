use tetra::{Context, State, window::quit};
use crate::{GC, Rcc, V2, button::Button, grid::{Grid, UIAlignment}, label::Label, loading_scene::LoadingScene, lobby_scene::LobbyScene, ui_element::{UIReactor, UIState}};
use super::scenes::{Scene, SceneType};

pub struct MenuScene {
    pub grid: Grid,
    private_game_button: Rcc<Button<PrivateGameReactor>>,
    online_game_button: Rcc<Button<OnlineGameReactor>>,
    exit_button: Rcc<Button<ExitReactor>>,
    game: GC
}

impl MenuScene {
    pub fn new(ctx: &mut Context, game: GC) -> tetra::Result<MenuScene> {
        let mut grid = Grid::new(ctx, UIAlignment::Horizontal,
            V2::zero(), V2::one() * 250.0, 10.0)?;
        let label = grid.add_element(Label::new(ctx, "Blackbeard", true,
            5.0, game.clone())?);
                
        let private_game_button = grid.add_element(Button::new(ctx, "Private Game",
            V2::new(125.0, 35.0), 0.0, PrivateGameReactor::new(), game.clone())?);
        let online_game_button = grid.add_element(Button::new(ctx, "Online Game",
            V2::new(125.0, 35.0), 5.0, OnlineGameReactor::new(), game.clone())?);
        let exit_button = grid.add_element(Button::new(ctx, "Exit",
            V2::new(75.0, 35.0), 15.0, ExitReactor::new(), game.clone())?);
        
        Ok(MenuScene {
            grid, private_game_button, online_game_button, exit_button,
            game: game.clone()
        })
    }
}

impl Scene for MenuScene {
    fn get_grid(&self) -> &Grid {
        &self.grid
    }

    fn get_grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    fn poll(&self, ctx: &mut Context) -> tetra::Result<Option<Box<dyn Scene>>> {
        if self.private_game_button.borrow().reactor.state == UIState::Focus {
            return Ok(Some(Box::new(
                LoadingScene::new(ctx, SceneType::World, 5.0, self.game.clone())?)))
        }
        else if self.online_game_button.borrow().reactor.state == UIState::Focus {
            return Ok(Some(Box::new(
                LobbyScene::new(ctx, self.game.clone())?)))
        }
        else if self.exit_button.borrow().reactor.state == UIState::Focus {
            quit(ctx);
            return Ok(None)
        }
        Ok(None)
    }

    fn get_type(&self) -> SceneType {
        SceneType::Menu
    }
}

impl State for MenuScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }
}

struct PrivateGameReactor {
    state: UIState
}

impl PrivateGameReactor {
    fn new() -> PrivateGameReactor {
        PrivateGameReactor {
            state: UIState::Idle
        }
    }
}

impl UIReactor for PrivateGameReactor {
    fn get_state(&self) -> UIState {
        self.state
    }

    fn set_state(&mut self, state: UIState) {
        self.state = state;
    }
}

struct OnlineGameReactor {
    state: UIState
}

impl OnlineGameReactor {
    pub fn new() -> OnlineGameReactor {
        OnlineGameReactor {
            state: UIState::Idle
        }
    }
}

impl UIReactor for OnlineGameReactor {
    fn get_state(&self) -> UIState {
        self.state.clone()
    }

    fn set_state(&mut self, state: UIState) {
        self.state = state;
    }
}

struct ExitReactor {
    state: UIState
}

impl ExitReactor {
    pub fn new() -> ExitReactor {
        ExitReactor {
            state: UIState::Idle
        }
    }
}

impl UIReactor for ExitReactor {
    fn get_state(&self) -> UIState {
        self.state
    }

    fn set_state(&mut self, state: UIState) {
        self.state = state;
    }
}
