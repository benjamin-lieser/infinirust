mod texture_atlas;
mod gl_smart_pointers;
mod program;
mod text;

use std::ffi::CStr;

pub use gl_smart_pointers::VBOWithStorage;
pub use gl_smart_pointers::VAO;
pub use gl_smart_pointers::VBO;
pub use texture_atlas::TextureAtlas;
pub use program::create_program;
pub use program::Program;

pub fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}