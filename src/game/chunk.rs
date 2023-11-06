use std::io::{Write, Read};

use crate::mygl::{TextureAtlas, VBOWithStorage, VAO};

use crate::game::Direction;

pub const CHUNK_SIZE: usize = 16;

/// Range y chunks go from -Y_RANGE to Y_RANGE - 1
pub const Y_RANGE: i32 = 4;

/// Data of a chunk. Is used by server and client
pub struct ChunkData {
    blocks : Vec<u8>
}

impl ChunkData {

    pub fn empty() -> Self {
        ChunkData { blocks: vec![0;CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE]}
    }

    pub fn get(&self, pos: [usize; 3]) -> u8 {
        self.blocks[pos[0] * CHUNK_SIZE * CHUNK_SIZE + pos[1] * CHUNK_SIZE + pos[2]]
    }

    pub fn set(&mut self, pos: [usize; 3], block: u8) {
        self.blocks[pos[0] * CHUNK_SIZE * CHUNK_SIZE + pos[1] * CHUNK_SIZE + pos[2]] = block
    }

    pub fn read_from(&mut self, reader : &mut impl Read) {
        reader.read_exact(&mut self.blocks).unwrap();
    }

    pub fn request_from(&mut self, pos : &[i32;3], stream : &mut (impl Read + Write)) {
        stream.write_all(crate::misc::as_bytes(pos)).unwrap();
        self.read_from(stream);
    }
}

pub struct Chunk {
    /// Array of blocks in the chunk
    blocks: ChunkData,
    /// [0,0,0] is the chunk at origion in the positive directions
    position: [i32; 3],
    vao: VAO,
    vertex_pos: VBOWithStorage<u8>,
    texture_pos: VBOWithStorage<f32>,
}

impl Chunk {
    /// The next bytes in data have to represent the chunk data
    pub fn new(position: [i32; 3], data: &mut (impl Read + Write)) -> Self {
        let mut chunk = Chunk {
            blocks: ChunkData::empty(),
            position,
            vao: VAO::new(),
            vertex_pos: VBOWithStorage::new(),
            texture_pos: VBOWithStorage::new(),
        };

        chunk.blocks.request_from(&position, data);

        chunk
            .vao
            .attrib_pointer(0, &chunk.vertex_pos.vbo, 3, 0, 0, false);
        chunk
            .vao
            .attrib_pointer(1, &chunk.texture_pos.vbo, 2, 0, 0, false);
        chunk.vao.enable_array(0);
        chunk.vao.enable_array(1);

        chunk
    }

    pub fn change_pos(&mut self, new_pos : [i32;3], server : &mut (impl Write + Read)) {
        self.blocks.request_from(&new_pos, server);
        self.position = new_pos;
    }

    pub fn write_vbo(&mut self, atlas: &TextureAtlas) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if self.blocks.get([x, y, z]) > 0 {
                        if z == CHUNK_SIZE - 1 || self.blocks.get([x, y, z + 1]) == 0 {
                            add_face(
                                &mut self.vertex_pos.data,
                                &mut self.texture_pos.data,
                                atlas,
                                0,
                                [x as u8, y as u8, z as u8],
                                Direction::PosZ,
                            );
                        }
                        if z == 0 || self.blocks.get([x, y, z - 1]) == 0 {
                            add_face(
                                &mut self.vertex_pos.data,
                                &mut self.texture_pos.data,
                                atlas,
                                0,
                                [x as u8, y as u8, z as u8],
                                Direction::NegZ,
                            );
                        }
                        if x == 0 || self.blocks.get([x - 1, y, z]) == 0 {
                            add_face(
                                &mut self.vertex_pos.data,
                                &mut self.texture_pos.data,
                                atlas,
                                0,
                                [x as u8, y as u8, z as u8],
                                Direction::NegX,
                            );
                        }
                        if x == CHUNK_SIZE - 1 || self.blocks.get([x + 1, y, z]) == 0 {
                            add_face(
                                &mut self.vertex_pos.data,
                                &mut self.texture_pos.data,
                                atlas,
                                0,
                                [x as u8, y as u8, z as u8],
                                Direction::PosX,
                            );
                        }
                        if y == CHUNK_SIZE - 1 || self.blocks.get([x, y + 1, z]) == 0 {
                            add_face(
                                &mut self.vertex_pos.data,
                                &mut self.texture_pos.data,
                                atlas,
                                1,
                                [x as u8, y as u8, z as u8],
                                Direction::PosY,
                            );
                        }
                        if y == 0 || self.blocks.get([x, y - 1, z]) == 0 {
                            add_face(
                                &mut self.vertex_pos.data,
                                &mut self.texture_pos.data,
                                atlas,
                                23,
                                [x as u8, y as u8, z as u8],
                                Direction::NegY,
                            );
                        }
                    }
                }
            }
        }
        self.vertex_pos.copy();
        self.texture_pos.copy();
    }

    pub fn draw(&self) {
        self.vao.bind();
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, self.texture_pos.data.len() as i32 / 2);
        }
    }

    pub fn position(&self) -> &[i32; 3] {
        &self.position
    }
}

fn add_face(
    vertex_data: &mut Vec<u8>,
    texture_data: &mut Vec<f32>,
    atlas: &TextureAtlas,
    texture: u32,
    pos: [u8; 3],
    dir: Direction,
) {
    let (tex_x, tex_y) = atlas.get_position(texture);
    let (size_x, size_y) = TextureAtlas::get_size();
    //We do counter clockwiese triangles

    //bottom left
    texture_data.push(tex_x);
    texture_data.push(tex_y);
    match dir {
        Direction::NegX | Direction::PosY | Direction::PosZ => {
            //top right
            texture_data.push(tex_x + size_x);
            texture_data.push(tex_y + size_y);
            //top left
            texture_data.push(tex_x);
            texture_data.push(tex_y + size_y);
        }
        Direction::PosX | Direction::NegY | Direction::NegZ => {
            //top left
            texture_data.push(tex_x);
            texture_data.push(tex_y + size_y);
            //top right
            texture_data.push(tex_x + size_x);
            texture_data.push(tex_y + size_y);
        }
    }

    //bottom left
    texture_data.push(tex_x);
    texture_data.push(tex_y);
    match dir {
        Direction::NegX | Direction::PosY | Direction::PosZ => {
            //bottom right
            texture_data.push(tex_x + size_x);
            texture_data.push(tex_y);
            //top right
            texture_data.push(tex_x + size_x);
            texture_data.push(tex_y + size_y);
        }
        Direction::PosX | Direction::NegY | Direction::NegZ => {
            //top right
            texture_data.push(tex_x + size_x);
            texture_data.push(tex_y + size_y);
            //bottom right
            texture_data.push(tex_x + size_x);
            texture_data.push(tex_y);
        }
    }

    let mut insert = |x, y, z| {
        vertex_data.push(pos[0] + x);
        vertex_data.push(pos[1] + y);
        vertex_data.push(pos[2] + z);
    };

    match dir {
        Direction::PosY => {
            insert(0, 1, 0);
            insert(1, 1, 1);
            insert(1, 1, 0);

            insert(0, 1, 0);
            insert(0, 1, 1);
            insert(1, 1, 1);
        }
        Direction::NegY => {
            insert(0, 0, 0);
            insert(1, 0, 0);
            insert(1, 0, 1);

            insert(0, 0, 0);
            insert(1, 0, 1);
            insert(0, 0, 1);
        }
        Direction::NegX => {
            insert(0, 0, 0);
            insert(0, 1, 1);
            insert(0, 1, 0);

            insert(0, 0, 0);
            insert(0, 0, 1);
            insert(0, 1, 1);
        }
        Direction::PosX => {
            insert(1, 0, 0);
            insert(1, 1, 0);
            insert(1, 1, 1);

            insert(1, 0, 0);
            insert(1, 1, 1);
            insert(1, 0, 1);
        }
        Direction::PosZ => {
            insert(0, 0, 1);
            insert(1, 1, 1);
            insert(0, 1, 1);

            insert(0, 0, 1);
            insert(1, 0, 1);
            insert(1, 1, 1);
        }
        Direction::NegZ => {
            insert(0, 0, 0);
            insert(0, 1, 0);
            insert(1, 1, 0);

            insert(0, 0, 0);
            insert(1, 1, 0);
            insert(1, 0, 0);
        }
    }
}
