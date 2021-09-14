use std::{collections::HashMap, path::{Path, PathBuf}};
use tetra::{Context, graphics::{Texture, text::Font}};

pub const ASSETS_ROOT_PATH: &str = "assets";
pub const TEXTURES_PATH: &str = "textures";

pub struct Assets {
    pub small_font: Font,
    pub font: Font,
    pub header_font: Font,
    cached_textures: HashMap<String, Texture>
}

impl Assets {
    pub fn load(ctx: &mut Context) -> tetra::Result<Assets> {
        let font_path = Self::get_full_path("Calisto.ttf".to_owned());
        Ok(Assets {
            small_font: Font::vector(ctx, font_path.clone(), 17.0)?,
            font: Font::vector(ctx, font_path.clone(), 20.0)?,
            header_font: Font::vector(ctx, font_path, 35.0)?,
            cached_textures: HashMap::new()
        })
    }

    pub fn load_texture(&mut self, ctx: &mut Context, name: String, cache: bool)
        -> tetra::Result<Texture> {
        if cache {
            if let Some(cached_tex) = self.cached_textures.get(&name) {
                return Ok(cached_tex.clone())
            }
        }

        let tex = Texture::new(ctx, Self::get_full_texture_path(name.clone()))?;
        if cache {
            self.cached_textures.insert(name, tex.clone());
        }
        Ok(tex)
    }

    fn get_full_path(file_name: String) -> PathBuf {
        Path::new(ASSETS_ROOT_PATH).join(file_name)
    }

    fn get_full_texture_path(texture_name: String) -> PathBuf {
        Path::new(ASSETS_ROOT_PATH).join(TEXTURES_PATH).join(texture_name)
    }
}
