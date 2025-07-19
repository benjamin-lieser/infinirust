#![allow(dead_code)]
use ab_glyph::{Font, FontRef, point};

use crate::mygl::{GLToken, texture_array::TextureArray};

pub struct FontAtlas {
    texture: TextureArray,
}

impl FontAtlas {
    pub fn new(glt: GLToken, font_data: &[u8], chars: &str) -> Self {
        let texture = TextureArray::new(glt, gl::LINEAR, gl::NEAREST, gl::CLAMP_TO_EDGE);

        let font = FontRef::try_from_slice(font_data).unwrap();

        let mut image = image::RgbaImage::new(32, 32);

        for char in chars.chars() {

            dbg!(font.h_advance_unscaled(font.glyph_id(char)));

            let glyph = font
                .glyph_id(char)
                .with_scale_and_position(40.0, point(0.0, 0.0));

            let q = font.outline_glyph(glyph).unwrap();

            image.pixels_mut().for_each(|f| {
                *f = image::Rgba([255, 255, 255, 0]);
            });

            println!("Drawing character: {}", char);
            dbg!(q.px_bounds());

            q.draw(|x, y, v| {
                let pixel = image.get_pixel_mut(x, y);
                let alpha = (v * 255.0) as u8;
                *pixel = image::Rgba([255, 255, 255, alpha]);
            });

            image.save_with_format(format!("debug/{char}.png"), image::ImageFormat::Png).unwrap();
        }

        todo!();
    }
}
