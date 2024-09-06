use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    mem,
};

use bytemuck::{cast_slice, Pod, Zeroable};
use glam::{Mat3, Mat4, Quat};
use log::info;
use wgpu::util::DeviceExt;

use crate::render::{
    material::{simple::SimpleMaterial, MaterialManager},
    mesh::{MeshAssets, VertexTrait},
    shader::ShaderAssets,
    texture::{Texture, TextureAssets},
};

use super::Layouts;

pub const DEFAULT_SHADER: &str = "model";
pub const DEFAULT_TEXTURE: &str = "white";
pub const DEFAULT_MATERIAL: &str = "model";

pub struct Bundle {
    pub pipelines: HashMap<String, Pipeline>,
    registered_shaders: HashSet<String>,
}

impl Bundle {
    pub fn new(
        shaders: &mut ShaderAssets,
        textures: &mut TextureAssets,
        materials: &mut MaterialManager,
    ) -> Self {
        textures.load(DEFAULT_TEXTURE);

        let mut registered_shaders = HashSet::new();
        registered_shaders.insert(DEFAULT_SHADER.to_string());
        shaders.load(DEFAULT_SHADER);

        let material = SimpleMaterial::new(DEFAULT_SHADER, DEFAULT_TEXTURE);
        materials.add(DEFAULT_MATERIAL, material);

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
                Pipeline::new(device, config, layouts, module, shader_id),
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

struct MaterialData {
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}

#[derive(Default)]
struct InstanceArray {
    buffer: Option<wgpu::Buffer>,
    data: Vec<Instance>,
}

#[derive(Hash, PartialEq, Eq)]
struct Key {
    mesh_id: String,
    material_id: String,
}

#[derive(Default)]
pub struct Batches {
    materials: HashMap<String, MaterialData>,
    instances: HashMap<Key, InstanceArray>,
}

impl Batches {
    pub fn add_model(
        &mut self,
        mesh_id: String,
        material_id: String,
        instance: Instance,
    ) {
        let key = Key {
            mesh_id,
            material_id,
        };
        self.instances.entry(key).or_default().data.push(instance);
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layouts: &Layouts,
        textures: &TextureAssets,
        materials: &MaterialManager,
    ) {
        for (key, instances) in &mut self.instances {
            if instances.data.is_empty() {
                continue;
            }

            let Some(texture_id) = materials.get_texture_id(&key.material_id)
            else {
                return;
            };
            if let (Some(texture), Some(uniform_data)) = (
                textures.get(&texture_id),
                materials.get_uniform_data_bytes(&key.material_id),
            ) {
                match self.materials.entry(key.material_id.clone()) {
                    Entry::Occupied(entry) => {
                        let material_data = entry.get();
                        queue.write_buffer(
                            &material_data.buffer,
                            0,
                            &uniform_data,
                        );
                    }
                    Entry::Vacant(entry) => {
                        let buffer = device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some(&format!(
                                    "model_{}_uniform",
                                    key.material_id
                                )),
                                contents: &uniform_data,
                                usage: wgpu::BufferUsages::UNIFORM
                                    | wgpu::BufferUsages::COPY_DST,
                            },
                        );
                        let bind_group =
                            layouts.model.bind(device, &buffer, texture);
                        entry.insert(MaterialData { bind_group, buffer });
                    }
                };
            }

            instances.buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("model_{}_instance", key.material_id)),
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
        materials: &MaterialManager,
    ) {
        for (key, instances) in &self.instances {
            if instances.data.is_empty() {
                continue;
            }

            let Some(shader_id) = materials.get_shader_id(&key.material_id)
            else {
                continue;
            };
            let (
                Some(mesh),
                Some(material_data),
                Some(pipeline),
                Some(instances_buffer),
            ) = (
                meshes.get(&key.mesh_id),
                self.materials.get(&key.material_id),
                bundle.pipelines.get(&shader_id),
                &instances.buffer,
            )
            else {
                continue;
            };

            rpass.set_pipeline(&pipeline.pipeline);
            rpass.set_bind_group(2, &material_data.bind_group, &[]);
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
        label: &str,
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
                label: Some(&format!("model_{}_pipeline", label)),
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
