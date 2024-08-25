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
