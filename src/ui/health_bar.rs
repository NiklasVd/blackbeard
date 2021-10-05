use tetra::{Context, graphics::{Color, DrawParams, Texture, text::Text}};
use crate::{GC, V2};

pub const HEALTH_BAR_WIDTH: f32 = 200.0;
pub const HEALTH_BAR_HEIGHT: f32 = 20.0;

pub struct HealthBar {
    label: Text,
    life_tex: Texture,
    red_tex: Texture,
    curr_health: u16,
    max_health: u16,
    curr_health_rel: f32,
    label_color: Color,
    label_rel_centre: V2
}

impl HealthBar {
    pub fn new(ctx: &mut Context, name: String, label_color: Color, max_health: u16,
        game: GC) -> tetra::Result<HealthBar> {
        let (life_tex, red_tex, font) = {
            let game_ref = game.borrow();
            (game_ref.assets.get_cached_texture("Green".to_owned()),
                game_ref.assets.get_cached_texture("Red".to_owned()),
                game_ref.assets.header2_font.clone())
        };
        let mut label = Text::new(name, font);
        let label_rel_centre = label.get_bounds(ctx).unwrap().center();
        Ok(HealthBar {
            label, life_tex, red_tex,
            curr_health: max_health, max_health, curr_health_rel: 1.0,
            label_color, label_rel_centre
        })
    }

    pub fn set_info(&mut self, curr_health: u16) {
        assert!(curr_health <= self.max_health);
        self.curr_health_rel = curr_health as f32 / self.max_health as f32;
        self.curr_health = curr_health;
    }

    pub fn draw(&mut self, ctx: &mut Context, mut pos: V2) {
        pos -= V2::new(0.0, 30.0);
        self.label.draw(ctx, DrawParams {
            position: pos - self.label_rel_centre, rotation: 0.0, scale: V2::one(),
            origin: V2::zero(), color: self.label_color
        });
        pos += V2::new(-HEALTH_BAR_WIDTH * 0.5, self.label_rel_centre.y * 1.85);
        self.red_tex.draw(ctx, DrawParams {
            position: pos, rotation: 0.0, origin: V2::zero(),
            scale: V2::new(HEALTH_BAR_WIDTH, HEALTH_BAR_HEIGHT),
            color: Color::WHITE
        });
        self.life_tex.draw(ctx, DrawParams {
            position: pos, rotation: 0.0, origin: V2::zero(),
            scale: V2::new(HEALTH_BAR_WIDTH * self.curr_health_rel, HEALTH_BAR_HEIGHT),
            color: Color::WHITE
        });
    }
}
