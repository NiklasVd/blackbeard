use std::{collections::HashMap, iter::FromIterator, path::{Path, PathBuf}};
use tetra::{Context, graphics::{Texture, text::Font}};

pub const ASSETS_ROOT_PATH: &str = "assets";
pub const TEXTURES_PATH: &str = "textures";

pub struct Assets {
    pub small_font: Font,
    pub font: Font,
    pub header_font: Font,
    pub header2_font: Font,
    cached_textures: HashMap<String, Texture>
}

impl Assets {
    pub fn load(ctx: &mut Context) -> tetra::Result<Assets> {
        let font_path = Self::get_full_path("Calisto.ttf".to_owned());
        let green_tex = Texture::from_rgba(ctx, 1, 1, &[51, 204, 51, 255])?;
        let red_tex = Texture::from_rgba(ctx, 1, 1, &[204, 0, 0, 255])?;

        Ok(Assets {
            small_font: Font::vector(ctx, font_path.clone(), 17.0)?,
            font: Font::vector(ctx, font_path.clone(), 20.0)?,
            header_font: Font::vector(ctx, font_path.clone(), 35.0)?,
            header2_font: Font::vector(ctx, font_path, 60.0)?,
            cached_textures: HashMap::from_iter([
                ("Green".to_owned(), green_tex), ("Red".to_owned(), red_tex)])
        })
    }

    pub fn load_texture(&mut self, ctx: &mut Context, name: String, cache: bool)
        -> tetra::Result<Texture> {
        if cache {
            if let Some(cached_texture) = self.cached_textures.get(&name) {
                return Ok(cached_texture.clone())
            }
        }

        let texture = Texture::new(ctx, Self::get_full_texture_path(name.clone()))?;
        if cache {
            self.cache_texture(name, texture.clone());
        }
        Ok(texture)
    }

    pub fn get_cached_texture(&self, name: String) -> Texture {
        return self.cached_textures[&name].clone()
    }

    fn cache_texture(&mut self, name: String, texture: Texture) {
        self.cached_textures.insert(name, texture);
    }

    fn get_full_path(file_name: String) -> PathBuf {
        Path::new(ASSETS_ROOT_PATH).join(file_name)
    }

    fn get_full_texture_path(texture_name: String) -> PathBuf {
        Path::new(ASSETS_ROOT_PATH).join(TEXTURES_PATH).join(texture_name)
    }
}
