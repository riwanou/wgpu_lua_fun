local player = require("assets/scripts/player")
local camera = require("assets/scripts/camera")

function init(ctx: Context)
	print("init")

	for _, mesh_id in { "cube", "plane", "bunny", "dragon" } do
		ctx.graphics:load_mesh(mesh_id)
	end

	for _, texture_id in { "blue_nebula", "dragon", "256-palette", "weave" } do
		ctx.graphics:load_texture(texture_id)
	end

	camera:init(ctx)
	player:init()
end

function update(ctx: Context, dt: number, elapsed: number)
	camera:update(ctx, dt)

	player:update(dt, elapsed)
	player:render(ctx)

	-- on the objects
	ctx.scene:point_light(Vec3.new(-2, 2, -10))
	ctx.scene:point_light(Vec3.new(-2, 2, -2))

	-- on the ground
	ctx.scene:point_light(Vec3.new(2, -5, 8))
	ctx.scene:point_light(Vec3.new(-5, -5, 5))
	ctx.scene:point_light(Vec3.new(8, -5, -3))

	local plane = Transform.new(Vec3.new(0, -4.5, 0))
	local size = 25
	plane.scale = Vec3.new(size, 0, size)
	ctx.scene:batch_model("plane", "weave", plane)

	ctx.scene:batch_model("cube", "256-palette", Transform.new(Vec3.new(2, 2, -10)))
	ctx.scene:batch_model("cube", "blue_nebula", Transform.new(Vec3.new(-2, 2, -2)))
	ctx.scene:batch_model("cube", "dragon", Transform.new(Vec3.new(2, -2, 1)))
	ctx.scene:batch_model("cube", nil, Transform.new(Vec3.new(-3, -2, 3)))
end
