use std::{collections::HashMap, io::Write};

use nalgebra_glm as glm;
use noise::Perlin;

use crate::mygl::TextureAtlas;

use super::{Camera, Chunk, FreeCamera, CHUNK_SIZE, chunk::ChunkData};

pub struct ServerWorld {
    generator : Perlin,
    loaded_chunks : HashMap<[i32; 3], ChunkData>
}

impl ServerWorld {
    pub fn new(seed : u32) -> Self {
        ServerWorld { generator: Perlin::new(seed), loaded_chunks: HashMap::new() }
    }
    pub fn write_chunk(&mut self, pos : &[i32;3], writer : &mut impl Write) {
        if let Some(chunk) = self.loaded_chunks.get(pos) {
            chunk.write_to(writer);
        } else {
            let new_chunk = ChunkData::generate(&self.generator, pos);
            new_chunk.write_to(writer);
            self.loaded_chunks.insert(*pos, new_chunk);
        }
        println!("write chunk {:?}", pos);
    }
}

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

            let projection_view =projection * camera.view_matrix();

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
                let mvp: glm::Mat4 = projection_view * model;
                gl::UniformMatrix4fv(mvp_location, 1, 0, mvp.as_ptr());
                chunk.draw();
            }
        }
    }
}
