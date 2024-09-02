local player = require("assets/scripts/player")

function init(ctx: Context)
	print("init")

	for _, mesh_id in { "cube", "bunny", "dragon" } do
		ctx.graphics:load_mesh(mesh_id)
	end

	for _, texture_id in { "blue_nebula", "dragon" } do
		ctx.graphics:load_texture(texture_id)
	end

	player:init()
	ctx.scene.camera.transform.pos = Vec3.new(0.5, 0, 10)
end

function update_camera(camera: Camera, inputs: Inputs, dt: number)
	local direction = Vec3.splat(0)
	if inputs:pressed("forward") then
		direction.z -= 1
	end
	if inputs:pressed("backward") then
		direction.z += 1
	end
	if inputs:pressed("left") then
		direction.x -= 1
	end
	if inputs:pressed("right") then
		direction.x += 1
	end
	if inputs:pressed("down") then
		direction.y -= 1
	end
	if inputs:pressed("up") then
		direction.y += 1
	end

	camera.transform.pos += direction * 15 * dt
end

function update(ctx: Context, dt: number, elapsed: number)
	update_camera(ctx.scene.camera, ctx.inputs, dt)

	player:update(dt, elapsed)
	player:render(ctx)

	ctx.scene:batch_model("cube", nil, Transform.new(Vec3.new(2, 2, -10)))
	ctx.scene:batch_model("cube", "blue_nebula", Transform.new(Vec3.new(-2, 2, -2)))
	ctx.scene:batch_model("cube", "dragon", Transform.new(Vec3.new(2, -2, 1)))
	ctx.scene:batch_model("cube", nil, Transform.new(Vec3.new(-3, -2, 3)))
end
