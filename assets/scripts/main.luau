--!strict

print("Main luau version 1")

function init(camera: Camera)
	camera.pos = Vec3.new(0.5, 0, 2)
	print(camera.fovy, camera.pos)
end

function update(dt: number, camera: Camera)
	if camera.pos.x > 0 then
		camera.pos -= Vec3.new(dt, 0, 0)
	end
end
