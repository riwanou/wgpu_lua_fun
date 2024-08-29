local player = require("assets/scripts/player")

function init()
	print("init")

	for _, mesh_id in { "cube", "bunny", "dragon" } do
		game.load_mesh(mesh_id)
	end

	player:init()
	game.camera.transform.pos = Vec3.new(0.5, 0, 10)
end

function update(dt: number, elapsed: number)
	elapsed += dt

	player:update(dt, elapsed)
	player:render()

	game.render_model("cube", Transform.new(Vec3.new(2, 2, -10)))
	game.render_model("cube", Transform.new(Vec3.new(-2, 2, -2)))
	game.render_model("cube", Transform.new(Vec3.new(2, -2, 1)))
	game.render_model("cube", Transform.new(Vec3.new(-2, -2, 10)))

	if game.camera.transform.pos.x > 0 then
		game.camera.transform.pos.x -= dt
	end
end
