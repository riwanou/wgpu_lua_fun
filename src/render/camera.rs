use glam::{Mat4, Vec3};

use crate::{lua::shared::Shared, transform::Transform};

#[derive(Debug)]
pub struct Camera {
    pub fovy: f32,
    pub transform: Shared<Transform>,
    pub zfar: f32,
    pub znear: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            fovy: 45.0,
            transform: Shared::new(Transform::from_pos(Shared::new(
                Vec3::new(0.0, 0.0, 2.0),
            ))),
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
        self.transform.borrow().build_matrix().inverse()
    }
}
