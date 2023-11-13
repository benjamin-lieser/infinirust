mod camera;
mod chunk;
mod input;
pub mod misc;
mod overlay;
mod world;
mod renderer;
mod blocks;
mod background;

use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;

use serde::Deserialize;
use winit::dpi::PhysicalSize;

pub use camera::{Camera, FreeCamera};
pub use chunk::Chunk;
pub use chunk::CHUNK_SIZE;
pub use chunk::Y_RANGE;
pub use input::Controls;
pub use world::World;
pub use renderer::Renderer;

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
    renderer: Renderer,
}

impl Game {
    pub fn new(render_size: PhysicalSize<u32>, tcp: TcpStream) -> Self {
        let world = World::new();
        let world = Arc::new(Mutex::new(world));

        let renderer = Renderer::new(world.clone(), render_size);

        


        Self { renderer }
    }

    pub fn draw(&mut self, delta_t: f32) {
        self.renderer.draw(delta_t);
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(size);
    }

    pub fn mouse_input(&mut self, delta: (f64, f64)) {
        self.renderer.mouse_input(delta);
    }

    pub fn keyboard_input(&mut self, key: Key, pressed: bool) {
        self.renderer.keyboard_input(key, pressed);
    }
}