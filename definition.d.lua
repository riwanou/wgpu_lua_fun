declare class Vec3
  x: number
  y: number
  z: number
  function __add(self, other: Vec3): Vec3
  function __sub(self, other: Vec3): Vec3
  function __mul(self, other: Vec3): Vec3
  function __div(self, other: Vec3): Vec3
end

declare Vec3: {
  new: (x: number, y: number, z: number) -> Vec3,
  splat: (val: number) -> Vec3,
}

declare class Transform
  pos: Vec3
  scale: Vec3
  function rotate_x(self, angle: number): ()
  function rotate_y(self, angle: number): ()
  function rotate_z(self, angle: number): ()
end

declare Transform: {
  new: (pos: Vec3) -> Transform,
}

declare class Camera
	transform: Transform
	fovy: number
end

declare game: {
  camera: Camera,
  load_mesh: (mesh_id: string) -> (),
  render_model: (mesh_id: string, transform: Transform) -> (),
}

declare entities: { [string]: any }
declare function entity(id: string): any
