use std::{
    collections::{HashMap, HashSet},
    io::Cursor,
    ops::Deref,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    time::Instant,
};

use anyhow::Result;
use assets_manager::{loader, Asset, AssetCache};
use bytemuck::{cast_slice, Pod};
use log::{error, info};
use wgpu::util::DeviceExt;

use crate::app::{get_pool, RELOAD_DEBOUNCE};

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
        label: &str,
    ) -> Self {
        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("mesh_{}_vertex_buffer", label)),
                contents: cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("mesh_{}_index_buffer", label)),
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

type LoadResult = (String, Result<(Box<Vec<model::Vertex>>, Box<Vec<u32>>)>);

pub struct MeshAssets {
    cache: Arc<AssetCache>,
    last_reload: Instant,
    load_rx: Receiver<LoadResult>,
    load_tx: Sender<LoadResult>,
    loaded: HashSet<String>,
    meshes: HashMap<String, Mesh>,
}

impl MeshAssets {
    pub fn new() -> Self {
        let (load_tx, load_rx) = channel();
        Self {
            cache: Arc::new(AssetCache::new("assets/meshes").unwrap()),
            last_reload: Instant::now(),
            load_rx,
            load_tx,
            loaded: HashSet::new(),
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
                self.last_reload = Instant::now();
                self.load_internal(&mesh_id);
            }
        }

        if let Ok((mesh_id, result)) = self.load_rx.try_recv() {
            match result {
                Ok((vertices, indices)) => {
                    info!("Mesh loaded: {}", mesh_id);
                    self.meshes.insert(
                        mesh_id.clone(),
                        Mesh::new(device, &vertices, &indices, &mesh_id),
                    );
                }
                Err(err) => {
                    error!("load\n{:?}", err);
                    self.loaded.remove(&mesh_id);
                }
            };
        }
    }

    pub fn get(&self, mesh_id: &str) -> Option<&Mesh> {
        self.meshes.get(mesh_id)
    }

    fn load_internal(&mut self, mesh_id: &str) {
        let cache = self.cache.clone();
        let mesh_id = mesh_id.to_string();
        let load_tx = self.load_tx.clone();

        get_pool().execute(move || {
            let result = (|| {
                let handle = cache.load::<ObjSource>(&mesh_id)?;
                let data = handle.read();
                let mut cursor = Cursor::new(data.0.deref());
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

                Ok((Box::new(vertices), Box::new(indices)))
            })();
            load_tx.send((mesh_id, result)).unwrap();
        });
    }

    pub fn load(&mut self, mesh_id: &str) {
        if self.loaded.contains(mesh_id) {
            return;
        }
        self.loaded.insert(mesh_id.to_string());
        self.load_internal(mesh_id);
    }
}
