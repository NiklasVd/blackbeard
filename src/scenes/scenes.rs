use tetra::{Context, Event, State, graphics::{Color, DrawParams, Texture}};
use crate::{Entity, GC, Timer, V2, world_scene::WorldScene};

pub enum SceneType {
    Startup,
    Menu,
    World
}

pub trait Scene : State {
    fn enter(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }

    fn exit(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }

    fn poll(&self, ctx: &mut Context) -> tetra::Result<Option<Box<dyn Scene>>>;
    fn get_type(&self) -> SceneType;
}

pub struct Scenes {
    pub curr_scene: Box<dyn Scene>,
    game: GC
}

impl Scenes {
    pub fn setup(ctx: &mut Context, game: GC) -> tetra::Result<Scenes> {
        let mut startup_scene = StartupScene::new(ctx, game.clone())?;
        startup_scene.enter(ctx);
        Ok(Scenes {
            curr_scene: Box::new(startup_scene),
            game
        })
    }

    pub fn load_scene(&mut self, ctx: &mut Context, mut scene: Box<dyn Scene>)
        -> tetra::Result {
        self.curr_scene.exit(ctx)?;
        
        scene.enter(ctx)?;
        self.curr_scene = scene;
        Ok(())
    }
}

impl State for Scenes {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.curr_scene.update(ctx)?;
        if let Some(next_scene) = self.curr_scene.poll(ctx)? {
            self.load_scene(ctx, next_scene)?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.curr_scene.draw(ctx)
    }

    fn event(&mut self, ctx: &mut Context, event: Event) -> tetra::Result {
        self.curr_scene.event(ctx, event)
    }
}

pub struct StartupScene {
    timer: Timer,
    logo: Texture,
    game: GC
}

impl StartupScene {
    pub fn new(ctx: &mut Context, game: GC) -> tetra::Result<StartupScene> {
        Ok(StartupScene {
            timer: Timer::new(3.0),
            logo: game.borrow_mut().assets.load_texture(ctx, "Logo.png".to_owned(), false)?,
            game: game.clone()
        })
    }
}

impl Scene for StartupScene {
    fn poll(&self, ctx: &mut Context) -> tetra::Result<Option<Box<dyn Scene>>> {
        if self.timer.is_over() {
            return Ok(Some(Box::new(WorldScene::new(ctx, self.game.clone())?)))
        }

        Ok(None)
    }

    fn get_type(&self) -> SceneType {
        SceneType::Startup
    }
}

impl State for StartupScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.timer.update();
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
