use tetra::{Context, State};
use crate::{GC, Rcc, V2, button::{Button, DefaultButton}, grid::{Grid, UIAlignment}, label::Label, lobby_scene::LobbyScene, menu_scene::MenuScene, textbox::Textbox, ui_element::{DefaultUIReactor, UIElement, UIState}};
use super::scenes::{Scene, SceneType};

const DEFAULT_HOST_PORT: u16 = 8080;

pub struct ConnectionScene {
    pub grid: Grid,
    back_button: Rcc<DefaultButton>,
    create_button: Rcc<DefaultButton>,
    join_button: Rcc<DefaultButton>,
    join_endpoint_txt: Rcc<Textbox>,
    disconnected: bool,
    game: GC
}

impl ConnectionScene {
    pub fn new(ctx: &mut Context, game: GC) -> tetra::Result<ConnectionScene> {
        let mut grid = Grid::new(ctx, UIAlignment::Vertical, V2::zero(),
            V2::new(200.0, 100.0), 0.0)?;
        let back_button = grid.add_element(Button::new(ctx, "Back to Menu",
            V2::new(120.0, 30.0), 5.0, DefaultUIReactor::new(), game.clone())?);
        
        let create_label = grid.add_element(Label::new(ctx, "Create Match", false,
            5.0, game.clone())?);
        let create_button = grid.add_element(Button::new(ctx, "Create", V2::new(75.0, 30.0),
            5.0, DefaultUIReactor::new(), game.clone())?);
        
        let join_label = grid.add_element(Label::new(ctx, "Join Match", false,
            5.0, game.clone())?);
        let mut join_grid = Grid::new(ctx, UIAlignment::Horizontal,
            V2::zero(), V2::new(150.0, 30.0), 5.0)?;
        let join_endpoint_txt = join_grid.add_element(Textbox::new(ctx,
            format!("127.0.0.1:{}", DEFAULT_HOST_PORT).as_str(), V2::new(200.0, 30.0),
            5.0, game.clone())?);
        let join_button = join_grid.add_element(Button::new(ctx, "Join",
            V2::new(65.0, 30.0), 5.0, DefaultUIReactor::new(), game.clone())?);
        let join_grid = grid.add_element(join_grid);
        
        Ok(ConnectionScene {
            grid, back_button, create_button, join_button, join_endpoint_txt,
            disconnected: true, game
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

impl Scene for ConnectionScene {
    fn get_type(&self) -> SceneType {
        SceneType::Connection
    }

    fn get_grid(&self) -> &Grid {
        &self.grid
    }

    fn get_grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    fn poll(&self, ctx: &mut Context) -> tetra::Result<Option<Box<dyn Scene + 'static>>> {
        if self.back_button.borrow().is_pressed() {
            return Ok(Some(Box::new(MenuScene::new(ctx, self.game.clone())?)))
        }
        if self.create_button.borrow().is_pressed() {
            return Ok(Some(Box::new(LobbyScene::create(ctx, DEFAULT_HOST_PORT,
                self.game.clone())?)))
        }
        if self.join_button.borrow().is_pressed() {
            return Ok(Some(Box::new(LobbyScene::join(ctx,
                self.join_endpoint_txt.borrow().get_text().to_owned(), self.game.clone())?)))
        }

        Ok(None)
    }
}

impl State for ConnectionScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.check_buttons();
        Ok(())
    }
}
