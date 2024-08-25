use super::shader::ShaderAssets;

pub mod globals;
pub mod model;

pub struct Layouts {
    globals: globals::Layout,
}

impl Layouts {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            globals: globals::Layout::new(device),
        }
    }
}

pub struct Bundles {
    pub globals: globals::Bundle,
    pub model: model::Bundle,
}

impl Bundles {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layouts: &Layouts,
        shaders: &mut ShaderAssets,
    ) -> Self {
        Self {
            globals: globals::Bundle::new(device, layouts),
            model: model::Bundle::new(device, config, layouts, shaders),
        }
    }

    pub fn hot_reload(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layouts: &Layouts,
        shaders: &mut ShaderAssets,
    ) {
        self.model.hot_reload(device, config, layouts, shaders);
    }
}
