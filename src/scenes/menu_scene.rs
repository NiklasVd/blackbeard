use tetra::{Context, State, window::quit};
use crate::{BbResult, GC, Rcc, TransformResult, V2, button::{Button, DefaultButton}, connection_scene::ConnectionScene, grid::{Grid, UIAlignment}, label::{FontSize, Label}, ui_element::{DefaultUIReactor}};
use super::scenes::{Scene, SceneType};

pub struct MenuScene {
    pub grid: Grid,
    online_game_button: Rcc<DefaultButton>,
    exit_button: Rcc<DefaultButton>,
    game: GC
}

impl MenuScene {
    pub fn new(ctx: &mut Context, game: GC) -> tetra::Result<MenuScene> {
        let mut grid = Grid::default(ctx, UIAlignment::Horizontal,
            V2::zero(), V2::one() * 250.0, 10.0)?;
        let label = grid.add_element(Label::new(ctx, "Blackbeard", FontSize::Header,
            5.0, game.clone())?);
                
        // let private_game_button = grid.add_element(Button::new(ctx, "Private Game",
        //     V2::new(125.0, 30.0), 5.0, DefaultUIReactor::new(), game.clone())?);
        let online_game_button = grid.add_element(Button::new(ctx, "Play Online",
            V2::new(130.0, 35.0), 2.0, DefaultUIReactor::new(), game.clone())?);
        let exit_button = grid.add_element(Button::new(ctx, "Exit",
            V2::new(80.0, 35.0), 2.0, DefaultUIReactor::new(), game.clone())?);
        
        Ok(MenuScene {
            grid, online_game_button, exit_button,
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

    fn poll(&self, ctx: &mut Context) -> BbResult<Option<Box<dyn Scene>>> {
        // if self.private_game_button.borrow().is_pressed() {
        //     return Ok(Some(Box::new(
        //         LoadingScene::new(ctx, vec![(ID::new("Jack Sparrow".to_owned(), 0), ShipType::Caravel)],
        //             self.game.clone()).convert()?)))
        // }
        if self.online_game_button.borrow().is_pressed() {
            return Ok(Some(Box::new(
                ConnectionScene::new(ctx, self.game.clone()).convert()?)))
        }
        else if self.exit_button.borrow().is_pressed() {
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
