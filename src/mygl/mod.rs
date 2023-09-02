use std::ffi::CStr;

/// Will panic if the shader source does not compile
pub fn create_and_compile_shader(shader_type: gl::types::GLenum, source: &CStr) -> gl::types::GLuint {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        if shader == 0 {
            panic!("Could not create a shader obejct");
        }
        gl::ShaderSource(shader, 1, [source.as_ptr()].as_ptr(), std::ptr::null());
        gl::CompileShader(shader);
    
        //check for compile errors
        let mut status : gl::types::GLint = 0;
        let mut error_length : gl::types::GLsizei = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        if status == 0 {
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut error_length);
            let mut buffer = vec![0 as u8; error_length as usize];
            gl::GetShaderInfoLog(shader, error_length, std::ptr::null_mut(), buffer.as_mut_ptr().cast());
            println!("Shader Compile Error: {}", std::str::from_utf8(&buffer).unwrap());
            panic!();
        }
        shader
    }
}

pub fn create_program(vertex_source : &CStr, fragment_source : &CStr) -> gl::types::GLuint {
    unsafe {
        let program = gl::CreateProgram();
        let vertex_shader = create_and_compile_shader(gl::VERTEX_SHADER, vertex_source);
        let fragment_shader = create_and_compile_shader(gl::FRAGMENT_SHADER, fragment_source);

        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);

        gl::LinkProgram(program);

        //check for link errors
        let mut status : gl::types::GLint = 0;
        let mut error_length : gl::types::GLsizei = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        if status == 0 {
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut error_length);
            let mut buffer = vec![0 as u8; error_length as usize];
            gl::GetProgramInfoLog(program, error_length, std::ptr::null_mut(), buffer.as_mut_ptr().cast());
            println!("Shader Linking Error: {}", std::str::from_utf8(&buffer).unwrap());
            panic!();
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        

        program
    }
}