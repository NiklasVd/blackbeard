use tetra::{Context, State};
use crate::{GC, Timer, V2, grid::{Grid, UIAlignment}, image::Image, label::Label, world_scene::WorldScene};
use super::scenes::{Scene, SceneType};

const LOADING_HINTS: [&str; 4] = [
    "Caravels were developed by the Portugese in the 15th century.",
    "Though the game bears his name, Blackbeard wasn't such a rad dude overall.",
    "Ship collisions stun the crew. Duration and damage depend on the ship's defence value.",
    ""
];

pub struct LoadingScene {
    pub next_scene: SceneType,
    pub min_load_time: f32,
    min_load_timer: Timer,
    grid: Grid,
    game: GC
}

impl LoadingScene {
    pub fn new(ctx: &mut Context, next_scene: SceneType, min_load_time: f32, game: GC)
        -> tetra::Result<LoadingScene> {
        let font = game.borrow().assets.font.clone();
        let mut grid = Grid::new(ctx, UIAlignment::Vertical,
            V2::zero(), V2::one() * 200.0, 0.0)?;
        let mut title_grid = Grid::new(ctx, UIAlignment::Horizontal, V2::zero(),
            V2::new(200.0, 30.0), 0.0)?;
        
        let loading_label = Label::new(ctx, "Loading...", true, 5.0, game.clone())?;
        title_grid.add_element(loading_label, 0);
        let loading_hint = Label::new(ctx,
            &format!("Hint: {}", LOADING_HINTS[0 /* Change to random */]), false, 5.0, game.clone())?;
        title_grid.add_element(loading_hint, 1);
        grid.add_element(title_grid, 0);
        
        let loading_image = Image::new(ctx, V2::new(200.0 * 4.0, 125.0 * 4.0), 0.0,
            "UI/Sunk Ship Painting.png".to_owned(), false, game.clone())?;
        grid.add_element(loading_image, 1);

        Ok(LoadingScene {
            next_scene, min_load_time, min_load_timer: Timer::start(min_load_time),
            grid, game
        })
    }
}

impl Scene for LoadingScene {
    fn get_type(&self) -> SceneType {
        SceneType::Loading
    }

    fn get_grid(&self) -> &Grid {
        &self.grid
    }

    fn get_grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    fn poll(&self, ctx: &mut Context) -> tetra::Result<Option<Box<dyn Scene + 'static>>> {
        if self.min_load_timer.is_over() {
            return Ok(match self.next_scene {
                SceneType::World => Some(Box::new(
                    WorldScene::new(ctx, self.game.clone())?)),
                _ => None
            })
        }
        Ok(None)
    }
}

impl State for LoadingScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.min_load_timer.update(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }
}
