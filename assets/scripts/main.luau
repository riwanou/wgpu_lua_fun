--!strict

print("Main luau version 1")

-- for key, value in pairs(scene.camera) do
-- 	print(key, value)
-- end

function init(camera)
	print(camera.pos.z)
	print(camera.pos)
	print(camera.fovy)
	camera.fovy = 45
	camera.pos = Vec3.new(0.5, 0, 2)
	print(camera)
end

function update(dt: number, camera)
	camera.pos += Vec3.new(dt, 0, 0)
	-- scene.camera.translation[0] = 2
	-- print("FPS:" .. dt * 1000)
end
