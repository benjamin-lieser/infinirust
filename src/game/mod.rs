mod camera;
mod chunk;
mod world;
mod input;

pub use camera::{Camera, FreeCamera};
pub use chunk::Chunk;
pub use input::Controls;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ
}

#[derive(Debug, Clone, Copy)]
pub enum Key {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down
}