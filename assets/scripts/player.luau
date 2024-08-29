type Player = {
	transform: Transform,
	init: (self: Player) -> (),
	update: (self: Player, dt: number, elapsed: number) -> (),
	render: (self: Player) -> (),
}

local player: Player = entity("player")

function player.init(self)
	player.transform = Transform.new(Vec3.new(0, 0, 0))
	player.transform.scale = Vec3.splat(0.5)
end

function player.update(self, dt, elapsed)
	player.transform.pos.y = math.cos(elapsed) * 1.5
	print(player.transform.pos.y)
	player.transform:rotate_y(dt)
	player.transform:rotate_z(dt)
end

function player.render(self)
	game.render_model("dragon", self.transform)
end

return player
