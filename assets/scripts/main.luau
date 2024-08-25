function init(scene: Scene)
	print("init")
	scene.camera.pos = Vec3.new(0.5, 0, 2)
end

function update(dt: number, scene: Scene)
	if scene.camera.pos.x > 0 then
		scene.camera.pos -= Vec3.new(dt, 0, 0)
	end
end
