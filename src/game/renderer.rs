use std::sync::Mutex;

use glm::Mat4;
use nalgebra_glm as glm;

use crate::mygl::{get_gl_string, Program, TextureAtlas};

use super::World;

const NEAR_PLAIN: f32 = 0.3;
const FAR_PLAIN: f32 = 100.0;

/// This struct holds all GL relevant things
/// All the functions have to be called from the GL thread
/// It holds a Mutex of the World to render it
pub struct Renderer {
    world: Mutex<World>,
    program: Program,
    atlas: TextureAtlas,
    projection: Mat4,
}

impl Renderer {
    pub fn new(world: Mutex<World>, render_size: winit::dpi::PhysicalSize<u32>) -> Self {
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

            let program = Program::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);

            let mut atlas = crate::mygl::TextureAtlas::new();
            atlas.add_texture("textures/grass_side.png", 0).unwrap();
            atlas.add_texture("textures/grass_top.png", 1).unwrap();
            atlas.add_texture("textures/dirt.png", 23).unwrap();
            //atlas.save("temp.png").unwrap();
            atlas.bind_texture(gl::TEXTURE0);
            atlas.finalize();

            let projection = glm::perspective(
                render_size.width as f32 / render_size.height as f32,
                0.785398,
                NEAR_PLAIN,
                FAR_PLAIN,
            );
            Self { world, program, atlas, projection }
        }
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
