use std::{cell::RefCell , rc::Rc};
use tetra::{Context, Event, State, graphics::{self, Color}};
use crate::{Assets, BbResult, Cam, Physics, Settings, TransformResult, WorldSettings, network::Network, scenes::scenes::{Scenes}};

pub type Rcc<T> = Rc<RefCell<T>>;
pub type GC = Rcc<GameContainer>;

pub fn wrap_rcc<T>(obj: T) -> Rcc<T> {
    Rc::new(RefCell::new(obj))
}

pub trait AppState {
    fn update_bb(&mut self, ctx: &mut Context) -> BbResult {
        self.update(ctx).convert()
    }

    fn draw_bb(&mut self, ctx: &mut Context) -> BbResult {
        self.draw(ctx).convert()
    }

    fn event_bb(&mut self, ctx: &mut Context, event: Event) -> BbResult {
        self.event(ctx, event).convert()
    }

    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event) -> tetra::Result {
        Ok(())
    }
}

pub struct GameContainer {
    pub assets: Assets,
    pub physics: Physics,
    pub settings: Settings,
    pub world: WorldSettings,
    pub cam: Cam,
    pub network: Option<Network>
}

pub struct Game {
    pub container: GC,
    pub scenes: Scenes
}

impl Game {
    pub fn new(ctx: &mut Context) -> tetra::Result<Game> {
        let gc = wrap_rcc(GameContainer {
            assets: Assets::load(ctx)?,
            physics: Physics::setup(),
            settings: Settings::new(),
            world: WorldSettings::new(),
            cam: Cam::setup(550.0),
            network: None
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
            if let Some(network) = game_ref.network.as_mut() {
                network.update(ctx)?;
            }
        }
        self.scenes.update(ctx)
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb8(0, 102, 255));
        graphics::set_transform_matrix(ctx,
            self.container.borrow().cam.instance.as_matrix());
        self.scenes.draw(ctx)?;
        
        graphics::reset_transform_matrix(ctx);
        self.scenes.draw_ui(ctx)?;
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: tetra::Event) -> tetra::Result {
        self.scenes.event(ctx, event)
    }
}