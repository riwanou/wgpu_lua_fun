type Player = {
	pos: Vec3,
	init: (self: Player) -> (),
	update: (self: Player, dt: number) -> (),
	render: (self: Player, scene: Scene) -> (),
}

local player: Player = entity("player")

function player.init(self)
	player.pos = Vec3.new(0, 0, 0)
end

function player.update(self, dt)
	player.pos += Vec3.new(0, 0, 0)
end

function player.render(self, scene)
	scene.render_model("cube", self.pos)
end

return player
