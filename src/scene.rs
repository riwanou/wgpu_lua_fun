use crate::render::{
    bundle::model::{self, Batches},
    camera::Camera,
};

pub struct Scene {
    pub camera: Camera,
    pub mesh_id: String,
    pub _model_batches: model::Batches,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(),
            mesh_id: "cube".to_string(),
            _model_batches: Batches::default(),
        }
    }
}
