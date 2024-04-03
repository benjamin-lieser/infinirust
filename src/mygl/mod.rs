mod texture_atlas;
mod gl_smart_pointers;
mod program;
mod text;

use std::ffi::CStr;
use std::marker::PhantomData;

pub use gl_smart_pointers::VBOWithStorage;
pub use gl_smart_pointers::VAO;
pub use gl_smart_pointers::VBO;
pub use texture_atlas::TextureAtlas;
pub use program::Program;

pub fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

/// Non thread safe
/// Allows calling gl functions without having to check if the context is current
#[derive(Clone, Copy)]
pub struct GLToken {
    make_unsend_and_unsync: PhantomData<* const()>,
}

impl GLToken {
    /// # Safety
    /// The GLToken must be created in the same thread as the OpenGL context
    pub unsafe fn new() -> Self {
        Self {
            make_unsend_and_unsync: PhantomData,
        }
    }
}