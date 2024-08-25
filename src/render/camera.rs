use glam::{Mat4, Quat, Vec3};

#[derive(Debug)]
pub struct Camera {
    pub fovy: f32,
    pub rot: Quat,
    pub pos: Vec3,
    pub zfar: f32,
    pub znear: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            fovy: 45.0,
            rot: Quat::IDENTITY,
            pos: Vec3::new(0.0, 0.0, 2.0),
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn build_projection(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh(
            self.fovy.to_radians(),
            aspect_ratio,
            self.znear,
            self.zfar,
        )
    }

    pub fn build_view(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rot, self.pos).inverse()
    }
}
