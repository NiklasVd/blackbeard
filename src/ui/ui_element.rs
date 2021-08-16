use tetra::{Context, State, graphics::{Color, DrawParams, Rectangle, mesh::{Mesh, GeometryBuilder, ShapeStyle}}, input::{MouseButton, get_mouse_position, is_mouse_button_pressed}};
use crate::{Rcc, V2, grid::UIAlignment};

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
        let rect = Self::get_rect_shape(ctx, position, size)?;
        Ok(Self {
            position, size, uniform_size, padding, rect
        })
    }

    pub fn default(ctx: &mut Context, size: V2, uniform_size: V2, padding: f32)
        -> tetra::Result<Self> {
        Self::new(ctx, V2::zero(), size, uniform_size, padding)
    }

    #[cfg(not(debug_assertions))]
    fn get_rect_shape(ctx: &mut Context, pos: V2, size: V2) -> tetra::Result<Option<Mesh>> {
        Ok(None)
    }

    #[cfg(debug_assertions)]
    fn get_rect_shape(ctx: &mut Context, pos: V2, size: V2) -> tetra::Result<Option<Mesh>> {
        Ok(Some(GeometryBuilder::new().set_color(Color::WHITE).rectangle(
            ShapeStyle::Stroke(1.0),Rectangle::new(0.0, 0.0, size.x, size.y))?
            .build_mesh(ctx)?))
    }

    pub fn get_centre(&self) -> V2 {
        self.position + self.size * 0.5
    }

    pub fn get_rect(&self) -> Rectangle {
        Rectangle::new(self.position.x, self.position.y, self.size.x, self.size.y)
    }

    pub fn get_padded_pos(&self) -> V2 {
        self.position + self.padding
    }

    pub fn get_padded_size(&self) -> V2 {
        self.size - self.padding
    }

    pub fn get_padded_rect(&self) -> Rectangle {
        let padded_pos = self.get_padded_pos();
        let padded_size = self.get_padded_size();
        Rectangle::new(padded_pos.x, padded_pos.y, padded_size.x, padded_size.y)
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

    pub fn get_draw_params(&self) -> DrawParams {
        DrawParams {
            position: self.get_padded_pos(), rotation: 0.0, origin: V2::zero(),
            scale: self.calc_scale(), color: Color::WHITE
        }
    }

    fn calc_scale(&self) -> V2 {
        let rel_x = 1.0 / self.uniform_size.x;
        let rel_y = 1.0 / self.uniform_size.y;
        let size = self.get_padded_size();
        V2::new(size.x * rel_x, size.y * rel_y)
    }

    pub fn draw_rect(&mut self, ctx: &mut Context) {
        if let Some(rect) = self.rect.as_ref() {
            // Update pos?
            rect.draw(ctx, self.position);
        }
    }
}

pub trait UIElement : State {
    fn get_name(&self) -> &str;
    fn get_reactor(&self) -> Option<Rcc<dyn UIReactor>>;

    fn get_transform(&self) -> &UITransform;
    fn get_transform_mut(&mut self) -> &mut UITransform;

    fn update_reactor(&mut self, ctx: &mut Context) -> tetra::Result {
        let rect = self.get_transform().get_rect();
        if let Some(reactor) = self.get_reactor() {
            let mut reactor_ref = reactor.borrow_mut();
            let mouse_pos = get_mouse_position(ctx);
            let mouse_in_rect = rect.contains_point(mouse_pos);
            if mouse_in_rect && is_mouse_button_pressed(ctx, MouseButton::Left) {
                reactor_ref.set_state(UIState::Click);
                reactor_ref.on_click(ctx)?;
            }
            else {
                if mouse_in_rect && reactor_ref.get_state() != UIState::Hover {
                    reactor_ref.set_state(UIState::Hover);
                    reactor_ref.on_hover(ctx)?;
                }
                else if !mouse_in_rect && reactor_ref.get_state() == UIState::Hover {
                    reactor_ref.set_state(UIState::Idle);
                    reactor_ref.on_unhover(ctx)?;
                }
                else if !mouse_in_rect {
                    reactor_ref.set_state(UIState::Idle);
                }
            }
        }
        Ok(())
    }

    fn draw_rect(&mut self, ctx: &mut Context) {
        self.get_transform_mut().draw_rect(ctx);
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UIState {
    Idle,
    Hover,
    Click
}

pub trait UIReactor {
    fn get_state(&self) -> UIState;
    fn set_state(&mut self, state: UIState);

    fn on_hover(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }
    fn on_unhover(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }
    fn on_click(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }
}
