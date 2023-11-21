use std::sync::Arc;

use anyhow::Ok;
use serde::{Deserialize, Serialize};

use super::{Client, UID};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub name: String,
    pub pos: [f64; 3],
    pub pitch: f32,
    pub yaw: f32,
}

impl Player {
    pub fn new(name: String) -> Self {
        Player {
            name,
            pos: [0.0; 3],
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct ServerPlayer {
    pub player: Player,
    pub package_writer: Client,
    pub uid: usize,
}

/// Both fields have to be same length, online is None when not logged in
#[derive(Debug)]
pub struct Players {
    registered: Vec<Player>,
    online: Vec<Option<ServerPlayer>>,
}

impl Players {
    pub fn new(world_directory: &std::path::Path) -> Self {
        let player_file =
            std::fs::read_to_string(world_directory.join("players.json")).unwrap_or("[]".into());

        let players: Vec<Player> =
            serde_json::from_str(&player_file).expect("Could not parse players.json");
        let online = (0..players.len()).map(|_| None).collect();

        Players {
            registered: players,
            online,
        }
    }

    pub fn login(&mut self, name: String, client: Client) -> Option<UID> {
        let pos = self
            .registered
            .iter()
            .enumerate()
            .find(|(_idx, p)| p.name == name);
        if let Some((pos, player)) = pos {
            //Already registered
            if self.online[pos].is_none() {
                self.online[pos] = Some(ServerPlayer {
                    player: player.clone(),
                    package_writer: client,
                    uid: pos,
                });
                Some(pos)
            } else {
                None
            }
        } else {
            //Not registered
            let uid = self.registered.len();
            self.registered.push(Player::new(name));
            self.online.push(Some(ServerPlayer {
                player: self.registered[uid].clone(),
                package_writer: client,
                uid,
            }));
            Some(uid)
        }
    }

    pub fn logout(&mut self, uid: UID) {
        let player = self.online[uid].take();
        self.registered[uid] = player.unwrap().player;
    }

    pub fn sync_to_disk(&mut self, world_directory: &std::path::Path) -> Result<(), anyhow::Error> {
        for (disk, ram) in self.registered.iter_mut().zip(self.online.iter()) {
            if let Some(p) = ram {
                *disk = p.player.clone();
            }
        }

        let json = serde_json::to_string(&self.registered)?;

        std::fs::write(world_directory.join("players.json"), json)?;

        Ok(())
    }

    pub fn client(&self, uid: UID) -> &Client {
        &self.online[uid].as_ref().unwrap().package_writer
    }

    /// Sends a package to all logged in players
    pub fn broadcast(&self, package: Arc<[u8]>) {
        for player in self.online.iter().flatten() {
            // Package gets lost if the write channel is full
            _ = player.package_writer.try_send(package.clone());
        }
    }
}
