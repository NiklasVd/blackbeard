use tetra::{Context, State, graphics::Texture};
use crate::{Entity, EntityType, GC, Sprite, SpriteOrigin, Transform, V2};

pub struct Object {
    pub transform: Transform,
    sprite: Sprite,
    game: GC
}

impl Object {
    pub fn build_island(ctx: &mut Context, game: GC, pos: V2, rot: f32)
        -> tetra::Result<Object> {
        Self::build_object(ctx, game, "Island 1.png".to_owned(), pos, rot)
    }

    pub fn build_ship_wreck(ctx: &mut Context, game: GC, pos: V2, rot: f32)
        -> tetra::Result<Object> {
        Self::build_object(ctx, game, "Destroyed Caravel.png".to_owned(), pos, rot)
    }

    fn build_object(ctx: &mut Context, game: GC, tex_name: String, pos: V2, rot: f32)
        -> tetra::Result<Object> {
        let mut game_ref = game.borrow_mut();
        let sprite = Sprite::new(game_ref.assets.load_texture(
            ctx, tex_name.to_owned(), true)?, SpriteOrigin::Centre);
        let handle = game_ref.physics.build_object_collider(
            sprite.texture.width() as f32 * 0.5, sprite.texture.height() as f32 * 0.5);
        std::mem::drop(game_ref);

        let mut transform = Transform::new(handle, game.clone());
        transform.set_pos(pos, rot);
        Ok(Object {
            transform, sprite, game
        })
    }
}

impl Entity for Object {
    fn get_type(&self) -> EntityType {
        EntityType::Object
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.sprite.draw2(ctx, self.transform.get_translation());
        Ok(())
    }
}
