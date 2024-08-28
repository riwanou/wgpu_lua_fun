use crate::render::{
    bundle::model::{self, Batches},
    camera::Camera,
};

pub struct Scene {
    pub camera: Camera,
    pub model_batches: model::Batches,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(),
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
