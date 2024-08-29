use glam::{Mat3, Mat4, Quat, Vec3};

use crate::lua::shared::Shared;

#[derive(Debug, Clone)]
pub struct Transform {
    pub pos: Shared<Vec3>,
    pub rot: Quat,
    pub scale: Shared<Vec3>,
    pub up: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: Shared::new(Vec3::ZERO),
            rot: Quat::IDENTITY,
            scale: Shared::new(Vec3::splat(1.0)),
            up: Vec3::Y,
        }
    }
}

impl Transform {
    pub fn build_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            *self.scale.borrow(),
            self.rot,
            *self.pos.borrow(),
        )
    }

    pub fn from_pos(pos: Shared<Vec3>) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }

    pub fn rotate(&mut self, axis: Vec3, angle: f32) {
        self.rot *= Quat::from_axis_angle(axis, angle);
    }

    pub fn rotate_x(&mut self, angle: f32) {
        self.rotate(Vec3::X, angle);
    }

    pub fn rotate_y(&mut self, angle: f32) {
        self.rotate(Vec3::Y, angle);
    }

    pub fn rotate_z(&mut self, angle: f32) {
        self.rotate(Vec3::Z, angle);
    }

    pub fn look_at(&mut self, target: Vec3) {
        let forward = (*self.pos.borrow() - target).normalize();
        self.look_to(forward);
    }

    pub fn look_to(&mut self, forward: Vec3) {
        let right = self.up.cross(forward).normalize();
        let local_up = forward.cross(right);
        self.rot = Quat::from_mat3(&Mat3::from_cols(right, local_up, forward));
    }
}
