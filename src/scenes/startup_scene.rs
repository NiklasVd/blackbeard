use tetra::{Context, State, graphics::{Color, DrawParams, Texture}};
use crate::{GC, Timer, V2, grid::{Grid, UIAlignment}, menu_scene::MenuScene};
use super::scenes::{Scene, SceneType};

const STARTUP_TIME: f32 = 3.0;
const STARTUP_TIME_DEBUG: f32 = 0.5; 

#[cfg(not(debug_assertions))]
fn get_startup_time() -> f32 {
    STARTUP_TIME
}

#[cfg(debug_assertions)]
fn get_startup_time() -> f32 {
    STARTUP_TIME_DEBUG
}

pub struct StartupScene {
    grid: Grid,
    timer: Timer,
    logo: Texture,
    game: GC
}

impl StartupScene {
    pub fn new(ctx: &mut Context, game: GC) -> tetra::Result<StartupScene> {
        let logo = game.borrow_mut().assets.load_texture(ctx,
            "UI/Logo.png".to_owned(), false)?;
        Ok(StartupScene {
            grid: Grid::new(ctx, UIAlignment::Vertical, V2::zero(), V2::one(), 0.0)?,
            timer: Timer::start(get_startup_time()),
            logo, game: game.clone()
        })
    }
}

impl Scene for StartupScene {
    fn get_grid(&self) -> &Grid {
        &self.grid
    }

    fn get_grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    fn poll(&self, ctx: &mut Context) -> tetra::Result<Option<Box<dyn Scene + 'static>>> {
        if self.timer.is_over() {
            return Ok(Some(Box::new(MenuScene::new(ctx, self.game.clone())?)))
        }

        Ok(None)
    }

    fn get_type(&self) -> SceneType {
        SceneType::Startup
    }
}

impl State for StartupScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.timer.update(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.logo.draw(ctx, DrawParams {
            position: V2::new(
                -self.logo.width() as f32 * 0.5, -self.logo.height() as f32 * 0.5),
            scale: V2::one(),
            rotation: 0.0,
            origin: V2::zero(),
            color: Color::WHITE
        });
        Ok(())
    }
}
