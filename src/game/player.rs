use gl::types::GLint;
use nalgebra_glm::{self as glm, DVec3, Vec3};
use zerocopy::transmute;

use crate::{
    mygl::{GLToken, VAO, VBO},
    net::ServerPackagePlayerPosition,
    server::UID,
};

use super::Camera;

pub struct Player {
    pub name: String,
    pub position: DVec3, // Position in x, y, z
    pub pitch: f32,      // Pitch in radians
    pub yaw: f32,        // Yaw in radians
    pub uid: UID,
    pub velocity: Vec3,     // Velocity in x, y, z
    pub on_ground: bool,    // Whether the player is on the ground
    pub jump_duration: f32, // How long the player has been jumping
}

impl Camera for Player {
    fn camera_position(&self) -> [f64; 3] {
        transmute!((self.position + DVec3::new(0.25, 1.5, 0.25)).data.0)
    }

    fn pitch(&self) -> f32 {
        self.pitch
    }

    fn yaw(&self) -> f32 {
        self.yaw
    }

    fn change_pitch(&mut self, diff: f32) {
        self.pitch =
            (self.pitch + diff).clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
    }

    fn change_yaw(&mut self, diff: f32) {
        self.yaw = (self.yaw + diff) % std::f32::consts::TAU;
    }
}

impl Player {
    pub fn bounding_box_size(&self) -> DVec3 {
        // x y z
        DVec3::new(0.5, 1.75, 0.5)
    }

    pub fn update_pos_pitch_yaw(&mut self, pos: [f64; 3], pitch: f32, yaw: f32) {
        self.position = DVec3::new(pos[0], pos[1], pos[2]);
        self.pitch = pitch;
        self.yaw = yaw;
    }
}

pub struct Players {
    /// Other Players
    players: Vec<Player>,
    pub local_player: Player,
    render: PlayerRender,
}

impl Players {
    pub fn new(glt: GLToken, local_player: Player) -> Self {
        Self {
            players: vec![],
            local_player,
            render: PlayerRender::new(glt),
        }
    }

    pub fn add_player(&mut self, name: String, uid: UID) {
        self.players.push(Player {
            name,
            position: DVec3::new(0.0, 0.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,
            uid,
            velocity: Vec3::zeros(),
            on_ground: false,
            jump_duration: 0.0,
        });
    }

    pub fn update(&mut self, package: &ServerPackagePlayerPosition) {
        for player in self.players.iter_mut() {
            if player.uid == package.uid as usize {
                player.update_pos_pitch_yaw(package.pos, package.pitch, package.yaw);
            }
        }
        if self.local_player.uid == package.uid as usize {
            self.local_player
                .update_pos_pitch_yaw(package.pos, package.pitch, package.yaw);
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
            let [x, y, z] = player.camera_position();

            //TODO this is still a bit off

            let model_center = glm::translation(&glm::vec3(-0.5, -0.5, -0.5));

            let model_trans = glm::translation(&glm::vec3(
                (x - pos[0]) as f32,
                (y - pos[1]) as f32,
                (z - pos[2]) as f32,
            ));
            let model = model_trans * player.inverse_view_matrix() * model_center;
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
    pub fn new(glt: GLToken) -> Self {
        let mut vao = VAO::new(glt);
        let mut vertex_vbo = VBO::new(glt);
        let mut texture_vbo = VBO::new(glt);

        vao.attrib_pointer(glt, 0, &vertex_vbo, 3, 0, 0, false);
        vao.attrib_pointer(glt, 1, &texture_vbo, 2, 0, 0, false);
        vao.enable_array(glt, 0);
        vao.enable_array(glt, 1);

        // Make it a cube of obsidian with a furnace face for now
        //let mut vertex_data = vec![];
        //let mut texture_data = vec![];

        /*add_face(
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
        );*/

        //vertex_vbo.copy(glt, &vertex_data);
        //texture_vbo.copy(glt, &texture_data);

        Self {
            vao,
            vertex_vbo,
            texture_vbo,
        }
    }

    pub fn draw(&self, glt: GLToken) {
        self.vao.bind(glt);
        // TODO different redering
        //gl::DrawArrays(gl::TRIANGLES, 0, 36);
    }

    pub fn delete(self, glt: GLToken) {
        self.vao.delete(glt);
        self.vertex_vbo.delete(glt);
        self.texture_vbo.delete(glt);
    }
}
