use glm::Mat4;
use nalgebra_glm as glm;

pub trait Camera {
    fn position(&self) -> [f64; 3];
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
}

impl Camera for FreeCamera {
    fn position(&self) -> [f64; 3] {
        self.pos
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
        let orig_view_vec = glm::vec4(0.0,0.0,-1.0,1.0);
        let pitch = glm::rotation(-self.pitch, &glm::vec3(1.0, 0.0, 0.0));
        let yaw = glm::rotation(-self.yaw, &glm::vec3(0.0, 1.0, 0.0));
        (yaw * pitch * orig_view_vec).xyz()
    }
}
