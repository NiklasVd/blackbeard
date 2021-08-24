use tetra::{Context, Event, graphics::{Color, DrawParams}, input::{MouseButton, get_mouse_position, is_mouse_button_pressed}};
use crate::{V2, ui_transform::UITransform};

pub trait UIElement {
    fn get_name(&self) -> &str;
    fn get_reactor(&self) -> Option<&dyn UIReactor> {
        None
    }
    fn get_reactor_mut(&mut self) -> Option<&mut dyn UIReactor> {
        None
    }

    fn get_transform(&self) -> &UITransform;
    fn get_transform_mut(&mut self) -> &mut UITransform;

    fn get_draw_params(&self, parent_pos: V2) -> DrawParams {
        let transform = self.get_transform();
        DrawParams {
            position: parent_pos + transform.get_padded_pos(), rotation: 0.0,
            origin: V2::zero(), scale: transform.calc_scale(),
            color: match self.is_disabled() {
                false => Color::WHITE,
                true => Color::rgb8(150, 150, 150)
            }
        }
    }

    fn is_disabled(&self) -> bool {
        if let Some(reactor) = self.get_reactor() {
            return reactor.get_state() == UIState::Disabled
        }
        return false
    }

    fn set_disabled(&mut self, state: bool) {
        if let Some(reactor) = self.get_reactor_mut().as_mut() {
            reactor.set_state(match state {
                true => UIState::Disabled,
                false => UIState::Idle
            });
        }
    }

    fn update_reactor(&mut self, ctx: &mut Context, parent_pos: V2) -> tetra::Result {
        let rect = self.get_transform().get_padded_rect(parent_pos);
        if let Some(reactor) = self.get_reactor_mut() {
            if reactor.get_state() == UIState::Disabled {
                return Ok(())
            }
            
            let mouse_pos = get_mouse_position(ctx);
            let mouse_in_rect = rect.contains_point(mouse_pos);
            if is_mouse_button_pressed(ctx, MouseButton::Left) {
                if mouse_in_rect {
                    reactor.set_state(UIState::Focus);
                    reactor.on_click(ctx)?;
                }
                else {
                    reactor.set_state(UIState::Idle);
                    reactor.on_unfocus(ctx)?;
                }
            }
            else {
                if mouse_in_rect && reactor.get_state() == UIState::Idle {
                    reactor.set_state(UIState::Hover);
                    reactor.on_hover(ctx)?;
                }
                else if !mouse_in_rect && reactor.get_state() == UIState::Hover {
                    reactor.set_state(UIState::Idle);
                    reactor.on_unhover(ctx)?;
                }
            }
        }
        Ok(())
    }

    fn draw_rect(&mut self, ctx: &mut Context, parent_pos: V2) {
        self.get_transform_mut().draw_rect(ctx, parent_pos);
    }

    fn update_element(&mut self, ctx: &mut Context, parent_pos: V2) -> tetra::Result {
        Ok(())
    }
    fn draw_element(&mut self, ctx: &mut Context, parent_pos: V2) -> tetra::Result {
        Ok(())
    }
    fn event_element(&mut self, ctx: &mut Context, event: Event, parent_pos: V2)
        -> tetra::Result {
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UIState {
    Idle,
    Hover,
    Focus,
    Disabled
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
    fn on_unfocus(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }
}

pub struct DefaultUIReactor {
    pub state: UIState
}

impl DefaultUIReactor {
    pub fn new() -> DefaultUIReactor {
        DefaultUIReactor {
            state: UIState::Idle
        }
    }
}

impl UIReactor for DefaultUIReactor {
    fn get_state(&self) -> UIState {
        self.state.clone()
    }

    fn set_state(&mut self, state: UIState) {
        self.state = state;
    }
}
