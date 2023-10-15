use std::fs;
use std::sync::Arc;

use self::player::{Player, ServerPlayer};
use self::world::ServerWorld;

pub mod player;
pub mod world;
mod handlers;

type Client = tokio::sync::mpsc::Sender<Arc<[u8]>>;

#[derive(Debug)]
pub enum BlockUpdateMode {
    Destroy,
    Place,
}

#[derive(Debug)]
pub enum Command {
    ChunkData([i32; 3], Client),
    Login(String, Client),
    Logout(usize),
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
            Command::Login(name, client) => {
                handlers::login(&mut server, name, client);
            }
            Command::Logout(player_id) => {}
            Command::ChunkData(pos, client) => {
                // If the buffer is full or client disconnect, this package will not be send
                _ = client.try_send(server.world.get_chunk_data(&pos));
            }
            Command::BlockUpdate(pos, mode, block) => {}
        }
    }
}

struct Server {
    world: ServerWorld,
    players: Vec<Player>,
    connected_players: Vec<Option<ServerPlayer>>,
}

impl Server {
    fn new(world_directory: &std::path::Path) -> Self {
        let player_file = fs::read_to_string(world_directory.join("players.json"))
            .expect("Could not open players.json");

        let players = serde_json::from_str(&player_file).expect("Could not parse players.json");

        let world = ServerWorld::from_files(&world_directory);

        Server {
            world,
            players,
            connected_players: vec![],
        }
    }

    fn is_logged_in(&self, name: &str) -> bool {
        for player in &self.connected_players {
            match player {
                Some(p) => {
                    if p.player.name == name {
                        return true;
                    }
                }
                None => {}
            }
        }
        return false;
    }

    fn is_known(&self, name: &str) -> Option<Player> {
        for player in &self.players {
            if player.name == name {
                return Some(player.clone());
            }
        }
        return None;
    }
}
