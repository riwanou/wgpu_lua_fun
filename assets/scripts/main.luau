local player = require("assets/scripts/player")

function init(ctx: Context)
	print("init")

	for _, mesh_id in { "cube", "bunny", "dragon" } do
		ctx.graphics:load_mesh(mesh_id)
	end

	player:init()
	ctx.scene.camera.transform.pos = Vec3.new(0.5, 0, 10)
end

function update_camera(camera: Camera, dt: number) end

function update(ctx: Context, dt: number, elapsed: number)
	local camera = ctx.scene.camera
	if camera.transform.pos.x > 0 then
		camera.transform.pos.x -= dt
	end

	player:update(dt, elapsed)
	player:render(ctx)

	ctx.scene:batch_model("cube", Transform.new(Vec3.new(2, 2, -10)))
	ctx.scene:batch_model("cube", Transform.new(Vec3.new(-2, 2, -2)))
	ctx.scene:batch_model("cube", Transform.new(Vec3.new(2, -2, 1)))
	ctx.scene:batch_model("cube", Transform.new(Vec3.new(-3, -2, 3)))
end
