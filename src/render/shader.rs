use std::{
    collections::{HashMap, HashSet},
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

type LoadResult = (String, Result<String>);

pub struct ShaderAssets {
    cache: Arc<AssetCache>,
    pub frame_reloaded: Option<String>,
    last_reload: Instant,
    load_rx: Receiver<LoadResult>,
    load_tx: Sender<LoadResult>,
    loaded: HashSet<String>,
    modules: HashMap<String, wgpu::ShaderModule>,
}

impl ShaderAssets {
    pub fn new() -> Self {
        let (load_tx, load_rx) = channel();
        Self {
            cache: Arc::new(AssetCache::new("assets/shaders").unwrap()),
            frame_reloaded: None,
            last_reload: Instant::now(),
            load_rx,
            load_tx,
            loaded: HashSet::new(),
            modules: HashMap::new(),
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

        if let Ok((shader_id, result)) = self.load_rx.try_recv() {
            match result {
                Ok(source) => {
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
                    self.loaded.remove(&shader_id);
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
                Ok(source.0.clone())
            })();
            load_tx.send((module_id, result)).unwrap();
        });
    }

    pub fn load(&mut self, shader_id: &str) {
        if self.loaded.contains(shader_id) {
            return;
        }
        self.loaded.insert(shader_id.to_string());
        self.load_internal(shader_id);
    }
}
