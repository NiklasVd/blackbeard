use tetra::{Context, State};
use crate::{BbResult, GC, Timer, V2, grid::{Grid, UIAlignment}, image::Image, label::Label, rand_u32, world_scene::WorldScene};
use super::scenes::{Scene, SceneType};

const MIN_LOADING_TIME: f32 = 1.0;
const LOADING_HINTS: [&str; 5] = [
    "Caravels were developed by the Portugese in the 15th century.",
    "Though the game bears his name, Blackbeard wasn't such a rad dude overall.",
    "Ship collisions stun the crew. Duration and damage depend on the ship's defence value.",
    "Use the Q and E keys to shoot cannons on star- and portside.",
    "What smaller ships lack in firepower, they make up in mobility."
];

pub struct LoadingScene {
    pub next_scene: SceneType,
    min_load_timer: Timer,
    grid: Grid,
    image_loaded: bool,
    game: GC
}

impl LoadingScene {
    pub fn new(ctx: &mut Context, next_scene: SceneType, game: GC)
        -> tetra::Result<LoadingScene> {
        let mut grid = Grid::new(ctx, UIAlignment::Vertical,
            V2::zero(), V2::one() * 200.0, 0.0)?;
        let mut title_grid = Grid::new(ctx, UIAlignment::Horizontal, V2::zero(),
            V2::new(200.0, 42.0), 0.0)?;
        
        let loading_label = Label::new(ctx, "Loading...", true, 5.0, game.clone())?;
        title_grid.add_element(loading_label);
        let hint_index = rand_u32(0, LOADING_HINTS.len() as u32 - 1) as usize;
        let loading_hint = Label::new(ctx,
            &format!("Hint: {}", LOADING_HINTS[hint_index]), false, 5.0, game.clone())?;
        title_grid.add_element(loading_hint);
        grid.add_element(title_grid);
        
        Ok(LoadingScene {
            next_scene, min_load_timer: Timer::start(MIN_LOADING_TIME),
            grid, image_loaded: false, game
        })
    }

    fn load_image(&mut self, ctx: &mut Context) -> tetra::Result {
        if !self.image_loaded {
            let loading_image = Image::new(ctx, V2::new(200.0 * 4.0, 125.0 * 4.0), 0.0,
                "UI/Sunk Ship Painting.png".to_owned(), false, self.game.clone())?;
            self.grid.add_element(loading_image);
            self.image_loaded = true;
        }
        Ok(())
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

    fn poll(&self, ctx: &mut Context) -> BbResult<Option<Box<dyn Scene + 'static>>> {
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
        if self.min_load_timer.max > 0.0 && self.min_load_timer.curr_time > 0.1 {
            self.load_image(ctx)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }
}
