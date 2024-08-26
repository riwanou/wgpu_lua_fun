use std::mem;

use bytemuck::{Pod, Zeroable};
use log::info;

use crate::render::{mesh::VertexTrait, shader::ShaderAssets};

use super::Layouts;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
    pub normal: [f32; 3],
}

impl VertexTrait for Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x2,
            2 => Float32x3
        ];
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

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
                "Reloading model pipeline from {}.wgsl",
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
        let shader_id = "model".to_string();
        let module = shaders.load(&shader_id, device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("model_layout"),
                bind_group_layouts: &[&layouts.globals.layout],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("model_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
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
