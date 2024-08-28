use std::{collections::HashMap, mem};

use bytemuck::{Pod, Zeroable};
use encase::ShaderType;
use glam::{Mat3, Mat4, Quat};
use log::info;
use wgpu::util::DeviceExt;

use crate::render::{
    mesh::{MeshAssets, VertexTrait},
    shader::ShaderAssets,
};

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
                "Reloading model pipeline from {}.wgsl",
                self.pipeline.shader_id
            );
            self.pipeline = Pipeline::new(device, config, layouts, shaders);
        }
    }
}

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

#[derive(ShaderType)]
pub struct Instance {
    pub world_local: Mat4,
    pub normal: Mat3,
}

impl Instance {
    pub fn new(transform: Mat4, rotation: Quat) -> Self {
        Self {
            world_local: transform,
            normal: Mat3::from_quat(rotation),
        }
    }

    fn slice_as_bytes(instances: &[Instance]) -> Vec<u8> {
        let mut buffer = encase::StorageBuffer::new(Vec::<u8>::new());
        buffer.write(instances).unwrap();
        buffer.into_inner()
    }

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
            3 => Float32x4,
            4 => Float32x4,
            5 => Float32x4,
            6 => Float32x4,
            7 => Float32x3,
            8 => Float32x3,
            9 => Float32x3
        ];
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}

#[derive(Default)]
struct InstanceArray {
    data: Vec<Instance>,
    buffer: Option<wgpu::Buffer>,
}

#[derive(Default)]
pub struct Batches {
    instances: HashMap<String, InstanceArray>,
}

impl Batches {
    pub fn add_model(&mut self, mesh_id: String, instance: Instance) {
        self.instances
            .entry(mesh_id)
            .or_default()
            .data
            .push(instance);
    }

    pub fn prepare(&mut self, device: &wgpu::Device) {
        for instances in self.instances.values_mut() {
            if instances.data.is_empty() {
                continue;
            }
            instances.buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("model_instance"),
                    contents: &Instance::slice_as_bytes(&instances.data),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ))
        }
    }

    pub fn render(&self, rpass: &mut wgpu::RenderPass, meshes: &MeshAssets) {
        for (mesh_id, instances) in &self.instances {
            if instances.data.is_empty() {
                continue;
            }
            let (Some(mesh), Some(instances_buffer)) =
                (meshes.get(mesh_id), &instances.buffer)
            else {
                continue;
            };
            rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            rpass.set_index_buffer(
                mesh.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            rpass.set_vertex_buffer(1, instances_buffer.slice(..));
            rpass.draw_indexed(
                0..mesh.num_indices,
                0,
                0..instances.data.len() as u32,
            );
        }
    }

    pub fn clear(&mut self) {
        self.instances.clear();
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
                    buffers: &[Vertex::desc(), Instance::desc()],
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
