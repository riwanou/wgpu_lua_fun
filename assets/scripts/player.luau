type Player = {
	transform: Transform,
	init: (self: Player, ctx: Context) -> (),
	update: (self: Player, dt: number, elapsed: number) -> (),
	render: (self: Player, ctx: Context) -> (),
}

local player: Player = cached_table("player")

function player.init(self, ctx)
	self.transform = Transform.new(Vec3.new(0, 0, 0))
	self.transform.scale = Vec3.splat(0.5)

	ctx.graphics:add_material("simple", {
		key = "player",
		texture = "blue_nebula",
		shader = "animated",
	})
end

function player.update(self, dt, elapsed)
	self.transform.pos.y = math.cos(elapsed) * 3.5
	self.transform:rotate(Vec3.Y, dt)
	self.transform:rotate(Vec3.Z, dt)
end

function player.render(self, ctx)
	local shader = "nebula"
	if ctx.inputs:pressed("interact") then
		shader = "player"
	end
	ctx.graphics:material_data("player", {
		color = Vec3.new(self.transform.pos.y, 1.0, 0.2),
	})
	ctx.scene:batch_model("dragon", shader, self.transform)
end

return player
