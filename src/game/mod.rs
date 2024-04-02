mod camera;
mod chunk;
mod input;
pub mod misc;
mod overlay;
mod world;
mod renderer;
mod blocks;
mod background;
mod player;

use std::net::TcpStream;
use std::sync::Arc;

use winit::dpi::PhysicalSize;

pub use camera::{Camera, FreeCamera};
pub use chunk::Chunk;
pub use chunk::CHUNK_SIZE;
pub use chunk::Y_RANGE;
pub use input::Controls;
pub use world::World;
pub use renderer::Renderer;

use crate::mygl::GLToken;

use self::background::chunk_loader;

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
    renderer: Renderer,
    background_thread: std::thread::JoinHandle<()>,
}

impl Game {
    pub fn new(glt : GLToken, render_size: PhysicalSize<u32>, tcp: TcpStream) -> Self {
        let world = World::new(glt);
        let world = Arc::new(world);

        let (update_tx, update_rx) = tokio::sync::mpsc::channel(100);

        let renderer = Renderer::new(glt, world.clone(), render_size, update_tx);
        let atlas = renderer.atlas();


        let chunk_loader_world = world.clone();
        let background_thread = std::thread::spawn(|| chunk_loader(tcp, chunk_loader_world, update_rx, atlas));


        Self { renderer , background_thread}
    }

    pub fn draw(&mut self, glt : GLToken, delta_t: f32) {
        self.renderer.draw(glt, delta_t);
    }

    pub fn resize(&mut self, glt : GLToken, size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(glt, size);
    }

    pub fn mouse_input(&mut self, delta: (f64, f64)) {
        self.renderer.mouse_input(delta);
    }

    pub fn keyboard_input(&mut self, key: Key, pressed: bool) {
        self.renderer.keyboard_input(key, pressed);
    }

    pub fn exit(self, glt : GLToken) {
        // Exit the background thread
        self.renderer.send_exit();
        self.background_thread.join().unwrap();

        unsafe {
            self.renderer.delete(glt);
        }

    }
}