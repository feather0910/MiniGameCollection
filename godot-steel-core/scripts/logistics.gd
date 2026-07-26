class_name Logistics
extends RefCounted
## Conveyor belt item movement simulation.

var _accum: float = 0.0
const MOVE_INTERVAL := 0.18


func tick(game_map: GameMap, inventory: Dictionary, delta: float) -> void:
	_accum += delta
	while _accum >= MOVE_INTERVAL:
		_accum -= MOVE_INTERVAL
		_step(game_map, inventory)


func _step(game_map: GameMap, inventory: Dictionary) -> void:
	## Collect moves first so simultaneous updates don't chain infinitely in one step.
	var moves: Array[Dictionary] = []
	for id in game_map.buildings.keys():
		var b: Dictionary = game_map.buildings[id]
		if int(b["type"]) != Config.BuildType.CONVEYOR:
			continue
		var item: int = int(b["item"])
		if item == Config.Item.NONE:
			continue
		var cell: Vector2i = b["cell"]
		var dir: int = int(b["dir"]) % 4
		var dest: Vector2i = cell + Config.DIR_VECTORS[dir]
		moves.append({"from_id": id, "item": item, "dest": dest})

	# Clear sources that successfully transfer
	for m in moves:
		var dest: Vector2i = m["dest"]
		var item: int = m["item"]
		var from_id: int = m["from_id"]
		if not game_map.buildings.has(from_id):
			continue
		var src: Dictionary = game_map.buildings[from_id]
		if int(src["item"]) != item:
			continue
		if _deposit(game_map, inventory, dest, item):
			src["item"] = Config.Item.NONE


func _deposit(game_map: GameMap, inventory: Dictionary, cell: Vector2i, item: int) -> bool:
	if not game_map.in_bounds_v(cell):
		return false
	var id := game_map.get_building_at(cell.x, cell.y)
	if id < 0:
		return false
	var b: Dictionary = game_map.buildings[id]
	var t: int = b["type"]
	if t == Config.BuildType.CORE:
		if item == Config.Item.COPPER:
			inventory["copper"] = int(inventory["copper"]) + 1
		elif item == Config.Item.LEAD:
			inventory["lead"] = int(inventory["lead"]) + 1
		return true
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
	return false
