mod camera;
mod chunk;
mod input;
pub mod misc;
mod overlay;
mod world;
mod renderer;
mod blocks;

use std::ffi::CStr;
use std::net::TcpStream;

use glm::Mat4;
use nalgebra_glm as glm;
use serde::Deserialize;
use winit::dpi::PhysicalSize;

pub use camera::{Camera, FreeCamera};
pub use chunk::Chunk;
pub use chunk::CHUNK_SIZE;
pub use chunk::Y_RANGE;
pub use input::Controls;
pub use world::World;
pub use renderer::Renderer;

use self::misc::CubeOutlines;
use self::overlay::Overlay;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[repr(u8)]
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
    render_size: PhysicalSize<u32>,
    projection: Mat4,
    cube_outlines: CubeOutlines,
    overlay: Overlay,
}

const NEAR_PLAIN: f32 = 0.3;
const FAR_PLAIN: f32 = 100.0;

impl Game {
    pub fn new(render_size: PhysicalSize<u32>, tcp: TcpStream) -> Self {
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

            let world = World::new(&atlas, tcp);

            let projection = glm::perspective(
                render_size.width as f32 / render_size.height as f32,
                0.785398,
                NEAR_PLAIN,
                FAR_PLAIN,
            );

            Self {
                program,
                world,
                camera: FreeCamera::new([0.0, 0.0, 0.0]),
                render_size,
                controls: Controls::default(),
                projection,
                cube_outlines: CubeOutlines::new(),
                overlay: Overlay::new(render_size),
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

        self.world
            .draw(self.program, &self.projection, &self.camera);

        let distance_to_screen_mid = unsafe {
            let mut depth: f32 = 0.0;
            gl::ReadPixels(
                self.render_size.width as i32 / 2,
                self.render_size.height as i32 / 2,
                1,
                1,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                (&mut depth as *mut f32).cast(),
            );
            let ndc = depth * 2.0 - 1.0;
            (2.0 * NEAR_PLAIN * FAR_PLAIN)
                / (FAR_PLAIN + NEAR_PLAIN - ndc * (FAR_PLAIN - NEAR_PLAIN))
        };

        if distance_to_screen_mid <= 10.0 {

            let [x,y,z] = self.camera.position();

            let look_pos = self.camera.view_direction() * (distance_to_screen_mid);

            let mut abs_look_pos = [
                look_pos.x as f64 + x,
                look_pos.y as f64 + y,
                look_pos.z as f64 + z,
            ];
            
            let diff_to_int = abs_look_pos.map(|x| (x.round() - x).abs());

            let direction = diff_to_int
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.total_cmp(b))
                .map(|(index, _)| index).unwrap();

            
            abs_look_pos[direction] = abs_look_pos[direction].round();

            let mut look_block = abs_look_pos.map(|x| x.floor());

            look_block[direction] += if self.camera.view_direction()[direction] <= 0.0 { -1.0 } else { 0.0 };

            println!(
                "{},{},{},{}",
                self.camera.view_direction(),
                look_pos.x as f64 + x,
                look_pos.y as f64 + y,
                look_pos.z as f64 + z
            );

            let model = glm::translation(&glm::vec3(
                (look_block[0] - x) as f32,
                (look_block[1] - y) as f32,
                (look_block[2] - z) as f32,
            ));

            self.cube_outlines
                .draw(&(self.projection * self.camera.view_matrix() * model));
        }

        self.overlay.draw();
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.render_size = size;
        unsafe {
            gl::Viewport(0, 0, size.width as i32, size.height as i32);
        }
        self.projection = glm::perspective(
            size.width as f32 / size.height as f32,
            0.785398,
            NEAR_PLAIN,
            FAR_PLAIN,
        );
        self.overlay.resize(size);
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



