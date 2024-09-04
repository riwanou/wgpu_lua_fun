use std::fmt;

use crate::render::{
    bundle::{
        lights,
        model::{self, Batches},
    },
    camera::Camera,
};

pub struct Scene {
    pub camera: Camera,
    pub model_batches: model::Batches,
    pub point_lights: Vec<lights::PointLight>,
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
            point_lights: Vec::new(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.model_batches.clear();
        self.point_lights.clear();
    }
}
