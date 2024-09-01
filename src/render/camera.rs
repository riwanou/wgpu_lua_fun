use glam::{Mat4, Vec3};

use crate::transform::Transform;

#[derive(Debug)]
pub struct Camera {
    pub fovy: f32,
    pub transform: Transform,
    pub zfar: f32,
    pub znear: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            fovy: 45.0,
            transform: Transform::from_pos(Vec3::new(0.0, 0.0, 2.0)),
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
        self.transform.build_matrix().inverse()
    }
}
