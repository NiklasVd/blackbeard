use tetra::{Context, State, window::quit};
use crate::{GC, Rcc, V2, button::Button, grid::{Grid, UIAlignment}, label::Label, loading_scene::LoadingScene, ui_element::{UIReactor, UIState}, wrap_rcc};
use super::scenes::{Scene, SceneType};

pub struct MenuScene {
    pub grid: Grid,
    new_game_reactor: Rcc<NewGameReactor>,
    exit_reactor: Rcc<ExitReactor>,
    game: GC
}

impl MenuScene {
    pub fn new(ctx: &mut Context, game: GC) -> tetra::Result<MenuScene> {
        let mut grid = Grid::new(ctx, UIAlignment::Horizontal,
            V2::zero(), V2::one() * 250.0, 10.0)?;
        let title_label = Label::new(ctx, "Blackbeard", true, 5.0, game.clone())?;
        grid.add_element(title_label, 0);

        let new_game_reactor = wrap_rcc(NewGameReactor::new());
        let new_game_button = Button::new(ctx, "New Game", V2::new(125.0, 35.0), 5.0,
            new_game_reactor.clone(), game.clone())?;
        grid.add_element(new_game_button, 1);
        
        let exit_reactor = wrap_rcc(ExitReactor::new());
        let exit_button = Button::new(ctx, "Exit", V2::new(75.0, 35.0), 5.0,
            exit_reactor.clone(), game.clone())?;
        grid.add_element(exit_button, 2);

        Ok(MenuScene {
            grid, new_game_reactor, exit_reactor, game: game.clone()
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
        if self.new_game_reactor.borrow().new_game {
            return Ok(Some(Box::new(
                LoadingScene::new(ctx, SceneType::World, 5.0, self.game.clone())?)))
        }
        else if self.exit_reactor.borrow().exit {
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

struct NewGameReactor {
    state: UIState,
    new_game: bool
}

impl NewGameReactor {
    fn new() -> NewGameReactor {
        NewGameReactor {
            state: UIState::Idle, new_game: false
        }
    }
}

impl UIReactor for NewGameReactor {
    fn get_state(&self) -> UIState {
        self.state
    }

    fn set_state(&mut self, state: UIState) {
        self.state = state;
    }

    fn on_click(&mut self, ctx: &mut Context) -> tetra::Result {
        self.new_game = true;
        Ok(())
    }
}

struct ExitReactor {
    state: UIState,
    exit: bool
}

impl ExitReactor {
    pub fn new() -> ExitReactor {
        ExitReactor {
            state: UIState::Idle, exit: false
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

    fn on_click(&mut self, ctx: &mut Context) -> tetra::Result {
        self.exit = true;
        Ok(())
    }
}
