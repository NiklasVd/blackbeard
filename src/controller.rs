use tetra::{Context, Event, State, graphics::{Color, DrawParams, Texture}, input::{MouseButton}};
use crate::{Entity, GC, Rcc, Ship, V2, get_texture_origin};

pub struct Controller {
    //pub players: HashMap<String, Player>
    pub possessed_ship: Option<Rcc<Ship>>,
    target_x: Texture,
    target_x_origin: V2,
    game: GC
}

impl Controller {
    pub fn possess_ship(&mut self, ship: Rcc<Ship>) {
        self.possessed_ship = Some(ship);
    }
}

impl Entity<Controller> for Controller {
    fn init(ctx: &mut Context, game: GC) -> tetra::Result<Controller> {
        let target_x = game.borrow_mut().assets.load_texture(
            ctx, "X.png".to_owned(), false)?;
        Ok(Controller {
            possessed_ship: None, target_x: target_x.clone(),
            target_x_origin: get_texture_origin(target_x), game
        })
    }
}

impl State for Controller {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        if self.possessed_ship.is_none() {
            return Ok(())
        }
        
        if let Some(target_pos) = self.possessed_ship.as_ref().unwrap().borrow().target_pos {
            self.target_x.draw(ctx, DrawParams {
                position: target_pos, origin: self.target_x_origin, scale: V2::one(),
                rotation: 0.0, color: Color::WHITE
            });
        }
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event) -> tetra::Result {
        if self.possessed_ship.is_none() {
            return Ok(())
        }

        match event {
            Event::MouseButtonPressed { button } if button == MouseButton::Right => {
                let mouse_pos = self.game.borrow().cam.get_mouse_pos(ctx);
                self.possessed_ship.as_ref().unwrap().borrow_mut()
                    .set_target_pos(mouse_pos);
            }
            _ => ()
        }
        Ok(())
    }
}
