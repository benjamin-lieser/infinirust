use image::RgbaImage;
use image::io::Reader;
use anyhow::{anyhow, Ok};


/// 16x16 pixel texture atlas. It prevents bleeding and supports Mipmapping
pub struct TextureAtlas {
    texture : gl::types::GLuint,
    positions : Vec<(f32, f32)>,
    image : RgbaImage
}

impl TextureAtlas {
    pub fn new() -> Self {
        let mut texture : gl::types::GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
        }
        assert!(texture != 0);
        TextureAtlas { texture, positions: vec![(0.0,0.0);484], image : RgbaImage::new(1024, 1024) }
    }


    pub fn add_texture(&mut self, path : impl AsRef<std::path::Path>, id : u32) -> anyhow::Result<()> {
        assert!(id <= 484);
        let mut img = Reader::open(path)?.decode()?.to_rgba8();
        if img.dimensions() != (16,16) {
            return Err(anyhow!("Image has to have 16x16 pixels"));
        }

        //Opengl has (0,0) in the bottom left, image at the top left
        image::imageops::flip_vertical_in_place(&mut img);

        let x = (id % 22) as i32;
        let y = (id / 22) as i32;

        let pixel_x = x * 48;
        let pixel_y = y * 48;

        //Copy into the image buffer and extend the border to a 48 * 48
        for xx in -16i32..32 {
            for yy in -16i32..32 {

                let img_x = xx.clamp(0, 15) as u32;
                let img_y = yy.clamp(0, 15) as u32;


                self.image.put_pixel((pixel_x + xx).clamp(0, 1023) as u32, (pixel_y + yy).clamp(0, 1023) as u32, *img.get_pixel(img_x, img_y));
            }
        }

        self.positions[id as usize] = (pixel_x as f32 / 1024.0, pixel_y as f32 / 1024.0);

        Ok(())
    }

    pub fn save(&self, path : impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        self.image.save(path)?;
        Ok(())
    }

    pub fn get_position(&self, id : u32) -> (f32, f32) {
        self.positions[id as usize]
    }

    pub fn bind_texture(&mut self, texture_unit : gl::types::GLenum) {
        unsafe {
            gl::ActiveTexture(texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
        }
    }

    /// Call bind texture first
    pub unsafe fn finalize(&mut self) {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 4);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as gl::types::GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as gl::types::GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as gl::types::GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as gl::types::GLint);

        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA8 as gl::types::GLint, 1024, 1024, 0, gl::RGBA, gl::UNSIGNED_BYTE, self.image.as_raw().as_ptr().cast());
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

}