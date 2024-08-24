use std::{ops::Deref, time::Instant};

use assets_manager::{loader, Asset, AssetCache};

use crate::app::RELOAD_DEBOUNCE;

pub struct WgslSource(String);

impl From<String> for WgslSource {
    fn from(value: String) -> Self {
        WgslSource(value)
    }
}

impl Asset for WgslSource {
    const EXTENSION: &'static str = "wgsl";
    type Loader = loader::LoadFrom<String, loader::StringLoader>;
}

pub struct ShaderAssets {
    cache: AssetCache,
    last_reload: Instant,
}

impl ShaderAssets {
    pub fn new() -> Self {
        Self {
            cache: AssetCache::new("assets/shaders").unwrap(),
            last_reload: Instant::now(),
        }
    }

    pub fn hot_reload(&mut self) {
        self.cache.hot_reload();
    }

    pub fn reloaded(&mut self, shader_id: &str) -> bool {
        let handle = self.cache.load_expect::<WgslSource>(shader_id);
        if self.last_reload.elapsed() >= RELOAD_DEBOUNCE {
            self.last_reload = Instant::now();
            return handle.reloaded_global();
        }
        false
    }

    pub fn load_module(
        &self,
        shader_id: &str,
        device: &wgpu::Device,
    ) -> wgpu::ShaderModule {
        let source = self.cache.load_expect::<WgslSource>(shader_id).read();
        let module =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(shader_id),
                source: wgpu::ShaderSource::Wgsl(source.0.deref().into()),
            });
        module
    }
}
