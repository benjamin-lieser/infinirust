use std::sync::Arc;

use glm::Mat4;
use nalgebra_glm as glm;

use crate::mygl::{get_gl_string, GLToken, Program, TextureAtlas};

use super::{
    background::Update, misc::CubeOutlines, overlay::Overlay, Camera, Controls, Key, World,
};

const NEAR_PLAIN: f32 = 0.2;
const FAR_PLAIN: f32 = 300.0;

/// This struct holds all GL relevant things
/// All the functions have to be called from the GL thread
/// It holds a Arc of the World to render it
pub struct Renderer {
    world: Arc<World>,
    program: Program,
    atlas: Arc<TextureAtlas>,
    projection: Mat4,
    controls: Controls,
    cube_outlines: CubeOutlines,
    overlay: Overlay,
    render_size: winit::dpi::PhysicalSize<u32>,
    updates: tokio::sync::mpsc::Sender<Update>,
    last_pos_update: std::time::Instant,
    last_block_remove_place: std::time::Instant,
}

impl Renderer {
    pub fn new(
        glt: GLToken,
        world: Arc<World>,
        atlas: Arc<TextureAtlas>,
        render_size: winit::dpi::PhysicalSize<u32>,
        updates: tokio::sync::mpsc::Sender<Update>,
    ) -> Self {
        if let Some(renderer) = get_gl_string(gl::RENDERER) {
            println!("Running on {}", renderer.to_string_lossy());
        }
        if let Some(version) = get_gl_string(gl::VERSION) {
            println!("OpenGL Version {}", version.to_string_lossy());
        }

        if let Some(shaders_version) = get_gl_string(gl::SHADING_LANGUAGE_VERSION) {
            println!("Shaders version on {}", shaders_version.to_string_lossy());
        }

        let program = Program::new(glt, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);

        let projection = glm::perspective(
            render_size.width as f32 / render_size.height as f32,
            std::f32::consts::FRAC_PI_4,
            NEAR_PLAIN,
            FAR_PLAIN,
        );
        Self {
            world,
            program,
            atlas,
            projection,
            controls: Controls::default(),
            cube_outlines: CubeOutlines::new(glt),
            overlay: Overlay::new(glt, render_size),
            render_size,
            updates,
            last_pos_update: std::time::Instant::now(),
            last_block_remove_place: std::time::Instant::now(),
        }
    }

    pub fn draw(&mut self, glt: GLToken, delta_t: f32) {
        let speed = 5.0;

        let camera = {
            let mut players = self.world.players.lock().unwrap();
            let camera = &mut players.local_player.camera;

            if self.controls.forward {
                camera.go_forward(delta_t * speed);
            }

            if self.controls.backward {
                camera.go_forward(-delta_t * speed);
            }

            if self.controls.left {
                camera.go_left(delta_t * speed);
            }

            if self.controls.right {
                camera.go_left(-delta_t * speed);
            }

            if self.controls.up {
                camera.go_up(delta_t * speed);
            }

            if self.controls.down {
                camera.go_up(-delta_t * speed);
            }
            camera.clone()
        };

        self.world
            .draw(glt, &self.program, &self.projection, &camera);

        //Update background about the current position
        //For position its ok if it gets lost, for blockupdate not to much TODO
        if self.last_pos_update.elapsed().as_secs_f32() > 0.05 {
            self.last_pos_update = std::time::Instant::now();
            _ = self.updates.try_send(Update::Pos(camera.clone()));
        }

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
            let [x, y, z] = camera.position();

            let look_pos = camera.view_direction() * (distance_to_screen_mid);

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
                .map(|(index, _)| index)
                .unwrap();

            abs_look_pos[direction] = abs_look_pos[direction].round();

            let mut look_block = abs_look_pos.map(|x| x.floor());

            look_block[direction] += if camera.view_direction()[direction] <= 0.0 {
                -1.0
            } else {
                0.0
            };

            let highlighted_block = look_block.map(|x| x as i32);

            // Remove block update if left click
            if self.controls.left_click && self.last_block_remove_place.elapsed().as_secs_f32() > 0.05{
                self.updates
                    .try_send(Update::Block(highlighted_block, 0))
                    .unwrap();
                self.last_block_remove_place = std::time::Instant::now();
            }

            let model = glm::translation(&glm::vec3(
                (look_block[0] - x) as f32,
                (look_block[1] - y) as f32,
                (look_block[2] - z) as f32,
            ));

            self.cube_outlines
                .draw(glt, &(self.projection * camera.view_matrix() * model));
        }
        self.overlay.draw(glt);
    }

    pub fn atlas(&self) -> Arc<TextureAtlas> {
        self.atlas.clone()
    }

    pub fn resize(&mut self, glt: GLToken, size: winit::dpi::PhysicalSize<u32>) {
        self.render_size = size;
        unsafe {
            gl::Viewport(0, 0, size.width as i32, size.height as i32);
        }
        self.projection = glm::perspective(
            size.width as f32 / size.height as f32,
            std::f32::consts::FRAC_PI_4,
            NEAR_PLAIN,
            FAR_PLAIN,
        );
        self.overlay.resize(glt, size);
    }

    /// Only mouse movement, clicking is handled in `keyboard_input`
    pub fn mouse_input(&mut self, delta: (f64, f64)) {
        let camera = &mut self.world.players.lock().unwrap().local_player.camera;
        camera.change_pitch(delta.1 as f32 / 100.0);
        camera.change_yaw(delta.0 as f32 / 100.0);
    }

    /// also manages clicking
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
            Key::LeftClick => {
                self.controls.left_click = pressed;
            }
            Key::RightClick => {
                self.controls.right_click = pressed;
            }
        }
    }
    /// Sends a exit signal to the background
    pub fn send_exit(&self) {
        self.updates.blocking_send(Update::Exit).unwrap();
    }

    /// # Safety
    /// This function has to be called after the exit thread has been joined
    /// Otherwise some drop glue will panic
    pub unsafe fn delete(self, glt: GLToken) {
        self.cube_outlines.delete(glt);
        self.overlay.delete(glt);
        Arc::<World>::into_inner(self.world)
            .expect("After the background therad joind this should be the only reference to world")
            .delete(glt);
        self.program.delete(glt);
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
