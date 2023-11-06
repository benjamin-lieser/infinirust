//! Defined the different types of blocks in the world

use std::collections::HashMap;

use serde::Deserialize;


pub struct Blocks {

}
#[derive(Debug, Deserialize)]
pub struct BlockConfig {
    pub name: String,
    ///Keys are 'default' 'pos_x' 'neg_x' ... 'neg_z', values are the texture files
    pub faces: HashMap<String, String>
}