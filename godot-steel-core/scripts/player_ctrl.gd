class_name PlayerCtrl
extends RefCounted
## Player unit: movement, mining, shooting, build selection.

var pos: Vector2 = Vector2.ZERO
var radius: float = 11.0
var speed: float = 160.0
var mine_range: float = Config.TILE_SIZE * 1.6
var mine_cd: float = 0.0
var shoot_cd: float = 0.0
var selected_build: int = Config.BuildType.DRILL
var build_dir: int = 0  # 0=R 1=D 2=L 3=U

const MINE_INTERVAL := 0.28
const SHOOT_INTERVAL := 0.22

var _prev_keys: Dictionary = {}


func setup(map_center: Vector2) -> void:
	pos = map_center + Vector2(0, Config.TILE_SIZE * 2)


func tick(
	delta: float,
	game_map: GameMap,
	inventory: Dictionary,
	combat: Combat,
	mouse_world: Vector2,
	wants_shoot: bool
) -> void:
	mine_cd = maxf(0.0, mine_cd - delta)
	shoot_cd = maxf(0.0, shoot_cd - delta)

	var dir := Vector2.ZERO
	if Input.is_physical_key_pressed(KEY_W) or Input.is_action_pressed("move_up"):
		dir.y -= 1
	if Input.is_physical_key_pressed(KEY_S) or Input.is_action_pressed("move_down"):
		dir.y += 1
	if Input.is_physical_key_pressed(KEY_A) or Input.is_action_pressed("move_left"):
		dir.x -= 1
	if Input.is_physical_key_pressed(KEY_D) or Input.is_action_pressed("move_right"):
		dir.x += 1
	if dir.length_squared() > 0.0:
		dir = dir.normalized()
		pos = _move_with_collision(game_map, pos + dir * speed * delta)

	if Input.is_physical_key_pressed(KEY_E) or Input.is_action_pressed("mine"):
		_try_mine(game_map, inventory)

	if wants_shoot and shoot_cd <= 0.0:
		shoot_cd = SHOOT_INTERVAL
		combat.player_shoot(pos, mouse_world, 11.0)


func poll_hotkeys() -> void:
	if _edge(KEY_1) or Input.is_action_just_pressed("build_1"):
		selected_build = Config.BuildType.DRILL
	if _edge(KEY_2) or Input.is_action_just_pressed("build_2"):
		selected_build = Config.BuildType.CONVEYOR
	if _edge(KEY_3) or Input.is_action_just_pressed("build_3"):
		selected_build = Config.BuildType.WALL
	if _edge(KEY_4) or Input.is_action_just_pressed("build_4"):
		selected_build = Config.BuildType.DUO
	if _edge(KEY_5) or Input.is_action_just_pressed("build_5"):
		selected_build = Config.BuildType.SCATTER
	# Keep edge state fresh for unused keys
	_edge(KEY_R)
	_edge(KEY_X)


func rotate_build() -> void:
	build_dir = (build_dir + 1) % 4


func _edge(keycode: Key) -> bool:
	var down := Input.is_physical_key_pressed(keycode)
	var was: bool = _prev_keys.get(keycode, false)
	_prev_keys[keycode] = down
	return down and not was


func _try_mine(game_map: GameMap, inventory: Dictionary) -> void:
	if mine_cd > 0.0:
		return
	var cell := game_map.world_to_cell(pos)
	var best_ore := Config.Ore.NONE
	var best_d := 999.0
	for dy in range(-2, 3):
		for dx in range(-2, 3):
			var c := cell + Vector2i(dx, dy)
			if not game_map.in_bounds_v(c):
				continue
			var ore := game_map.get_ore(c.x, c.y)
			if ore == Config.Ore.NONE:
				continue
			var wp := game_map.cell_to_world_center(c)
			var d := pos.distance_to(wp)
			if d <= mine_range and d < best_d:
				best_d = d
				best_ore = ore
	if best_ore == Config.Ore.NONE:
		return
	mine_cd = MINE_INTERVAL
	if best_ore == Config.Ore.COPPER:
		inventory["copper"] = int(inventory["copper"]) + 1
	elif best_ore == Config.Ore.LEAD:
		inventory["lead"] = int(inventory["lead"]) + 1


func _move_with_collision(game_map: GameMap, next: Vector2) -> Vector2:
	var map_size := game_map.map_pixel_size()
	next.x = clampf(next.x, radius, map_size.x - radius)
	next.y = clampf(next.y, radius, map_size.y - radius)
	var try_pos := next
	var cell := game_map.world_to_cell(try_pos)
	for dy in range(-1, 2):
		for dx in range(-1, 2):
			var c := cell + Vector2i(dx, dy)
			if not game_map.in_bounds_v(c):
				continue
			if not game_map.is_solid_at(c.x, c.y):
				continue
			var tl := Vector2(c.x * Config.TILE_SIZE, c.y * Config.TILE_SIZE)
			var br := tl + Vector2(Config.TILE_SIZE, Config.TILE_SIZE)
			var closest := Vector2(
				clampf(try_pos.x, tl.x, br.x),
				clampf(try_pos.y, tl.y, br.y)
			)
			var delta := try_pos - closest
			var d2 := delta.length_squared()
			if d2 < radius * radius and d2 > 0.0001:
				var d := sqrt(d2)
				try_pos = closest + delta / d * radius
			elif d2 <= 0.0001:
				try_pos += Vector2(radius, 0)
	return try_pos
