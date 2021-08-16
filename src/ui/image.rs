use tetra::{Context, State, graphics::Texture};
use crate::{GC, Rcc, V2, ui_element::{UIElement, UIReactor, UITransform}};

pub struct Image {
    pub transform: UITransform,
    image: Texture
}

impl Image {
    pub fn new(ctx: &mut Context, size: V2, padding: f32, image_path: String,
        cache: bool, game: GC) -> tetra::Result<Image> {
        let image = game.borrow_mut().assets.load_texture(ctx, image_path, cache)?;
        Ok(Image {
            transform: UITransform::default(ctx, size,
                V2::new(image.width() as f32, image.height() as f32), padding)?,
            image
        })
    }
}

impl UIElement for Image {
    fn get_name(&self) -> &str {
        "Image"
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

impl State for Image {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.image.draw(ctx, self.transform.get_draw_params());
        Ok(())
    }
}
