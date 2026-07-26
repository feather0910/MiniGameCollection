class_name GameMap
extends RefCounted
## Grid map: ores, occupancy, buildings, pathfinding (BFS / A*).

var width: int = Config.MAP_W
var height: int = Config.MAP_H
var ores: PackedInt32Array = PackedInt32Array()
## building id at cell, -1 = empty
var occupancy: PackedInt32Array = PackedInt32Array()
var buildings: Dictionary = {}  # id -> Dictionary building data
var next_building_id: int = 1
var core_id: int = -1
var core_origin: Vector2i = Vector2i.ZERO

## Cached path from each edge spawn; rebuilt when solid buildings change.
var _path_dirty: bool = true
var _path_cache: Dictionary = {}  # Vector2i spawn -> Array[Vector2i]


func _init(w: int = Config.MAP_W, h: int = Config.MAP_H) -> void:
	width = w
	height = h
	var n := width * height
	ores.resize(n)
	occupancy.resize(n)
	ores.fill(Config.Ore.NONE)
	occupancy.fill(-1)
	_generate_ores()
	_place_core()


func idx(x: int, y: int) -> int:
	return y * width + x


func in_bounds(x: int, y: int) -> bool:
	return x >= 0 and y >= 0 and x < width and y < height


func in_bounds_v(p: Vector2i) -> bool:
	return in_bounds(p.x, p.y)


func get_ore(x: int, y: int) -> int:
	if not in_bounds(x, y):
		return Config.Ore.NONE
	return ores[idx(x, y)]


func get_building_at(x: int, y: int) -> int:
	if not in_bounds(x, y):
		return -1
	return occupancy[idx(x, y)]


func get_building(id: int) -> Dictionary:
	return buildings.get(id, {})


func is_solid_at(x: int, y: int) -> bool:
	var id := get_building_at(x, y)
	if id < 0:
		return false
	var b: Dictionary = buildings[id]
	return b.get("solid", true)


func is_walkable(x: int, y: int) -> bool:
	if not in_bounds(x, y):
		return false
	return not is_solid_at(x, y)


func world_to_cell(pos: Vector2) -> Vector2i:
	return Vector2i(floori(pos.x / Config.TILE_SIZE), floori(pos.y / Config.TILE_SIZE))


func cell_to_world_center(cell: Vector2i) -> Vector2:
	return Vector2(
		cell.x * Config.TILE_SIZE + Config.TILE_SIZE * 0.5,
		cell.y * Config.TILE_SIZE + Config.TILE_SIZE * 0.5
	)


func map_pixel_size() -> Vector2:
	return Vector2(width * Config.TILE_SIZE, height * Config.TILE_SIZE)


func core_center_world() -> Vector2:
	var c := core_origin + Vector2i(Config.CORE_SIZE / 2, Config.CORE_SIZE / 2)
	return cell_to_world_center(c)


func mark_path_dirty() -> void:
	_path_dirty = true
	_path_cache.clear()


func _generate_ores() -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 42
	# Copper patches
	_scatter_ore_patch(rng, Config.Ore.COPPER, 7, 3, 6)
	# Lead patches
	_scatter_ore_patch(rng, Config.Ore.LEAD, 5, 2, 5)


func _scatter_ore_patch(rng: RandomNumberGenerator, ore: int, count: int, rmin: int, rmax: int) -> void:
	var cx := width / 2
	var cy := height / 2
	for _i in count:
		var px := rng.randi_range(2, width - 3)
		var py := rng.randi_range(2, height - 3)
		# Keep core area relatively clear
		if abs(px - cx) < 5 and abs(py - cy) < 5:
			px = rng.randi_range(2, width - 3)
			py = rng.randi_range(2, height - 3)
		var rad := rng.randi_range(rmin, rmax)
		for dy in range(-rad, rad + 1):
			for dx in range(-rad, rad + 1):
				if dx * dx + dy * dy > rad * rad:
					continue
				var x := px + dx
				var y := py + dy
				if not in_bounds(x, y):
					continue
				if abs(x - cx) <= 2 and abs(y - cy) <= 2:
					continue
				if rng.randf() < 0.72:
					ores[idx(x, y)] = ore


func _place_core() -> void:
	core_origin = Vector2i(
		(width - Config.CORE_SIZE) / 2,
		(height - Config.CORE_SIZE) / 2
	)
	var b := {
		"id": next_building_id,
		"type": Config.BuildType.CORE,
		"cell": core_origin,
		"size": Config.CORE_SIZE,
		"dir": 0,
		"hp": 1200.0,
		"max_hp": 1200.0,
		"solid": true,
		"ammo": 0,
		"ammo_max": 0,
		"fire_cd": 0.0,
		"item": Config.Item.NONE,
		"extract_cd": 0.0,
	}
	core_id = next_building_id
	buildings[core_id] = b
	next_building_id += 1
	_occupy(b)


func _occupy(b: Dictionary) -> void:
	var cell: Vector2i = b["cell"]
	var size: int = b["size"]
	for dy in size:
		for dx in size:
			occupancy[idx(cell.x + dx, cell.y + dy)] = b["id"]


func _unoccupy(b: Dictionary) -> void:
	var cell: Vector2i = b["cell"]
	var size: int = b["size"]
	for dy in size:
		for dx in size:
			var i := idx(cell.x + dx, cell.y + dy)
			if occupancy[i] == b["id"]:
				occupancy[i] = -1


func can_place(build_type: int, cell: Vector2i, _dir: int = 0) -> bool:
	var defs := Config.building_defs()
	if not defs.has(build_type):
		return false
	var def: Dictionary = defs[build_type]
	var size: int = def["size"]
	if cell.x < 0 or cell.y < 0 or cell.x + size > width or cell.y + size > height:
		return false
	for dy in size:
		for dx in size:
			var x := cell.x + dx
			var y := cell.y + dy
			if get_building_at(x, y) >= 0:
				return false
	if def.get("needs_ore", false):
		# Drill must sit on ore
		if get_ore(cell.x, cell.y) == Config.Ore.NONE:
			return false
	return true


func place_building(build_type: int, cell: Vector2i, dir: int = 0) -> int:
	if not can_place(build_type, cell, dir):
		return -1
	var defs := Config.building_defs()
	var def: Dictionary = defs[build_type]
	var b := {
		"id": next_building_id,
		"type": build_type,
		"cell": cell,
		"size": def["size"],
		"dir": dir % 4,
		"hp": float(def["hp"]),
		"max_hp": float(def["hp"]),
		"solid": def.get("solid", true),
		"ammo": 0,
		"ammo_max": int(def.get("ammo_max", 0)),
		"fire_cd": 0.0,
		"item": Config.Item.NONE,
		"extract_cd": 0.0,
	}
	var id: int = next_building_id
	buildings[id] = b
	next_building_id += 1
	_occupy(b)
	if b["solid"]:
		mark_path_dirty()
	return id


func remove_building(id: int, refund_to: Dictionary = {}) -> bool:
	if id == core_id:
		return false
	if not buildings.has(id):
		return false
	var b: Dictionary = buildings[id]
	var solid: bool = b.get("solid", true)
	_unoccupy(b)
	# Refund 50%
	var defs := Config.building_defs()
	var def: Dictionary = defs[b["type"]]
	if not refund_to.is_empty():
		refund_to["copper"] = refund_to.get("copper", 0) + int(def["cost_copper"] / 2)
		refund_to["lead"] = refund_to.get("lead", 0) + int(def["cost_lead"] / 2)
	buildings.erase(id)
	if solid:
		mark_path_dirty()
	return true


func damage_building(id: int, amount: float) -> bool:
	## Returns true if destroyed.
	if not buildings.has(id):
		return false
	var b: Dictionary = buildings[id]
	b["hp"] = float(b["hp"]) - amount
	if float(b["hp"]) <= 0.0:
		if id == core_id:
			b["hp"] = 0.0
			return true
		remove_building(id)
		return true
	return false


func core_alive() -> bool:
	if not buildings.has(core_id):
		return false
	return float(buildings[core_id]["hp"]) > 0.0


func accept_item(cell: Vector2i, item: int) -> bool:
	## Try to put item into building at cell (core inventory handled externally via callback).
	var id := get_building_at(cell.x, cell.y)
	if id < 0:
		return false
	var b: Dictionary = buildings[id]
	var t: int = b["type"]
	if t == Config.BuildType.CONVEYOR:
		if int(b["item"]) == Config.Item.NONE:
			b["item"] = item
			return true
		return false
	if t == Config.BuildType.DUO or t == Config.BuildType.SCATTER:
		var defs := Config.building_defs()
		var def: Dictionary = defs[t]
		var need: int = def.get("ammo_item", Config.Item.NONE)
		if item == need and int(b["ammo"]) < int(b["ammo_max"]):
			b["ammo"] = int(b["ammo"]) + 1
			return true
		return false
	if t == Config.BuildType.CORE:
		return true  # caller adds to inventory
	return false


## --- Pathfinding ---

func find_path(from: Vector2i, to: Vector2i) -> Array[Vector2i]:
	## BFS shortest path on walkable cells. Returns cells including from, excluding blocked to if needed.
	if not in_bounds_v(from) or not in_bounds_v(to):
		return [] as Array[Vector2i]
	# Allow destination even if solid (attack target) — path to nearest walkable adjacent if solid.
	var goal := to
	if is_solid_at(to.x, to.y):
		var adj := _nearest_walkable_adjacent(to, from)
		if adj == Vector2i(-999, -999):
			return [] as Array[Vector2i]
		goal = adj

	var came: Dictionary = {}
	var q: Array[Vector2i] = [from]
	came[from] = Vector2i(-1, -1)
	var head := 0
	var found := false
	while head < q.size():
		var cur: Vector2i = q[head]
		head += 1
		if cur == goal:
			found = true
			break
		for d in Config.DIR_VECTORS:
			var n: Vector2i = cur + d
			if not in_bounds_v(n):
				continue
			if came.has(n):
				continue
			if not is_walkable(n.x, n.y) and n != goal:
				continue
			came[n] = cur
			q.append(n)
	if not found:
		return [] as Array[Vector2i]
	var path: Array[Vector2i] = []
	var c := goal
	while c != Vector2i(-1, -1):
		path.push_front(c)
		c = came[c]
	return path


func get_path_to_core(spawn: Vector2i) -> Array[Vector2i]:
	if _path_cache.has(spawn) and not _path_dirty:
		return _path_cache[spawn]
	var target := core_origin + Vector2i(Config.CORE_SIZE / 2, Config.CORE_SIZE / 2)
	var path := find_path(spawn, target)
	_path_cache[spawn] = path
	_path_dirty = false
	return path


func _nearest_walkable_adjacent(solid_cell: Vector2i, prefer: Vector2i) -> Vector2i:
	var best := Vector2i(-999, -999)
	var best_d := 999999
	for d in Config.DIR_VECTORS:
		var n: Vector2i = solid_cell + d
		if not in_bounds_v(n):
			continue
		if not is_walkable(n.x, n.y):
			continue
		var dist := absi(n.x - prefer.x) + absi(n.y - prefer.y)
		if dist < best_d:
			best_d = dist
			best = n
	return best


func edge_spawn_cells() -> Array[Vector2i]:
	var cells: Array[Vector2i] = []
	for x in width:
		if is_walkable(x, 0):
			cells.append(Vector2i(x, 0))
		if is_walkable(x, height - 1):
			cells.append(Vector2i(x, height - 1))
	for y in range(1, height - 1):
		if is_walkable(0, y):
			cells.append(Vector2i(0, y))
		if is_walkable(width - 1, y):
			cells.append(Vector2i(width - 1, y))
	return cells


func a_star(from: Vector2i, to: Vector2i) -> Array[Vector2i]:
	## A* alternative used when BFS paths get blocked; Manhattan heuristic.
	if not in_bounds_v(from):
		return [] as Array[Vector2i]
	var goal := to
	if is_solid_at(to.x, to.y):
		var adj := _nearest_walkable_adjacent(to, from)
		if adj == Vector2i(-999, -999):
			return [] as Array[Vector2i]
		goal = adj

	var open: Array[Vector2i] = [from]
	var came: Dictionary = {}
	var g: Dictionary = {from: 0}
	var f: Dictionary = {from: _heuristic(from, goal)}
	var closed: Dictionary = {}

	while not open.is_empty():
		var best_i := 0
		var best_f: int = f[open[0]]
		for i in range(1, open.size()):
			var fi: int = f.get(open[i], 999999)
			if fi < best_f:
				best_f = fi
				best_i = i
		var cur: Vector2i = open[best_i]
		open.remove_at(best_i)
		if cur == goal:
			var path: Array[Vector2i] = []
			var c := cur
			path.push_front(c)
			while came.has(c):
				c = came[c]
				path.push_front(c)
			return path
		closed[cur] = true
		for d in Config.DIR_VECTORS:
			var n: Vector2i = cur + d
			if not in_bounds_v(n):
				continue
			if closed.has(n):
				continue
			if not is_walkable(n.x, n.y) and n != goal:
				continue
			var ng: int = int(g[cur]) + 1
			if not g.has(n) or ng < int(g[n]):
				came[n] = cur
				g[n] = ng
				f[n] = ng + _heuristic(n, goal)
				if not open.has(n):
					open.append(n)
	return [] as Array[Vector2i]


func _heuristic(a: Vector2i, b: Vector2i) -> int:
	return absi(a.x - b.x) + absi(a.y - b.y)
