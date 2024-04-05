use glm::Mat4;
use nalgebra_glm as glm;

pub trait Camera {
    fn position(&self) -> [f64; 3];
    fn pitch(&self) -> f32;
    fn yaw(&self) -> f32;
    fn view_matrix(&self) -> Mat4;
    fn view_direction(&self) -> glm::Vec3;

    fn change_pitch(&mut self, diff: f32);
    fn change_yaw(&mut self, diff: f32);

    /// z direction
    fn go_forward(&mut self, diff: f32);
    /// x direction
    fn go_left(&mut self, diff: f32);
    /// y direction
    fn go_up(&mut self, diff: f32);
}
#[derive(Debug, Clone)]
pub struct FreeCamera {
    pos: [f64; 3],
    pitch: f32,
    yaw: f32,
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
    fn position(&self) -> [f64; 3] {
        self.pos
    }
    fn pitch(&self) -> f32 {
        self.pitch
    }
    fn yaw(&self) -> f32 {
        self.yaw
    }

    fn change_pitch(&mut self, diff: f32) {
        self.pitch = (self.pitch + diff).clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
    }

    fn change_yaw(&mut self, diff: f32) {
        self.yaw = (self.yaw + diff) % std::f32::consts::TAU;
    }

    fn go_forward(&mut self, diff: f32) {
        self.pos[2] -= (diff * self.yaw.cos()) as f64;
        self.pos[0] += (diff * self.yaw.sin()) as f64;
    }

    fn go_left(&mut self, diff: f32) {
        self.pos[2] -= (diff * self.yaw.sin()) as f64;
        self.pos[0] -= (diff * self.yaw.cos()) as f64;
    }

    fn go_up(&mut self, diff: f32) {
        self.pos[1] += diff as f64;
    }

    fn view_matrix(&self) -> Mat4 {
        let pitch = glm::rotation(self.pitch, &glm::vec3(1.0, 0.0, 0.0));
        let yaw = glm::rotation(self.yaw, &glm::vec3(0.0, 1.0, 0.0));
        pitch * yaw
    }

    fn view_direction(&self) -> glm::Vec3 {
        // TODO optimize this
        let orig_view_vec = glm::vec4(0.0,0.0,-1.0,1.0);
        let pitch = glm::rotation(-self.pitch, &glm::vec3(1.0, 0.0, 0.0));
        let yaw = glm::rotation(-self.yaw, &glm::vec3(0.0, 1.0, 0.0));
        (yaw * pitch * orig_view_vec).xyz()
    }
}

impl FreeCamera {
    pub fn inverse_view_matrix(&self) -> Mat4 {
        let pitch = glm::rotation(-self.pitch, &glm::vec3(1.0, 0.0, 0.0));
        let yaw = glm::rotation(-self.yaw, &glm::vec3(0.0, 1.0, 0.0));
        yaw * pitch
    }
}