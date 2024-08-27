use encase::ShaderType;
use glam::Mat4;
use wgpu::util::DeviceExt;

use crate::scene::Scene;

use super::Layouts;

pub struct Bundle {
    pub bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}

impl Bundle {
    pub fn new(device: &wgpu::Device, layouts: &Layouts) -> Self {
        let buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("globals_buffer"),
                contents: &Uniform::default().as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            });
        let bind_group = layouts.globals.bind(device, &buffer);
        Self { bind_group, buffer }
    }

    pub fn prepare(
        &self,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        scene: &Scene,
        elapsed: f32,
    ) {
        let aspect_ratio = config.width as f32 / config.height as f32;
        let uniform = Uniform {
            clip_view: scene.camera.build_projection(aspect_ratio),
            view_world: scene.camera.build_view(),
            elapsed,
        };
        queue.write_buffer(&self.buffer, 0, &uniform.as_bytes());
    }
}

#[derive(Default, ShaderType)]
pub struct Uniform {
    clip_view: Mat4,
    view_world: Mat4,
    elapsed: f32,
}

impl Uniform {
    pub fn as_bytes(&self) -> Vec<u8> {
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
                label: Some("globals_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX
                        | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        Self { layout }
    }

    pub fn bind(
        &self,
        device: &wgpu::Device,
        buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("globals_bind_group"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }
}
