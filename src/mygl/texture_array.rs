use gl::types::{GLenum, GLsizei, GLuint};

use crate::mygl::GLToken;

pub struct TextureArray {
    id: GLuint,
}

impl TextureArray {
    pub fn new(_: GLToken, min_filter: GLenum, mag_filter: GLenum, wrapping: GLenum) -> Self {
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, id);
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MIN_FILTER,
                min_filter as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MAG_FILTER,
                mag_filter as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, wrapping as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, wrapping as i32);
        }
        Self { id }
    }

    pub fn upload_png<'a>(
        &self,
        glt: GLToken,
        pngs: impl ExactSizeIterator<Item = &'a str>,
        mipmap: GLsizei,
    ) {
        self.bind(glt);

        let mut all_width: u32 = 0;
        let mut all_height: u32 = 0;

        let len = pngs.len();

        for (i, png) in pngs.into_iter().enumerate() {
            let mut image = image::open("textures/".to_owned() + png)
                .expect("Failed to open image")
                .to_rgba8();
            image::imageops::flip_vertical_in_place(&mut image);
            let (width, height) = image.dimensions();

            if i == 0 {
                unsafe {
                    gl::TexStorage3D(
                        gl::TEXTURE_2D_ARRAY,
                        mipmap, // mipmap level
                        gl::RGBA8,
                        width as i32,
                        height as i32,
                        len as i32,
                    );
                }
                all_width = width;
                all_height = height;
            } else {
                assert_eq!(width, all_width);
                assert_eq!(height, all_height);
            }

            unsafe {
                gl::TexSubImage3D(
                    gl::TEXTURE_2D_ARRAY,
                    0, // mipmap level
                    0,
                    0,
                    i as i32,
                    width as i32,
                    height as i32,
                    1, // depth
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    image.as_raw().as_ptr().cast(),
                );
            }
        }
        unsafe {
            gl::GenerateMipmap(gl::TEXTURE_2D_ARRAY);
        }
    }

    pub fn bind(&self, _: GLToken) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.id);
        }
    }

    pub fn delete(mut self, _: GLToken) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
        self.id = 0;
    }
}
