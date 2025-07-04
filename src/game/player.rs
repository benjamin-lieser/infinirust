use gl::types::GLint;
use nalgebra_glm::{self as glm, Vec3};

use crate::{
    mygl::{GLToken, TextureAtlas, VAO, VBO},
    net::ServerPackagePlayerPosition,
    server::UID,
};

use super::{chunk::add_face, Camera, FreeCamera};

pub struct Player {
    pub name: String,
    pub camera: FreeCamera, // Also contains the position and rotation
    pub uid: UID,
    pub velocity: Vec3, // Velocity in x, y, z
}

impl Player {
    pub fn bounding_box_pos(&self) -> [f64; 3] {
        let [x, y, z] = self.camera.position();

        [x - 0.4, y - 1.7, z - 0.4]
    }

    pub fn bounding_box_size(&self) -> [f64; 3] {
        // x y z
        [0.8, 1.8, 0.8]
    }
}

pub struct Players {
    /// Other Players
    players: Vec<Player>,
    pub local_player: Player,
    render: PlayerRender,
}

impl Players {
    pub fn new(glt: GLToken, atlas: &TextureAtlas, local_player: Player) -> Self {
        Self {
            players: vec![],
            local_player,
            render: PlayerRender::new(glt, atlas),
        }
    }

    pub fn add_player(&mut self, name: String, uid: UID, camera: FreeCamera) {
        self.players.push(Player { name, camera, uid, velocity: Vec3::zeros() });
    }

    pub fn update(&mut self, package: &ServerPackagePlayerPosition) {
        for player in self.players.iter_mut() {
            if player.uid == package.uid as usize {
                player
                    .camera
                    .update(package.pos, package.pitch, package.yaw);
            }
        }
        if self.local_player.uid == package.uid as usize {
            self.local_player
                .camera
                .update(package.pos, package.pitch, package.yaw);
        }
    }

    pub unsafe fn draw(
        &self,
        glt: GLToken,
        projection_view: &nalgebra_glm::Mat4,
        pos: &[f64; 3],
        mvp_location: GLint,
    ) {
        for player in self.players.iter() {
            let [x, y, z] = player.camera.position();

            //TODO this is still a bit off

            let model_center = glm::translation(&glm::vec3(-0.5, -0.5, -0.5));

            let model_trans = glm::translation(&glm::vec3(
                (x - pos[0]) as f32,
                (y - pos[1]) as f32,
                (z - pos[2]) as f32,
            ));
            let model = model_trans * player.camera.inverse_view_matrix() * model_center;
            let mvp = projection_view * model;
            gl::UniformMatrix4fv(mvp_location, 1, 0, mvp.as_ptr());
            self.render.draw(glt);
        }
    }
    pub fn delete(self, glt: GLToken) {
        self.render.delete(glt);
    }
}

pub struct PlayerRender {
    vao: VAO,
    vertex_vbo: VBO<u8>,
    texture_vbo: VBO<f32>,
}

impl PlayerRender {
    pub fn new(glt: GLToken, atlas: &TextureAtlas) -> Self {
        let mut vao = VAO::new(glt);
        let mut vertex_vbo = VBO::new(glt);
        let mut texture_vbo = VBO::new(glt);

        vao.attrib_pointer(glt, 0, &vertex_vbo, 3, 0, 0, false);
        vao.attrib_pointer(glt, 1, &texture_vbo, 2, 0, 0, false);
        vao.enable_array(glt, 0);
        vao.enable_array(glt, 1);

        // Make it a cube of obsidian with a furnace face for now
        let mut vertex_data = vec![];
        let mut texture_data = vec![];

        add_face(
            &mut vertex_data,
            &mut texture_data,
            atlas,
            "head.png",
            [0, 0, 0],
            super::Direction::NegX,
        );
        add_face(
            &mut vertex_data,
            &mut texture_data,
            atlas,
            "head.png",
            [0, 0, 0],
            super::Direction::PosX,
        );
        add_face(
            &mut vertex_data,
            &mut texture_data,
            atlas,
            "head.png",
            [0, 0, 0],
            super::Direction::NegY,
        );
        add_face(
            &mut vertex_data,
            &mut texture_data,
            atlas,
            "head.png",
            [0, 0, 0],
            super::Direction::PosY,
        );
        add_face(
            &mut vertex_data,
            &mut texture_data,
            atlas,
            "face.png",
            [0, 0, 0],
            super::Direction::NegZ,
        );
        add_face(
            &mut vertex_data,
            &mut texture_data,
            atlas,
            "head.png",
            [0, 0, 0],
            super::Direction::PosZ,
        );

        vertex_vbo.copy(glt, &vertex_data);
        texture_vbo.copy(glt, &texture_data);

        Self {
            vao,
            vertex_vbo,
            texture_vbo,
        }
    }

    pub fn draw(&self, glt: GLToken) {
        self.vao.bind(glt);
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }
    }

    pub fn delete(self, glt: GLToken) {
        self.vao.delete(glt);
        self.vertex_vbo.delete(glt);
        self.texture_vbo.delete(glt);
    }
}
