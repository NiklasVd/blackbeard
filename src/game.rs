use std::{cell::RefCell , rc::Rc};
use tetra::{Context, State, graphics::{self, Color, text::Text}, window::{get_height, get_width}};
use crate::{Assets, Cam, Physics, Settings, V2, WorldSettings, economy::Economy, get_version, network::Network, scenes::scenes::{Scenes}};

pub type Rcc<T> = Rc<RefCell<T>>;
pub type GC = Rcc<GameContainer>;

pub fn wrap_rcc<T>(obj: T) -> Rcc<T> {
    Rc::new(RefCell::new(obj))
}

pub struct GameContainer {
    pub assets: Assets,
    pub physics: Physics,
    pub settings: Settings,
    pub world: WorldSettings,
    pub cam: Cam,
    pub network: Option<Network>,
    pub economy: Economy,
    pub simulation_state: bool
}

impl GameContainer {
    pub fn new(ctx: &mut Context) -> tetra::Result<GameContainer> {
        Ok(GameContainer {
            assets: Assets::load(ctx)?,
            physics: Physics::setup(),
            settings: Settings::new(),
            world: WorldSettings::new(),
            cam: Cam::setup(ctx, 700.0),
            network: None,
            economy: Economy::new(),
            simulation_state: true
        })
    }
}

impl State for GameContainer {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.cam.update(ctx)?;
        if let Some(network) = self.network.as_mut() {
            network.update(ctx)?;
        }
        if self.simulation_state { // Dont simulate physics if simulation is halted
            self.physics.update(ctx)?;
        }
        Ok(())
    }
}

pub struct Game {
    pub container: GC,
    pub scenes: Scenes,
    watermark: Text
}

impl Game {
    pub fn new(ctx: &mut Context) -> tetra::Result<Game> {
        let container = wrap_rcc(GameContainer::new(ctx)?);
        let watermark = Text::new(format!("Blackbeard Alpha {}", get_version()),
                container.borrow().assets.small_font.clone());
        let scenes = Scenes::setup(ctx, container.clone())?;

        Ok(Game {
            container, scenes, watermark
        })
    }
}

impl State for Game {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.scenes.update(ctx)?;
        self.container.borrow_mut().update(ctx)
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