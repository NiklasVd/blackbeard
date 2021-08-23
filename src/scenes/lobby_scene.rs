use tetra::{Context, State};
use crate::{GC, Rcc, V2, button::Button, grid::{Grid, UIAlignment}, label::Label, textbox::Textbox, ui_element::{UIElement, UIReactor, UIState}};
use super::scenes::{Scene, SceneType};

const DEFAULT_HOST_PORT: u16 = 8080;

pub struct LobbyScene {
    pub grid: Grid,
    create_button: Rcc<Button<CreateGameReactor>>,
    join_button: Rcc<Button<JoinGameReactor>>,
    disconnected: bool,
    game: GC
}

impl LobbyScene {
    pub fn new(ctx: &mut Context, game: GC) -> tetra::Result<LobbyScene> {
        let mut grid = Grid::new(ctx, UIAlignment::Vertical, V2::zero(),
            V2::new(200.0, 100.0), 0.0)?;
        
        let create_label = grid.add_element(Label::new(ctx, "Create Match", false,
            5.0, game.clone())?);
        let create_button = grid.add_element(Button::new(ctx, "Create", V2::new(75.0, 35.0),
            5.0, CreateGameReactor::new(), game.clone())?);
        
        let join_label = grid.add_element(Label::new(ctx, "Join Match", false,
            5.0, game.clone())?);
        let mut join_grid = Grid::new(ctx, UIAlignment::Horizontal,
            V2::zero(), V2::new(150.0, 30.0), 5.0)?;
        let join_endpoint_txt = join_grid.add_element(Textbox::new(ctx,
            format!("127.0.0.1:{}", DEFAULT_HOST_PORT).as_str(), V2::new(200.0, 30.0),
            5.0, game.clone())?);
        let join_button = join_grid.add_element(Button::new(ctx, "Join",
            V2::new(65.0, 35.0), 5.0, JoinGameReactor::new(), game.clone())?);
        let join_grid = grid.add_element(join_grid);
        
        Ok(LobbyScene {
            grid, create_button, join_button, disconnected: true, game
        })
    }

    fn check_buttons(&mut self) {
        if !self.disconnected {
            return
        }

        let mut create_button_ref = self.create_button.borrow_mut();
        let mut join_button_ref = self.join_button.borrow_mut();
        if create_button_ref.reactor.state == UIState::Focus
            || join_button_ref.reactor.state == UIState::Focus {
            create_button_ref.set_disabled(true);
            join_button_ref.set_disabled(true);
            self.disconnected = false;
        }
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
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.check_buttons();
        Ok(())
    }
}

struct CreateGameReactor {
    pub state: UIState
}

impl CreateGameReactor {
    pub fn new() -> CreateGameReactor {
        CreateGameReactor {
            state: UIState::Idle
        }
    }
}

impl UIReactor for CreateGameReactor {
    fn get_state(&self) -> UIState {
        self.state.clone()
    }

    fn set_state(&mut self, state: UIState) {
        self.state = state;
    }
}

struct JoinGameReactor {
    pub state: UIState
}

impl JoinGameReactor {
    pub fn new() -> JoinGameReactor {
        JoinGameReactor {
            state: UIState::Idle
        }
    }
}

impl UIReactor for JoinGameReactor {
    fn get_state(&self) -> UIState {
        self.state.clone()
    }

    fn set_state(&mut self, state: UIState) {
        self.state = state;
    }
}
