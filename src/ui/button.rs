use tetra::{Context, graphics::{text::Text}};
use crate::{GC, V2, spritesheet::Spritesheet, ui_element::{DefaultUIReactor, UIElement, UIReactor, UIState}, ui_transform::UITransform};

pub type DefaultButton = Button<DefaultUIReactor>;

pub struct Button<T: UIReactor + 'static> {
    pub transform: UITransform,
    pub reactor: T,
    text: Text,
    spritesheet: Spritesheet
}

impl<T: UIReactor + 'static> Button<T> {
    pub fn new(ctx: &mut Context, text: &str, size: V2, padding: f32, reactor: T,
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
            text: Text::new(text, font), spritesheet, reactor
        })
    }

    pub fn is_pressed(&self) -> bool {
        self.reactor.get_state() == UIState::Focus
    }

    fn update_spritesheet(&mut self) {
        match self.reactor.get_state() {
            UIState::Idle | UIState::Disabled => self.spritesheet.set_curr_index(0),
            UIState::Hover => self.spritesheet.set_curr_index(1),
            UIState::Focus => self.spritesheet.set_curr_index(2),
            _ => ()
        }
    }
}

impl<T: UIReactor + 'static> UIElement for Button<T> {
    fn get_name(&self) -> &str {
        "Button"
    }

    fn get_reactor(&self) -> Option<&dyn UIReactor> {
        Some(&self.reactor)
    }

    fn get_reactor_mut(&mut self) -> Option<&mut dyn UIReactor> {
        Some(&mut self.reactor)
    }

    fn get_transform(&self) -> &UITransform {
        &self.transform
    }

    fn get_transform_mut(&mut self) -> &mut UITransform {
        &mut self.transform
    }

    fn draw_element(&mut self, ctx: &mut Context, parent_pos: V2) -> tetra::Result {
        self.update_spritesheet();
        self.spritesheet.draw(ctx, self.get_draw_params(parent_pos));
        self.text.draw(ctx, parent_pos + self.transform.get_padded_pos());
        Ok(())
    }

    fn update_element(&mut self, ctx: &mut Context, parent_pos: V2) -> tetra::Result {
        if self.reactor.get_state() == UIState::Focus {
            self.reactor.set_state(UIState::Idle);
        }
        Ok(())
    }
}
