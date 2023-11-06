//! Defined the different types of blocks in the world

use std::collections::HashMap;

use serde::Deserialize;

use super::Direction;

pub struct Blocks {

}
#[derive(Debug, Deserialize)]
pub struct BlockJson {
    name: String,
    faces: HashMap<Direction, String>
}