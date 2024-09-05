use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    time::Instant,
};

use anyhow::Result;
use assets_manager::{loader::Loader, Asset, AssetCache, BoxedError};
use image::{DynamicImage, GenericImageView};
use log::{error, info};

use crate::app::{get_pool, RELOAD_DEBOUNCE};

pub struct Texture {
    pub sampler: wgpu::Sampler,
    pub view: wgpu::TextureView,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat =
        wgpu::TextureFormat::Depth32Float;

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &DynamicImage,
    ) -> Self {
        let rgba = image.to_rgba8();
        let dimensions = image.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("image_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            ..Default::default()
        });

        queue.write_texture(
            texture.as_image_copy(),
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        Self { sampler, view }
    }

    pub fn create_depth(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self { sampler, view }
    }
}

pub struct Image(DynamicImage);

pub struct ImageLoader;
impl Loader<Image> for ImageLoader {
    fn load(content: Cow<[u8]>, _ext: &str) -> Result<Image, BoxedError> {
        Ok(Image(image::load_from_memory(&content)?))
    }
}

impl Asset for Image {
    const EXTENSIONS: &'static [&'static str] = &["jpeg", "png"];
    type Loader = ImageLoader;
}

type LoadResult = Result<(String, Box<DynamicImage>)>;

pub struct TextureAssets {
    cache: Arc<AssetCache>,
    last_reload: Instant,
    load_rx: Receiver<LoadResult>,
    load_tx: Sender<LoadResult>,
    textures: HashMap<String, Texture>,
}

impl TextureAssets {
    pub fn new() -> Self {
        let (load_tx, load_rx) = channel();
        Self {
            cache: Arc::new(AssetCache::new("assets/textures").unwrap()),
            last_reload: Instant::now(),
            load_rx,
            load_tx,
            textures: HashMap::new(),
        }
    }

    pub fn hot_reload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.cache.hot_reload();

        let keys = self.textures.keys().cloned().collect::<Vec<_>>();
        for texture_id in keys {
            let handle = self.cache.load_expect::<Image>(&texture_id);
            if self.last_reload.elapsed() >= RELOAD_DEBOUNCE
                && handle.reloaded_global()
            {
                self.last_reload = Instant::now();
                self.load_internal(&texture_id);
            }
        }

        if let Ok(result) = self.load_rx.try_recv() {
            match result {
                Ok((texture_id, image)) => {
                    info!("Texture loaded: {}", texture_id);
                    self.textures.insert(
                        texture_id,
                        Texture::from_image(device, queue, &image),
                    );
                }
                Err(err) => {
                    error!("load\n{:?}", err);
                }
            };
        }
    }

    pub fn get(&self, texture_id: &str) -> Option<&Texture> {
        self.textures.get(texture_id)
    }

    fn load_internal(&mut self, texture_id: &str) {
        let cache = self.cache.clone();
        let texture_id = texture_id.to_string();
        let load_tx = self.load_tx.clone();

        get_pool().execute(move || {
            let result = (|| {
                let handle = cache.load::<Image>(&texture_id)?;
                let data = handle.read().0.clone();
                Ok((texture_id, Box::new(data)))
            })();
            load_tx.send(result).unwrap();
        });
    }

    pub fn load(&mut self, texture_id: &str) {
        if self.cache.contains::<Image>(texture_id) {
            return;
        }
        self.load_internal(texture_id);
    }
}
