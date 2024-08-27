local meshes = {
	cube = "cube",
	bunny = "bunny",
	dragon = "dragon",
}

local player: { pos: Vec3 } = entity("player")

function init(scene: Scene)
	print("init")

	for _, val in pairs(meshes) do
		scene.meshes:load(val)
	end

	player.pos = Vec3.new(0, 10, 0)
	scene.camera.pos = Vec3.new(0.5, 0, 20)
end

function update(dt: number, scene: Scene)
	player.pos += Vec3.new(1, 0, 0)

	scene.mesh_id:set(meshes.dragon)

	if scene.camera.pos.x > 0 then
		scene.camera.pos -= Vec3.new(dt, 0, 0)
	end
end
