use winit::dpi::PhysicalSize;

use crate::mygl::{Program, VBO, VAO};

pub struct Overlay {
    program : Program,
    vbo : VBO<f32>,
    vao : VAO
}

const CROSSHAIR_SIZE : f32 = 0.02;

impl Overlay {
    pub fn new(render_size : PhysicalSize<u32>) -> Self {
        let program = Program::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
        let mut vbo = VBO::new();
        let mut vao = VAO::new();

        let aspect = render_size.width as f32 / render_size.height as f32;

        let data = [0.0 , CROSSHAIR_SIZE * aspect, 0.0, -CROSSHAIR_SIZE * aspect, -CROSSHAIR_SIZE, 0.0, CROSSHAIR_SIZE, 0.0];
        vbo.copy(&data);
        vao.attrib_pointer(0, &vbo, 2, 0, 0, false);
        vao.enable_array(0);

        Self { program, vbo, vao }
    }

    pub fn resize(&mut self, render_size : PhysicalSize<u32>) {
        let aspect = render_size.width as f32 / render_size.height as f32;

        let data = [0.0 , CROSSHAIR_SIZE * aspect, 0.0, -CROSSHAIR_SIZE * aspect, -CROSSHAIR_SIZE, 0.0, CROSSHAIR_SIZE, 0.0];
        self.vbo.copy(&data);
    }

    pub fn draw(&self) {
        self.program.bind();
        self.vao.bind();
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::DrawArrays(gl::LINES, 0, 4);
        }
    }
}

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 410 core
precision highp float;

layout(location=0) in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 410 core
precision highp float;

layout(location=0) out vec4 fragColor;

void main() {
    fragColor = vec4(0.8,0.8,0.8,1.0);
}
\0";