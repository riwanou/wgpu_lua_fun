local player = require("assets/scripts/player")

function init(scene: Scene)
	print("init")

	for _, mesh_id in { "cube", "bunny", "dragon" } do
		scene.meshes:load(mesh_id)
	end

	scene.camera.pos = Vec3.new(0.5, 0, 20)
	player:init()
end

local elapsed = 0
function update(dt: number, scene: Scene)
	elapsed += dt

	player:update(dt)
	player:render(scene)

	scene.render_model("cube", Vec3.new(2, 2, -10))
	scene.render_model("cube", Vec3.new(-2, 2, -2))
	scene.render_model("cube", Vec3.new(2, -2, 1))
	scene.render_model("cube", Vec3.new(-2, -2, 10))

	if scene.camera.pos.x > 0 then
		scene.camera.pos -= Vec3.new(dt, 0, 0)
	end
end
