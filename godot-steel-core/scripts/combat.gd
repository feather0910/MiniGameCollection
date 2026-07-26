class_name Combat
extends RefCounted
## Waves, enemies, turrets, and bullets.

signal wave_started(wave: int)
signal wave_cleared(wave: int)
signal victory()
signal defeat()

var enemies: Array[Dictionary] = []
var bullets: Array[Dictionary] = []
var next_enemy_id: int = 1
var next_bullet_id: int = 1

var wave: int = 0
var wave_active: bool = false
var between_wave_timer: float = 60.0
var spawn_queue: int = 0
var spawn_timer: float = 0.0
var game_over: bool = false
var won: bool = false

var _rng := RandomNumberGenerator.new()


func _init() -> void:
	_rng.randomize()


func reset() -> void:
	enemies.clear()
	bullets.clear()
	wave = 0
	wave_active = false
	between_wave_timer = 60.0
	spawn_queue = 0
	spawn_timer = 0.0
	game_over = false
	won = false


func tick(game_map: GameMap, delta: float) -> void:
	if game_over:
		return
	_tick_waves(game_map, delta)
	_tick_enemies(game_map, delta)
	_tick_turrets(game_map, delta)
	_tick_bullets(game_map, delta)
	_check_end(game_map)


func _tick_waves(game_map: GameMap, delta: float) -> void:
	if wave_active:
		if spawn_queue > 0:
			spawn_timer -= delta
			if spawn_timer <= 0.0:
				_spawn_one(game_map)
				spawn_queue -= 1
				spawn_timer = maxf(0.5, 1.2 - wave * 0.03)
		elif enemies.is_empty():
			wave_active = false
			wave_cleared.emit(wave)
			if wave >= Config.WIN_WAVE:
				won = true
				game_over = true
				victory.emit()
			else:
				between_wave_timer = 8.0
		return

	between_wave_timer -= delta
	if between_wave_timer <= 0.0 and wave < Config.WIN_WAVE:
		_start_wave(game_map)


func _start_wave(game_map: GameMap) -> void:
	wave += 1
	wave_active = true
	spawn_queue = 2 + wave * 2
	spawn_timer = 3.0
	game_map.mark_path_dirty()
	wave_started.emit(wave)


func _spawn_one(game_map: GameMap) -> void:
	var edges := game_map.edge_spawn_cells()
	if edges.is_empty():
		return
	var spawn: Vector2i = edges[_rng.randi_range(0, edges.size() - 1)]
	var path := game_map.get_path_to_core(spawn)
	if path.is_empty():
		path = game_map.a_star(spawn, game_map.core_origin + Vector2i(1, 1))
	var hp := 40.0 + wave * 12.0
	var speed := 32.0 + wave * 2.5
	enemies.append({
		"id": next_enemy_id,
		"pos": game_map.cell_to_world_center(spawn),
		"hp": hp,
		"max_hp": hp,
		"speed": speed,
		"damage": 8.0 + wave * 1.2,
		"attack_cd": 0.0,
		"path": path,
		"path_i": 0,
		"radius": 10.0,
	})
	next_enemy_id += 1


func _tick_enemies(game_map: GameMap, delta: float) -> void:
	var core_pos := game_map.core_center_world()
	var to_remove: Array[int] = []
	for i in enemies.size():
		var e: Dictionary = enemies[i]
		if float(e["hp"]) <= 0.0:
			to_remove.append(i)
			continue
		e["attack_cd"] = float(e["attack_cd"]) - delta
		var pos: Vector2 = e["pos"]
		# Attack nearby solid buildings
		var cell := game_map.world_to_cell(pos)
		var attacked := false
		var check_cells: Array[Vector2i] = [cell]
		for d in Config.DIR_VECTORS:
			check_cells.append(cell + d)
		for n in check_cells:
			if not game_map.in_bounds_v(n):
				continue
			var bid := game_map.get_building_at(n.x, n.y)
			if bid < 0:
				continue
			var b: Dictionary = game_map.buildings[bid]
			if not b.get("solid", true):
				continue
			var bcell: Vector2i = b["cell"]
			var bsize: int = int(b["size"])
			var bpos := game_map.cell_to_world_center(
				bcell + Vector2i(bsize / 2, bsize / 2)
			)
			if pos.distance_to(bpos) > Config.TILE_SIZE * (0.9 + float(bsize) * 0.5):
				continue
			attacked = true
			if float(e["attack_cd"]) <= 0.0:
				e["attack_cd"] = 0.7
				var destroyed := game_map.damage_building(bid, float(e["damage"]))
				if destroyed and bid == game_map.core_id:
					game_over = true
					defeat.emit()
			break
		if attacked:
			continue

		# Follow path
		var path: Array = e["path"]
		var pi: int = e["path_i"]
		if path.is_empty() or pi >= path.size():
			# Repath
			var new_path := game_map.a_star(cell, game_map.core_origin + Vector2i(1, 1))
			e["path"] = new_path
			e["path_i"] = 0
			path = new_path
			pi = 0
			if path.is_empty():
				# Move directly toward core
				var dir := (core_pos - pos).normalized()
				e["pos"] = pos + dir * float(e["speed"]) * delta
				continue
		var target_cell: Vector2i = path[mini(pi, path.size() - 1)]
		var target := game_map.cell_to_world_center(target_cell)
		var to_t := target - pos
		var dist := to_t.length()
		var step := float(e["speed"]) * delta
		if dist <= step:
			e["pos"] = target
			e["path_i"] = pi + 1
		else:
			e["pos"] = pos + to_t.normalized() * step

	# Remove dead (reverse order)
	to_remove.reverse()
	for i in to_remove:
		enemies.remove_at(i)


func _tick_turrets(game_map: GameMap, delta: float) -> void:
	var defs := Config.building_defs()
	for id in game_map.buildings.keys():
		var b: Dictionary = game_map.buildings[id]
		var t: int = b["type"]
		if t != Config.BuildType.DUO and t != Config.BuildType.SCATTER:
			continue
		b["fire_cd"] = float(b["fire_cd"]) - delta
		if int(b["ammo"]) <= 0:
			continue
		if float(b["fire_cd"]) > 0.0:
			continue
		var def: Dictionary = defs[t]
		var origin := game_map.cell_to_world_center(b["cell"])
		var range_px: float = def["range"]
		var target := _nearest_enemy(origin, range_px)
		if target.is_empty():
			continue
		b["fire_cd"] = float(def["fire_interval"])
		b["ammo"] = int(b["ammo"]) - 1
		var aim: Vector2 = target["pos"]
		var dir := (aim - origin).normalized()
		if t == Config.BuildType.SCATTER:
			var pellets: int = def.get("pellets", 4)
			for p in pellets:
				var ang := dir.angle() + deg_to_rad(_rng.randf_range(-18.0, 18.0))
				_spawn_bullet(origin, Vector2.from_angle(ang), def, Config.Item.LEAD)
		else:
			_spawn_bullet(origin, dir, def, Config.Item.COPPER)


func _spawn_bullet(origin: Vector2, dir: Vector2, def: Dictionary, ammo_kind: int) -> void:
	bullets.append({
		"id": next_bullet_id,
		"pos": origin,
		"vel": dir.normalized() * float(def["bullet_speed"]),
		"damage": float(def["damage"]),
		"life": 1.2,
		"kind": ammo_kind,
		"radius": 4.0,
	})
	next_bullet_id += 1


func _nearest_enemy(origin: Vector2, range_px: float) -> Dictionary:
	var best: Dictionary = {}
	var best_d := range_px
	for e in enemies:
		if float(e["hp"]) <= 0.0:
			continue
		var d: float = origin.distance_to(e["pos"])
		if d <= best_d:
			best_d = d
			best = e
	return best


func _tick_bullets(game_map: GameMap, delta: float) -> void:
	var to_remove: Array[int] = []
	for i in bullets.size():
		var bul: Dictionary = bullets[i]
		bul["life"] = float(bul["life"]) - delta
		if float(bul["life"]) <= 0.0:
			to_remove.append(i)
			continue
		bul["pos"] = bul["pos"] + bul["vel"] * delta
		var pos: Vector2 = bul["pos"]
		# Out of map
		var map_size := game_map.map_pixel_size()
		if pos.x < -20 or pos.y < -20 or pos.x > map_size.x + 20 or pos.y > map_size.y + 20:
			to_remove.append(i)
			continue
		# Hit enemy
		var hit := false
		for e in enemies:
			if float(e["hp"]) <= 0.0:
				continue
			if pos.distance_to(e["pos"]) <= float(e["radius"]) + float(bul["radius"]):
				e["hp"] = float(e["hp"]) - float(bul["damage"])
				hit = true
				break
		if hit:
			to_remove.append(i)
	to_remove.reverse()
	for i in to_remove:
		if i < bullets.size():
			bullets.remove_at(i)


func player_shoot(origin: Vector2, aim: Vector2, damage: float = 10.0) -> void:
	var dir := (aim - origin).normalized()
	if dir.length_squared() < 0.01:
		dir = Vector2.RIGHT
	bullets.append({
		"id": next_bullet_id,
		"pos": origin,
		"vel": dir * 500.0,
		"damage": damage,
		"life": 0.9,
		"kind": Config.Item.COPPER,
		"radius": 3.5,
	})
	next_bullet_id += 1


func _check_end(game_map: GameMap) -> void:
	if game_over:
		return
	if not game_map.core_alive():
		game_over = true
		defeat.emit()
