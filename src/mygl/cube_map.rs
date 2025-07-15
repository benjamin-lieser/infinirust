use std::path::PathBuf;

use gl::types::GLuint;

use crate::mygl::GLToken;

pub struct CubeMap {
    id: GLuint,
}

impl CubeMap {
    pub fn new(_: GLToken, textures: &Vec<PathBuf>) -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, id);
        }
        assert!(id != 0);
        unsafe {
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_R,
                gl::CLAMP_TO_EDGE as i32,
            );
        }

        for (i, path) in textures.iter().enumerate() {
            let image = image::open(path).expect("Failed to load cube map texture");
            let image = image.into_rgba8();
            let (width, height) = image.dimensions();
            assert!(width == height, "Cube map textures must be square");

            unsafe {
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                    0,
                    gl::RGBA as i32,
                    width as i32,
                    height as i32,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    image.as_raw().as_ptr() as *const _,
                );
            }
        }

        CubeMap { id }
    }

    pub fn bind(&self, _: GLToken) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.id);
        }
    }

    pub fn delete(mut self, _: GLToken) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
        self.id = 0; // Mark as deleted
    }
}
