use std::ffi::c_void;
use std::marker::PhantomData;

use gl::types::GLuint;
use gl::types::GLint;
use gl::types::GLenum;
use gl::types::GLsizeiptr;

pub struct VBO<T : ToGlType> {
    id : GLuint,
    _phantom : PhantomData<T>
}

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


impl<T : ToGlType> VBO<T> {
    pub fn new() -> Self {
        let mut id : GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        VBO {id, _phantom : PhantomData::default()}
    }
    
    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }

    pub fn copy(&mut self, data : &[T]) {
        self.bind();
        unsafe {
            gl::BufferData(gl::ARRAY_BUFFER, (data.len() * std::mem::size_of::<T>()) as GLsizeiptr, data.as_ptr().cast(), gl::STATIC_DRAW);
        }
    }
}

impl<T : ToGlType> Drop for VBO<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.id);
        }
    }
}

pub struct VAO {
    id : GLuint
}

impl VAO {
    pub fn new() -> VAO {
        let mut id : GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        VAO {id}
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn attrib_pointer<T : ToGlType>(&mut self, index : GLuint, buffer : &VBO<T>, number_components : u8, stride : usize, offset : usize, normalized : bool) {
        let data_type = T::to_gl_type();
        self.bind();
        buffer.bind();
        unsafe {
            gl::VertexAttribPointer(index, number_components as GLint, data_type, normalized as u8, stride as gl::types::GLsizei, offset as *const c_void)
        }
    }

    pub fn enable_array(&mut self, index : GLuint) {
        self.bind();
        unsafe {
            gl::EnableVertexAttribArray(index);
        }
    }
}

impl Drop for VAO {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &mut self.id);
        }
    }
}

pub struct VBOWithStorage<T : ToGlType> {
    pub vbo : VBO<T>,
    pub data : Vec<T>
}

impl<T : ToGlType> VBOWithStorage<T> {
    pub fn new() -> Self {
        VBOWithStorage { vbo: VBO::new(), data: Vec::new() }
    }

    pub fn copy(&mut self) {
        self.vbo.copy(&self.data);
    }
}