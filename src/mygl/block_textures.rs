use std::collections::HashMap;

use crate::mygl::GLToken;

pub struct BlockTextures {
    texture_array: super::texture_array::TextureArray,
    positions: HashMap<String, f32>,
}

impl BlockTextures {
    pub fn new(glt: GLToken, pngs: &[&str]) -> Self {
        let texture_array =
            super::texture_array::TextureArray::new(glt, gl::LINEAR, gl::LINEAR, gl::CLAMP_TO_EDGE);
        texture_array.upload_png(glt, pngs.into_iter().copied(), 4);
        let mut positions = HashMap::new();
        for (i, png) in pngs.into_iter().enumerate() {
            // Store the position of the texture in the atlas
            positions.insert(png.to_string(), i as f32);
        }

        BlockTextures {
            texture_array,
            positions,
        }
    }

    pub fn bind_texture(&self, glt: GLToken) {
        self.texture_array.bind(glt);
    }

    pub fn get_texture_position(&self, name: &str) -> f32 {
        self.positions.get(name).copied().unwrap()
    }
}
