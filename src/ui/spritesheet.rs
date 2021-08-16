use tetra::{Context, graphics::{DrawParams, Rectangle, Texture}};

use crate::GC;

pub struct Spritesheet {
    pub texture: Texture,
    pub rects: Vec<Rectangle>,
    curr_index: usize
}

impl Spritesheet {
    pub fn new(texture: Texture, game: GC, cell_width: f32, cell_height: f32,
        cell_count: usize) -> Spritesheet {
        Spritesheet {
            texture, rects: Rectangle::row(
                0.0, 0.0, cell_width, cell_height).take(cell_count).collect(),
            curr_index: 0
        }
    }

    pub fn set_curr_index(&mut self, index: usize) {
        assert!(index < self.rects.len());
        self.curr_index = index;
    }

    pub fn get_curr_index(&self) -> usize {
        self.curr_index
    }

    pub fn get_curr_rect(&self) -> Rectangle {
        self.rects[self.curr_index]
    }

    pub fn draw(&self, ctx: &mut Context, params: DrawParams) {
        self.texture.draw_region(ctx, self.get_curr_rect(), params)
    }
}
