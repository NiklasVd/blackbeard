use std::{collections::HashMap};
use tetra::{Context, Event, State, input::{MouseButton}};
use crate::{GC, Player, Rcc, Sprite, SpriteOrigin, wrap_rcc};

pub struct Controller {
    pub players: HashMap<u16, Rcc<Player>>,
    pub local_player: Option<Rcc<Player>>,
    target_x: Sprite,
    game: GC
}

impl Controller {
    pub fn new(ctx: &mut Context, game: GC) -> tetra::Result<Controller> {
        let target_x = game.borrow_mut().assets.load_texture(
            ctx, "X.png".to_owned(), false)?;
        Ok(Controller {
            players: HashMap::new(), local_player: None,
            target_x: Sprite::new(target_x, SpriteOrigin::Centre, None), game
        })
    }

    pub fn add_player(&mut self, player: Player) -> Rcc<Player> {
        let player_idn = player.id.n;
        let player_ref = wrap_rcc(player);
        self.players.insert(player_idn, player_ref.clone());
        player_ref
    }

    pub fn set_local_player(&mut self, local_player: Rcc<Player>) {
        self.local_player = Some(local_player)
    }
}

impl State for Controller {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        if let Some(target_pos) = self.local_player.as_ref().unwrap().borrow()
            .possessed_ship.borrow().target_pos {
            self.target_x.draw(ctx, target_pos, 0.0);
        }
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event) -> tetra::Result {
        match event {
            Event::MouseButtonPressed { button } if button == MouseButton::Right => {
                let mouse_pos = self.game.borrow().cam.get_mouse_pos(ctx);
                self.local_player.as_ref().unwrap().borrow().possessed_ship.borrow_mut()
                    .set_target_pos(mouse_pos);
            }
            _ => ()
        }
        Ok(())
    }
}
