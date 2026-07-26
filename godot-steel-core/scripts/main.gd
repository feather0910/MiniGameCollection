extends Node2D
## Root controller — game loop orchestration for 钢核防线.

var game_map: GameMap
var logistics: Logistics
var combat: Combat
var player: PlayerCtrl
var inventory: Dictionary = {"copper": Config.START_COPPER, "lead": Config.START_LEAD}

@onready var camera: Camera2D = $Camera2D
@onready var world: Node2D = $World
@onready var hud: CanvasLayer = $HUD

var _paused_end: bool = false


func _ready() -> void:
	# Ensure scripts with class_name are available; construct systems.
	game_map = GameMap.new()
	logistics = Logistics.new()
	combat = Combat.new()
	player = PlayerCtrl.new()
	player.setup(game_map.core_center_world())

	# Wire world view
	if world.has_method("setup"):
		world.setup(game_map, combat, player)
	elif world.get_script() == null:
		# Attach world_view if missing
		var wv := load("res://scripts/world_view.gd")
		world.set_script(wv)
		world.setup(game_map, combat, player)

	# Camera centered on map
	var map_size := game_map.map_pixel_size()
	camera.position = map_size * 0.5
	camera.make_current()

	# HUD signals
	if hud.has_signal("build_selected"):
		hud.build_selected.connect(_on_build_selected)
	combat.wave_started.connect(_on_wave_started)
	combat.wave_cleared.connect(_on_wave_cleared)
	combat.victory.connect(_on_victory)
	combat.defeat.connect(_on_defeat)

	if hud.has_method("show_message"):
		hud.show_message("保护核心 · 建造防线", 2.5)

	# Background color
	RenderingServer.set_default_clear_color(Config.COL_BG)


func _process(delta: float) -> void:
	if _paused_end:
		if Input.is_physical_key_pressed(KEY_ENTER) or Input.is_physical_key_pressed(KEY_SPACE):
			_restart()
		return

	player.poll_hotkeys()

	var mouse_world := get_global_mouse_position()
	var wants_shoot := Input.is_action_pressed("shoot") or Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
	# Don't shoot when clicking UI buttons roughly at top-left
	if mouse_world.y < 0:  # shouldn't happen
		wants_shoot = false
	var screen_mouse := get_viewport().get_mouse_position()
	if screen_mouse.y < 140 and screen_mouse.x < 420:
		# Likely HUD — allow shoot only with Space
		wants_shoot = Input.is_action_pressed("shoot") or Input.is_physical_key_pressed(KEY_SPACE)

	player.tick(delta, game_map, inventory, combat, mouse_world, wants_shoot)

	# Build / demolish
	var hover := game_map.world_to_cell(mouse_world)
	var ghost_ok := game_map.can_place(player.selected_build, hover, player.build_dir)
	if world.has_method("set_ghost"):
		world.set_ghost(hover, ghost_ok)

	if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) and not (
		screen_mouse.y < 140 and screen_mouse.x < 420
	):
		# Place while holding? single click better — use just pressed
		pass
	if Input.is_action_just_pressed("ui_accept"):
		pass

	# Use InputEvent edge via cached — check just pressed mouse
	# Handled in _unhandled_input

	Buildings.tick_drills(game_map, delta)
	Buildings.push_item_from_drill(game_map, inventory)
	logistics.tick(game_map, inventory, delta)
	combat.tick(game_map, delta)

	# Light camera follow
	camera.position = camera.position.lerp(player.pos, clampf(delta * 3.0, 0.0, 1.0))

	_update_hud()


func _unhandled_input(event: InputEvent) -> void:
	if _paused_end:
		return
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if not mb.pressed:
			return
		var cell := game_map.world_to_cell(get_global_mouse_position())
		if mb.button_index == MOUSE_BUTTON_LEFT:
			_try_build(cell)
		elif mb.button_index == MOUSE_BUTTON_RIGHT:
			Buildings.try_demolish(game_map, inventory, cell)
	elif event is InputEventKey:
		var key := event as InputEventKey
		if not key.pressed or key.echo:
			return
		if key.physical_keycode == KEY_X:
			var cell := game_map.world_to_cell(get_global_mouse_position())
			Buildings.try_demolish(game_map, inventory, cell)
		elif key.physical_keycode == KEY_R:
			player.rotate_build()


func _try_build(cell: Vector2i) -> void:
	Buildings.try_place(game_map, inventory, player.selected_build, cell, player.build_dir)


func _on_build_selected(build_type: int) -> void:
	player.selected_build = build_type


func _update_hud() -> void:
	if not hud.has_method("update_state"):
		return
	var core_hp := 0.0
	var core_max := 1200.0
	if game_map.buildings.has(game_map.core_id):
		var core: Dictionary = game_map.buildings[game_map.core_id]
		core_hp = float(core["hp"])
		core_max = float(core["max_hp"])
	hud.update_state(
		inventory,
		combat.wave,
		combat.wave_active,
		combat.between_wave_timer,
		player.selected_build,
		core_hp,
		core_max,
		combat.enemies.size()
	)


func _on_wave_started(wave: int) -> void:
	if hud.has_method("show_message"):
		hud.show_message("第 %d 波来袭！" % wave, 2.0)


func _on_wave_cleared(wave: int) -> void:
	if hud.has_method("show_message"):
		hud.show_message("第 %d 波已清除" % wave, 1.8)


func _on_victory() -> void:
	_paused_end = true
	if hud.has_method("show_message"):
		hud.show_message("胜利！核心屹立 — 按 Enter 重开", 99.0)


func _on_defeat() -> void:
	_paused_end = true
	if hud.has_method("show_message"):
		hud.show_message("核心被摧毁 — 按 Enter 重开", 99.0)


func _restart() -> void:
	get_tree().reload_current_scene()
