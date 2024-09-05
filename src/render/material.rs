use std::{any::Any, collections::HashMap};

use encase::ShaderType;

pub trait Material: 'static {
    type Uniform: ShaderType;

    fn shader() -> String;
    fn uniform_data(&self) -> Self::Uniform;
}

pub struct MaterialManager {
    materials: HashMap<String, Box<dyn Any>>,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
        }
    }

    pub fn add<M: Material>(&mut self, material: M) {
        self.materials.insert(M::shader(), Box::new(material));
    }

    pub fn get<M: Material>(&self) -> Option<&M> {
        self.materials
            .get(&M::shader())
            .and_then(|boxed_material| boxed_material.downcast_ref::<M>())
    }

    pub fn get_mut<M: Material>(&mut self) -> Option<&mut M> {
        self.materials
            .get_mut(&M::shader())
            .and_then(|boxed_material| boxed_material.downcast_mut::<M>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(ShaderType, Debug, PartialEq, Eq)]
    struct SimpleUniform {
        bloup: u32,
    }

    struct SimpleMaterial {
        foo: u32,
    }

    impl Material for SimpleMaterial {
        type Uniform = SimpleUniform;

        fn shader() -> String {
            "foo".to_string()
        }

        fn uniform_data(&self) -> Self::Uniform {
            SimpleUniform { bloup: self.foo }
        }
    }

    #[test]
    fn test_material() {
        let material = SimpleMaterial { foo: 0 };
        assert_eq!(SimpleMaterial::shader(), "foo");
        assert_eq!(material.uniform_data(), SimpleUniform { bloup: 0 });

        let mut manager = MaterialManager::new();
        manager.add(material);

        let material = manager.get_mut::<SimpleMaterial>().unwrap();
        assert_eq!(material.uniform_data(), SimpleUniform { bloup: 0 });
        material.foo = 42;

        let material = manager.get::<SimpleMaterial>().unwrap();
        assert_eq!(material.uniform_data(), SimpleUniform { bloup: 42 });
    }
}
