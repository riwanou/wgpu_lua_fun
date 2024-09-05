type Player = {
	transform: Transform,
	init: (self: Player) -> (),
	update: (self: Player, dt: number, elapsed: number) -> (),
	render: (self: Player, ctx: Context) -> (),
}

local player: Player = cached_table("player")

function player.init(self)
	self.transform = Transform.new(Vec3.new(0, 0, 0))
	self.transform.scale = Vec3.splat(0.5)
end

function player.update(self, dt, elapsed)
	self.transform.pos.y = math.cos(elapsed) * 1.5
	self.transform:rotate(Vec3.Y, dt)
	self.transform:rotate(Vec3.Z, dt)
end

function player.render(self, ctx)
	local shader = nil
	if ctx.inputs:pressed("interact") then
		shader = "animated"
	end
	ctx.scene:batch_model("dragon", "blue_nebula", shader, self.transform)
end

return player
