use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    time::Instant,
};

use anyhow::Result;
use assets_manager::{loader, Asset, AssetCache};
use log::{error, info};

use crate::app::{get_pool, RELOAD_DEBOUNCE};

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

type LoadResult = Result<(String, String)>;

pub struct ShaderAssets {
    cache: Arc<AssetCache>,
    last_reload: Instant,
    load_rx: Receiver<LoadResult>,
    load_tx: Sender<LoadResult>,
    modules: HashMap<String, wgpu::ShaderModule>,
    pub frame_reloaded: Option<String>,
}

impl ShaderAssets {
    pub fn new() -> Self {
        let (load_tx, load_rx) = channel();
        Self {
            cache: Arc::new(AssetCache::new("assets/shaders").unwrap()),
            last_reload: Instant::now(),
            modules: HashMap::new(),
            load_rx,
            load_tx,
            frame_reloaded: None,
        }
    }

    pub fn hot_reload(&mut self, device: &wgpu::Device) {
        self.frame_reloaded = None;
        self.cache.hot_reload();

        let keys = self.modules.keys().cloned().collect::<Vec<_>>();
        for shader_id in keys {
            let handle = self.cache.load_expect::<WgslSource>(&shader_id);
            if self.last_reload.elapsed() >= RELOAD_DEBOUNCE
                && handle.reloaded_global()
            {
                self.last_reload = Instant::now();
                self.load_internal(&shader_id);
            }
        }

        if let Ok(result) = self.load_rx.try_recv() {
            match result {
                Ok((shader_id, source)) => {
                    info!("Shader loaded: {}", shader_id);
                    let module = device.create_shader_module(
                        wgpu::ShaderModuleDescriptor {
                            label: Some(&format!("{}_module", shader_id)),
                            source: wgpu::ShaderSource::Wgsl(source.into()),
                        },
                    );
                    self.frame_reloaded = Some(shader_id.clone());
                    self.modules.insert(shader_id, module);
                }
                Err(err) => {
                    error!("load\n{:?}", err);
                }
            };
        }
    }

    pub fn reloaded(&mut self, shader_id: &str) -> bool {
        let handle = self.cache.load_expect::<WgslSource>(shader_id);
        if self.last_reload.elapsed() >= RELOAD_DEBOUNCE {
            self.last_reload = Instant::now();
            return handle.reloaded_global();
        }
        false
    }

    pub fn get(&self, shader_id: &str) -> Option<&wgpu::ShaderModule> {
        self.modules.get(shader_id)
    }

    fn load_internal(&mut self, shader_id: &str) {
        let cache = self.cache.clone();
        let module_id = shader_id.to_string();
        let load_tx = self.load_tx.clone();

        get_pool().execute(move || {
            let result = (|| {
                let source = cache.load::<WgslSource>(&module_id)?.read();
                Ok((module_id, source.0.clone()))
            })();
            load_tx.send(result).unwrap();
        });
    }

    pub fn load(&mut self, shader_id: &str) {
        if self.cache.contains::<WgslSource>(shader_id) {
            return;
        }
        self.load_internal(shader_id);
    }
}
