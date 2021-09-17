use std::{cell::RefCell , rc::Rc};
use tetra::{Context, Event, State, graphics::{self, Color, text::Text}, window::{get_height, get_width}};
use crate::{Assets, BbResult, Cam, Physics, Settings, TransformResult, V2, WorldSettings, get_version, network::Network, scenes::scenes::{Scenes}};

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
    pub scenes: Scenes,
    watermark: Text
}

impl Game {
    pub fn new(ctx: &mut Context) -> tetra::Result<Game> {
        let gc = wrap_rcc(GameContainer {
            assets: Assets::load(ctx)?,
            physics: Physics::setup(),
            settings: Settings::new(),
            world: WorldSettings::new(),
            cam: Cam::setup(ctx, 550.0),
            network: None
        });
        let watermark = Text::new(format!("Blackbeard Alpha {}", get_version()),
                gc.borrow().assets.small_font.clone());
        
        Ok(Game {
            container: gc.clone(),
            scenes: Scenes::setup(ctx, gc)?,
            watermark
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
        if self.container.borrow().settings.show_watermark {
            self.watermark.draw(ctx, V2::new(get_width(ctx) as f32, get_height(ctx) as f32)
                - V2::new(self.watermark.content().len() as f32 * 17.0 * 0.5, 20.0));
        }
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: tetra::Event) -> tetra::Result {
        self.scenes.event(ctx, event)
    }
}