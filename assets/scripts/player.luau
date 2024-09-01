type Player = {
	transform: Transform,
	init: (self: Player) -> (),
	update: (self: Player, dt: number, elapsed: number) -> (),
	render: (self: Player, ctx: Context) -> (),
}

local player: Player = entity("player")

function player.init(self)
	player.transform = Transform.new(Vec3.new(0, 0, 0))
	player.transform.scale = Vec3.splat(0.5)
end

function player.update(self, dt, elapsed)
	self.transform.pos.y = math.cos(elapsed) * 1.5
	self.transform:rotate(Vec3.new(0, 1, 0), dt)
	self.transform:rotate(Vec3.new(0, 0, 1), dt)
end

function player.render(self, ctx)
	ctx.scene:batch_model("dragon", self.transform)
end

return player
