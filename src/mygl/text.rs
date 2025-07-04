#![allow(dead_code)]
use ab_glyph::FontRef;

pub struct FontAtlas {
    texture: gl::types::GLuint,
}

impl FontAtlas {
    pub fn new(font_data: &[u8], _chars: &str) -> Self {
        let mut texture: gl::types::GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
        }

        let _font = FontRef::try_from_slice(font_data).unwrap();

        todo!()
    }
}
