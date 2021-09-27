use tetra::{Context};
use crate::{ANY_COLL_GROUP, GC, SMALL_SHIP_COLL_GROUP, Sprite, SpriteOrigin, Transform, V2, entity::{Entity, EntityType, GameState}};

// Reefs only pose hazard to big ships (with greater keel depth), hence providing
// a way of escape for smaller ships

#[derive(Debug, Clone, Copy)]
pub enum ObjectType {
    Island,
    Reef,
    Shipwreck
}

pub struct Object {
    pub transform: Transform,
    pub obj_type: ObjectType,
    pub sprite: Sprite,
    destroy: bool,
    game: GC
}

impl Object {
    pub fn build_island(ctx: &mut Context, game: GC, pos: V2, rot: f32, island_type: u32)
        -> tetra::Result<Object> {
        let island_tex = match island_type {
            1 => "Island 1.png",
            2 => "Island 2.png",
            3 => "Island 3.png",
            4 => "Island 4.png",
            n @ _ => panic!("Island type {} doesn't exist", n)
        };
        Self::build_object(ctx, ObjectType::Island, game, island_tex.to_owned(),
            pos, rot, ANY_COLL_GROUP)
    }

    pub fn build_ship_wreck(ctx: &mut Context, game: GC, pos: V2, rot: f32)
        -> tetra::Result<Object> {
        // IDEA: Add timer that removes wreck after some time to avoid cluttering?
        // Only for practical reasons; the idea of 'ship graveyards' forming sounds
        // strangely appealing too.
        // Regardless, shipwrecks should take up less space and visual focus.
        // Add a transparent gradient for submerging effect.
        Self::build_object(ctx, ObjectType::Shipwreck, game, "Destroyed Caravel.png".to_owned(),
            pos, rot, ANY_COLL_GROUP)
    }

    pub fn build_reef(ctx: &mut Context, game: GC, pos: V2, rot: f32)
        -> tetra::Result<Object> {
        Self::build_object(ctx, ObjectType::Reef, game.clone(), "Reef.png".to_owned(),
            pos, rot, ANY_COLL_GROUP ^ SMALL_SHIP_COLL_GROUP)
    }

    fn build_object(ctx: &mut Context, obj_type: ObjectType, game: GC, tex_name: String,
        pos: V2, rot: f32, filter_groups: u32)
        -> tetra::Result<Object> {
        let mut game_ref = game.borrow_mut();
        let sprite = Sprite::new(game_ref.assets.load_texture(
            ctx, tex_name.to_owned(), true)?, SpriteOrigin::Centre, None);
        let handle = game_ref.physics.build_object_collider(
            sprite.texture.width() as f32 * 0.4, sprite.texture.height() as f32 * 0.4,
            obj_type, filter_groups);
        std::mem::drop(game_ref);

        let mut transform = Transform::new(handle, game.clone());
        transform.set_pos(pos, rot);
        Ok(Object {
            transform, obj_type, sprite, destroy: false, game
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

    fn marked_destroy(&self) -> bool {
        self.destroy
    }

    fn destroy(&mut self) {
        self.destroy = true;
    }
}

impl GameState for Object {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.sprite.draw2(ctx, self.transform.get_translation());
        Ok(())
    }
}
