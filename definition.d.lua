declare class Vec3
  x: number
  y: number
  z: number
  function __add(self, other: Vec3 | number): Vec3
  function __sub(self, other: Vec3 | number): Vec3
  function __mul(self, other: Vec3 | number): Vec3
  function __div(self, other: Vec3 | number): Vec3
end

declare Vec3: {
  X: Vec3,
  Y: Vec3,
  Z: Vec3,
  new: (x: number, y: number, z: number) -> Vec3,
  splat: (val: number) -> Vec3,
}

declare class Quat
end

declare Quat: {
  default: () -> Quat
}

declare class Transform
  pos: Vec3
  rot: Quat
  scale: Vec3
  function forward(self): Vec3
  function right(self): Vec3
  function rotate(self, axis: Vec3, angle: number): ()
  function rotate_local(self, axis: Vec3, angle: number): ()
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
  function batch_model(self, mesh_id: string, texture_id: string?, transform: Transform): ()
  function point_light(self, pos: Vec3, radius: number): ()
end

type Action = "forward" | "backward" | "right" | "left" | "up" | "down" | "focus"

declare class Inputs
  function cursor_in_window(self): boolean
  function focused(self): boolean
  function pressed(self, action: Action): boolean
  function just_pressed(self, action: Action): boolean
  function mouse_pressed(self, button: "left" | "right"): boolean
  function mouse_just_pressed(self, button: "left" | "right"): boolean
  function mouse_delta(self): { x: number, y: number }
end

declare class Window 
  function grab_cursor(self): ()
  function release_cursor(self): ()
end

declare class Graphics
  function load_mesh(self, mesh_id: string): ()
  function load_texture(self, texture_id: string): ()
end

export type Context = {
  scene: Scene,
  inputs: Inputs,
  window: Window,
  graphics: Graphics,
}

declare function cached_table(id: string): any
