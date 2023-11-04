use std::sync::Arc;

use self::player::Players;
use self::world::ServerWorld;

pub mod player;
pub mod world;
pub mod handlers;

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
    BlockUpdate([i32; 3], u8),
    Shutdown
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
                let uid = server.players.login(name, client);
                _ = back.send(uid);
            }
            Command::Logout(uid) => {
                server.players.logout(uid);
            }
            Command::ChunkData(pos, uid) => {
                // If the buffer is full or client disconnect, this package will not be send
                _ = server.players.client(uid).try_send(server.world.get_chunk_data(&pos));
            }
            Command::BlockUpdate(pos, block) => {
                let package = server.world.process_block_update(&pos, block);
                server.players.broadcast(package);
            }
            Command::Shutdown => {
                server.players.sync_to_disk(&world_directory).unwrap();
                //Todo Sync chunk data
                println!("Shutting down.");
                std::process::exit(0);
            }
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

        let world = ServerWorld::from_files(world_directory);

        Server {
            world,
            players,
        }
    }
}
