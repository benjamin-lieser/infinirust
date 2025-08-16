use std::sync::Arc;

use zerocopy::IntoBytes;

use crate::net::{ServerPackageLogout, ServerPackagePlayerPosition, ServerPlayerLogin};

use self::player::Players;
use self::world::ServerWorld;
use crate::net::Package;

pub mod player;
pub mod stdin;
pub mod world;
pub mod world_generator;

pub type Client = tokio::sync::mpsc::Sender<Arc<[u8]>>;
pub type ServerCommand = tokio::sync::mpsc::Sender<(UID, Command)>;
pub type UID = usize;
pub const NOUSER: UID = usize::MAX;

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
    Shutdown,
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
                back.send(uid).expect("Server: Could not send uid back");
                if let Some(uid) = uid {
                    //send login success package
                    let mut package =
                        vec![0x02u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
                    package[2..].copy_from_slice((uid as u64).as_bytes());
                    server
                        .players
                        .client(uid)
                        .try_send(Arc::from(package))
                        .unwrap();
                    //Send position package to the new player
                    let player = server.players.get_player_mut(uid);
                    let package = ServerPackagePlayerPosition {
                        uid: uid as u64,
                        pos: player.pos,
                        pitch: player.pitch,
                        yaw: player.yaw,
                    };
                    server
                        .players
                        .client(uid)
                        .try_send(package.to_arc())
                        .unwrap();

                    //Send login package of all online players
                    for player in server.players.online() {
                        if player.uid != uid {
                            let package = ServerPlayerLogin {
                                uid: player.uid as u64,
                                name: player.player.name.clone(),
                            };
                            server
                                .players
                                .client(uid)
                                .try_send(package.to_arc())
                                .unwrap();
                        }
                    }

                    //Send all the other players that this on logged in
                    let package = ServerPlayerLogin {
                        uid: uid as u64,
                        name: server.players.get_player_mut(uid).name.clone(),
                    };
                    server
                        .players
                        .broadcast_filtered(package.to_arc(), |p| p.uid != uid);
                }
            }
            Command::Logout => {
                server.players.logout(uid);
                server
                    .players
                    .broadcast_filtered(ServerPackageLogout { uid: uid as u64 }.to_arc(), |p| {
                        p.uid != uid
                    });
            }
            Command::ChunkData(pos) => {
                // If the buffer is full or client disconnect, this package will not be send
                _ = server
                    .players
                    .client(uid)
                    .try_send(server.world.get_chunk_data(&pos));
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
                server
                    .players
                    .broadcast_filtered(package.to_arc(), |p| p.uid != uid);
            }
            Command::BlockUpdate(pos, block) => {
                let package = server.world.process_block_update(&pos, block);
                server.players.broadcast(package);
            }
            Command::Shutdown => {
                server.players.sync_to_disk(&world_directory).unwrap();
                server.world.sync_to_disk(&world_directory).unwrap();
                eprintln!("Server: shut down after saving to disk");
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

        Server { world, players }
    }
}
