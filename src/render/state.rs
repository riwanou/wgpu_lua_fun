use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use super::{
    bundle::{model, Bundles, Layouts},
    camera::Camera,
    mesh::Mesh,
    shader::ShaderAssets,
};

pub struct Scene {
    pub camera: Camera,
    mesh: Mesh,
}

impl Scene {
    pub fn new(device: &wgpu::Device) -> Self {
        let mesh = Mesh::new(
            device,
            &[
                model::Vertex {
                    position: [0.0, 0.5, 0.0],
                    tex_coord: [0.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                },
                model::Vertex {
                    position: [-0.5, -0.5, 0.0],
                    tex_coord: [0.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                },
                model::Vertex {
                    position: [0.5, -0.5, 0.0],
                    tex_coord: [0.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                },
            ],
            &[0, 1, 2],
        );
        Self {
            camera: Camera::new(),
            mesh,
        }
    }
}

pub struct RenderState {
    _adapter: wgpu::Adapter,
    _instance: wgpu::Instance,
    bundles: Bundles,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    layouts: Layouts,
    queue: wgpu::Queue,
    pub scene: Scene,
    shaders: ShaderAssets,
    surface: wgpu::Surface<'static>,
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
        let layouts = Layouts::new(&device);
        let bundles = Bundles::new(&device, &config, &layouts, &mut shaders);
        let scene = Scene::new(&device);

        Self {
            _adapter: adapter,
            bundles,
            config,
            device,
            _instance: instance,
            layouts,
            queue,
            scene,
            shaders,
            surface,
        }
    }

    pub fn hot_reload(&mut self) {
        self.shaders.hot_reload();
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
    }

    pub fn prepare(&mut self, elapsed: f32) {
        self.bundles.globals.prepare(
            &self.queue,
            &self.config,
            &self.scene,
            elapsed,
        );
    }

    pub fn render(&mut self) {
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
                    ..Default::default()
                });

            rpass.set_pipeline(&self.bundles.model.pipeline.pipeline);

            rpass.set_bind_group(0, &self.bundles.globals.bind_group, &[]);
            rpass.set_vertex_buffer(0, self.scene.mesh.vertex_buffer.slice(..));
            rpass.set_index_buffer(
                self.scene.mesh.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );

            rpass.draw_indexed(0..self.scene.mesh.num_indices, 0, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
