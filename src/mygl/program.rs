use std::ffi::CStr;

use nalgebra_glm::Mat4;

use super::GLToken;

pub struct Program {
    pub program: gl::types::GLuint,
}

impl Program {
    pub fn new(glt: GLToken, vertex_source: &[u8], fragment_source: &[u8]) -> Self {
        let program = create_program(
            glt,
            CStr::from_bytes_with_nul(vertex_source).unwrap(),
            CStr::from_bytes_with_nul(fragment_source).unwrap(),
        );

        Self { program }
    }

    pub fn bind(&self, _: GLToken) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }

    pub fn uniform_mat4(&self, _: GLToken, name: &CStr, mat: &Mat4) {
        unsafe {
            let location = gl::GetUniformLocation(self.program, name.as_ptr());
            gl::UniformMatrix4fv(location, 1, 0, mat.as_ptr());
        }
    }

    pub fn delete(mut self, _: GLToken) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
        self.program = 0;
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        if self.program != 0 {
            panic!("Program was not deleted before being dropped");
        }
    }
}

/// Will panic if the shader source does not compile
fn create_and_compile_shader(
    _: GLToken,
    shader_type: gl::types::GLenum,
    source: &CStr,
) -> gl::types::GLuint {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        if shader == 0 {
            panic!("Could not create a shader obejct");
        }
        gl::ShaderSource(shader, 1, [source.as_ptr()].as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        //check for compile errors
        let mut status: gl::types::GLint = 0;
        let mut error_length: gl::types::GLsizei = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        if status == 0 {
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut error_length);
            let mut buffer = vec![0u8; error_length as usize];
            gl::GetShaderInfoLog(
                shader,
                error_length,
                std::ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
            );
            println!(
                "Shader Compile Error: {}",
                std::str::from_utf8(&buffer).unwrap()
            );
            panic!();
        }
        shader
    }
}

/// Panics if there are shader compiling or linking errors
fn create_program(
    glt: GLToken,
    vertex_source: &CStr,
    fragment_source: &CStr,
) -> gl::types::GLuint {
    unsafe {
        let program = gl::CreateProgram();
        let vertex_shader = create_and_compile_shader(glt, gl::VERTEX_SHADER, vertex_source);
        let fragment_shader = create_and_compile_shader(glt, gl::FRAGMENT_SHADER, fragment_source);

        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);

        gl::LinkProgram(program);

        //check for link errors
        let mut status: gl::types::GLint = 0;
        let mut error_length: gl::types::GLsizei = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        if status == 0 {
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut error_length);
            let mut buffer = vec![0u8; error_length as usize];
            gl::GetProgramInfoLog(
                program,
                error_length,
                std::ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
            );
            println!(
                "Shader Linking Error: {}",
                std::str::from_utf8(&buffer).unwrap()
            );
            panic!();
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        program
    }
}
