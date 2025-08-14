use std::{collections::HashMap, sync::Mutex, vec};

use nalgebra_glm::{self as glm, DVec3};
use zerocopy::transmute;

use crate::{
    game::{chunk::block_position_to_chunk_index, player::Player},
    mygl::{BlockTextures, GLToken, Program, TextRenderer},
};

use super::{CHUNK_SIZE, Camera, Chunk, Y_RANGE, player::Players};

pub const VIEW_DISTANCE: i32 = 8;

/// The maximum number of chunks that can be loaded at once
const MAX_CHUNKS: usize =
    4 * (VIEW_DISTANCE as usize + 1) * (VIEW_DISTANCE as usize + 1) * 2 * Y_RANGE as usize;

pub struct World {
    pub chunks: Mutex<HashMap<[i32; 3], Chunk>>,
    pub unused_chunks: Mutex<Vec<Chunk>>,
    pub players: Mutex<Players>,
}

impl World {
    pub fn new(glt: GLToken, local_player: Player, inv_aspect_ratio: f32) -> Self {
        let mut unused_chunks = Vec::new();
        for _ in 0..MAX_CHUNKS {
            unused_chunks.push(Chunk::new_empty(glt));
        }

        let players = Players::new(glt, local_player, inv_aspect_ratio);

        Self {
            chunks: Mutex::new(HashMap::with_capacity(MAX_CHUNKS)),
            unused_chunks: Mutex::new(unused_chunks),
            players: Mutex::new(players),
        }
    }

    pub fn is_block(pos: [i32; 3], chunks: &HashMap<[i32; 3], Chunk>) -> bool {
        let (chunk_index, block_index) = block_position_to_chunk_index(pos);
        if let Some(chunk) = chunks.get(&chunk_index) {
            return chunk.blocks.get(block_index) != 0;
        }
        false
    }

    pub fn is_block_at(pos: DVec3, chunks: &HashMap<[i32; 3], Chunk>) -> bool {
        let pos = pos.map(|x| x.floor() as i32);
        Self::is_block(transmute!(pos.data.0), chunks)
    }

    pub fn game_update(&self, delta_t: f32, controls: &super::Controls) {
        let acceleration = 90.0;

        let mut players = self.players.lock().unwrap();
        // Make sure the chunks are loaded around the player

        let player_pos = players.local_player.position.map(|x| x.floor() as i32);
        let player_chunk_index = [
            player_pos[0].div_euclid(CHUNK_SIZE as i32),
            0,
            player_pos[2].div_euclid(CHUNK_SIZE as i32),
        ];
        if !self
            .chunks
            .lock()
            .unwrap()
            .contains_key(&player_chunk_index)
        {
            return; // No chunk loaded for the player
        }

        let player = &mut players.local_player;

        // Friction in x and z directions
        player.velocity[0] -= player.velocity[0] * delta_t * 10.0;
        player.velocity[2] -= player.velocity[2] * delta_t * 10.0;
        // Gravity
        player.velocity[1] -= delta_t * 50.0;

        if controls.forward {
            player.velocity += player.forward_dir() * delta_t * acceleration;
        }
        if controls.backward {
            player.velocity -= player.forward_dir() * delta_t * acceleration;
        }
        if controls.left {
            player.velocity += player.left_dir() * delta_t * acceleration;
        }
        if controls.right {
            player.velocity -= player.left_dir() * delta_t * acceleration;
        }
        if controls.up && player.on_ground {
            player.velocity[1] += delta_t * 140.0;
            player.jump_duration += delta_t;
            if player.jump_duration > 0.10 {
                player.on_ground = false; // jump is finished
                player.jump_duration = 0.0; // Reset jump duration
            }
        }
        //if controls.down {
        //    player.velocity[1] -= delta_t * acceleration;
        //}

        let bounding_box_size = player.bounding_box_size();

        let chunks = self.chunks.lock().unwrap();

        for move_direction in 0..3 {
            // Movement update in this direction
            player.position[move_direction] +=
                player.velocity[move_direction] as f64 * delta_t as f64;

            if player.velocity[move_direction] == 0.0 {
                continue; // No movement in this direction
            }

            let mut points_to_check = vec![];

            let offset = if player.velocity[move_direction] > 0.0 {
                bounding_box_size[move_direction]
            } else {
                0.0
            };

            for i in 0..2 {
                for j in 0..2 {
                    let mut point = player.position;
                    point[move_direction] += offset;
                    point[(move_direction + 1) % 3] +=
                        i as f64 * bounding_box_size[(move_direction + 1) % 3];
                    point[(move_direction + 2) % 3] +=
                        j as f64 * bounding_box_size[(move_direction + 2) % 3];
                    points_to_check.push(point);
                }
            }

            if points_to_check
                .iter()
                .any(|pos| World::is_block_at(*pos, &chunks))
            {
                // Collision detected, move the camera to the edge of the bounding box
                if player.velocity[move_direction] > 0.0 {
                    player.position[move_direction] = (player.position[move_direction]
                        + bounding_box_size[move_direction])
                        .floor()
                        - bounding_box_size[move_direction]
                        - 1e-5;
                    player.velocity[move_direction] = 0.0; // Stop the movement in this direction
                } else if player.velocity[move_direction] < 0.0 {
                    player.position[move_direction] = player.position[move_direction].ceil() + 1e-5;
                    player.velocity[move_direction] = 0.0; // Stop the movement in this direction
                    if move_direction == 1 {
                        player.on_ground = true; // If we hit the ground, we are on the ground
                    }
                }
            }
        }
    }

    pub fn draw(
        &self,
        glt: GLToken,
        program: &Program,
        projection: &nalgebra_glm::Mat4,
        camera: &impl Camera,
        text_renderer: &TextRenderer,
        block_texture: &BlockTextures
    ) {
        unsafe {
            program.bind(glt);
            block_texture.bind_texture(glt);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);

            let [x, y, z] = camera.camera_position();

            let mvp_location = gl::GetUniformLocation(program.program, c"mvp".as_ptr().cast());
            let texture_location =
                gl::GetUniformLocation(program.program, c"tex_atlas".as_ptr().cast());

            gl::Uniform1i(texture_location, 0);

            gl::Clear(gl::DEPTH_BUFFER_BIT);

            let projection_view = projection * camera.view_matrix();

            let mut chunks = self.chunks.lock().unwrap();

            for chunk in chunks.values_mut() {
                let [cx, cy, cz] = chunk.position();

                let cx = *cx as f64 * CHUNK_SIZE as f64;
                let cy = *cy as f64 * CHUNK_SIZE as f64;
                let cz = *cz as f64 * CHUNK_SIZE as f64;

                let view_direction = camera.view_direction().cast::<f64>();
                let cam_position = glm::TVec3::<f64>::from(camera.camera_position());
                let chunk_position = glm::vec3(cx, cy, cz);

                let cutoff = cam_position - view_direction * 2.0 * CHUNK_SIZE as f64;

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
                &camera.camera_position(),
                mvp_location,
                program,
                text_renderer,
            );
        }
    }

    pub fn delete(self, glt: GLToken) {
        // Delete all the active chunks
        for chunk in self.chunks.into_inner().unwrap().into_values() {
            chunk.delete(glt);
        }
        // Delete all the unused chunks
        for chunk in self.unused_chunks.into_inner().unwrap() {
            chunk.delete(glt);
        }
        self.players.into_inner().unwrap().delete(glt);
    }
}
