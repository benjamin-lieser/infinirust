use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub name: String,
    pub pos: [f64; 3],
    pub pitch: f32,
    pub yaw: f32,
}

impl Player {
    pub fn new(name: String) -> Self {
        Player { name, pos: [0.0;3], pitch: 0.0, yaw: 0.0 }
    }
}

#[derive(Debug)]
pub struct ServerPlayer {
    pub player: Player,
    pub package_writer: tokio::sync::mpsc::Sender<Arc<[u8]>>,
    /// Is unique among the currently logged in users, but assigned dynamically
    pub player_id: usize,
}