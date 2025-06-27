const CHUNK_SIZE: usize = crate::game::CHUNK_SIZE as usize;
use crate::game::{LocalBlockIndex, Y_RANGE};

use std::fs::File;
use std::io::{Read, Write};
use std::{collections::HashMap, sync::Arc};

use noise::NoiseFn;
use noise::Perlin;
use zerocopy::IntoBytes;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    seed: u32,
}

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


    pub fn get(&self, pos: LocalBlockIndex) -> u8 {
        let chunk_size_usize: usize = CHUNK_SIZE as usize;
        self.blocks[pos[0] as usize * chunk_size_usize * chunk_size_usize + pos[1] as usize * chunk_size_usize + pos[2] as usize]
    }
    
    pub fn set(&mut self, pos: [usize; 3], block: u8) {
        self.blocks[pos[0] * CHUNK_SIZE * CHUNK_SIZE + pos[1] * CHUNK_SIZE + pos[2]] = block
    }

    pub fn block_mut(&mut self, pos: LocalBlockIndex) -> &mut u8 {
        let chunk_size_usize: usize = CHUNK_SIZE as usize;
        &mut self.blocks[pos[0] as usize * chunk_size_usize * chunk_size_usize + pos[1] as usize * chunk_size_usize + pos[2] as usize]
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
}

impl ServerWorld {
    pub fn from_files(world_directory: &std::path::Path) -> Self {
        let settings_file = std::fs::read_to_string(world_directory.join("settings.json"))
            .expect("Could not open settings.json");
        let settings: Settings =
            serde_json::from_str(&settings_file).expect("Could not parse settings.json");

        // Load chunk data from file
        let loaded_chunks = if let Ok(mut chunk_data) = File::open(world_directory.join("chunks.dat")) {
            let mut loaded_chunks = HashMap::new();
            let mut pos = [0i32; 3];
            while let Ok(_) = chunk_data.read_exact(&mut pos.as_mut_bytes()) {
                let mut chunk = ChunkData::empty();
                chunk_data.read_exact(&mut chunk.blocks).expect("Could not read chunk data");
                loaded_chunks.insert(pos, chunk);    
            }
            loaded_chunks
        } else {
            HashMap::new()
        };


        ServerWorld {
            generator: Perlin::new(settings.seed),
            loaded_chunks,
        }
    }

    pub fn sync_to_disk(&self, world_directory: &std::path::Path) -> std::io::Result<()> {
        // Save chunk data
        let mut chunk_file = File::create(world_directory.join("chunks.dat"))?;
        for (pos, chunk) in &self.loaded_chunks {
            chunk_file.write_all(&pos.as_bytes())?;
            chunk_file.write_all(&chunk.blocks)?;
        }

        Ok(())
    }

    /// Gets a reference to a block or None if this position is not loaded
    pub fn get_block_mut(&mut self, pos: &[i32; 3]) -> Option<&mut u8> {
        let (chunk_pos, in_chunk_pos) = crate::game::chunk::block_position_to_chunk_index(*pos);
        if let Some(chunk) = self.loaded_chunks.get_mut(&chunk_pos) {
            Some(chunk.block_mut(in_chunk_pos))
        } else {
            None
        }
    }

    pub fn get_chunk_data(&mut self, pos: &[i32; 3]) -> Arc<[u8]> {
        if let Some(chunk) = self.loaded_chunks.get(pos) {
            create_chunk_package(chunk, pos)
        } else {
            let new_chunk = ChunkData::generate(&self.generator, pos);
            let package = create_chunk_package(&new_chunk, pos);
            self.loaded_chunks.insert(*pos, new_chunk);
            package
        }
    }
    /// If the block is in unloaded chunks it will be ignored
    pub fn process_block_update(&mut self, pos: &[i32; 3], new_block: u8) -> Arc<[u8]> {
        //Send empty package if block is in unloaded chunk
        if let Some(block) = self.get_block_mut(pos) {
            if new_block == 0 {
                //Destroy
                // This will always succed and leave an empty block
                *block = 0;
                create_block_update_package(pos, 0)
            } else {
                //Place
                // This will only succeed when the block is empty before
                if *block == 0 {
                    *block = new_block;
                    create_block_update_package(pos, new_block)
                } else {
                    create_block_update_package(pos, *block)
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
    package[2..14].copy_from_slice(pos.as_bytes());
    package[14..].copy_from_slice(&chunk.blocks);
    Arc::from(package)
    //Todo: Check if this is efficient
}

fn create_block_update_package(pos: &[i32; 3], block: u8) -> Arc<[u8]> {
    let mut package = [0u8; 2 + 12 + 4];
    package[0] = 0x0B;
    package[2..14].copy_from_slice(pos.as_bytes());
    package[14] = block;
    Arc::from(package)
    //Todo: Check if this is efficient
}
