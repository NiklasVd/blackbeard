use tetra::{Context, graphics::{Color, DrawParams, Texture, text::Text}, input::{self, Key, is_key_down}};
use crate::{GC, V2, ui_element::{UIElement, UIReactor, UIState}, ui_transform::UITransform};

pub struct Textbox {
    pub transform: UITransform,
    texture: Texture,
    text: Text,
    reactor: TextboxReactor
}

impl Textbox {
    pub fn new(ctx: &mut Context, default_text: &str, size: V2, padding: f32, game: GC)
        -> tetra::Result<Textbox> {
        let mut game_ref = game.borrow_mut();
        let texture = game_ref.assets.load_texture(ctx, "UI/Textbox.png".to_owned(), true)?;
        let font = game_ref.assets.font.clone();
        std::mem::drop(game_ref);

        Ok(Textbox {
            transform: UITransform::default(ctx, size, V2::new(
                texture.width() as f32, texture.height() as f32), padding)?,
            texture, text: Text::new(default_text, font),
            reactor: TextboxReactor::new()
        })
    }

    pub fn get_text(&self) -> &str {
        self.text.content()
    }
}

impl UIElement for Textbox {
    fn get_name(&self) -> &str {
        "Textbox"
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

    fn update_element(&mut self, ctx: &mut Context, parent_pos: V2) -> tetra::Result {
        if self.reactor.get_state() != UIState::Focus {
            return Ok(())
        }

        if is_key_down(ctx, Key::Backspace) {
            self.text.pop();
        }
        if let Some(input) = input::get_text_input(ctx) {
            self.text.push_str(input);
        }
        Ok(())
    }

    fn draw_element(&mut self, ctx: &mut Context, parent_pos: V2) -> tetra::Result {
        self.texture.draw(ctx, self.get_draw_params(parent_pos));
        self.text.draw(ctx, DrawParams {
            position: parent_pos + self.transform.get_padded_pos() * 1.1, rotation: 0.0,
            scale: V2::one(), origin: V2::zero(), color: Color::BLACK
        });
        Ok(())
    }
}

struct TextboxReactor {
    state: UIState
}

impl TextboxReactor {
    fn new() -> TextboxReactor {
        TextboxReactor {
            state: UIState::Idle
        }
    }
}

impl UIReactor for TextboxReactor {
    fn get_state(&self) -> UIState {
        self.state
    }

    fn set_state(&mut self, state: UIState) {
        self.state = state;
    }
}