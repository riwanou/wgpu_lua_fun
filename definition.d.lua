declare class Vec3
  x: number
  y: number
  z: number
  function __add(self, other: Vec3): Vec3
  function __sub(self, other: Vec3): Vec3
end

declare Vec3: {
  new: (x: number, y: number, z: number) -> Vec3,
}

declare class Camera
	pos: Vec3
	fovy: number
end

declare class String
  function get(self): string
  function set(self, val: string): ()
end

declare class MeshAssets
  function load(self, mesh_id: string): ()
end

export type Scene = {
  camera: Camera,
  mesh_id: String,
  meshes: MeshAssets,
  render_model: (mesh_id: string, pos: Vec3) -> (),
}

declare entities: { [string]: any }
declare function entity(id: string): any
