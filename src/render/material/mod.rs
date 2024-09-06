use std::{any::Any, collections::HashMap};

use encase::{internal::WriteInto, ShaderType};

pub mod simple;

pub trait Material: 'static {
    type Uniform: ShaderType + WriteInto;

    fn shader_id(&self) -> String;
    fn texture_id(&self) -> String;
    fn uniform_data(&self) -> &Self::Uniform;
}

type GetShaderId = Box<dyn Fn(&Box<dyn Any>) -> String>;
type GetTextureId = Box<dyn Fn(&Box<dyn Any>) -> String>;
type GetUniformDataBytes = Box<dyn Fn(&Box<dyn Any>) -> Vec<u8>>;

pub struct InternalMaterial {
    get_shader_id: GetShaderId,
    get_texture_id: GetTextureId,
    get_uniform_data_bytes: GetUniformDataBytes,
    material: Box<dyn Any>,
}

impl InternalMaterial {
    fn new<M: Material>(material: M) -> Self {
        let get_shader_id: GetShaderId = Box::new(|any| {
            let material = any.downcast_ref::<M>().unwrap();
            material.shader_id()
        });
        let get_texture_id: GetTextureId = Box::new(|any| {
            let material = any.downcast_ref::<M>().unwrap();
            material.texture_id()
        });
        let get_uniform_data_bytes: GetUniformDataBytes = Box::new(|any| {
            let material = any.downcast_ref::<M>().unwrap();
            let mut buffer = encase::UniformBuffer::new(Vec::<u8>::new());
            buffer.write(&material.uniform_data()).unwrap();
            buffer.into_inner()
        });
        Self {
            get_shader_id,
            get_texture_id,
            get_uniform_data_bytes,
            material: Box::new(material),
        }
    }
}

pub struct MaterialManager {
    materials: HashMap<String, InternalMaterial>,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
        }
    }

    pub fn add<M: Material>(&mut self, key: &str, material: M) {
        self.materials
            .insert(key.to_string(), InternalMaterial::new(material));
    }

    pub fn get_shader_id(&self, key: &str) -> Option<String> {
        self.materials
            .get(key)
            .map(|data| (data.get_shader_id)(&data.material))
    }

    pub fn get_texture_id(&self, key: &str) -> Option<String> {
        self.materials
            .get(key)
            .map(|data| (data.get_texture_id)(&data.material))
    }

    pub fn get_uniform_data_bytes(&self, key: &str) -> Option<Vec<u8>> {
        self.materials
            .get(key)
            .map(|data| (data.get_uniform_data_bytes)(&data.material))
    }

    pub fn get_any(&self, key: &str) -> Option<&Box<dyn Any>> {
        self.materials.get(key).map(|data| &data.material)
    }

    pub fn get_mut_any(&mut self, key: &str) -> Option<&mut Box<dyn Any>> {
        self.materials.get_mut(key).map(|data| &mut data.material)
    }
}
