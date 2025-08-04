use std::collections::HashMap;

use fontdue::{Font, FontSettings};
use crate::mygl::{GLToken, Texture};

pub struct TextRenderer {
    texture: Texture,
    texture_coordinates: HashMap<char, (f32, f32, f32, f32)>, // (x, y, width, height)
    vertex_data: HashMap<char, (f32, f32, f32, f32)>, // (x, y, width, height) relative to origin
    advance_data: HashMap<char, f32>, // advance for each character
}

impl TextRenderer {
    pub fn new(glt: GLToken, font_data: &[u8], chars: &str) -> Self {
        let texture = Texture::new(glt, gl::LINEAR, gl::NEAREST, gl::CLAMP_TO_EDGE);

        let font = Font::from_bytes(font_data, FontSettings::default()).unwrap();

        let mut image = image::RgbaImage::new(1024, 1024);

        for char in chars.chars() {
            let metrics = font.metrics(char, 64.0);
            dbg!(metrics);

            // Here you would typically draw the glyph onto the image.
            // For simplicity, we are not implementing the actual drawing logic.
            // You would use `image::RgbaImage` methods to draw the glyph pixels.

            // Example: image.put_pixel(x, y, image::Rgba([r, g, b, a]));
            
        }

        todo!();
    }
}
