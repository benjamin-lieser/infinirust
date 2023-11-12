use std::collections::HashMap;

use anyhow::{anyhow, Ok};
use image::io::Reader;
use image::RgbaImage;

/// 16x16 pixel texture atlas. It prevents bleeding and supports Mipmapping
pub struct TextureAtlas {
    texture: gl::types::GLuint,
    positions: HashMap<String, (f32, f32)>,
    next_free: u32,
    image: RgbaImage,
}

impl TextureAtlas {
    /// Needs GL context
    pub fn new() -> Self {
        let mut texture: gl::types::GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
        }
        assert!(texture != 0);
        TextureAtlas {
            texture,
            positions: HashMap::new(),
            next_free: 0,
            image: RgbaImage::new(1024, 1024),
        }
    }

    pub fn add_texture(&mut self, path: &str) -> anyhow::Result<(f32, f32)> {
        //Already loaded
        if let Some((x,y)) = self.positions.get(path) {
            return Ok((*x,*y));
        }

        if self.next_free == 22*22 {
            return Err(anyhow!("Texture atlas is full"));
        }
        
        let mut img = Reader::open(path)?.decode()?.to_rgba8();
        if img.dimensions() != (16, 16) {
            return Err(anyhow!("Image has to have 16x16 pixels"));
        }

        //Opengl has (0,0) in the bottom left, image at the top left
        image::imageops::flip_vertical_in_place(&mut img);

        let x = (self.next_free % 22) as i32;
        let y = (self.next_free / 22) as i32;

        let pixel_x = x * 48;
        let pixel_y = y * 48;

        //Copy into the image buffer and extend the border to a 48 * 48
        for xx in -16i32..32 {
            for yy in -16i32..32 {
                let img_x = xx.clamp(0, 15) as u32;
                let img_y = yy.clamp(0, 15) as u32;

                self.image.put_pixel(
                    (pixel_x + xx).clamp(0, 1023) as u32,
                    (pixel_y + yy).clamp(0, 1023) as u32,
                    *img.get_pixel(img_x, img_y),
                );
            }
        }

        self.next_free += 1;

        self.positions.insert(path.into(), (pixel_x as f32 / 1024.0, pixel_y as f32 / 1024.0));
        Ok((pixel_x as f32 / 1024.0, pixel_y as f32 / 1024.0))
    }

    /// Saves the internal atlas to disk, mostly for debug
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        self.image.save(path)?;
        Ok(())
    }

    pub fn get_position(&self, path: &str) -> Option<(f32, f32)> {
        self.positions.get(path).copied()
    }

    pub fn get_size() -> (f32, f32) {
        (16.0 / 1024.0, 16.0 / 1024.0)
    }

    pub fn bind_texture(&mut self, texture_unit: gl::types::GLenum) {
        unsafe {
            gl::ActiveTexture(texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
        }
    }

    /// Call bind texture first
    pub unsafe fn finalize(&mut self) {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 4);
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::NEAREST as gl::types::GLint,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            gl::NEAREST as gl::types::GLint,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as gl::types::GLint,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as gl::types::GLint,
        );

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA8 as gl::types::GLint,
            1024,
            1024,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            self.image.as_raw().as_ptr().cast(),
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }
}
