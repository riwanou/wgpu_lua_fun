use std::{
    collections::{HashMap, HashSet},
    mem,
};

use bytemuck::{cast_slice, Pod, Zeroable};
use encase::ShaderType;
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
pub const DEFAULT_SHADER: &str = "model";

pub struct Bundle {
    pub pipelines: HashMap<String, Pipeline>,
    registered_shaders: HashSet<String>,
}

impl Bundle {
    pub fn new(
        shaders: &mut ShaderAssets,
        textures: &mut TextureAssets,
    ) -> Self {
        textures.load(DEFAULT_DIFFUSE_TEXTURE);

        let mut registered_shaders = HashSet::new();
        registered_shaders.insert(DEFAULT_SHADER.to_string());
        shaders.load(DEFAULT_SHADER);

        Self {
            pipelines: HashMap::new(),
            registered_shaders,
        }
    }

    pub fn hot_reload(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layouts: &Layouts,
        shaders: &mut ShaderAssets,
    ) {
        let Some(shader_id) = &shaders.frame_reloaded else {
            return;
        };

        if self.registered_shaders.contains(shader_id) {
            info!("Pipeline loaded with shader: {}", shader_id);
            let module = shaders.get(shader_id).unwrap();
            self.pipelines.insert(
                shader_id.clone(),
                Pipeline::new(device, config, layouts, module),
            );
        }
    }

    pub fn register_shader(&mut self, shader_id: &str) {
        self.registered_shaders.insert(shader_id.to_string());
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
    shader_id: String,
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
        shader_id: String,
        instance: Instance,
    ) {
        let key = Key {
            mesh_id,
            texture_id,
            shader_id,
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
                    .or_insert_with(|| {
                        let buffer = device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("model_uniform"),
                                contents: &Uniform::default().as_bytes(),
                                usage: wgpu::BufferUsages::UNIFORM,
                            },
                        );
                        layouts.model.bind(device, &buffer, texture)
                    });
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

    pub fn render(
        &self,
        rpass: &mut wgpu::RenderPass,
        bundle: &Bundle,
        meshes: &MeshAssets,
    ) {
        for (key, instances) in &self.instances {
            if instances.data.is_empty() {
                continue;
            }

            let (
                Some(mesh),
                Some(bind_group),
                Some(pipeline),
                Some(instances_buffer),
            ) = (
                meshes.get(&key.mesh_id),
                self.bind_groups.get(&key.texture_id),
                bundle.pipelines.get(&key.shader_id),
                &instances.buffer,
            )
            else {
                continue;
            };

            rpass.set_pipeline(&pipeline.pipeline);
            rpass.set_bind_group(2, bind_group, &[]);
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

#[derive(ShaderType)]
pub struct Uniform {
    bloup: f32,
}

impl Default for Uniform {
    fn default() -> Self {
        Self { bloup: 42.0 }
    }
}

impl Uniform {
    fn as_bytes(&self) -> Vec<u8> {
        let mut buffer = encase::UniformBuffer::new(Vec::<u8>::new());
        buffer.write(self).unwrap();
        buffer.into_inner()
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
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::default(),
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
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
        uniform: &wgpu::Buffer,
        texture: &Texture,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("model_bind_group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        uniform.as_entire_buffer_binding(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        })
    }
}

pub struct Pipeline {
    pub pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layouts: &Layouts,
        module: &wgpu::ShaderModule,
    ) -> Self {
        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("model_layout"),
                bind_group_layouts: &[
                    &layouts.globals.layout,
                    &layouts.lights.layout,
                    &layouts.model.layout,
                ],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("model_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc(), Instance::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module,
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

        Self { pipeline }
    }
}
