use image::RgbaImage;
use image::io::Reader;
use anyhow::anyhow;


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
        let img = Reader::open(path)?.decode()?.to_rgba8();
        if img.dimensions() != (16,16) {
            return Err(anyhow!("Image has to have 16x16 pixels"));
        }

        let x = (id % 22) as u32;
        let y = (id / 22) as u32;

        let pixel_x = x * 48;
        let pixel_y = y * 48;

        //Copy into the image buffer
        for xx in 0..16 {
            for yy in 0..16 {
                self.image.put_pixel(pixel_x + xx, pixel_y + yy, *img.get_pixel(xx, yy));
            }
        }

        self.positions[id as usize] = (pixel_x as f32 / 1024.0, pixel_y as f32 / 1024.0);

        Ok(())
    }

}