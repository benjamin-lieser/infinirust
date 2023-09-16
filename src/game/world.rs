use std::collections::HashMap;

use nalgebra_glm as glm;
use noise::Perlin;

use crate::mygl::TextureAtlas;

use super::{Camera, Chunk, FreeCamera, CHUNK_SIZE};

pub struct World {
    chunks: HashMap<[i32; 3], Chunk>,
    generator: Perlin,
}

impl World {
    pub fn new(atlas: &TextureAtlas) -> Self {
        let mut chunks = HashMap::new();
        let generator = Perlin::new(42);

        for x in -8..8 {
            for y in -8..8 {
                for z in -8..8 {
                    let pos = [x, y, z];
                    chunks.insert(pos, Chunk::new(pos, &generator));
                    chunks.get_mut(&pos).unwrap().write_vbo(atlas);
                }
            }
        }

        Self { chunks, generator }
    }

    pub fn draw(
        &self,
        program: gl::types::GLuint,
        projection: &nalgebra_glm::Mat4,
        camera: &FreeCamera,
    ) {
        unsafe {
            gl::UseProgram(program);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);

            let [x, y, z] = camera.position();

            let mvp_location = gl::GetUniformLocation(program, "mvp\0".as_ptr().cast());
            let texture_location = gl::GetUniformLocation(program, "tex_atlas\0".as_ptr().cast());

            gl::Uniform1i(texture_location, 0);

            gl::ClearColor(0.1, 0.1, 0.1, 0.9);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            for (_, chunk) in &self.chunks {
                let [cx, cy, cz] = chunk.position();

                let cx = *cx as f64 * CHUNK_SIZE as f64;
                let cy = *cy as f64 * CHUNK_SIZE as f64;
                let cz = *cz as f64 * CHUNK_SIZE as f64;

                let model = glm::translation(&glm::vec3(
                    (cx - x) as f32,
                    (cy - y) as f32,
                    (cz - z) as f32,
                ));
                let mvp: glm::TMat4<f32> = projection * camera.view_matrix() * model;
                gl::UniformMatrix4fv(mvp_location, 1, 0, mvp.as_ptr());
                chunk.draw();
            }
        }
    }
}
