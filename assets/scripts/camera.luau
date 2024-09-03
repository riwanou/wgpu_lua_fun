type Camera = {
	enabled: boolean,
	init: (self: Camera, ctx: Context) -> (),
	update: (self: Camera, ctx: Context, dt: number) -> (),
}

local camera: Camera = cached_table("camera")

function camera.init(self, ctx)
	local window = ctx.window
	local camera = ctx.scene.camera

	self.enabled = true
	window:grab_cursor()
	camera.transform.pos = Vec3.new(0.5, 0, 10)
	camera.transform.rot = Quat.default()
end

function camera.update(self, ctx, dt)
	local inputs = ctx.inputs
	local window = ctx.window
	local camera = ctx.scene.camera

	if inputs:just_pressed("focus") then
		self.enabled = not self.enabled
		if self.enabled then
			window:grab_cursor()
		else
			window:release_cursor()
		end
	end

	if not self.enabled or not ctx.inputs:focused() then
		return
	end

	local direction = Vec3.splat(0)
	local right = camera.transform:right()
	local forward = camera.transform:forward()
	if inputs:pressed("backward") then
		direction += forward * -1
	end
	if inputs:pressed("forward") then
		direction += forward * 1
	end
	if inputs:pressed("left") then
		direction += right * -1
	end
	if inputs:pressed("right") then
		direction += right * 1
	end
	if inputs:pressed("down") then
		direction.y -= 1
	end
	if inputs:pressed("up") then
		direction.y += 1
	end

	local yaw, pitch = 0, 0
	yaw = -inputs:mouse_delta().x * 0.4 * dt
	pitch = -inputs:mouse_delta().y * 0.4 * dt

	camera.transform.pos += direction * 15 * dt
	camera.transform:rotate(Vec3.Y, yaw)
	camera.transform:rotate_local(Vec3.X, pitch)
end

return camera
