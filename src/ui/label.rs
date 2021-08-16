use tetra::{Context, State, graphics::text::Text};
use crate::{GC, Rcc, V2, ui_element::{UIElement, UIReactor, UITransform}};

pub struct Label {
    pub transform: UITransform,
    text: Text,
}

impl Label {
    pub fn new(ctx: &mut Context, text: &str, header: bool, padding: f32, game: GC)
        -> tetra::Result<Label> {
        let (font, font_size) = match header {
            true => (game.borrow().assets.title_font.clone(), 35.0),
            false => (game.borrow().assets.font.clone(), 20.0)
        } ;
        let x_size = text.len() as f32 * font_size * 0.5;
        let text = Text::new(text, font);
        Ok(Label {
            transform: UITransform::default(ctx, V2::new(x_size, font_size * 1.2),
                V2::one(), padding)?,
            text
        })
    }

    pub fn set_text(&mut self, text: &str) {
        self.text.set_content(text);
    }
}

impl UIElement for Label {
    fn get_name(&self) -> &str {
        "Label"
    }

    fn get_reactor(&self) -> Option<Rcc<dyn UIReactor>> {
        None
    }

    fn get_transform(&self) -> &UITransform {
        &self.transform
    }

    fn get_transform_mut(&mut self) -> &mut UITransform {
        &mut self.transform
    }
}

impl State for Label {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.text.draw(ctx, self.transform.get_padded_pos());
        Ok(())
    }
}
