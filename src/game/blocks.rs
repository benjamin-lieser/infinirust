//! Defines the different types of blocks in the world

use std::{collections::HashMap, path::Path};

use serde::Deserialize;

use crate::game::Direction;

#[derive(Debug, Deserialize)]
pub struct BlockConfig {
    pub id: u32,
    pub name: String,
    pub texture: String,
    pub top_texture: String,
    pub bottom_texture: String,
}

pub struct BlocksConfig {
    pub blocks: Vec<BlockConfig>,
    textures: HashMap<String, u16>,
}

impl BlocksConfig {
    pub fn new(file: &Path) -> (Self, Vec<String>) {
        let file_content =
            std::fs::read_to_string(file).expect("Failed to read blocks config file");
        let mut blocks: Vec<BlockConfig> =
            serde_json::from_str(&file_content).expect("Failed to parse blocks config file");

        blocks.insert(
            0,
            BlockConfig {
                id: 0,
                name: "air".to_string(),
                texture: "".to_string(),
                top_texture: "".to_string(),
                bottom_texture: "".to_string(),
            },
        );

        let mut textures = HashMap::new();
        let mut textures_vec = Vec::new();

        for block in &blocks[1..] {
            if !textures.contains_key(&block.texture) {
                textures.insert(block.texture.clone(), textures_vec.len() as u16);
                textures_vec.push(block.texture.clone());
            }
            if !textures.contains_key(&block.top_texture) {
                textures.insert(block.top_texture.clone(), textures_vec.len() as u16);
                textures_vec.push(block.top_texture.clone());
            }
            if !textures.contains_key(&block.bottom_texture) {
                textures.insert(block.bottom_texture.clone(), textures_vec.len() as u16);
                textures_vec.push(block.bottom_texture.clone());
            }
        }

        let blocks_config = Self { blocks, textures };

        (blocks_config, textures_vec)
    }

    pub fn get_texture(&self, block_type: u8, dir: Direction) -> u16 {
        let block = &self.blocks[block_type as usize];
        let texture_name = match dir {
            Direction::PosY => &block.top_texture,
            Direction::NegY => &block.bottom_texture,
            _ => &block.texture,
        };
        self.textures
            .get(texture_name)
            .copied()
            .expect("Texture not found in blocks config")
    }
}
