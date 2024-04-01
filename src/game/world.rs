use std::{collections::HashMap, sync::Mutex};

use nalgebra_glm as glm;

use crate::mygl::{GLToken, Program};

use super::{Camera, Chunk, FreeCamera, CHUNK_SIZE, Y_RANGE};

const VIEW_DISTANCE: i32 = 8;

/// The maximum number of chunks that can be loaded at once
const MAX_CHUNKS: usize =
    4 * (VIEW_DISTANCE as usize + 1) * (VIEW_DISTANCE as usize + 1) * Y_RANGE as usize;

pub struct World {
    pub chunks: Mutex<HashMap<[i32; 3], Chunk>>,
    /// Can be used to optain an unsued chunk
    pub unused_chunks_rx: Mutex<tokio::sync::mpsc::Receiver<Chunk>>,
    /// If a chunk is unused it should be sent here
    unsued_chunks_tx: tokio::sync::mpsc::Sender<Chunk>,
    center: [i32; 3],
}

impl World {
    pub fn new(glt: GLToken) -> Self {
        let (unused_chunks_tx, unused_chunks_rx) = tokio::sync::mpsc::channel(MAX_CHUNKS);
        for _ in 0..MAX_CHUNKS {
            unused_chunks_tx
                .try_send(Chunk::new_empty(glt))
                .expect("Channel can't be full");
        }
        Self {
            chunks: Mutex::new(HashMap::new()),
            unused_chunks_rx : Mutex::new(unused_chunks_rx),
            unsued_chunks_tx : unused_chunks_tx,
            center: [0, 0, 0],
        }
    }

    pub fn update_center(&mut self, camera_pos: &[f64; 3]) {
        let camera_center = [
            camera_pos[0] as i32 / CHUNK_SIZE as i32,
            camera_pos[1] as i32 / CHUNK_SIZE as i32,
            camera_pos[2] as i32 / CHUNK_SIZE as i32,
        ];
        // x
        if self.center[0] != camera_center[0] {
            for x in (camera_center[0] - VIEW_DISTANCE)..(self.center[0] - VIEW_DISTANCE) {
                for y in -Y_RANGE..Y_RANGE {
                    for z in (self.center[2] - VIEW_DISTANCE)..(self.center[0] - VIEW_DISTANCE) {
                       
                    }
                }
            }
        }
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

            for chunk in chunks.values() {
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
        for (_, chunk) in self.chunks.into_inner().unwrap().drain() {
            chunk.delete(glt);
        }
        // Delete all the unused chunks
        let mut unused_chunks = self.unused_chunks_rx.into_inner().unwrap();
        unused_chunks.close();
        while let Some(chunk) = unused_chunks.blocking_recv() {
            chunk.delete(glt);
        }
    }
}
