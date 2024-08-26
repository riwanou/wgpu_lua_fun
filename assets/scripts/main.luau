local meshes = {
	cube = "cube",
	bunny = "bunny",
	-- dragon = "dragon",
}

function init(scene: Scene)
	print("init")
	for _, val in pairs(meshes) do
		scene.meshes:load(val, scene.device)
	end
	scene.camera.pos = Vec3.new(0.5, 0, 5)
end

function update(dt: number, scene: Scene)
	scene.mesh_id:set(meshes.bunny)
	if scene.camera.pos.x > 0 then
		scene.camera.pos -= Vec3.new(dt, 0, 0)
	end
end
