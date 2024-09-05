use encase::{ArrayLength, ShaderType};
use glam::Vec3;
use wgpu::util::DeviceExt;

use super::Layouts;

pub struct Bundle {
    pub bind_group: wgpu::BindGroup,
    point_lights_buffer: wgpu::Buffer,
}

impl Bundle {
    pub fn new(device: &wgpu::Device, layouts: &Layouts) -> Self {
        let point_lights_buffer = Self::create_point_lights_buffer(
            device,
            &PointLightData::default(),
        );
        let bind_group = layouts.lights.bind(device, &point_lights_buffer);
        Self {
            bind_group,
            point_lights_buffer,
        }
    }

    fn create_point_lights_buffer(
        device: &wgpu::Device,
        data: &PointLightData,
    ) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("point_lights_buffer"),
            contents: &data.as_bytes(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        layouts: &Layouts,
        point_lights: &[PointLight],
    ) {
        self.point_lights_buffer = Self::create_point_lights_buffer(
            device,
            &PointLightData {
                len: ArrayLength,
                data: point_lights.to_vec(),
            },
        );
        self.bind_group =
            layouts.lights.bind(device, &self.point_lights_buffer);
    }
}

#[derive(Debug, Default, ShaderType, Clone)]
pub struct PointLight {
    pub pos: Vec3,
    pub radius: f32,
}

#[derive(Default, ShaderType)]
pub struct PointLightData {
    len: ArrayLength,
    #[size(runtime)]
    data: Vec<PointLight>,
}

impl PointLightData {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buffer = encase::StorageBuffer::new(Vec::<u8>::new());
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
                label: Some("lights_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: true,
                        },
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
        point_lights_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lights_bind_group"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: point_lights_buffer.as_entire_binding(),
            }],
        })
    }
}
