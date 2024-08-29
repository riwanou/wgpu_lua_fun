use std::fmt;

use crate::{
    lua::shared::Shared,
    render::{
        bundle::model::{self, Batches},
        camera::Camera,
    },
};

pub struct Scene {
    pub camera: Shared<Camera>,
    pub model_batches: model::Batches,
}

impl fmt::Debug for Scene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scene")
    }
}

impl Scene {
    pub fn new() -> Self {
        Self {
            camera: Shared::new(Camera::new()),
            model_batches: Batches::default(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.model_batches.clear();
    }

    pub fn prepare(&mut self, device: &wgpu::Device) {
        self.model_batches.prepare(device);
    }
}
