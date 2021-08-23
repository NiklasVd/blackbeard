use tetra::{Context, State, graphics::Texture};
use crate::{GC, V2, ui_element::{UIElement}, ui_transform::UITransform};

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

    fn get_transform(&self) -> &UITransform {
        &self.transform
    }

    fn get_transform_mut(&mut self) -> &mut UITransform {
        &mut self.transform
    }

    fn draw_element(&mut self, ctx: &mut Context, parent_pos: V2) -> tetra::Result {
        self.image.draw(ctx, self.get_draw_params(parent_pos));
        Ok(())
    }
}
