local player = require("assets/scripts/player")
local camera = require("assets/scripts/camera")

function init(ctx: Context)
	print("init")

	for _, mesh_id in { "cube", "bunny", "dragon" } do
		ctx.graphics:load_mesh(mesh_id)
	end

	for _, texture_id in { "blue_nebula", "dragon" } do
		ctx.graphics:load_texture(texture_id)
	end

	camera:init(ctx)
	player:init()
end

function update(ctx: Context, dt: number, elapsed: number)
	camera:update(ctx, dt)

	player:update(dt, elapsed)
	player:render(ctx)

	ctx.scene:batch_model("cube", nil, Transform.new(Vec3.new(2, 2, -10)))
	ctx.scene:batch_model("cube", "blue_nebula", Transform.new(Vec3.new(-2, 2, -2)))
	ctx.scene:batch_model("cube", "dragon", Transform.new(Vec3.new(2, -2, 1)))
	ctx.scene:batch_model("cube", nil, Transform.new(Vec3.new(-3, -2, 3)))
end
