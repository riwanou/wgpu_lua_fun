use std::{collections::HashMap, io::Cursor, ops::Deref, time::Instant};

use anyhow::{Context, Result};
use assets_manager::{loader, Asset, AssetCache};
use bytemuck::{cast_slice, Pod};
use log::info;
use wgpu::util::DeviceExt;

use crate::app::RELOAD_DEBOUNCE;

use super::bundle::model;

pub trait VertexTrait: Pod {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl Mesh {
    pub fn new<Vertex: VertexTrait>(
        device: &wgpu::Device,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Self {
        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("mesh_vertex_buffer"),
                contents: cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("mesh_index_buffer"),
                contents: cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}

pub struct ObjSource(String);

impl From<String> for ObjSource {
    fn from(value: String) -> Self {
        ObjSource(value)
    }
}

impl Asset for ObjSource {
    const EXTENSION: &'static str = "obj";
    type Loader = loader::LoadFrom<String, loader::StringLoader>;
}

pub struct MeshAssets {
    cache: AssetCache,
    last_reload: Instant,
    meshes: HashMap<String, Mesh>,
}

impl MeshAssets {
    pub fn new() -> Self {
        Self {
            cache: AssetCache::new("assets/meshes").unwrap(),
            last_reload: Instant::now(),
            meshes: HashMap::new(),
        }
    }

    pub fn hot_reload(&mut self, device: &wgpu::Device) {
        self.cache.hot_reload();
        let keys = self.meshes.keys().cloned().collect::<Vec<_>>();
        for mesh_id in keys {
            let handle = self.cache.load_expect::<ObjSource>(&mesh_id);
            if self.last_reload.elapsed() >= RELOAD_DEBOUNCE
                && handle.reloaded_global()
            {
                info!("Mesh reloaded: {}", mesh_id);
                self.last_reload = Instant::now();
                self.load(&mesh_id, device).unwrap();
            }
        }
    }

    pub fn contains(&self, mesh_id: &str) -> bool {
        self.meshes.contains_key(mesh_id)
    }

    pub fn get(&self, mesh_id: &str) -> Result<&Mesh> {
        self.meshes
            .get(mesh_id)
            .context(format!("Mesh not loaded: {}", mesh_id))
    }

    pub fn load(
        &mut self,
        mesh_id: &str,
        device: &wgpu::Device,
    ) -> Result<String> {
        let handle = self.cache.load::<ObjSource>(mesh_id)?;
        let data = handle.read();
        let mut cursor = Cursor::new(data.deref().0.as_bytes());
        let (obj_models, _) = tobj::load_obj_buf(
            &mut cursor,
            &tobj::GPU_LOAD_OPTIONS,
            |_| unreachable!(),
        )?;

        let mut vertices = Vec::<model::Vertex>::new();
        let mut indices = Vec::<u32>::new();

        for mut m in obj_models {
            for i in 0..m.mesh.positions.len() / 3 {
                vertices.push(model::Vertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coord: [
                        m.mesh.texcoords[i * 2],
                        1.0 - m.mesh.texcoords[i * 2 + 1],
                    ],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                });
            }
            indices.append(&mut m.mesh.indices);
        }

        let id = handle.id().to_string();
        self.meshes
            .insert(id.clone(), Mesh::new(device, &vertices, &indices));

        Ok(id)
    }
}
