mod background;
mod blocks;
mod camera;
pub mod chunk;
mod input;
pub mod misc;
mod overlay;
mod player;
mod renderer;
mod skybox;
mod world;

use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;

use nalgebra_glm::DVec3;
use nalgebra_glm::Vec3;
use winit::dpi::PhysicalSize;

pub use camera::{Camera, FreeCamera};
pub use chunk::CHUNK_SIZE;
pub use chunk::Chunk;
pub use chunk::Y_RANGE;
pub use input::Controls;
pub use renderer::Renderer;
pub use world::World;

use crate::game::blocks::BlocksConfig;
use crate::mygl::BlockTextures;
use crate::mygl::GLToken;
use crate::server::UID;

use self::background::background_thread;
use self::player::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

/// Represents an input for the game. These are abstracted from the actual input handling and could for example be key combinations or mouse clicks.
#[derive(Debug, Clone, Copy)]
pub enum Key {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
    LeftClick,
    RightClick,
    DebugScreen,
}

pub type ChunkIndex = [i32; 3];
pub type LocalBlockIndex = [u8; 3];
pub type BlockType = u8;

/// Represents the game state in the client.
/// Renderer holds everything needed in the game loop.
/// The background thread handles the network communication and other asynchronous tasks.
pub struct Game {
    renderer: Renderer,
    background_thread: std::thread::JoinHandle<()>,
}

fn create_block_texture(glt: GLToken) -> (BlocksConfig, BlockTextures) {
    let (blocks_config, texture_files) = BlocksConfig::new(Path::new("config/blocks.json"));
    let block_textures = BlockTextures::new(
        glt,
        texture_files
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .as_slice(),
    );

    (blocks_config, block_textures)
}

impl Game {
    pub fn new(
        glt: GLToken,
        render_size: PhysicalSize<u32>,
        tcp: TcpStream,
        uid: UID,
        name: String,
    ) -> Self {
        let (blocks_config, block_textures) = create_block_texture(glt);

        let local_player = Player {
            name,
            uid,
            position: DVec3::new(0.0, 0.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::zeros(),
            on_ground: false,
            jump_duration: 0.0,
        };

        let world = World::new(glt, local_player);
        let world = Arc::new(world);

        let (update_tx, update_rx) = tokio::sync::mpsc::channel(100);

        let renderer = Renderer::new(glt, world.clone(), block_textures, render_size, update_tx);

        let chunk_loader_world = world.clone();
        let background_thread = std::thread::spawn(move || {
            background_thread(
                tcp,
                chunk_loader_world,
                update_rx,
                Arc::new(blocks_config),
                uid,
            )
        });

        Self {
            renderer,
            background_thread,
        }
    }

    pub fn draw(&mut self, glt: GLToken, delta_t: f32) {
        self.renderer.draw(glt, delta_t);
    }

    pub fn resize(&mut self, glt: GLToken, size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(glt, size);
    }

    pub fn mouse_input(&mut self, delta: (f64, f64)) {
        self.renderer.mouse_input(delta);
    }

    pub fn keyboard_input(&mut self, key: Key, pressed: bool) {
        self.renderer.keyboard_input(key, pressed);
    }

    pub fn exit(self, glt: GLToken) {
        // Exit the background thread
        self.renderer.send_exit();
        self.background_thread.join().unwrap();

        unsafe {
            self.renderer.delete(glt);
        }
    }
}
