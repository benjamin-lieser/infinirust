use std::ffi::CStr;

use nalgebra_glm::Mat4;

use crate::mygl::{VAO, VBO, create_program};

/// draws the outlines of one cube
pub struct CubeOutlines {
    vao: VAO,
    #[allow(dead_code)]
    vbo: VBO<f32>,
    program: gl::types::GLuint
}

#[rustfmt::skip]
impl CubeOutlines {
    pub fn new() -> Self {
        let mut vao = VAO::new();
        let mut vbo = VBO::new();

        let lines = [
            // front face
            0.0, 0.0, 0.0,
            1.0, 0.0, 0.0,

            1.0, 0.0, 0.0,
            1.0, 1.0, 0.0,

            1.0, 1.0, 0.0,
            0.0, 1.0, 0.0,

            0.0, 1.0, 0.0,
            0.0, 0.0, 0.0,
            // back face
            0.0, 0.0, 1.0,
            1.0, 0.0, 1.0,

            1.0, 0.0, 1.0,
            1.0, 1.0, 1.0,

            1.0, 1.0, 1.0,
            0.0, 1.0, 1.0,

            0.0, 1.0, 1.0,
            0.0, 0.0, 1.0,
            // connections
            0.0, 0.0, 0.0,
            0.0, 0.0, 1.0,

            1.0, 0.0, 0.0,
            1.0, 0.0, 1.0,
            
            1.0, 1.0, 0.0,
            1.0, 1.0, 1.0,
            
            0.0, 1.0, 0.0,
            0.0, 1.0, 1.0
        ];

        vbo.copy(&lines);
        vao.attrib_pointer(0, &vbo, 3, 0, 0, false);
        vao.enable_array(0);

        let program = create_program(CStr::from_bytes_with_nul(VERTEX_SHADER_SOURCE).unwrap(), CStr::from_bytes_with_nul(FRAGMENT_SHADER_SOURCE).unwrap());

        CubeOutlines { vao, vbo, program }
    }

    pub fn draw(&self, mvp : &Mat4) {
        unsafe {
            gl::UseProgram(self.program);
            let mvp_location = gl::GetUniformLocation(self.program, "mvp\0".as_ptr().cast());
            self.vao.bind();
            gl::Disable(gl::DEPTH_TEST);
            gl::UniformMatrix4fv(mvp_location, 1, 0, mvp.as_ptr());
            gl::DrawArrays(gl::LINES, 0, 24);
        }
    }
}

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 410 core
precision highp float;

layout(location=0) in vec3 position;

uniform mat4 mvp;

void main() {
    gl_Position = mvp * vec4(position, 1.0);
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 410 core
precision highp float;

layout(location=0) out vec4 fragColor;

void main() {
    fragColor = vec4(1.0,1.0,1.0,0.5);
}
\0";