use std::collections::HashMap;

use anyhow::{anyhow, Ok};
use image::io::Reader;
use image::RgbaImage;

use super::GLToken;

/// *x* pixel texture atlas. It prevents bleeding and supports Mipmapping
pub struct TextureAtlas {
    texture: gl::types::GLuint,
    pixel_num: u32,
    texture_per_row: u32,
    positions: HashMap<String, (f32, f32)>,
    next_free: u32,
    image: RgbaImage,
}

const MAX_ATLAS_SIZE : u32 = 1024;

impl TextureAtlas {

    pub fn new(_: GLToken, pixel_num : u32) -> Self {
        let mut texture: gl::types::GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
        }
        assert!(texture != 0);

        // We need the biggest x such that x * pixel_num * 3 - 2 * pixel_num <= MAX_ATLAS_SIZE
        let x = (MAX_ATLAS_SIZE + 2 * pixel_num) / (pixel_num * 3);

        assert!(x > 0, "pixel_num is too high for MAX_ATLAS_SIZE");

        let actual_size = 3 * pixel_num * x - 2 * pixel_num;

        TextureAtlas {
            texture,
            pixel_num,
            texture_per_row: x,
            positions: HashMap::new(),
            next_free: 0,
            image: RgbaImage::new(actual_size, actual_size),
        }
    }

    pub fn add_texture(&mut self, path: &str) -> anyhow::Result<(f32, f32)> {
        //Already loaded
        if let Some((x,y)) = self.positions.get(path) {
            return Ok((*x,*y));
        }

        if self.next_free == self.texture_per_row * self.texture_per_row {
            return Err(anyhow!("Texture atlas is full"));
        }
        
        let mut img = Reader::open("textures/".to_owned() + path)?.decode()?.to_rgba8();
        if img.dimensions() != (self.pixel_num, self.pixel_num) {
            return Err(anyhow!("Image has to have pixel_num x pixel_num pixels"));
        }

        //Opengl has (0,0) in the bottom left, image at the top left
        image::imageops::flip_vertical_in_place(&mut img);

        let x = (self.next_free % self.texture_per_row) as i32;
        let y = (self.next_free / self.texture_per_row) as i32;

        let pixel_x = x * 3 * self.pixel_num as i32;
        let pixel_y = y * 3 * self.pixel_num as i32;

        //Copy into the image buffer and extend the border to a 3 * pixel_num * 3 * pixel_num
        for xx in -(self.pixel_num as i32)..2 * self.pixel_num as i32 {
            for yy in -(self.pixel_num as i32)..2 * self.pixel_num as i32 {
                let img_x = xx.clamp(0, self.pixel_num as i32 - 1) as u32;
                let img_y = yy.clamp(0, self.pixel_num as i32 - 1) as u32;

                self.image.put_pixel(
                    (pixel_x + xx).clamp(0, self.image.width() as i32 - 1) as u32,
                    (pixel_y + yy).clamp(0, self.image.width() as i32 - 1) as u32,
                    *img.get_pixel(img_x, img_y),
                );
            }
        }

        self.next_free += 1;

        let atlas_pixel_size = self.image.width() as f32;

        self.positions.insert(path.into(), (pixel_x as f32 / atlas_pixel_size, pixel_y as f32 / atlas_pixel_size));
        Ok((pixel_x as f32 / atlas_pixel_size, pixel_y as f32 / atlas_pixel_size))
    }

    /// Saves the internal atlas to disk, mostly for debug
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        self.image.save(path)?;
        Ok(())
    }

    pub fn get_position(&self, path: &str) -> Option<(f32, f32)> {
        self.positions.get(path).copied()
    }

    /// The size of a texture in texture coordinates
    pub fn get_size(&self) -> (f32, f32) {
        let size = self.pixel_num as f32 / self.image.width() as f32;
        (size, size)
    }

    pub fn bind_texture(&mut self, texture_unit: gl::types::GLenum) {
        unsafe {
            gl::ActiveTexture(texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
        }
    }

    /// # Safety
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
            self.image.width() as i32,
            self.image.height() as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            self.image.as_raw().as_ptr().cast(),
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }
}
