use tetra::{Context, State};

use crate::GC;

use super::scenes::{Scene, SceneType};

pub struct MenuScene {
    game: GC
}

impl MenuScene {
    pub fn new(ctx: &mut Context, game: GC) -> tetra::Result<MenuScene> {
        Ok(MenuScene {
            game
        })
    }
}

impl Scene for MenuScene {
    fn poll(&self, ctx: &mut Context) -> tetra::Result<Option<Box<dyn Scene>>> {
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
