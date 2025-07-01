mod background;
mod blocks;
mod camera;
pub mod chunk;
mod input;
pub mod misc;
mod overlay;
mod player;
mod renderer;
mod world;

use std::net::TcpStream;
use std::sync::Arc;

use winit::dpi::PhysicalSize;

pub use camera::{Camera, FreeCamera};
pub use chunk::Chunk;
pub use chunk::CHUNK_SIZE;
pub use chunk::Y_RANGE;
pub use input::Controls;
pub use renderer::Renderer;
pub use world::World;

use crate::mygl::GLToken;
use crate::mygl::TextureAtlas;
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
}

pub type ChunkIndex = [i32; 3];
pub type LocalBlockIndex = [u8; 3];
pub type BlockType = u8;

pub struct Game {
    renderer: Renderer,
    background_thread: std::thread::JoinHandle<()>,
}

fn create_atlas(glt: GLToken) -> TextureAtlas {
    let mut atlas = crate::mygl::TextureAtlas::new(glt, 128);
    atlas.add_texture("grass_side.png").unwrap();
    atlas.add_texture("grass_top.png").unwrap();
    atlas.add_texture("dirt.png").unwrap();
    atlas.add_texture("head.png").unwrap();
    atlas.add_texture("face.png").unwrap();
    //atlas.save("temp.png").unwrap();
    atlas.bind_texture(gl::TEXTURE0);
    unsafe {
        atlas.finalize();
    }
    atlas
}

impl Game {
    pub fn new(
        glt: GLToken,
        render_size: PhysicalSize<u32>,
        tcp: TcpStream,
        uid: UID,
        name: String,
    ) -> Self {
        let atlas = create_atlas(glt);

        let local_player = Player {
            name,
            uid,
            camera: FreeCamera::new([0.0, 0.0, 0.0]),
        };

        let world = World::new(glt, &atlas, local_player);
        let world = Arc::new(world);

        let (update_tx, update_rx) = tokio::sync::mpsc::channel(100);

        let renderer = Renderer::new(glt, world.clone(), Arc::new(atlas), render_size, update_tx);
        let atlas = renderer.atlas();

        let chunk_loader_world = world.clone();
        let background_thread = std::thread::spawn(move || {
            background_thread(tcp, chunk_loader_world, update_rx, atlas, uid)
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
