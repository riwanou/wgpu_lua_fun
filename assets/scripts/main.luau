local player = require("assets/scripts/player")
local camera = require("assets/scripts/camera")

function init(ctx: Context)
	print("init")

	for _, mesh_id in { "cube", "plane", "bunny", "dragon" } do
		ctx.graphics:load_mesh(mesh_id)
	end

	ctx.graphics:add_material("simple", {
		key = "nebula",
		texture = "blue_nebula",
	})
	ctx.graphics:add_material("simple", {
		key = "dragon",
		texture = "dragon",
	})
	ctx.graphics:add_material("simple", {
		key = "weave",
		texture = "weave",
	})
	ctx.graphics:add_material("simple", {
		key = "animated",
		texture = "blue_nebula",
		shader = "animated",
	})

	camera:init(ctx)
	player:init(ctx)
end

function update(ctx: Context, dt: number, elapsed: number)
	camera:update(ctx, dt)

	player:update(dt, elapsed)
	player:render(ctx)

	-- on the objects
	ctx.scene:point_light(Vec3.new(-2, 2, -10), 5)
	ctx.scene:point_light(Vec3.new(-2, 2, -2), 3)

	-- on the ground
	ctx.scene:point_light(Vec3.new(2, -4, 8), 7)
	ctx.scene:point_light(Vec3.new(-5, -4, 5), 5)
	ctx.scene:point_light(Vec3.new(8, -4, -3), 12)

	local plane = Transform.new(Vec3.new(0, -4.5, 0))
	local size = 25
	plane.scale = Vec3.new(size, 0, size)
	ctx.scene:batch_model("plane", "weave", plane)

	-- big cube
	local big_cube = Transform.new(Vec3.new(13, 2, -2))
	big_cube.scale = Vec3.splat(3.5)
	ctx.scene:batch_model("cube", nil, big_cube)

	-- small cubes
	ctx.scene:batch_model("cube", "animated", Transform.new(Vec3.new(2, 2, -10)))
	ctx.scene:batch_model("cube", "animated", Transform.new(Vec3.new(-2, 2, -2)))
	ctx.scene:batch_model("cube", "dragon", Transform.new(Vec3.new(2, -2, 1)))
	ctx.scene:batch_model("cube", nil, Transform.new(Vec3.new(-3, -2, 3)))
end
