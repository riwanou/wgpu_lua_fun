use glam::{Mat3, Mat4, Quat, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
    pub up: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            rot: Quat::IDENTITY,
            scale: Vec3::splat(1.0),
            up: Vec3::Y,
        }
    }
}

impl Transform {
    pub fn build_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rot, self.pos)
    }

    pub fn from_pos(pos: Vec3) -> Self {
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
        let forward = (self.pos - target).normalize();
        self.look_to(forward);
    }

    pub fn look_to(&mut self, forward: Vec3) {
        let right = self.up.cross(forward).normalize();
        let local_up = forward.cross(right);
        self.rot = Quat::from_mat3(&Mat3::from_cols(right, local_up, forward));
    }
}
