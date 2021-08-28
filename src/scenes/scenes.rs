use tetra::{Context, Event, State};
use crate::{BbResult, GC, TransformResult, V2, grid::Grid, startup_scene::StartupScene, ui_element::UIElement};

pub enum SceneType {
    Startup,
    Login,
    Menu,
    Loading,
    Connection,
    Lobby,
    World
}

pub trait Scene : State {
    fn get_grid(&self) -> &Grid;
    fn get_grid_mut(&mut self) -> &mut Grid;

    fn poll(&self, ctx: &mut Context) -> BbResult<Option<Box<dyn Scene + 'static>>>;
    fn get_type(&self) -> SceneType;
}

pub struct Scenes {
    pub curr_scene: Box<dyn Scene + 'static>,
    game: GC
}

impl Scenes {
    pub fn setup(ctx: &mut Context, game: GC) -> tetra::Result<Scenes> {
        let startup_scene = StartupScene::new(ctx, game.clone())?;
        Ok(Scenes {
            curr_scene: Box::new(startup_scene),
            game
        })
    }

    pub fn load_scene(&mut self, ctx: &mut Context, scene: Box<(dyn Scene + 'static)>)
        -> tetra::Result {
        self.curr_scene = scene;
        Ok(())
    }

    pub fn draw_ui(&mut self, ctx: &mut Context) -> tetra::Result {
        self.curr_scene.get_grid_mut().draw_element(ctx, V2::zero())
    }
}

impl State for Scenes {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.curr_scene.update(ctx)?;
        self.curr_scene.get_grid_mut().update_element(ctx, V2::zero())?;
        if let Some(next_scene) = self.curr_scene.poll(ctx).convert()? {
            self.load_scene(ctx, next_scene)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.curr_scene.draw(ctx)
        // UI is a special case, due to exclusion from camera matrix
    }

    fn event(&mut self, ctx: &mut Context, event: Event) -> tetra::Result {
        self.curr_scene.event(ctx, event.clone())?;
        self.curr_scene.get_grid_mut().event_element(ctx, event, V2::zero())
    }
}

