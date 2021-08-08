use std::time::Duration;

use tetra::{Context, graphics::{Color, DrawParams, Rectangle, Texture, animation::Animation}};
use crate::{GC, V2};

pub struct AnimatedSprite {
    pub anim: Animation,
    pub translation: Option<(V2, f32)>,
    sprite_count: usize
}

impl AnimatedSprite {
    pub fn new(spritesheet: Texture, sprite_count: usize, width: f32, height: f32,
        frame_length: f32, repeat: bool, translation: Option<(V2, f32)>) -> AnimatedSprite {
        let mut anim = Animation::new(spritesheet, Rectangle::row(
                0.0, 0.0, width, height).take(sprite_count).collect(),
            Duration::from_secs_f32(frame_length));
        anim.set_repeating(repeat);
        AnimatedSprite {
            anim, translation, sprite_count
        }
    }

    pub fn is_finished(&self) -> bool {
        self.anim.current_frame_index() == self.sprite_count - 1
    }

    pub fn draw(&mut self, ctx: &mut Context, translation: (V2, f32)) {
        self.anim.advance(ctx);
        self.anim.draw(ctx, DrawParams {
            position: translation.0, rotation: translation.1,
            scale: V2::one(), origin: V2::zero(), color: Color::WHITE
        })
    }

    pub fn draw2(&mut self, ctx: &mut Context) {
        if let Some(translation) = self.translation {
            self.draw(ctx, translation);
        }
        else {
            println!("Unable to draw animated sprite; no default translation given.")
        }
    }
}

pub fn build_water_splash_sprite(ctx: &mut Context, game: GC, pos: V2)
    -> tetra::Result<AnimatedSprite> {
    Ok(AnimatedSprite::new(
            game.borrow_mut().assets.load_texture(
            ctx, "Water Splash.png".to_owned(), true)?, 5, 15.0, 15.0, 0.2, false,
        Some((pos, 0.0))))
}
