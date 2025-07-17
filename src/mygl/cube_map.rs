use std::path::Path;

use gl::types::GLuint;

use crate::mygl::GLToken;

pub struct CubeMap {
    id: GLuint,
}

impl CubeMap {
    pub fn new(_: GLToken, cubemap: &Path) -> Self {
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

        let image = image::open(cubemap).expect("Failed to load cube map texture");
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();

        assert!(
            width / 4 == height / 3,
            "Cube map texture must be in 4:3 aspect ratio"
        );

        let width = width / 4;
        let height = height / 3;

        for i in 0..6 {
            let sub_image = match i {
                0 => image::GenericImageView::view(&image, 2 * width, height, width, height)
                    .to_image(), // POS X
                1 => image::GenericImageView::view(&image, 0, height, width, height).to_image(), // NEG X
                2 => image::GenericImageView::view(&image, width, 0, width, height).to_image(), // POS Y
                3 => image::GenericImageView::view(&image, width, height * 2, width, height)
                    .to_image(), // NEG Y
                4 => image::GenericImageView::view(&image, width, height, width, height).to_image(), // Pos Z
                5 => image::GenericImageView::view(&image, 3 * width, height, width, height)
                    .to_image(), // Neg Z
                _ => unreachable!(),
            };

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
                    sub_image.as_raw().as_ptr() as *const _,
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
