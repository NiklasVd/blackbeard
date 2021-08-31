use std::{collections::HashMap};
use tetra::{Context, Event, input::{Key, MouseButton}};
use crate::{GC, GameState, Player, Rcc, Sprite, SpriteOrigin, world::World, wrap_rcc};

pub struct Controller {
    pub players: HashMap<u16, Rcc<Player>>,
    pub local_player: Option<Rcc<Player>>,
    target_x: Sprite,
    game: GC
}

impl Controller {
    pub fn new(ctx: &mut Context, game: GC) -> tetra::Result<Controller> {
        let target_x = game.borrow_mut().assets.load_texture(
            ctx, "UI/X.png".to_owned(), false)?;
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

impl GameState for Controller {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        if let Some(target_pos) = self.local_player.as_ref().unwrap().borrow()
            .possessed_ship.borrow().target_pos {
            self.target_x.draw(ctx, target_pos, 0.0);
        }
        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event, world: &mut World)
        -> tetra::Result {
        if let Some(local_player) = self.local_player.as_ref() {
            match event {
                Event::MouseButtonPressed { button } if button == MouseButton::Right => {
                    let mouse_pos = self.game.borrow().cam.get_mouse_pos(ctx);
                    local_player.borrow().possessed_ship.borrow_mut()
                        .set_target_pos(mouse_pos);
                },
                Event::KeyPressed { key } => {
                    match key {
                        Key::Space => local_player.borrow().possessed_ship.borrow_mut()
                            .shoot_cannons(ctx, world)?,
                        Key::Q => local_player.borrow().possessed_ship.borrow_mut()
                            .shoot_cannons_on_side(ctx, crate::CannonSide::Bowside, world)?,
                        Key::E => local_player.borrow().possessed_ship.borrow_mut()
                            .shoot_cannons_on_side(ctx, crate::CannonSide::Portside, world)?,
                        _ => ()
                    }
                },
                _ => ()
            }
        }

        Ok(())
    }
}
