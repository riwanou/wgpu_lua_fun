use super::shader::ShaderAssets;

pub mod triangle;

pub struct Layouts {
    triangle: triangle::Layout,
}

impl Layouts {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            triangle: triangle::Layout::new(device),
        }
    }
}

pub struct Bundles {
    pub triangle: triangle::Bundle,
}

impl Bundles {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layouts: &Layouts,
        shaders: &mut ShaderAssets,
    ) -> Self {
        Self {
            triangle: triangle::Bundle::new(device, config, layouts, shaders),
        }
    }

    pub fn hot_reload(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layouts: &Layouts,
        shaders: &mut ShaderAssets,
    ) {
        self.triangle.hot_reload(device, config, layouts, shaders);
    }
}
