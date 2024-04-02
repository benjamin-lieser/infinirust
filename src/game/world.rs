use std::sync::Mutex;

use nalgebra_glm as glm;

use crate::mygl::{GLToken, Program};

use super::{Camera, Chunk, FreeCamera, CHUNK_SIZE, Y_RANGE};

const VIEW_DISTANCE: i32 = 8;

/// The maximum number of chunks that can be loaded at once
const MAX_CHUNKS: usize =
    4 * (VIEW_DISTANCE as usize + 1) * (VIEW_DISTANCE as usize + 1) * Y_RANGE as usize;

pub struct World {
    pub chunks: Mutex<Vec<Chunk>>,
    pub unused_chunks: Mutex<Vec<Chunk>>,
}

impl World {
    pub fn new(glt: GLToken) -> Self {
        let mut unused_chunks = Vec::new();
        for _ in 0..MAX_CHUNKS {
            unused_chunks.push(Chunk::new_empty(glt));
        }
        Self {
            chunks: Mutex::new(Vec::new()),
            unused_chunks : Mutex::new(unused_chunks)
        }
    }

    pub fn update_center(&mut self, camera_pos: &[f64; 3]) {
        let camera_center = [
            camera_pos[0] as i32 / CHUNK_SIZE as i32,
            camera_pos[1] as i32 / CHUNK_SIZE as i32,
            camera_pos[2] as i32 / CHUNK_SIZE as i32,
        ];
        todo!()
    }

    pub fn draw(
        &self,
        glt: GLToken,
        program: &Program,
        projection: &nalgebra_glm::Mat4,
        camera: &FreeCamera,
    ) {
        unsafe {
            program.bind(glt);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);

            let [x, y, z] = camera.position();

            let mvp_location = gl::GetUniformLocation(program.program, "mvp\0".as_ptr().cast());
            let texture_location =
                gl::GetUniformLocation(program.program, "tex_atlas\0".as_ptr().cast());

            gl::Uniform1i(texture_location, 0);

            gl::ClearColor(0.1, 0.1, 0.1, 0.9);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let projection_view = projection * camera.view_matrix();

            let chunks = self.chunks.lock().unwrap();

            for chunk in chunks.iter() {
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
                chunk.draw(glt);
            }
        }
    }

    pub fn delete(self, glt: GLToken) {
        // Delete all the active chunks
        for chunk in self.chunks.into_inner().unwrap() {
            chunk.delete(glt);
        }
        // Delete all the unused chunks
        for chunk in self.unused_chunks.into_inner().unwrap() {
            chunk.delete(glt);
        }
    }
}
