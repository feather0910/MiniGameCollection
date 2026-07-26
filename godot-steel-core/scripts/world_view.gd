extends Node2D
## Draws map, buildings, entities with CanvasItem drawing (stylized geometric).

var game_map: GameMap
var combat: Combat
var player: PlayerCtrl
var ghost_cell: Vector2i = Vector2i(-1, -1)
var ghost_ok: bool = false
var anim_t: float = 0.0


func setup(p_map: GameMap, p_combat: Combat, p_player: PlayerCtrl) -> void:
	game_map = p_map
	combat = p_combat
	player = p_player
	queue_redraw()


func _process(delta: float) -> void:
	anim_t += delta
	queue_redraw()


func _draw() -> void:
	if game_map == null:
		return
	_draw_floor()
	_draw_ores()
	_draw_buildings()
	_draw_items_on_belts()
	_draw_ghost()
	_draw_enemies()
	_draw_bullets()
	_draw_player()


func _draw_floor() -> void:
	var ts := Config.TILE_SIZE
	for y in game_map.height:
		for x in game_map.width:
			var col := Config.COL_FLOOR if ((x + y) % 2 == 0) else Config.COL_FLOOR_ALT
			draw_rect(Rect2(x * ts, y * ts, ts, ts), col)
	# Subtle grid
	for x in game_map.width + 1:
		draw_line(Vector2(x * ts, 0), Vector2(x * ts, game_map.height * ts), Config.COL_GRID, 1.0)
	for y in game_map.height + 1:
		draw_line(Vector2(0, y * ts), Vector2(game_map.width * ts, y * ts), Config.COL_GRID, 1.0)


func _draw_ores() -> void:
	var ts := Config.TILE_SIZE
	for y in game_map.height:
		for x in game_map.width:
			var ore := game_map.get_ore(x, y)
			if ore == Config.Ore.NONE:
				continue
			var col := Config.ore_color(ore)
			var cx := x * ts + ts * 0.5
			var cy := y * ts + ts * 0.5
			# Scattered small diamonds
			var base := Color(col.r, col.g, col.b, 0.55)
			draw_circle(Vector2(cx - 6, cy - 4), 3.5, base)
			draw_circle(Vector2(cx + 5, cy + 3), 4.0, base)
			draw_circle(Vector2(cx + 1, cy - 6), 2.8, base)
			draw_rect(Rect2(cx - 3, cy - 3, 6, 6), col)


func _draw_buildings() -> void:
	for id in game_map.buildings.keys():
		var b: Dictionary = game_map.buildings[id]
		match int(b["type"]):
			Config.BuildType.CORE:
				_draw_core(b)
			Config.BuildType.DRILL:
				_draw_drill(b)
			Config.BuildType.CONVEYOR:
				_draw_conveyor(b)
			Config.BuildType.WALL:
				_draw_wall(b)
			Config.BuildType.DUO:
				_draw_duo(b)
			Config.BuildType.SCATTER:
				_draw_scatter(b)


func _cell_rect(cell: Vector2i, size: int = 1) -> Rect2:
	var ts := Config.TILE_SIZE
	return Rect2(cell.x * ts, cell.y * ts, ts * size, ts * size)


func _draw_core(b: Dictionary) -> void:
	var cell: Vector2i = b["cell"]
	var r := _cell_rect(cell, int(b["size"]))
	var c := r.get_center()
	var pts := PackedVector2Array()
	var rad := r.size.x * 0.42
	for i in 6:
		var a := TAU * float(i) / 6.0 - PI / 2.0
		pts.append(c + Vector2(cos(a), sin(a)) * rad)
	draw_colored_polygon(pts, Config.COL_CORE_INNER)
	draw_polyline(pts + PackedVector2Array([pts[0]]), Config.COL_CORE, 3.0, true)
	var inner := PackedVector2Array()
	for i in 6:
		var a := TAU * float(i) / 6.0 - PI / 2.0 + anim_t * 0.4
		inner.append(c + Vector2(cos(a), sin(a)) * rad * 0.45)
	draw_colored_polygon(inner, Config.COL_CORE)
	_draw_hp_bar(r, float(b["hp"]), float(b["max_hp"]))


func _draw_drill(b: Dictionary) -> void:
	var cell: Vector2i = b["cell"]
	var r := _cell_rect(cell)
	draw_rect(r.grow(-2), Config.COL_DRILL)
	var c := r.get_center()
	var ang := anim_t * 4.0
	draw_line(c, c + Vector2(cos(ang), sin(ang)) * 10.0, Color(0.2, 0.25, 0.15), 3.0)
	draw_circle(c, 4.0, Color(0.25, 0.3, 0.18))
	if int(b["item"]) != Config.Item.NONE:
		draw_circle(c + Vector2(8, -8), 3.5, Config.item_color(int(b["item"])))
	_draw_hp_bar(r, float(b["hp"]), float(b["max_hp"]))


func _draw_conveyor(b: Dictionary) -> void:
	var cell: Vector2i = b["cell"]
	var r := _cell_rect(cell)
	draw_rect(r.grow(-1), Config.COL_CONVEYOR)
	var c := r.get_center()
	var d: Vector2 = Vector2(Config.DIR_VECTORS[int(b["dir"])])
	var tip := c + d * 10.0
	var left := tip - d * 8.0 + Vector2(-d.y, d.x) * 5.0
	var right := tip - d * 8.0 - Vector2(-d.y, d.x) * 5.0
	draw_colored_polygon(PackedVector2Array([tip, left, right]), Config.COL_CONVEYOR_ARROW)
	# Moving dashes
	var phase := fmod(anim_t * 2.5, 1.0)
	var p := c - d * 8.0 + d * phase * 16.0
	draw_circle(p, 2.0, Config.COL_CONVEYOR_ARROW.darkened(0.2))


func _draw_wall(b: Dictionary) -> void:
	var r := _cell_rect(b["cell"])
	draw_rect(r.grow(-1), Config.COL_WALL)
	draw_rect(r.grow(-6), Config.COL_WALL.darkened(0.25))
	_draw_hp_bar(r, float(b["hp"]), float(b["max_hp"]))


func _draw_duo(b: Dictionary) -> void:
	var r := _cell_rect(b["cell"])
	var c := r.get_center()
	draw_rect(r.grow(-2), Config.COL_DUO)
	draw_circle(c, 7.0, Config.COL_DUO.lightened(0.2))
	draw_rect(Rect2(c.x - 2, c.y - 12, 4, 10), Color(0.4, 0.3, 0.15))
	draw_rect(Rect2(c.x + 2, c.y - 10, 3, 8), Color(0.4, 0.3, 0.15))
	# Ammo pips
	var ammo: int = int(b["ammo"])
	if ammo > 0:
		draw_circle(c + Vector2(-8, 8), 2.5, Config.COL_COPPER)
	_draw_hp_bar(r, float(b["hp"]), float(b["max_hp"]))


func _draw_scatter(b: Dictionary) -> void:
	var r := _cell_rect(b["cell"])
	var c := r.get_center()
	draw_rect(r.grow(-2), Config.COL_SCATTER)
	draw_circle(c, 8.0, Config.COL_SCATTER.lightened(0.15))
	for i in 3:
		var a := -PI / 2 + (i - 1) * 0.35
		draw_line(c, c + Vector2(cos(a), sin(a)) * 12.0, Color(0.35, 0.2, 0.4), 2.5)
	if int(b["ammo"]) > 0:
		draw_circle(c + Vector2(-8, 8), 2.5, Config.COL_LEAD)
	_draw_hp_bar(r, float(b["hp"]), float(b["max_hp"]))


func _draw_items_on_belts() -> void:
	for id in game_map.buildings.keys():
		var b: Dictionary = game_map.buildings[id]
		if int(b["type"]) != Config.BuildType.CONVEYOR:
			continue
		var item: int = int(b["item"])
		if item == Config.Item.NONE:
			continue
		var c := _cell_rect(b["cell"]).get_center()
		draw_circle(c, 4.5, Config.item_color(item))


func _draw_ghost() -> void:
	if player == null or player.selected_build == Config.BuildType.NONE:
		return
	if ghost_cell.x < 0:
		return
	var col := Config.COL_GHOST_OK if ghost_ok else Config.COL_GHOST_BAD
	var r := _cell_rect(ghost_cell)
	draw_rect(r, col)
	if player.selected_build == Config.BuildType.CONVEYOR:
		var c := r.get_center()
		var d: Vector2 = Vector2(Config.DIR_VECTORS[player.build_dir])
		draw_line(c - d * 8.0, c + d * 8.0, Color(1, 1, 1, 0.6), 2.0)


func _draw_enemies() -> void:
	if combat == null:
		return
	for e in combat.enemies:
		var p: Vector2 = e["pos"]
		var pts := PackedVector2Array([
			p + Vector2(0, -10),
			p + Vector2(9, 8),
			p + Vector2(-9, 8),
		])
		draw_colored_polygon(pts, Config.COL_ENEMY)
		draw_polyline(pts + PackedVector2Array([pts[0]]), Config.COL_ENEMY.lightened(0.3), 1.5, true)
		# HP
		var ratio := float(e["hp"]) / float(e["max_hp"])
		draw_rect(Rect2(p.x - 10, p.y - 16, 20, 3), Color(0.1, 0.1, 0.1, 0.7))
		draw_rect(Rect2(p.x - 10, p.y - 16, 20 * ratio, 3), Color(0.95, 0.3, 0.25))


func _draw_bullets() -> void:
	if combat == null:
		return
	for bul in combat.bullets:
		var col := Config.COL_BULLET if int(bul["kind"]) == Config.Item.COPPER else Config.COL_BULLET_LEAD
		draw_circle(bul["pos"], float(bul["radius"]), col)


func _draw_player() -> void:
	if player == null:
		return
	var p := player.pos
	draw_circle(p, player.radius, Config.COL_PLAYER)
	draw_circle(p, player.radius * 0.45, Config.COL_PLAYER.darkened(0.35))
	# Facing mark toward mouse is handled lightly
	draw_arc(p, player.radius + 2.0, 0, TAU, 24, Config.COL_PLAYER.lightened(0.2), 1.5)


func _draw_hp_bar(r: Rect2, hp: float, max_hp: float) -> void:
	if hp >= max_hp * 0.99:
		return
	var ratio := clampf(hp / max_hp, 0.0, 1.0)
	var bar := Rect2(r.position.x + 2, r.position.y - 5, r.size.x - 4, 3)
	draw_rect(bar, Color(0.05, 0.05, 0.05, 0.75))
	draw_rect(Rect2(bar.position, Vector2(bar.size.x * ratio, bar.size.y)), Color(0.3, 0.9, 0.5))


func set_ghost(cell: Vector2i, ok: bool) -> void:
	ghost_cell = cell
	ghost_ok = ok
