extends CanvasLayer
## HUD: resource labels, wave info, build buttons, messages.

signal build_selected(build_type: int)

var label_resources: Label
var label_wave: Label
var label_hint: Label
var label_message: Label
var button_row: HBoxContainer
var _buttons: Dictionary = {}
var _msg_timer: float = 0.0


func _ready() -> void:
	layer = 10
	_build_ui()


func _process(delta: float) -> void:
	if _msg_timer > 0.0:
		_msg_timer -= delta
		if _msg_timer <= 0.0 and label_message:
			label_message.text = ""


func _build_ui() -> void:
	var root := Control.new()
	root.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	root.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(root)

	var panel := PanelContainer.new()
	panel.position = Vector2(12, 12)
	panel.custom_minimum_size = Vector2(320, 0)
	var sb := StyleBoxFlat.new()
	sb.bg_color = Config.COL_HUD_BG
	sb.corner_radius_top_left = 6
	sb.corner_radius_top_right = 6
	sb.corner_radius_bottom_left = 6
	sb.corner_radius_bottom_right = 6
	sb.content_margin_left = 12
	sb.content_margin_right = 12
	sb.content_margin_top = 8
	sb.content_margin_bottom = 8
	panel.add_theme_stylebox_override("panel", sb)
	root.add_child(panel)

	var vbox := VBoxContainer.new()
	panel.add_child(vbox)

	var title := Label.new()
	title.text = "钢核防线"
	title.add_theme_color_override("font_color", Config.COL_CORE)
	title.add_theme_font_size_override("font_size", 22)
	vbox.add_child(title)

	label_resources = Label.new()
	label_resources.add_theme_color_override("font_color", Config.COL_TEXT)
	label_resources.add_theme_font_size_override("font_size", 16)
	vbox.add_child(label_resources)

	label_wave = Label.new()
	label_wave.add_theme_color_override("font_color", Config.COL_TEXT)
	label_wave.add_theme_font_size_override("font_size", 15)
	vbox.add_child(label_wave)

	label_hint = Label.new()
	label_hint.add_theme_color_override("font_color", Config.COL_TEXT.darkened(0.15))
	label_hint.add_theme_font_size_override("font_size", 12)
	label_hint.text = "WASD移动  E采矿  空格/左键射击  1-5建造  R旋转  右键/X拆除"
	vbox.add_child(label_hint)

	button_row = HBoxContainer.new()
	button_row.add_theme_constant_override("separation", 6)
	vbox.add_child(button_row)

	var defs := Config.building_defs()
	var order := [
		Config.BuildType.DRILL,
		Config.BuildType.CONVEYOR,
		Config.BuildType.WALL,
		Config.BuildType.DUO,
		Config.BuildType.SCATTER,
	]
	var i := 1
	for t in order:
		var build_type: int = t
		var def: Dictionary = defs[build_type]
		var btn := Button.new()
		btn.text = "%d %s" % [i, def["name"]]
		btn.tooltip_text = "铜%d 铅%d" % [def["cost_copper"], def["cost_lead"]]
		btn.custom_minimum_size = Vector2(72, 28)
		btn.pressed.connect(_make_build_handler(build_type))
		button_row.add_child(btn)
		_buttons[build_type] = btn
		i += 1

	label_message = Label.new()
	label_message.position = Vector2(12, 200)
	label_message.add_theme_color_override("font_color", Config.COL_CORE)
	label_message.add_theme_font_size_override("font_size", 28)
	label_message.mouse_filter = Control.MOUSE_FILTER_IGNORE
	root.add_child(label_message)

	# Bottom center tip
	var tip := Label.new()
	tip.set_anchors_preset(Control.PRESET_CENTER_BOTTOM)
	tip.position = Vector2(400, 680)
	tip.add_theme_color_override("font_color", Color(0.7, 0.85, 0.84, 0.8))
	tip.add_theme_font_size_override("font_size", 13)
	tip.text = "存活至第 %d 波 · 保护核心" % Config.WIN_WAVE
	tip.mouse_filter = Control.MOUSE_FILTER_IGNORE
	root.add_child(tip)


func update_state(
	inventory: Dictionary,
	wave: int,
	wave_active: bool,
	between_timer: float,
	selected: int,
	core_hp: float,
	core_max: float,
	enemy_count: int
) -> void:
	if label_resources:
		label_resources.text = "铜 %d   铅 %d   核心 %.0f/%.0f" % [
			int(inventory.get("copper", 0)),
			int(inventory.get("lead", 0)),
			core_hp,
			core_max,
		]
	if label_wave:
		if wave_active:
			label_wave.text = "波次 %d / %d   敌人 %d" % [wave, Config.WIN_WAVE, enemy_count]
		elif wave >= Config.WIN_WAVE:
			label_wave.text = "全部波次已清除"
		else:
			var next_w := wave + 1
			label_wave.text = "下一波 %d  ·  %.1fs   已选: %s" % [
				next_w,
				maxf(0.0, between_timer),
				_build_name(selected),
			]
	for t in _buttons.keys():
		var btn: Button = _buttons[t]
		btn.modulate = Color(1.2, 1.4, 1.2) if t == selected else Color.WHITE


func show_message(text: String, duration: float = 3.0) -> void:
	if label_message:
		label_message.text = text
		_msg_timer = duration


func _make_build_handler(build_type: int) -> Callable:
	return func() -> void:
		build_selected.emit(build_type)


func _build_name(t: int) -> String:
	var defs: Dictionary = Config.building_defs()
	if defs.has(t):
		return str(defs[t]["name"])
	return "-"
