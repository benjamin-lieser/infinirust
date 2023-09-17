mod camera;
mod chunk;
mod input;
mod world;

use std::ffi::CStr;

use nalgebra_glm as glm;

pub use camera::{Camera, FreeCamera};
pub use chunk::Chunk;
pub use chunk::CHUNK_SIZE;
pub use input::Controls;
pub use world::World;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

#[derive(Debug, Clone, Copy)]
pub enum Key {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}

pub struct Game {
    program: gl::types::GLuint,
    world: World,
    camera: FreeCamera,
    controls: Controls,
    aspect: Option<f32>,
}

impl Game {
    pub fn new() -> Self {
        unsafe {

            if let Some(renderer) = get_gl_string(gl::RENDERER) {
                println!("Running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = get_gl_string(gl::VERSION) {
                println!("OpenGL Version {}", version.to_string_lossy());
            }

            if let Some(shaders_version) = get_gl_string(gl::SHADING_LANGUAGE_VERSION) {
                println!("Shaders version on {}", shaders_version.to_string_lossy());
            }

            let program = crate::mygl::create_program(
                CStr::from_bytes_with_nul(VERTEX_SHADER_SOURCE).unwrap(),
                CStr::from_bytes_with_nul(FRAGMENT_SHADER_SOURCE).unwrap(),
            );


            let mut atlas = crate::mygl::TextureAtlas::new();
            atlas.add_texture("textures/grass_side.png", 0).unwrap();
            atlas.add_texture("textures/grass_top.png", 1).unwrap();
            atlas.add_texture("textures/dirt.png", 23).unwrap();
            //atlas.save("temp.png").unwrap();
            atlas.bind_texture(gl::TEXTURE0);
            atlas.finalize();

            let world = World::new(&atlas);

            Self {
                program,
                world,
                camera: FreeCamera::new([0.0, 0.0, 0.0]),
                aspect: None,
                controls: Controls::default(),
            }
        }
    }

    pub fn draw(&mut self, delta_t: f32) {
        let speed = 35.0;

        if self.controls.forward {
            self.camera.go_forward(delta_t * speed);
        }

        if self.controls.backward {
            self.camera.go_forward(-delta_t * speed);
        }

        if self.controls.left {
            self.camera.go_left(delta_t * speed);
        }

        if self.controls.right {
            self.camera.go_left(-delta_t * speed);
        }

        if self.controls.up {
            self.camera.go_up(delta_t * speed);
        }

        if self.controls.down {
            self.camera.go_up(-delta_t * speed);
        }

        let projection = glm::perspective(self.aspect.unwrap(), 0.785398, 0.5, 300.0);

        self.world.draw(self.program, &projection, &self.camera)
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.aspect = Some(width as f32 / height as f32);
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }

    pub fn mouse_input(&mut self, delta: (f64, f64)) {
        self.camera.change_pitch(delta.1 as f32 / 100.0);
        self.camera.change_yaw(delta.0 as f32 / 100.0);
    }

    pub fn keyboard_input(&mut self, key: Key, pressed: bool) {
        match key {
            Key::Backward => {
                self.controls.backward = pressed;
            }
            Key::Down => {
                self.controls.down = pressed;
            }
            Key::Forward => {
                self.controls.forward = pressed;
            }
            Key::Left => {
                self.controls.left = pressed;
            }
            Key::Right => {
                self.controls.right = pressed;
            }
            Key::Up => {
                self.controls.up = pressed;
            }
        }
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}

fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 410 core
precision highp float;

layout(location=0) in vec3 position;
layout(location=1) in vec2 tex;

uniform mat4 mvp;

out vec2 texCord;

void main() {
    gl_Position = mvp * vec4(position, 1.0);
    texCord = tex;
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 410 core
precision highp float;

uniform sampler2D tex_atlas;

layout(location=0) out vec4 fragColor;

in vec2 texCord;

void main() {
    fragColor = texture(tex_atlas, texCord);
}
\0";