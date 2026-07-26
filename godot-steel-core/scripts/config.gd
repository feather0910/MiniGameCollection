class_name Config
extends RefCounted
## Global constants, building definitions, and theme colors for 钢核防线.

const TILE_SIZE := 32
const MAP_W := 40
const MAP_H := 30

const CORE_SIZE := 3
const WIN_WAVE := 15

const START_COPPER := 100
const START_LEAD := 30

# Directions: 0=Right, 1=Down, 2=Left, 3=Up
const DIR_VECTORS: Array[Vector2i] = [
	Vector2i(1, 0),
	Vector2i(0, 1),
	Vector2i(-1, 0),
	Vector2i(0, -1),
]

# --- Colors (dark teal industrial) ---
const COL_BG := Color(0.05, 0.10, 0.12)
const COL_GRID := Color(0.10, 0.18, 0.20, 0.55)
const COL_FLOOR := Color(0.09, 0.16, 0.18)
const COL_FLOOR_ALT := Color(0.08, 0.14, 0.16)
const COL_COPPER := Color(0.92, 0.55, 0.18)
const COL_LEAD := Color(0.62, 0.35, 0.78)
const COL_CORE := Color(0.18, 0.85, 0.78)
const COL_CORE_INNER := Color(0.10, 0.55, 0.52)
const COL_WALL := Color(0.35, 0.48, 0.50)
const COL_DRILL := Color(0.55, 0.62, 0.35)
const COL_CONVEYOR := Color(0.30, 0.42, 0.45)
const COL_CONVEYOR_ARROW := Color(0.55, 0.75, 0.78)
const COL_DUO := Color(0.75, 0.55, 0.25)
const COL_SCATTER := Color(0.70, 0.40, 0.80)
const COL_PLAYER := Color(0.40, 0.95, 0.85)
const COL_ENEMY := Color(0.90, 0.25, 0.22)
const COL_BULLET := Color(1.0, 0.92, 0.45)
const COL_BULLET_LEAD := Color(0.85, 0.55, 0.95)
const COL_GHOST_OK := Color(0.3, 0.9, 0.5, 0.35)
const COL_GHOST_BAD := Color(0.9, 0.2, 0.2, 0.35)
const COL_HUD_BG := Color(0.06, 0.12, 0.14, 0.88)
const COL_TEXT := Color(0.85, 0.95, 0.94)

enum Ore { NONE, COPPER, LEAD }
enum Item { NONE, COPPER, LEAD }
enum BuildType { NONE, DRILL, CONVEYOR, WALL, DUO, SCATTER, CORE }

static func item_color(item: int) -> Color:
	match item:
		Item.COPPER:
			return COL_COPPER
		Item.LEAD:
			return COL_LEAD
		_:
			return Color.WHITE


static func ore_color(ore: int) -> Color:
	match ore:
		Ore.COPPER:
			return COL_COPPER
		Ore.LEAD:
			return COL_LEAD
		_:
			return COL_FLOOR


## Building definition dictionary helper.
static func building_defs() -> Dictionary:
	return {
		BuildType.DRILL: {
			"name": "钻头",
			"key": 1,
			"size": 1,
			"cost_copper": 12,
			"cost_lead": 0,
			"hp": 80,
			"color": COL_DRILL,
			"solid": true,
			"needs_ore": true,
		},
		BuildType.CONVEYOR: {
			"name": "传送带",
			"key": 2,
			"size": 1,
			"cost_copper": 1,
			"cost_lead": 0,
			"hp": 40,
			"color": COL_CONVEYOR,
			"solid": false,
			"needs_ore": false,
			"rotatable": true,
		},
		BuildType.WALL: {
			"name": "墙",
			"key": 3,
			"size": 1,
			"cost_copper": 4,
			"cost_lead": 0,
			"hp": 200,
			"color": COL_WALL,
			"solid": true,
			"needs_ore": false,
		},
		BuildType.DUO: {
			"name": "双联炮",
			"key": 4,
			"size": 1,
			"cost_copper": 25,
			"cost_lead": 0,
			"hp": 120,
			"color": COL_DUO,
			"solid": true,
			"needs_ore": false,
			"ammo_item": Item.COPPER,
			"ammo_max": 20,
			"range": 6.5 * TILE_SIZE,
			"fire_interval": 0.35,
			"damage": 12,
			"bullet_speed": 420.0,
		},
		BuildType.SCATTER: {
			"name": "散射炮",
			"key": 5,
			"size": 1,
			"cost_copper": 20,
			"cost_lead": 15,
			"hp": 100,
			"color": COL_SCATTER,
			"solid": true,
			"needs_ore": false,
			"ammo_item": Item.LEAD,
			"ammo_max": 30,
			"range": 5.0 * TILE_SIZE,
			"fire_interval": 0.55,
			"damage": 8,
			"bullet_speed": 380.0,
			"pellets": 4,
		},
		BuildType.CORE: {
			"name": "核心",
			"key": 0,
			"size": CORE_SIZE,
			"cost_copper": 0,
			"cost_lead": 0,
			"hp": 1200,
			"color": COL_CORE,
			"solid": true,
			"needs_ore": false,
		},
	}


static func build_hotkey_type(key_index: int) -> int:
	## 1..5 → BuildType
	match key_index:
		1: return BuildType.DRILL
		2: return BuildType.CONVEYOR
		3: return BuildType.WALL
		4: return BuildType.DUO
		5: return BuildType.SCATTER
		_: return BuildType.NONE
