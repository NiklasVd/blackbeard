use std::{collections::HashMap, path::{Path, PathBuf}};
use tetra::{Context, graphics::{Texture, text::Font}};

pub const ASSETS_ROOT_PATH: &str = "assets";
pub const TEXTURES_PATH: &str = "textures";

pub struct Assets {
    pub font: Font,
    cached_textures: HashMap<String, Texture>
}

impl Assets {
    pub fn load(ctx: &mut Context) -> tetra::Result<Assets> {
        Ok(Assets {
            font: Font::vector(ctx, Self::get_full_path("Calisto.ttf".to_owned()), 20.0)?,
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
        return Ok(tex)
    }

    fn get_full_path(file_name: String) -> PathBuf {
        Path::new(ASSETS_ROOT_PATH).join(file_name)
    }

    fn get_full_texture_path(texture_name: String) -> PathBuf {
        Path::new(ASSETS_ROOT_PATH).join(TEXTURES_PATH).join(texture_name)
    }
}
