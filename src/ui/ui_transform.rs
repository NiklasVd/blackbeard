use tetra::{Context, graphics::{Color, Rectangle, mesh::{Mesh, GeometryBuilder, ShapeStyle}}};
use crate::{V2, grid::UIAlignment};

pub struct UITransform {
    pub position: V2,
    pub size: V2,
    pub padding: f32,
    pub uniform_size: V2,
    rect: Option<Mesh>
}

impl UITransform {
    pub fn new(ctx: &mut Context, position: V2, size: V2, uniform_size: V2, padding: f32)
        -> tetra::Result<Self> {
        let rect = Self::get_rect_shape(ctx, size)?;
        Ok(Self {
            position, size, uniform_size, padding, rect
        })
    }

    pub fn default(ctx: &mut Context, size: V2, uniform_size: V2, padding: f32)
        -> tetra::Result<Self> {
        Self::new(ctx, V2::zero(), size, uniform_size, padding)
    }

    #[cfg(not(debug_assertions))]
    fn get_rect_shape(ctx: &mut Context, size: V2) -> tetra::Result<Option<Mesh>> {
        Ok(None)
    }

    #[cfg(debug_assertions)]
    fn get_rect_shape(ctx: &mut Context, size: V2) -> tetra::Result<Option<Mesh>> {
        Ok(Some(GeometryBuilder::new().set_color(Color::WHITE).rectangle(
            ShapeStyle::Stroke(1.0), Rectangle::new(0.0, 0.0, size.x, size.y))?
            .build_mesh(ctx)?))
    }

    pub fn get_abs_pos(&self, parent_pos: V2) -> V2 {
        self.get_padded_pos() + parent_pos
    }

    pub fn get_centre(&self) -> V2 {
        self.position + self.size * 0.5
    }

    pub fn get_rect(&self, parent_pos: V2) -> Rectangle {
        let abs_pos = self.get_abs_pos(parent_pos);
        Rectangle::new(abs_pos.x, abs_pos.y, self.size.x, self.size.y)
    }

    pub fn get_padded_pos(&self) -> V2 {
        self.position + self.padding
    }

    pub fn get_padded_size(&self) -> V2 {
        self.size - self.padding
    }

    pub fn get_padded_rect(&self, parent_pos: V2) -> Rectangle {
        let abs_pos = self.get_abs_pos(parent_pos);
        let padded_size = self.get_padded_size();
        Rectangle::new(abs_pos.x, abs_pos.y, padded_size.x, padded_size.y)
    }

    pub fn get_next_aligned_pos(&self, alignment: UIAlignment) -> V2 {
        match alignment {
            UIAlignment::Vertical => {
                let pos_y_bot_left = self.position.y + self.size.y;
                V2::new(self.position.x, pos_y_bot_left)
            },
            UIAlignment::Horizontal => {
                let pos_x_bot_left = self.position.x + self.size.x;
                V2::new(pos_x_bot_left, self.position.y)
            },
        }
    }

    pub fn calc_scale(&self) -> V2 {
        let rel_x = 1.0 / self.uniform_size.x;
        let rel_y = 1.0 / self.uniform_size.y;
        let size = self.get_padded_size();
        V2::new(size.x * rel_x, size.y * rel_y)
    }

    pub fn draw_rect(&mut self, ctx: &mut Context, parent_pos: V2) {
        if let Some(rect) = self.rect.as_ref() {
            rect.draw(ctx, parent_pos + self.position);
        }
    }
}