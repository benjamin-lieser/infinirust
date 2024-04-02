use crate::{mygl::{GLToken, TextureAtlas, VAO, VBO}, server::UID};

use super::{chunk::add_face, FreeCamera};

pub struct Player {
    pub name: String,
    pub camera: FreeCamera,
    pub uid: UID,
    pub vao: VAO,
    pub vertex_vbo: VBO<u8>,
    pub texture_vbo: VBO<f32>,
}

pub struct PlayerRender {
    vao: VAO,
    vertex_vbo: VBO<u8>,
    texture_vbo: VBO<f32>,
}

impl PlayerRender {
    pub fn new(glt: GLToken, atlas : &TextureAtlas) -> Self {
        let mut vao = VAO::new(glt);
        let mut vertex_vbo = VBO::new(glt);
        let mut texture_vbo = VBO::new(glt);

        vao.attrib_pointer(glt, 0, &vertex_vbo, 3, 0, 0, false);
        vao.attrib_pointer(glt, 1, &texture_vbo, 2, 0, 0, false);
        vao.enable_array(glt, 0);
        vao.enable_array(glt, 1);


        // Make it a cube of end bricks for now
        let mut vertex_data = vec![];
        let mut texture_data = vec![];

        add_face(&mut vertex_data, &mut texture_data, atlas, "end_bricks.png", [0,0,0], super::Direction::NegX);
        add_face(&mut vertex_data, &mut texture_data, atlas, "end_bricks.png", [0,0,0], super::Direction::PosX);
        add_face(&mut vertex_data, &mut texture_data, atlas, "end_bricks.png", [0,0,0], super::Direction::NegY);
        add_face(&mut vertex_data, &mut texture_data, atlas, "end_bricks.png", [0,0,0], super::Direction::PosY);
        add_face(&mut vertex_data, &mut texture_data, atlas, "end_bricks.png", [0,0,0], super::Direction::NegZ);
        add_face(&mut vertex_data, &mut texture_data, atlas, "end_bricks.png", [0,0,0], super::Direction::PosZ);

        vertex_vbo.copy(glt, &vertex_data);
        texture_vbo.copy(glt, &texture_data);

        Self {
            vao,
            vertex_vbo,
            texture_vbo,
        }
    }

    pub fn draw(&self, glt: GLToken, camera: &FreeCamera) {
        self.vao.bind(glt);
        //TODO
    }
}