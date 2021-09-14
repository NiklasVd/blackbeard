use rapier2d::prelude::ColliderHandle;
use tetra::{Context, graphics::text::Text};

use crate::{GC, Sprite, SpriteOrigin, Transform, V2, entity::{Entity, EntityType, GameState}};

pub struct Harbour {
    pub transform: Transform,
    pub zone_handle: ColliderHandle,
    sprite: Sprite,
    name_label: Text,
    game: GC
}

impl Harbour {
    pub fn new(ctx: &mut Context, name: String, pos: V2, rot: f32, game: GC)
        -> tetra::Result<Harbour> {
        let mut game_ref = game.borrow_mut();
        let texture = game_ref.assets.load_texture(ctx, "Harbour.png".to_owned(), true)?;
        let sprite = Sprite::new(texture, SpriteOrigin::Centre, None);
        let sprite_size = sprite.get_size();
        let handle = game_ref.physics.build_harbour_collider(
            sprite_size.x * 0.5, sprite_size.y * 0.5);
        let zone_handle = game_ref.physics.build_harbour_zone(pos, rot,
            sprite_size.x * 1.5, sprite_size.y * 1.5);
        let name_label = Text::new(name.clone(), game_ref.assets.font.clone());
        std::mem::drop(game_ref);

        let mut transform = Transform::new(handle, game.clone());
        transform.set_pos(pos, rot);
        Ok(Harbour {
            transform, zone_handle,
            sprite, name_label, game
        })
    }
}

impl Entity for Harbour {
    fn get_type(&self) -> EntityType {
        EntityType::Harbour
    }
    
    fn get_name(&self) -> String {
        self.name_label.content().to_owned()
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

impl GameState for Harbour {    
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        let translation = self.transform.get_translation();
        self.sprite.draw2(ctx, translation);
        self.name_label.draw(ctx, translation.0 - V2::new(90.0, 15.0));
        Ok(())
    }
}
