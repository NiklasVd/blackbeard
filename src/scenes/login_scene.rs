use tetra::{Context, State};

use crate::{BbResult, GC, Rcc, TransformResult, V2, button::DefaultButton, grid::{Grid, UIAlignment}, image::Image, label::Label, menu_scene::MenuScene, textbox::Textbox, ui_element::DefaultUIReactor};

use super::scenes::{Scene, SceneType};

pub struct LoginScene {
    grid: Grid,
    name_txt: Rcc<Textbox>,
    login_button: Rcc<DefaultButton>,
    game: GC
}

impl LoginScene {
    pub fn new(ctx: &mut Context, game: GC) -> tetra::Result<LoginScene> {
        let mut grid = Grid::centered(ctx, UIAlignment::Vertical, V2::new(100.0, 250.0), 0.0)?;
        let header = grid.add_element(Image::new(ctx, V2::new(673.0, 117.0), 5.0,
            "UI/Header-Text.png".to_owned(), true, game.clone())?);
            header.borrow_mut().transform.position -= V2::new(200.0, 50.0);
        grid.add_element(Label::new(ctx, "Choose a name", false, 0.0, game.clone())?);
        let name_txt = grid.add_element(Textbox::new(ctx, "", V2::new(200.0, 30.0), 2.0, game.clone())?);
        let login_button = grid.add_element(DefaultButton::new(ctx, "Login",
            V2::new(70.0, 35.0), 5.0, DefaultUIReactor::new(), game.clone())?);
        Ok(LoginScene {
            grid, name_txt, login_button, game
        })
    }
}

impl Scene for LoginScene {
    fn get_type(&self) -> SceneType {
        SceneType::Login
    }

    fn get_grid(&self) -> &Grid {
        &self.grid
    }

    fn get_grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    fn poll(&self, ctx: &mut Context) -> BbResult<Option<Box<dyn Scene + 'static>>> {
        if self.login_button.borrow().is_pressed() &&
            self.game.borrow_mut().settings.set_name(self.name_txt.borrow().get_text().to_owned()) {
            return Ok(Some(Box::new(MenuScene::new(ctx, self.game.clone()).convert()?)))
        }
        Ok(None)
    }
}

impl State for LoginScene {
}
