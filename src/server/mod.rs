use std::sync::Arc;

use self::player::Players;
use self::world::ServerWorld;

pub mod player;
pub mod world;
mod handlers;

pub type Client = tokio::sync::mpsc::Sender<Arc<[u8]>>;
pub type UID = usize;

#[derive(Debug)]
pub enum BlockUpdateMode {
    Destroy,
    Place,
}

#[derive(Debug)]
pub enum Command {
    ChunkData([i32; 3], UID),
    Login(String, Client, tokio::sync::oneshot::Sender<Option<UID>>),
    Logout(UID),
    BlockUpdate([i32; 3], BlockUpdateMode, u8),
}

/// Supposed to be started in a new tread
pub fn start_world(
    mut input: tokio::sync::mpsc::Receiver<Command>,
    world_directory: std::path::PathBuf,
) {
    let mut server = Server::new(&world_directory);

    while let Some(command) = input.blocking_recv() {
        match command {
            Command::Login(name, client, back) => {
                let uuid = server.players.login(name, client);
                _ = back.send(uuid);
            }
            Command::Logout(uuid) => {
                server.players.logout(uuid);
            }
            Command::ChunkData(pos, uuid) => {
                // If the buffer is full or client disconnect, this package will not be send
                _ = server.players.client(uuid).try_send(server.world.get_chunk_data(&pos));
            }
            Command::BlockUpdate(pos, mode, block) => {}
        }
    }
}

struct Server {
    world: ServerWorld,
    players: Players,
}

impl Server {
    fn new(world_directory: &std::path::Path) -> Self {
        let players = Players::new(world_directory);

        let world = ServerWorld::from_files(&world_directory);

        Server {
            world,
            players,
        }
    }
}
