use std::{collections::HashMap, fs::File, hash::Hash, io::BufReader};

use gl::types::{GLint, GLuint};
use nalgebra_glm::{self as glm, DVec3, Vec3};
use obj::TexturedVertex;
use zerocopy::transmute;

use crate::{
    game::misc::{CubeOutlines, extract_group_range},
    mygl::{GLToken, IndexBuffer, VAO, VBO},
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
        DVec3::new(0.6, 1.625, 0.6)
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
    bounding_box_render: CubeOutlines,
}

impl Players {
    pub fn new(glt: GLToken, local_player: Player) -> Self {
        Self {
            players: vec![],
            local_player,
            render: PlayerRender::new(glt),
            bounding_box_render: CubeOutlines::new(glt),
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

    pub fn draw(
        &self,
        glt: GLToken,
        projection_view: &nalgebra_glm::Mat4,
        camera_pos: &[f64; 3],
        mvp_location: GLint,
    ) {
        self.render.vao.bind(glt);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.render.texture);
        }
        // The Model is centered on 0,0,0, we have the lower x y coordinates in pos
        let model_center = glm::translation(&glm::vec3(0.3/0.6, 0.0, 0.3/0.6));
        for player in self.players.iter() {
            let player_pos = player.position;

            let model_trans = glm::translation(&glm::vec3(
                (player_pos.x - camera_pos[0]) as f32,
                (player_pos.y - camera_pos[1]) as f32,
                (player_pos.z - camera_pos[2]) as f32,
            ));


            for (name, (start, end)) in &self.render.body_ranges {
                let model_local = if name == "head" {
                    let head_in_center = glm::translation(&glm::vec3(0.0, -2.3, 0.0));
                    let head_back = glm::translation(&glm::vec3(0.0, 2.3, 0.0));
                    model_center * head_back * player.inverse_view_matrix() * head_in_center
                } else {
                    glm::rotate(&model_center, -player.yaw + std::f32::consts::PI, &glm::vec3(0.0, 1.0, 0.0))
                };
                
                let model = glm::scale(&model_trans, &Vec3::new(0.6, 0.6, 0.6)) * model_local;
                let mvp = projection_view * model;
                unsafe {
                    gl::UniformMatrix4fv(mvp_location, 1, 0, mvp.as_ptr());
                }
                unsafe {
                    gl::DrawElements(
                        gl::TRIANGLES,
                        (end - start) as GLint * 3,
                        gl::UNSIGNED_INT,
                        (3 * start * std::mem::size_of::<u32>() as u32) as *const _,
                    );
                }
            }
            // Draw the bounding box
            let bounding_box_size = player.bounding_box_size().cast();

            let bounding_box_model = glm::scale(&model_trans, &bounding_box_size);
            self.bounding_box_render
                .draw(glt, &(projection_view * bounding_box_model));
        }
    }
    pub fn delete(self, glt: GLToken) {
        self.render.delete(glt);
        self.bounding_box_render.delete(glt);
    }
}

pub struct PlayerRender {
    vao: VAO,
    vertex_vbo: VBO<f32>,
    texture_vbo: VBO<f32>,
    index_buffer: IndexBuffer,
    body_ranges: HashMap<String, (u32, u32)>,
    texture: GLuint,
    num_indices: usize,
}

impl PlayerRender {
    pub fn new(glt: GLToken) -> Self {
        let mut vao = VAO::new(glt);
        let mut vertex_vbo = VBO::new(glt);
        let mut texture_vbo = VBO::new(glt);
        let mut index_buffer = IndexBuffer::new(glt);

        let mut texture: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
        }
        assert!(texture != 0);

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, texture);
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_WRAP_S,
                gl::TEXTURE_WRAP_S as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_WRAP_T,
                gl::TEXTURE_WRAP_T as i32,
            );

            gl::TexStorage3D(
                gl::TEXTURE_2D_ARRAY,
                4, // mipmap level
                gl::RGBA8,
                1024,
                1024,
                18,
            );
        }

        let pngs = ('a'..='r')
            .map(|c| format!("textures/players/texture-{}.png", c))
            .collect::<Vec<_>>();

        for (i, png) in pngs.iter().enumerate() {
            let mut image = image::open(png).expect("Failed to open image").to_rgba8();
            image::imageops::flip_vertical_in_place(&mut image);
            assert_eq!(image.width(), 1024);
            assert_eq!(image.height(), 1024);

            unsafe {
                gl::TexSubImage3D(
                    gl::TEXTURE_2D_ARRAY,
                    0, // mipmap level
                    0,
                    0,
                    i as i32,
                    image.width() as i32,
                    image.height() as i32,
                    1, // layer
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    image.as_ptr().cast(),
                );
            }
        }

        unsafe {
            gl::GenerateMipmap(gl::TEXTURE_2D_ARRAY);
        }

        vao.attrib_pointer(glt, 0, &vertex_vbo, 3, 0, 0, false);
        vao.attrib_pointer(glt, 1, &texture_vbo, 3, 0, 0, false);
        vao.enable_array(glt, 0);
        vao.enable_array(glt, 1);

        let model = BufReader::new(
            File::open("textures/players/model.obj").expect("Failed to open player model"),
        );

        let raw_obj = obj::raw::parse_obj(model).expect("Failed to parse player model");

        let body_ranges = [
            ("head", extract_group_range(&raw_obj, "head")),
            ("torso", extract_group_range(&raw_obj, "torso")),
            ("arm-left", extract_group_range(&raw_obj, "arm-left")),
            ("arm-right", extract_group_range(&raw_obj, "arm-right")),
            ("leg-left", extract_group_range(&raw_obj, "leg-left")),
            ("leg-right", extract_group_range(&raw_obj, "leg-right")),
        ]
        .iter()
        .map(|(name, range)| (name.to_string(), *range))
        .collect::<HashMap<String, (u32, u32)>>();

        let model_obj = obj::Obj::<TexturedVertex, u32>::new(raw_obj).unwrap();

        let vertex_data = model_obj
            .vertices
            .iter()
            .flat_map(|v: &TexturedVertex| v.position)
            .collect::<Vec<_>>();
        let texture_data = model_obj
            .vertices
            .iter()
            .flat_map(|v: &TexturedVertex| [v.texture[0], v.texture[1], 0.0])
            .collect::<Vec<_>>();

        vertex_vbo.copy(glt, &vertex_data);
        texture_vbo.copy(glt, &texture_data);
        index_buffer.copy(glt, &model_obj.indices);

        dbg!(body_ranges.clone());

        Self {
            vao,
            vertex_vbo,
            texture_vbo,
            index_buffer,
            body_ranges,
            texture,
            num_indices: model_obj.indices.len(),
        }
    }

    pub fn delete(self, glt: GLToken) {
        self.vao.delete(glt);
        self.vertex_vbo.delete(glt);
        self.texture_vbo.delete(glt);
    }
}
