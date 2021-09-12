use tetra::{Context, State};
use crate::{BbResult, GC, ID, ShipType, Timer, V2, grid::{Grid, UIAlignment}, image::Image, label::Label, rand_u32, world_scene::WorldScene};
use super::scenes::{Scene, SceneType};

const MIN_LOADING_TIME: f32 = 1.0;
const LOADING_HINTS: [&str; 11] = [
    "Ship collisions stun the crew. Duration and damage depend on the ship's defence value.",
    "Use the Q and E keys to shoot cannons on star- and portside respectively.",
    "What smaller ships lack in firepower, they make up in mobility.",
    "Sinking a ship by ramming yields 60% of escudos onboard.",
    "Sinking a ship via cannon shot yields 40% of escudos onboard.",
    "When your ship sinks due to an accidental collision, you lose 15% of escudos onboard.",
    "The more escudos you have, the slower your ship will sail.",
    "Use your escudos to buy upgrades and repairs at harbours.",
    "Reefs will prove an impassable barrier to heavier ships, while allowing lighter ships to pass.",
    "Beware of bandit outposts! They will shoot any ship on sight.",
    "Some ships can drop powder kegs into the water, which will detonate upon collision."
];

pub struct LoadingScene {
    players: Vec<(ID, ShipType)>,
    min_load_timer: Timer,
    grid: Grid,
    image_loaded: bool,
    game: GC
}

impl LoadingScene {
    pub fn new(ctx: &mut Context, players: Vec<(ID, ShipType)>, game: GC)
        -> tetra::Result<LoadingScene> {
        let mut grid = Grid::default(ctx, UIAlignment::Vertical,
            V2::zero(), V2::one() * 200.0, 0.0)?;
        let mut title_grid = Grid::default(ctx, UIAlignment::Horizontal,
            V2::zero(), V2::new(200.0, 42.0), 0.0)?;
        
        let loading_label = Label::new(ctx, "Loading...", true, 5.0, game.clone())?;
        title_grid.add_element(loading_label);
        let hint_index = rand_u32(0, LOADING_HINTS.len() as u32 - 1) as usize;
        let loading_hint = Label::new(ctx,
            &format!("Hint: {}", LOADING_HINTS[hint_index]), false, 5.0, game.clone())?;
        title_grid.add_element(loading_hint);
        grid.add_element(title_grid);
        
        Ok(LoadingScene {
            players, min_load_timer: Timer::start(MIN_LOADING_TIME),
            grid, image_loaded: false, game
        })
    }

    fn load_image(&mut self, ctx: &mut Context) -> tetra::Result {
        if !self.image_loaded { // TODO: Put into separate grid for central layout
            let loading_image = Image::new(ctx,
                V2::new(200.0 * 4.0, 125.0 * 4.0), 0.0,
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
        Ok(if self.min_load_timer.is_over() {
            Some(Box::new(WorldScene::new(ctx, self.players.clone(), self.game.clone())?))
        } else {
            None
        })
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
