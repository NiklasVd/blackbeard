use tetra::{Context, State, graphics::{text::Text}};
use crate::{GC, Rcc, V2, spritesheet::Spritesheet, ui_element::{UIElement, UIReactor, UIState, UITransform}};

pub struct Button<T: UIReactor + 'static> {
    pub transform: UITransform,
    text: Text,
    spritesheet: Spritesheet,
    reactor: Rcc<T>
}

impl<T: UIReactor + 'static> Button<T> {
    pub fn new(ctx: &mut Context, text: &str, size: V2, padding: f32, reactor: Rcc<T>,
        game: GC) -> tetra::Result<Button<T>> {
        let mut game_ref = game.borrow_mut();
        let texture = game_ref.assets.load_texture(ctx,"UI/Button.png".to_owned(), true)?;
        let font = game_ref.assets.font.clone();
        std::mem::drop(game_ref);
        
        let uniform_size = V2::new(10.0, 10.0);
        let spritesheet = Spritesheet::new(texture, game.clone(),
            uniform_size.x, uniform_size.y, 3);
        Ok(Button {
            transform: UITransform::default(ctx, size, uniform_size, padding)?,
            text: Text::new(text, font),
            spritesheet, reactor
        })
    }
}

impl<T: UIReactor + 'static> UIElement for Button<T> {
    fn get_name(&self) -> &str {
        "Button"
    }

    fn get_reactor(&self) -> Option<Rcc<dyn UIReactor>> {
        Some(self.reactor.clone())
    }

    fn get_transform(&self) -> &UITransform {
        &self.transform
    }

    fn get_transform_mut(&mut self) -> &mut UITransform {
        &mut self.transform
    }
}

impl<T: UIReactor + 'static> State for Button<T> {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.update_reactor(ctx)
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        match self.reactor.borrow().get_state() {
            UIState::Idle => self.spritesheet.set_curr_index(0),
            UIState::Hover => self.spritesheet.set_curr_index(1),
            UIState::Click => self.spritesheet.set_curr_index(2),
        }
        self.spritesheet.draw(ctx, self.transform.get_draw_params());
        self.text.draw(ctx, self.transform.get_padded_pos());
        Ok(())
    }
}
