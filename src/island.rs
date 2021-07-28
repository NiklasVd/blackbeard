use tetra::{Context, State, graphics::Texture};
use crate::{Entity, EntityType, GC, Transform, get_texture_origin};

pub struct Island {
    pub transform: Transform,
    texture: Texture,
    game: GC
}

impl Island {
    pub fn build(ctx: &mut Context, game: GC) -> tetra::Result<Island> {
        let mut game_ref = game.borrow_mut();
        let texture = game_ref.assets.load_texture(ctx, "Island 1.png".to_owned(), true)?;
        let handle = game_ref.physics.build_island_collider(
            texture.width() as f32 * 0.5, texture.height() as f32 * 0.5);
        std::mem::drop(game_ref);

        Ok(Island {
            transform: Transform::new(
                get_texture_origin(texture.clone()), handle, game.clone()),
            texture, game
        })
    }
}

impl Entity for Island {
    fn get_type(&self) -> EntityType {
        EntityType::Island
    }
}

impl State for Island {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.texture.draw(ctx, self.transform.get_draw_params());
        Ok(())
    }
}
