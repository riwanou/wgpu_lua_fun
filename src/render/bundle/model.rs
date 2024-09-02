use std::{collections::HashMap, mem};

use bytemuck::{cast_slice, Pod, Zeroable};
use glam::{Mat3, Mat4, Quat};
use log::info;
use wgpu::util::DeviceExt;

use crate::render::{
    mesh::{MeshAssets, VertexTrait},
    shader::ShaderAssets,
    texture::{Texture, TextureAssets},
};

use super::Layouts;

pub const DEFAULT_DIFFUSE_TEXTURE: &str = "white";

pub struct Bundle {
    pub pipeline: Pipeline,
}

impl Bundle {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layouts: &Layouts,
        shaders: &mut ShaderAssets,
        textures: &mut TextureAssets,
    ) -> Self {
        textures.load(DEFAULT_DIFFUSE_TEXTURE);
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

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Instance {
    pub world_local: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}

impl Instance {
    pub fn new(transform: Mat4, rotation: Quat) -> Self {
        Self {
            world_local: transform.to_cols_array_2d(),
            normal: Mat3::from_quat(rotation).to_cols_array_2d(),
        }
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
    buffer: Option<wgpu::Buffer>,
    data: Vec<Instance>,
}

#[derive(Hash, PartialEq, Eq)]
struct Key {
    mesh_id: String,
    texture_id: String,
}

#[derive(Default)]
pub struct Batches {
    bind_groups: HashMap<String, wgpu::BindGroup>,
    instances: HashMap<Key, InstanceArray>,
}

impl Batches {
    pub fn add_model(
        &mut self,
        mesh_id: String,
        texture_id: String,
        instance: Instance,
    ) {
        let key = Key {
            mesh_id,
            texture_id,
        };
        self.instances.entry(key).or_default().data.push(instance);
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        layouts: &Layouts,
        textures: &TextureAssets,
    ) {
        for (key, instances) in &mut self.instances {
            if instances.data.is_empty() {
                continue;
            }

            if let Some(texture) = textures.get(&key.texture_id) {
                self.bind_groups
                    .entry(key.texture_id.clone())
                    .or_insert_with(|| layouts.model.bind(device, texture));
            }

            instances.buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("model_instance"),
                    contents: cast_slice(&instances.data),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ))
        }
    }

    pub fn render(&self, rpass: &mut wgpu::RenderPass, meshes: &MeshAssets) {
        for (key, instances) in &self.instances {
            if instances.data.is_empty() {
                continue;
            }
            let (Some(mesh), Some(bind_group), Some(instances_buffer)) = (
                meshes.get(&key.mesh_id),
                self.bind_groups.get(&key.texture_id),
                &instances.buffer,
            ) else {
                continue;
            };
            rpass.set_bind_group(1, bind_group, &[]);
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

pub struct Layout {
    pub layout: wgpu::BindGroupLayout,
}

impl Layout {
    pub fn new(device: &wgpu::Device) -> Self {
        let layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("model_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::default(),
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                ],
            });
        Self { layout }
    }

    pub fn bind(
        &self,
        device: &wgpu::Device,
        texture: &Texture,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("model_bind_group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        })
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
                bind_group_layouts: &[
                    &layouts.globals.layout,
                    &layouts.model.layout,
                ],
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),

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
