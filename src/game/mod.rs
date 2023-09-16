mod camera;
mod chunk;
mod input;
mod world;

pub use camera::{Camera, FreeCamera};
pub use chunk::Chunk;
pub use chunk::CHUNK_SIZE;
pub use input::Controls;
pub use world::World;

#[derive(Debug, Clone, Copy)]
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
