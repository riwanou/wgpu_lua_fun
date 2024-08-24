use log::info;

use crate::render::shader::ShaderAssets;

use super::Layouts;

pub struct Bundle {
    pub pipeline: Pipeline,
}

impl Bundle {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layouts: &Layouts,
        shaders: &mut ShaderAssets,
    ) -> Self {
        Self {
            pipeline: Pipeline::new(device, config, layouts, shaders),
        }
    }

    pub fn hot_reload(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layouts: &Layouts,
        shaders: &mut ShaderAssets,
    ) {
        if shaders.reloaded(&self.pipeline.shader_id) {
            info!(
                "Reloading triangle pipeline from {}.wgsl",
                self.pipeline.shader_id
            );
            self.pipeline = Pipeline::new(device, config, layouts, shaders);
        }
    }
}

pub struct Pipeline {
    pub pipeline: wgpu::RenderPipeline,
    shader_id: String,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layouts: &Layouts,
        shaders: &mut ShaderAssets,
    ) -> Self {
        let shader_id = "triangle".to_string();
        let module = shaders.load_module(&shader_id, device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("triangle_layout"),
                bind_group_layouts: &[&layouts.globals.layout],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("triangle_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: "vs_main",
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: "fs_main",
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        Self {
            pipeline,
            shader_id,
        }
    }
}
