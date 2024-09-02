use std::fmt;

use crate::render::{
    bundle::{
        model::{self, Batches},
        Layouts,
    },
    camera::Camera,
    texture::TextureAssets,
};

pub struct Scene {
    pub camera: Camera,
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
            camera: Camera::new(),
            model_batches: Batches::default(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.model_batches.clear();
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        layouts: &Layouts,
        textures: &TextureAssets,
    ) {
        self.model_batches.prepare(device, layouts, textures);
    }
}
