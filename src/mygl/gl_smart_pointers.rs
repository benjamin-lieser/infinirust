use std::ffi::c_void;
use std::marker::PhantomData;

use gl::types::GLenum;
use gl::types::GLint;
use gl::types::GLsizeiptr;
use gl::types::GLuint;

use super::GLToken;

pub trait ToGlType {
    fn to_gl_type() -> GLenum;
}

impl ToGlType for f32 {
    fn to_gl_type() -> GLenum {
        gl::FLOAT
    }
}

impl ToGlType for u8 {
    fn to_gl_type() -> GLenum {
        gl::UNSIGNED_BYTE
    }
}

impl ToGlType for i8 {
    fn to_gl_type() -> GLenum {
        gl::BYTE
    }
}
pub struct VBO<T: ToGlType> {
    id: GLuint,
    _phantom: PhantomData<T>,
    /// It is not send, because the drop function need to be called in the same thread
    /// It is sync, because all GL calls require the gl token
    _unsend: PhantomData<crate::misc::UnSend>,
}

impl<T: ToGlType> VBO<T> {
    pub fn new(_: GLToken) -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        VBO {
            id,
            _phantom: PhantomData,
            _unsend: PhantomData,
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
}

impl<T: ToGlType> Drop for VBO<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

pub struct VAO {
    id: GLuint,
    _unsend: PhantomData<crate::misc::UnSend>,
}

impl VAO {
    pub fn new(_: GLToken) -> VAO {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        VAO {
            id,
            _unsend: PhantomData,
        }
    }

    pub fn bind(&self, _: GLToken) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn attrib_pointer<T: ToGlType>(
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
}

impl Drop for VAO {
    fn drop(&mut self) {
        // This is save because VOA is not Send and can only be created in the GL thread
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}

pub struct VBOWithStorage<T: ToGlType> {
    vbo: VBO<T>,
    data: Vec<T>,
    modified: bool,
}

impl<T: ToGlType> VBOWithStorage<T> {
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
}
