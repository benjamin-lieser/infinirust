use core::f32;

use glm::Mat4;
use nalgebra_glm as glm;

/// Anything that can be used as a camera in the game
pub trait Camera {
    fn camera_position(&self) -> [f64; 3];
    fn pitch(&self) -> f32;
    fn yaw(&self) -> f32;
    fn change_pitch(&mut self, diff: f32);
    fn change_yaw(&mut self, diff: f32);
    fn view_matrix(&self) -> Mat4 {
        let pitch = glm::rotation(self.pitch(), &glm::vec3(1.0, 0.0, 0.0));
        let yaw = glm::rotation(self.yaw(), &glm::vec3(0.0, 1.0, 0.0));
        pitch * yaw
    }

    fn view_direction(&self) -> glm::Vec3 {
        // TODO optimize this
        let orig_view_vec = glm::vec4(0.0, 0.0, -1.0, 1.0);
        let pitch = glm::rotation(-self.pitch(), &glm::vec3(1.0, 0.0, 0.0));
        let yaw = glm::rotation(-self.yaw(), &glm::vec3(0.0, 1.0, 0.0));
        (yaw * pitch * orig_view_vec).xyz()
    }
    fn inverse_view_matrix(&self) -> Mat4 {
        let pitch = glm::rotation(self.pitch(), &glm::vec3(1.0, 0.0, 0.0));
        let yaw = glm::rotation(-self.yaw() + f32::consts::PI, &glm::vec3(0.0, 1.0, 0.0));
        yaw * pitch
    }

    fn clone_into_free_camera(&self) -> FreeCamera {
        FreeCamera {
            pos: self.camera_position(),
            pitch: self.pitch(),
            yaw: self.yaw(),
        }
    }

    fn forward_dir(&self) -> glm::Vec3 {
        let yaw = self.yaw();
        glm::vec3(yaw.sin(), 0.0, -yaw.cos())
    }

    fn left_dir(&self) -> glm::Vec3 {
        let yaw = self.yaw();
        glm::vec3(-yaw.cos(), 0.0, -yaw.sin())
    }
}

#[derive(Debug, Clone)]
pub struct FreeCamera {
    pub pos: [f64; 3],
    pub pitch: f32,
    pub yaw: f32,
}

impl FreeCamera {
    pub fn new(pos: [f64; 3]) -> FreeCamera {
        FreeCamera {
            pos,
            pitch: 0.0,
            yaw: 0.0,
        }
    }

    pub fn update(&mut self, pos: [f64; 3], pitch: f32, yaw: f32) {
        self.pos = pos;
        self.pitch = pitch;
        self.yaw = yaw;
    }
}

impl Camera for FreeCamera {
    fn camera_position(&self) -> [f64; 3] {
        self.pos
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
