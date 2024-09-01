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
  function rotate(self, axis: Vec3, angle: number): ()
end

declare Transform: {
  new: (pos: Vec3) -> Transform,
}

declare class Camera
	transform: Transform
	fovy: number
end

declare class Scene
  camera: Camera
  function batch_model(self, mesh_id: string, transform: Transform): ()
end

declare class Graphics
  function load_mesh(self, mesh_id: string): ()
end

export type Context = {
  scene: Scene,
  graphics: Graphics,
}

declare function entity(id: string): any
