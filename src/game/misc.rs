use std::ffi::CStr;

use nalgebra_glm::Mat4;

use crate::mygl::{create_program, GLToken, Program, VAO, VBO};

/// draws the outlines of one cube
pub struct CubeOutlines {
    vao: VAO,
    vbo: VBO<f32>,
    program: Program,
}

#[rustfmt::skip]
impl CubeOutlines {
    pub fn new(glt : GLToken) -> Self {
        let mut vao = VAO::new(glt);
        let mut vbo = VBO::new(glt);

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

        vbo.copy(glt, &lines);
        vao.attrib_pointer(glt, 0, &vbo, 3, 0, 0, false);
        vao.enable_array(glt, 0);

        let program = Program::new(glt, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);

        CubeOutlines { vao, vbo, program }
    }

    pub fn draw(&self, glt : GLToken, mvp : &Mat4) {
        unsafe {
            self.program.bind(glt);
            self.program.uniform_mat4(glt, CStr::from_bytes_with_nul(b"mvp\0").unwrap(), mvp);
            self.vao.bind(glt);
            gl::Enable(gl::DEPTH_TEST);
            gl::DrawArrays(gl::LINES, 0, 24);
        }
    }

    pub fn delete(self, glt : GLToken) {
        self.vao.delete(glt);
        self.vbo.delete(glt);
        self.program.delete(glt);
    }
}

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 410 core
precision highp float;

layout(location=0) in vec3 position;

uniform mat4 mvp;

void main() {
    gl_Position = mvp * vec4(position, 1.0);
    gl_Position.z -= 1e-4;
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