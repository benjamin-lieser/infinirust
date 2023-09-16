use noise::{NoiseFn, Perlin};

use crate::mygl::{TextureAtlas, VBOWithStorage, VAO};

use crate::game::Direction;

pub const CHUNK_SIZE: usize = 16;

/// Range of y is from -MAX_Y to MAX_Y exclusive, has to be multiple of CHUNK_SIZE
pub const MAX_Y: i32 = 256;

pub struct Chunk {
    /// Array of blocks in the chunk
    blocks: Vec<u8>,
    /// [0,0,0] is the chunk at origion in the positive direction
    position: [i32; 3],
    vao: VAO,
    vertex_pos: VBOWithStorage<u8>,
    texture_pos: VBOWithStorage<f32>,
}

impl Chunk {
    pub fn new(position: [i32; 3], generator: &Perlin) -> Self {
        let mut chunk = Chunk {
            blocks: vec![0; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
            position,
            vao: VAO::new(),
            vertex_pos: VBOWithStorage::new(),
            texture_pos: VBOWithStorage::new(),
        };

        let [x, y, z] = position;

        for xx in 0..CHUNK_SIZE {
            for zz in 0..CHUNK_SIZE {
                let x = (x * CHUNK_SIZE as i32 + xx as i32) as f64 + 0.5;
                let z = (z * CHUNK_SIZE as i32 + zz as i32) as f64 + 0.5;
                let height = generator.get([x, z]);
                for yy in 0..CHUNK_SIZE {
                    let y = ((y * CHUNK_SIZE as i32 + yy as i32) / MAX_Y) as f64 + 0.5;
                    if y >= height {
                        chunk.set([xx, yy, zz], 1);
                    }
                }
            }
        }

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

    pub fn get(&self, pos: [usize; 3]) -> u8 {
        self.blocks[pos[0] * CHUNK_SIZE * CHUNK_SIZE + pos[1] * CHUNK_SIZE + pos[2]]
    }

    pub fn set(&mut self, pos: [usize; 3], block: u8) {
        self.blocks[pos[0] * CHUNK_SIZE * CHUNK_SIZE + pos[1] * CHUNK_SIZE + pos[2]] = block
    }

    pub fn write_vbo(&mut self, atlas: &TextureAtlas) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if self.get([x, y, z]) > 0 {
                        add_face(
                            &mut self.vertex_pos.data,
                            &mut self.texture_pos.data,
                            atlas,
                            0,
                            [x as u8, y as u8, z as u8],
                            Direction::PosZ,
                        );
                        add_face(
                            &mut self.vertex_pos.data,
                            &mut self.texture_pos.data,
                            atlas,
                            0,
                            [x as u8, y as u8, z as u8],
                            Direction::NegZ,
                        );
                        add_face(
                            &mut self.vertex_pos.data,
                            &mut self.texture_pos.data,
                            atlas,
                            0,
                            [x as u8, y as u8, z as u8],
                            Direction::NegX,
                        );
                        add_face(
                            &mut self.vertex_pos.data,
                            &mut self.texture_pos.data,
                            atlas,
                            0,
                            [x as u8, y as u8, z as u8],
                            Direction::PosX,
                        );
                        add_face(
                            &mut self.vertex_pos.data,
                            &mut self.texture_pos.data,
                            atlas,
                            23,
                            [x as u8, y as u8, z as u8],
                            Direction::NegY,
                        );
                        add_face(
                            &mut self.vertex_pos.data,
                            &mut self.texture_pos.data,
                            atlas,
                            1,
                            [x as u8, y as u8, z as u8],
                            Direction::PosY,
                        );
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
        _ => {}
    }
}
