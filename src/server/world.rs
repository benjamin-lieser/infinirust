use crate::game::CHUNK_SIZE;
use crate::game::Y_RANGE;
use crate::misc::as_bytes;
use std::hash::Hash;
use std::{collections::HashMap, sync::Arc};

use noise::NoiseFn;
use noise::Perlin;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    seed: u32,
}

use super::BlockUpdateMode;
pub struct ChunkData {
    blocks: Vec<u8>,
}

impl ChunkData {
    pub fn empty() -> Self {
        ChunkData {
            blocks: vec![0; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }

    pub fn generate(generator: &Perlin, pos: &[i32; 3]) -> Self {
        let mut chunk = Self::empty();

        let [x, y, z] = pos;

        for xx in 0..CHUNK_SIZE {
            for zz in 0..CHUNK_SIZE {
                let x = (x * CHUNK_SIZE as i32 + xx as i32) as f64 + 0.5;
                let z = (z * CHUNK_SIZE as i32 + zz as i32) as f64 + 0.5;
                let height =
                    generator.get([x / 50.0, z / 50.0]) * Y_RANGE as f64 * CHUNK_SIZE as f64 * 0.1;
                for yy in 0..CHUNK_SIZE {
                    let y = (y * CHUNK_SIZE as i32 + yy as i32) as f64 + 0.5;
                    if y <= height {
                        chunk.set([xx, yy, zz], 1);
                    }
                }
            }
        }

        chunk
    }

    pub fn get(&self, pos: [usize; 3]) -> u8 {
        self.blocks[pos[0] * CHUNK_SIZE * CHUNK_SIZE + pos[1] * CHUNK_SIZE + pos[2]]
    }

    pub fn set(&mut self, pos: [usize; 3], block: u8) {
        self.blocks[pos[0] * CHUNK_SIZE * CHUNK_SIZE + pos[1] * CHUNK_SIZE + pos[2]] = block
    }

    pub fn block_mut(&mut self, pos: [usize; 3]) -> &mut u8 {
        &mut self.blocks[pos[0] * CHUNK_SIZE * CHUNK_SIZE + pos[1] * CHUNK_SIZE + pos[2]]
    }
}
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ChunkMeta {
    pos: [i32; 3],
    /// Where to find the chunk data in the chunk file
    offset: usize,
}

pub struct ServerWorld {
    generator: Perlin,
    loaded_chunks: HashMap<[i32; 3], ChunkData>,
    chunk_meta: HashMap<[i32; 3], ChunkMeta>,
}

impl ServerWorld {
    pub fn new(seed: u32) -> Self {
        ServerWorld {
            generator: Perlin::new(seed),
            loaded_chunks: HashMap::new(),
            chunk_meta: HashMap::new(),
        }
    }

    pub fn from_files(world_directory: &std::path::Path) -> Self {
        let settings_file = std::fs::read_to_string(world_directory.join("settings.json"))
            .expect("Could not open settings.json");
        let settings: Settings =
            serde_json::from_str(&settings_file).expect("Could not parse settings.json");

        let chunks_file =
            std::fs::read_to_string(world_directory.join("chunks.json")).unwrap_or_default(); //If this file does not exist we assume it to be empty

        let chunk_meta_data : Vec<ChunkMeta> = serde_json::from_str(&chunks_file).expect("Could not parse chunks.json");
        let mut chunk_meta = HashMap::new();

        for meta in chunk_meta_data {
            chunk_meta.insert(meta.pos, meta);
        }

        ServerWorld {
            generator: Perlin::new(settings.seed),
            loaded_chunks: HashMap::new(),
            chunk_meta
        }
    }

    /// Gets a reference to a block or None if this position is not loaded
    pub fn get_block_mut(&mut self, pos: &[i32; 3]) -> Option<&mut u8> {
        let chunk_pos = pos.map(|x| x / CHUNK_SIZE as i32);
        let in_chunk_pos = pos.map(|x| x as usize % CHUNK_SIZE);
        if let Some(chunk) = self.loaded_chunks.get_mut(&chunk_pos) {
            return Some(chunk.block_mut(in_chunk_pos));
        } else {
            return None;
        }
    }

    pub fn get_chunk_data(&mut self, pos: &[i32; 3]) -> Arc<[u8]> {
        if let Some(chunk) = self.loaded_chunks.get(pos) {
            create_chunk_package(chunk, pos)
        } else {
            //TODO: Make sure the pos are reasonable (Maybe check that the player is actually near it, to prevent DDOS)
            let new_chunk = ChunkData::generate(&self.generator, pos);
            let package = create_chunk_package(&new_chunk, pos);
            self.loaded_chunks.insert(*pos, new_chunk);
            package
        }
    }
    /// If the block is in unloaded chunks it will be ignored
    pub fn process_block_update(
        &mut self,
        pos: &[i32; 3],
        mode: BlockUpdateMode,
        new_block: u8,
    ) -> Arc<[u8]> {
        //Send empty package if out of block in unloaded chunk
        if let Some(block) = self.get_block_mut(pos) {
            match mode {
                // This will always succed and leave an empty block
                BlockUpdateMode::Destroy => {
                    *block = 0;
                    create_block_update_package(pos, 0)
                }
                // This will only succeed when the block is empty before
                BlockUpdateMode::Place => {
                    if *block == 0 {
                        *block = new_block;
                        create_block_update_package(pos, new_block)
                    } else {
                        create_block_update_package(pos, *block)
                    }
                }
            }
        } else {
            Arc::new([])
        }
    }
}

/// The package has 2 bytes with the package id 0x0A 0x00, 12 bytes of position and 4096 bytes of chunk data
fn create_chunk_package(chunk: &ChunkData, pos: &[i32; 3]) -> Arc<[u8]> {
    let mut package = [0u8; 2 + 12 + 4096];
    package[0] = 0x0A;
    package[2..14].copy_from_slice(as_bytes(pos));
    package[14..].copy_from_slice(&chunk.blocks);
    Arc::from(package)
    //Todo: Check if this is efficient
}

fn create_block_update_package(pos: &[i32; 3], block: u8) -> Arc<[u8]> {
    let mut package = [0u8; 2 + 12 + 1];
    package[0] = 0x0B;
    package[2..14].copy_from_slice(as_bytes(pos));
    package[14] = block;
    Arc::from(package)
    //Todo: Check if this is efficient
}
