use std::any::Any;

use tetra::{Context};
use crate::{Entity, EntityType, GC, GameState, Sprite, SpriteOrigin, Transform, V2};

// Reefs only pose hazard to big ships (with greater keel depth), hence providing
// a way of escape for smaller ships

#[derive(Debug)]
pub enum ObjectType {
    Island,
    Reef
}

pub struct Object {
    pub transform: Transform,
    pub obj_type: ObjectType,
    sprite: Sprite,
    game: GC
}

impl Object {
    pub fn build_island(ctx: &mut Context, game: GC, pos: V2, rot: f32)
        -> tetra::Result<Object> {
        Self::build_object(ctx, ObjectType::Island, game, "Island 1.png".to_owned(),
            pos, rot)
    }

    pub fn build_ship_wreck(ctx: &mut Context, game: GC, pos: V2, rot: f32)
        -> tetra::Result<Object> {
        Self::build_object(ctx, ObjectType::Reef, game, "Destroyed Caravel.png".to_owned(),
            pos, rot)
    }

    fn build_object(ctx: &mut Context, obj_type: ObjectType, game: GC, tex_name: String,
        pos: V2, rot: f32)
        -> tetra::Result<Object> {
        let mut game_ref = game.borrow_mut();
        let sprite = Sprite::new(game_ref.assets.load_texture(
            ctx, tex_name.to_owned(), true)?, SpriteOrigin::Centre, None);
        let handle = game_ref.physics.build_object_collider(
            sprite.texture.width() as f32 * 0.4, sprite.texture.height() as f32 * 0.4);
        std::mem::drop(game_ref);

        let mut transform = Transform::new(handle, game.clone());
        transform.set_pos(pos, rot);
        Ok(Object {
            transform, obj_type, sprite, game
        })
    }
}

impl Entity for Object {
    fn get_type(&self) -> EntityType {
        EntityType::Object
    }

    fn get_name(&self) -> String {
        format!("{:?}", self.obj_type)
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl GameState for Object {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.sprite.draw2(ctx, self.transform.get_translation());
        Ok(())
    }
}
