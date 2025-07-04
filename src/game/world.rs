use std::sync::Mutex;

use nalgebra_glm as glm;

use crate::{
    game::player::{self, Player},
    mygl::{GLToken, Program, TextureAtlas},
};

use super::{player::Players, Camera, Chunk, FreeCamera, CHUNK_SIZE, Y_RANGE};

pub const VIEW_DISTANCE: i32 = 8;

/// The maximum number of chunks that can be loaded at once
const MAX_CHUNKS: usize =
    4 * (VIEW_DISTANCE as usize + 1) * (VIEW_DISTANCE as usize + 1) * 2 * Y_RANGE as usize;

pub struct World {
    /// The indicies have to be stable, therefore we have the Option, the manage_world function can efficintly index into the chunks
    pub chunks: Mutex<Vec<Option<Chunk>>>,
    pub unused_chunks: Mutex<Vec<Chunk>>,
    pub players: Mutex<Players>,
}

impl World {
    pub fn new(glt: GLToken, texture_atlas: &TextureAtlas, local_player: Player) -> Self {
        let mut unused_chunks = Vec::new();
        for _ in 0..MAX_CHUNKS {
            unused_chunks.push(Chunk::new_empty(glt));
        }
        let mut chunks = Vec::new();
        for _ in 0..MAX_CHUNKS {
            chunks.push(None);
        }
        Self {
            chunks: Mutex::new(chunks),
            unused_chunks: Mutex::new(unused_chunks),
            players: Mutex::new(Players::new(glt, texture_atlas, local_player)),
        }
    }

    pub fn game_update(&self, delta_t: f32, controls: &super::Controls) {
        let speed = 5.0;

        let mut players = self.players.lock().unwrap();
        
        let player = &mut players.local_player;

        if controls.forward {
            player.velocity += player.camera.forward_dir() * delta_t * speed;
        }
        if controls.backward {
            player.velocity -= player.camera.forward_dir() * delta_t * speed;
        }
        if controls.left {
            player.velocity += player.camera.left_dir() * delta_t * speed;
        }
        if controls.right {
            player.velocity -= player.camera.left_dir() * delta_t * speed;
        }

        // Collision detection
        let bounding_box_pos = player.bounding_box_pos();
        let bounding_box_size = player.bounding_box_size();

        let [x, y, z] = bounding_box_pos;
        let [bx, by, bz] = bounding_box_size;

        let camera = &mut player.camera;

        camera.pos = [
            camera.position()[0] + (player.velocity[0] * delta_t) as f64,
            camera.position()[1] + (player.velocity[1] * delta_t) as f64,
            camera.position()[2] + (player.velocity[2] * delta_t) as f64,
        ];

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

            let mut chunks = self.chunks.lock().unwrap();

            for chunk in chunks.iter_mut().flatten() {
                let [cx, cy, cz] = chunk.position();

                let cx = *cx as f64 * CHUNK_SIZE as f64;
                let cy = *cy as f64 * CHUNK_SIZE as f64;
                let cz = *cz as f64 * CHUNK_SIZE as f64;

                let view_direction = camera.view_direction().cast::<f64>();
                let cam_position = glm::TVec3::<f64>::from(camera.position());
                let chunk_position = glm::vec3(cx, cy, cz);

                let cutoff = cam_position - view_direction * CHUNK_SIZE as f64;

                if glm::dot(&(chunk_position - cutoff), &view_direction) < 0.0 {
                    continue;
                }

                let model = glm::translation(&glm::vec3(
                    (cx - x) as f32,
                    (cy - y) as f32,
                    (cz - z) as f32,
                ));
                let mvp: glm::Mat4 = projection_view * model;
                gl::UniformMatrix4fv(mvp_location, 1, 0, mvp.as_ptr());
                chunk.draw(glt);
            }

            self.players.lock().unwrap().draw(
                glt,
                &projection_view,
                &camera.position(),
                mvp_location,
            );
        }
    }

    pub fn delete(self, glt: GLToken) {
        // Delete all the active chunks
        for chunk in self.chunks.into_inner().unwrap() {
            if let Some(chunk) = chunk {
                chunk.delete(glt);
            }
        }
        // Delete all the unused chunks
        for chunk in self.unused_chunks.into_inner().unwrap() {
            chunk.delete(glt);
        }
        self.players.into_inner().unwrap().delete(glt);
    }
}
