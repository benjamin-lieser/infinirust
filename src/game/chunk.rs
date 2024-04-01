use crate::mygl::{GLToken, TextureAtlas, VBOWithStorage, VAO};

use crate::game::Direction;

pub const CHUNK_SIZE: usize = 16;

/// Range y chunks go from -Y_RANGE to Y_RANGE - 1
pub const Y_RANGE: i32 = 4;

/// Data of a chunk. The blocks are stored in a 1D array
pub struct ChunkData {
    blocks: Vec<u8>,
}

impl ChunkData {
    pub fn new(data: Vec<u8>) -> Self {
        ChunkData { blocks: data }
    }

    pub fn get(&self, pos: [usize; 3]) -> u8 {
        self.blocks[pos[0] * CHUNK_SIZE * CHUNK_SIZE + pos[1] * CHUNK_SIZE + pos[2]]
    }

    pub fn set(&mut self, pos: [usize; 3], block: u8) {
        self.blocks[pos[0] * CHUNK_SIZE * CHUNK_SIZE + pos[1] * CHUNK_SIZE + pos[2]] = block
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
    pub fn new(glt: GLToken, position: [i32; 3], data: Vec<u8>) -> Self {
        let mut chunk = Chunk {
            blocks: ChunkData::new(data),
            position,
            vao: VAO::new(glt),
            vertex_pos: VBOWithStorage::new(glt),
            texture_pos: VBOWithStorage::new(glt),
        };

        chunk
            .vao
            .attrib_pointer(glt, 0, &chunk.vertex_pos.vbo(), 3, 0, 0, false);
        chunk
            .vao
            .attrib_pointer(glt, 1, &chunk.texture_pos.vbo(), 2, 0, 0, false);
        chunk.vao.enable_array(glt, 0);
        chunk.vao.enable_array(glt, 1);

        chunk
    }

    pub fn write_vbo(&mut self, atlas: &TextureAtlas) {
        let mut vertex_pos = vec![];
        let mut texture_pos = vec![];

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if self.blocks.get([x, y, z]) > 0 {
                        if z == CHUNK_SIZE - 1 || self.blocks.get([x, y, z + 1]) == 0 {
                            add_face(
                                &mut vertex_pos,
                                &mut texture_pos,
                                atlas,
                                "grass_side.png",
                                [x as u8, y as u8, z as u8],
                                Direction::PosZ,
                            );
                        }
                        if z == 0 || self.blocks.get([x, y, z - 1]) == 0 {
                            add_face(
                                &mut vertex_pos,
                                &mut texture_pos,
                                atlas,
                                "grass_side.png",
                                [x as u8, y as u8, z as u8],
                                Direction::NegZ,
                            );
                        }
                        if x == 0 || self.blocks.get([x - 1, y, z]) == 0 {
                            add_face(
                                &mut vertex_pos,
                                &mut texture_pos,
                                atlas,
                                "grass_side.png",
                                [x as u8, y as u8, z as u8],
                                Direction::NegX,
                            );
                        }
                        if x == CHUNK_SIZE - 1 || self.blocks.get([x + 1, y, z]) == 0 {
                            add_face(
                                &mut vertex_pos,
                                &mut texture_pos,
                                atlas,
                                "grass_side.png",
                                [x as u8, y as u8, z as u8],
                                Direction::PosX,
                            );
                        }
                        if y == CHUNK_SIZE - 1 || self.blocks.get([x, y + 1, z]) == 0 {
                            add_face(
                                &mut vertex_pos,
                                &mut texture_pos,
                                atlas,
                                "grass_top.png",
                                [x as u8, y as u8, z as u8],
                                Direction::PosY,
                            );
                        }
                        if y == 0 || self.blocks.get([x, y - 1, z]) == 0 {
                            add_face(
                                &mut vertex_pos,
                                &mut texture_pos,
                                atlas,
                                "dirt.png",
                                [x as u8, y as u8, z as u8],
                                Direction::NegY,
                            );
                        }
                    }
                }
            }
        }

        self.vertex_pos.exchange_cpu_buffer(vertex_pos);
        self.texture_pos.exchange_cpu_buffer(texture_pos);
    }

    pub fn draw(&self, glt: GLToken) {
        self.vao.bind(glt);
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, self.texture_pos.len() as i32 / 2);
        }
    }

    pub fn position(&self) -> &[i32; 3] {
        &self.position
    }

    pub fn delete(self, glt: GLToken) {
        self.vao.delete(glt);
        self.vertex_pos.delete(glt);
        self.texture_pos.delete(glt);
    }
}

fn add_face(
    vertex_data: &mut Vec<u8>,
    texture_data: &mut Vec<f32>,
    atlas: &TextureAtlas,
    texture: &str,
    pos: [u8; 3],
    dir: Direction,
) {
    let (tex_x, tex_y) = atlas.get_position(texture).unwrap();
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
