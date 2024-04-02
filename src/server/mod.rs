use std::sync::Arc;

use crate::net::ServerPackagePlayerPosition;

use self::player::Players;
use self::world::ServerWorld;
use crate::net::Package;

pub mod player;
pub mod world;
pub mod stdin;

pub type Client = tokio::sync::mpsc::Sender<Arc<[u8]>>;
pub type ServerCommand = tokio::sync::mpsc::Sender<(UID, Command)>;
pub type UID = usize;
pub const NOUSER: UID = std::usize::MAX;

#[derive(Debug)]
pub enum BlockUpdateMode {
    Destroy,
    Place,
}

#[derive(Debug)]
pub enum Command {
    ChunkData([i32; 3]),
    Login(String, Client, tokio::sync::oneshot::Sender<Option<UID>>),
    Logout,
    BlockUpdate([i32; 3], u8),
    PlayerPosition([f64; 3], f32, f32),
    Shutdown
}

/// Supposed to be started in a new tread
pub fn start_world(
    mut input: tokio::sync::mpsc::Receiver<(UID, Command)>,
    world_directory: std::path::PathBuf,
) {
    let mut server = Server::new(&world_directory);

    while let Some((uid, command)) = input.blocking_recv() {
        match command {
            Command::Login(name, client, back) => {
                let uid = server.players.login(name, client);
                back.send(uid).expect("Could not send uuid back");
            }
            Command::Logout => {
                server.players.logout(uid);
            }
            Command::ChunkData(pos) => {
                // If the buffer is full or client disconnect, this package will not be send
                _ = server.players.client(uid).try_send(server.world.get_chunk_data(&pos));
            }
            Command::PlayerPosition(pos, pitch, yaw) => {
                let player = server.players.get_player_mut(uid);
                player.pos = pos;
                player.pitch = pitch;
                player.yaw = yaw;
                let package = ServerPackagePlayerPosition {
                    uid: uid as u64,
                    pos,
                    pitch,
                    yaw,
                };
                // Send it to all other players
                server.players.broadcast_filtered(package.to_arc(), |p| p.uid != uid);
            }
            Command::BlockUpdate(pos, block) => {
                let package = server.world.process_block_update(&pos, block);
                server.players.broadcast(package);
            }
            Command::Shutdown => {
                server.players.sync_to_disk(&world_directory).unwrap();
                //Todo Sync chunk data
                eprintln!("Server shutted down after savin to disk");
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
