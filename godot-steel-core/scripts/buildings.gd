class_name Buildings
extends RefCounted
## Building placement helpers and extractor (drill) ticking.

static func try_place(
	game_map: GameMap,
	inventory: Dictionary,
	build_type: int,
	cell: Vector2i,
	dir: int
) -> bool:
	if build_type == Config.BuildType.NONE or build_type == Config.BuildType.CORE:
		return false
	var defs := Config.building_defs()
	if not defs.has(build_type):
		return false
	var def: Dictionary = defs[build_type]
	var cost_c: int = def["cost_copper"]
	var cost_l: int = def["cost_lead"]
	if int(inventory.get("copper", 0)) < cost_c:
		return false
	if int(inventory.get("lead", 0)) < cost_l:
		return false
	if not game_map.can_place(build_type, cell, dir):
		return false
	var id := game_map.place_building(build_type, cell, dir)
	if id < 0:
		return false
	inventory["copper"] = int(inventory["copper"]) - cost_c
	inventory["lead"] = int(inventory["lead"]) - cost_l
	return true


static func try_demolish(game_map: GameMap, inventory: Dictionary, cell: Vector2i) -> bool:
	var id := game_map.get_building_at(cell.x, cell.y)
	if id < 0 or id == game_map.core_id:
		return false
	return game_map.remove_building(id, inventory)


static func tick_drills(game_map: GameMap, delta: float) -> void:
	## Drills extract ore into their own item slot (then logistics moves it).
	const EXTRACT_INTERVAL := 0.85
	for id in game_map.buildings.keys():
		var b: Dictionary = game_map.buildings[id]
		if int(b["type"]) != Config.BuildType.DRILL:
			continue
		if int(b["item"]) != Config.Item.NONE:
			continue
		var cell: Vector2i = b["cell"]
		var ore := game_map.get_ore(cell.x, cell.y)
		if ore == Config.Ore.NONE:
			continue
		b["extract_cd"] = float(b["extract_cd"]) - delta
		if float(b["extract_cd"]) > 0.0:
			continue
		b["extract_cd"] = EXTRACT_INTERVAL
		if ore == Config.Ore.COPPER:
			b["item"] = Config.Item.COPPER
		elif ore == Config.Ore.LEAD:
			b["item"] = Config.Item.LEAD


static func push_item_from_drill(game_map: GameMap, inventory: Dictionary) -> void:
	## After extract, try to dump onto adjacent conveyor / core / turret.
	for id in game_map.buildings.keys():
		var b: Dictionary = game_map.buildings[id]
		if int(b["type"]) != Config.BuildType.DRILL:
			continue
		var item: int = int(b["item"])
		if item == Config.Item.NONE:
			continue
		var cell: Vector2i = b["cell"]
		# Prefer conveyor in facing dirs, then any neighbor
		var neighbors: Array[Vector2i] = []
		for d in Config.DIR_VECTORS:
			neighbors.append(cell + d)
		for n in neighbors:
			if _try_deposit(game_map, inventory, n, item):
				b["item"] = Config.Item.NONE
				break


static func _try_deposit(game_map: GameMap, inventory: Dictionary, cell: Vector2i, item: int) -> bool:
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
