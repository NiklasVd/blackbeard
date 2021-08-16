use std::{cell::RefCell , rc::Rc};
use tetra::{Context, State, graphics::{self, Color}};
use crate::{Assets, Cam, Physics, Settings, World, scenes::scenes::{Scenes}};

pub type Rcc<T> = Rc<RefCell<T>>;
pub type GC = Rcc<GameContainer>;

pub fn wrap_rcc<T>(obj: T) -> Rcc<T> {
    Rc::new(RefCell::new(obj))
}

pub struct GameContainer {
    pub assets: Assets,
    pub physics: Physics,
    pub settings: Settings,
    pub world: World,
    pub cam: Cam
}

pub struct Game {
    pub container: GC,
    pub scenes: Scenes
}

impl Game {
    pub fn new(ctx: &mut Context) -> tetra::Result<Game> {
        let settings = Settings::load();
        let cam = Cam::setup(400.0);
        let gc = wrap_rcc(GameContainer {
            assets: Assets::load(ctx)?,
            physics: Physics::setup(),
            settings,
            world: World::new(),
            cam
        });
        
        Ok(Game {
            container: gc.clone(),
            scenes: Scenes::setup(ctx, gc)?
        })
    }
}

impl State for Game {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        {
            let mut game_ref = self.container.borrow_mut();
            game_ref.cam.update(ctx)?;
            game_ref.physics.update(ctx)?;
        }
        self.scenes.update(ctx)
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb8(0, 102, 255));
        graphics::set_transform_matrix(ctx,
            self.container.borrow().cam.instance.as_matrix());
        self.scenes.draw(ctx)?;
        
        graphics::reset_transform_matrix(ctx);
        self.scenes.draw_ui(ctx)
    }

    fn event(&mut self, ctx: &mut Context, event: tetra::Event) -> tetra::Result {
        self.scenes.event(ctx, event)
    }
}