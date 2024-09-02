use std::{fmt, sync::Arc};

use winit::{dpi::PhysicalSize, window::Window};

use crate::scene::Scene;

use super::{
    bundle::{Bundles, Layouts},
    mesh::MeshAssets,
    shader::ShaderAssets,
    texture::{Texture, TextureAssets},
};

pub struct RenderState {
    _adapter: wgpu::Adapter,
    _instance: wgpu::Instance,
    bundles: Bundles,
    config: wgpu::SurfaceConfiguration,
    pub depth: Texture,
    pub device: wgpu::Device,
    layouts: Layouts,
    pub meshes: MeshAssets,
    queue: wgpu::Queue,
    shaders: ShaderAssets,
    surface: wgpu::Surface<'static>,
    pub textures: TextureAssets,
}

impl fmt::Debug for RenderState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RenderState")
    }
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let size = window.inner_size();
        let surface = instance.create_surface(window.clone()).unwrap();

        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        let mut shaders = ShaderAssets::new();
        let mut textures = TextureAssets::new();
        let meshes = MeshAssets::new();
        let layouts = Layouts::new(&device);
        let bundles = Bundles::new(
            &device,
            &config,
            &layouts,
            &mut shaders,
            &mut textures,
        );
        let depth = Texture::create_depth(&device, &config);

        Self {
            _adapter: adapter,
            bundles,
            config,
            depth,
            device,
            _instance: instance,
            layouts,
            meshes,
            queue,
            shaders,
            surface,
            textures,
        }
    }

    pub fn hot_reload(&mut self) {
        self.shaders.hot_reload();
        self.meshes.hot_reload(&self.device);
        self.textures.hot_reload(&self.device, &self.queue);
        self.bundles.hot_reload(
            &self.device,
            &self.config,
            &self.layouts,
            &mut self.shaders,
        );
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth = Texture::create_depth(&self.device, &self.config);
    }

    pub fn render(&mut self, elapsed: f32, scene: &mut Scene) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );

        self.bundles.globals.prepare(
            &self.queue,
            &self.config,
            elapsed,
            &scene.camera,
        );
        scene.prepare(&self.device, &self.layouts, &self.textures);

        {
            let mut rpass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.03,
                                    g: 0.03,
                                    b: 0.03,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachment {
                            view: &self.depth.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        },
                    ),
                    ..Default::default()
                });

            rpass.set_pipeline(&self.bundles.model.pipeline.pipeline);
            rpass.set_bind_group(0, &self.bundles.globals.bind_group, &[]);
            scene.model_batches.render(&mut rpass, &self.meshes);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
