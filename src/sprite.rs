use tetra::{Context, graphics::{Color, DrawParams, Texture}};
use crate::V2;

pub enum SpriteOrigin {
    TopLeft,
    BottomRight,
    Centre
}

pub struct Sprite {
    pub texture: Texture,
    pub origin: V2
}

impl Sprite {
    pub fn new(texture: Texture, origin: SpriteOrigin) -> Sprite {
        let origin = match origin {
            SpriteOrigin::TopLeft => V2::zero(),
            SpriteOrigin::BottomRight => V2::new(texture.width() as f32,
                texture.height() as f32),
            SpriteOrigin::Centre => V2::new(texture.width() as f32 * 0.5,
                texture.height() as f32 * 0.5)
        };
        Sprite {
            texture, origin
        }
    }

    pub fn draw(&self, ctx: &mut Context, position: V2, rotation: f32) {
        self.texture.draw(ctx, DrawParams {
            position, rotation, origin: self.origin, scale: V2::one(), color: Color::WHITE
        })
    }

    pub fn draw2(&self, ctx: &mut Context, translation: (V2, f32)) {
        self.draw(ctx, translation.0, translation.1);
    }
}
