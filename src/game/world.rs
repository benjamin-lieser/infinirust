use std::{collections::HashMap, net::TcpStream, io::Write};

use nalgebra_glm as glm;


use crate::{mygl::TextureAtlas, misc::cast_bytes};

use super::{Camera, Chunk, FreeCamera, CHUNK_SIZE, Y_RANGE};

const VIEW_DISTANCE : i32 = 8;



pub struct World {
    chunks : HashMap<[i32; 3], Chunk>,
    center : [i32;3],
}

impl World {
    pub fn new(atlas: &TextureAtlas, mut server : TcpStream) -> Self {
        let mut chunks = HashMap::new();

        for x in -VIEW_DISTANCE..VIEW_DISTANCE {
            for y in -Y_RANGE..Y_RANGE {
                for z in -VIEW_DISTANCE..VIEW_DISTANCE {
                    let pos: [i32;3] = [x, y, z];
                    server.write_all(b"\x0A\x00").unwrap();
                    server.write_all(cast_bytes(&pos)).unwrap();
                }
            }
        }

        

        Self { chunks, center : [0,0,0], server }
    }

    pub fn update_center(&mut self, camera_pos : &[f64;3]) {
        let camera_center = [camera_pos[0] as i32 / CHUNK_SIZE as i32, camera_pos[1] as i32 / CHUNK_SIZE as i32, camera_pos[2] as i32 / CHUNK_SIZE as i32];
        // x
        if self.center[0] != camera_center[0] {
            for x in (camera_center[0] - VIEW_DISTANCE)..(self.center[0] - VIEW_DISTANCE) {

                for y in -Y_RANGE..Y_RANGE {
                    for z in (self.center[2] - VIEW_DISTANCE)..(self.center[0] - VIEW_DISTANCE) {
                        let mut chunk = self.chunks.remove(&[x,y,z]).unwrap();
                    }
                }
            }
        }
        todo!()
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
