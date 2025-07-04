use std::ffi::c_void;
use std::marker::PhantomData;

use gl::types::GLenum;
use gl::types::GLint;
use gl::types::GLsizeiptr;
use gl::types::GLuint;

use super::GLToken;

pub trait GLType {
    fn to_gl_type() -> GLenum;
}

impl GLType for f32 {
    fn to_gl_type() -> GLenum {
        gl::FLOAT
    }
}

impl GLType for u8 {
    fn to_gl_type() -> GLenum {
        gl::UNSIGNED_BYTE
    }
}

impl GLType for i8 {
    fn to_gl_type() -> GLenum {
        gl::BYTE
    }
}
pub struct VBO<T: GLType> {
    id: GLuint,
    _phantom: PhantomData<T>,
}

impl<T: GLType> VBO<T> {
    pub fn new(_: GLToken) -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        VBO {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn bind(&self, _: GLToken) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }

    pub fn copy(&mut self, glt: GLToken, data: &[T]) {
        self.bind(glt);
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(data) as GLsizeiptr,
                data.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
        }
    }

    pub fn delete(mut self, _: GLToken) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
        // This marks the struct as safe to drop
        self.id = 0;
    }
}

impl<T: GLType> Drop for VBO<T> {
    fn drop(&mut self) {
        if self.id != 0 {
            panic!("VBO was not deleted before drop");
        }
    }
}

pub struct VAO {
    id: GLuint,
}

impl VAO {
    pub fn new(_: GLToken) -> VAO {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        VAO { id }
    }

    pub fn bind(&self, _: GLToken) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn attrib_pointer<T: GLType>(
        &mut self,
        glt: GLToken,
        index: GLuint,
        buffer: &VBO<T>,
        number_components: u8,
        stride: usize,
        offset: usize,
        normalized: bool,
    ) {
        let data_type = T::to_gl_type();
        self.bind(glt);
        buffer.bind(glt);
        unsafe {
            gl::VertexAttribPointer(
                index,
                number_components as GLint,
                data_type,
                normalized as u8,
                stride as gl::types::GLsizei,
                offset as *const c_void,
            )
        }
    }

    pub fn enable_array(&mut self, glt: GLToken, index: GLuint) {
        self.bind(glt);
        unsafe {
            gl::EnableVertexAttribArray(index);
        }
    }

    pub fn delete(mut self, _: GLToken) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
        // This marks the struct as safe to drop
        self.id = 0;
    }
}

impl Drop for VAO {
    fn drop(&mut self) {
        if self.id != 0 {
            panic!("VAO was not deleted before drop");
        }
    }
}

pub struct VBOWithStorage<T: GLType> {
    vbo: VBO<T>,
    data: Vec<T>,
    modified: bool,
}

impl<T: GLType> VBOWithStorage<T> {
    pub fn new(glt: GLToken) -> Self {
        VBOWithStorage {
            vbo: VBO::new(glt),
            data: Vec::new(),
            modified: false,
        }
    }

    /// Copies the content of the CPU buffer to the GPU buffer if modified
    /// This is supposed to be called in the by the main game loop every iteration
    pub fn copy(&mut self, glt: GLToken) {
        if self.modified {
            self.vbo.copy(glt, &self.data);
            self.modified = false;
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn vbo(&self) -> &VBO<T> {
        &self.vbo
    }

    pub fn exchange_cpu_buffer(&mut self, buffer: Vec<T>) {
        self.data = buffer;
        self.modified = true;
    }

    pub fn delete(self, glt: GLToken) {
        self.vbo.delete(glt);
    }
}
