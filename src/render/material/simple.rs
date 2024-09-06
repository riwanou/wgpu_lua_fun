use encase::ShaderType;
use glam::Vec3;

use super::Material;

#[derive(ShaderType, Debug)]
pub struct Uniform {
    pub color: Vec3,
}

pub struct SimpleMaterial {
    shader_id: String,
    texture_id: String,
    pub uniform: Uniform,
}

impl SimpleMaterial {
    pub fn new(shader_id: &str, texture_id: &str) -> Self {
        Self {
            shader_id: shader_id.to_string(),
            texture_id: texture_id.to_string(),
            uniform: Uniform {
                color: Vec3::new(1.0, 0.2, 0.3),
            },
        }
    }
}

impl Material for SimpleMaterial {
    type Uniform = Uniform;

    fn shader_id(&self) -> String {
        self.shader_id.clone()
    }

    fn texture_id(&self) -> String {
        self.texture_id.clone()
    }

    fn uniform_data(&self) -> &Self::Uniform {
        &self.uniform
    }
}
