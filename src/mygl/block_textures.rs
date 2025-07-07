use std::collections::HashMap;

use crate::mygl::GLToken;

pub struct BlockTextures {
    texture: gl::types::GLuint,
    positions: HashMap<String, f32>,
}

impl BlockTextures {
    pub fn new(_: GLToken, size_pixel: u32, pngs: &[String]) -> Self {
        let mut texture: gl::types::GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
        }
        assert!(texture != 0);

        let mut positions = HashMap::new();

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, texture);
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as i32,
            );

            gl::TexStorage3D(
                gl::TEXTURE_2D_ARRAY,
                4, // mipmap level
                gl::RGBA8,
                size_pixel as i32,
                size_pixel as i32,
                pngs.len() as i32,
            );

            for (i, png) in pngs.iter().enumerate() {
                let mut image = image::open("textures/".to_string() + png).expect("Failed to open image").to_rgba8();
                image::imageops::flip_vertical_in_place(&mut image);
                assert_eq!(image.width(), size_pixel);
                assert_eq!(image.height(), size_pixel);

                gl::TexSubImage3D(
                    gl::TEXTURE_2D_ARRAY,
                    0, // mipmap level
                    0,
                    0,
                    i as i32,
                    size_pixel as i32,
                    size_pixel as i32,
                    1, // depth
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    image.as_raw().as_ptr().cast(),
                );

                // Store the position of the texture in the atlas
                positions.insert(png.to_string(), i as f32);
            }

            gl::GenerateMipmap(gl::TEXTURE_2D_ARRAY);
        }

        BlockTextures { texture, positions }
    }

    pub fn bind_texture(&self, _: GLToken, texture_unit: gl::types::GLenum) {
        unsafe {
            gl::ActiveTexture(texture_unit);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.texture);
        }
    }

    pub fn get_texture_position(&self, name: &str) -> f32 {
        self.positions.get(name).copied().unwrap()
    }
}
