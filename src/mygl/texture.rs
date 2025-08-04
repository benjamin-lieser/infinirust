use gl::types::{GLenum, GLint, GLuint};

use crate::mygl::GLToken;

pub struct Texture {
    id: GLuint,
}

impl Texture {
    pub fn new(_glt: GLToken, min_filter: GLenum, mag_filter: GLenum, wrapping: GLenum) -> Self {
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrapping as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrapping as i32);
        }
        Self { id }
    }

    pub fn bind(&self, _glt: GLToken) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn upload(&self, glt: GLToken, image: &image::RgbaImage) {
        self.bind(glt);
        let (width, height) = image.dimensions();
        let data = image.as_raw();
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr().cast(),
            );
        }
    }

    pub fn delete(mut self, _glt: GLToken) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
        self.id = 0;
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        assert!(self.id == 0, "Texture was not deleted");
    }
}