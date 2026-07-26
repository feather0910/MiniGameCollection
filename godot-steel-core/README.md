# 钢核防线

Mindustry 风格的工厂塔防精简版 —— **Godot 4.3 GDScript** 实现。

用钻头开采铜/铅矿，传送带运输资源，建造双联炮与散射炮，抵御从地图边缘袭来的敌浪，保护中央核心。

## 运行方式

1. 安装 [Godot 4.3+](https://godotengine.org/)（标准版即可，无需 .NET）
2. 打开 Godot → **导入** → 选择本目录下的 `project.godot`
3. 按 F5 运行，主场景为 `scenes/main.tscn`

## 操作

| 按键 | 作用 |
|------|------|
| WASD | 移动单位 |
| E | 靠近矿脉时手动采矿（资源进库存） |
| Space / 左键 | 射击 |
| 1–5 | 选择建筑：钻头 / 传送带 / 墙 / 双联炮 / 散射炮 |
| R | 旋转传送带朝向 |
| 右键 / X | 拆除建筑（返还约 50% 造价） |
| 左键（空地） | 放置当前建筑 |

## 目标

- **胜利**：存活至第 15 波并清除敌人  
- **失败**：核心被摧毁  

开局资源：铜 100、铅 30。

## 建筑

| 建筑 | 造价 | 说明 |
|------|------|------|
| 钻头 | 铜 12 | 必须建在矿上，自动开采 |
| 传送带 | 铜 1 | 可旋转，运输物品至核心/炮塔 |
| 墙 | 铜 4 | 阻挡并承受伤害 |
| 双联炮 | 铜 25 | 消耗铜弹药 |
| 散射炮 | 铜 20 + 铅 15 | 消耗铅弹药，多弹丸 |

## 架构

```
scripts/
  config.gd       # 常量、颜色、建筑定义 (class_name Config)
  game_map.gd     # 网格、矿脉、占用、BFS/A* 寻路 (class_name GameMap)
  buildings.gd    # 建造/拆除/钻头逻辑 (class_name Buildings)
  logistics.gd    # 传送带步进 (class_name Logistics)
  combat.gd       # 波次、敌人、炮塔、子弹 (class_name Combat)
  player_ctrl.gd  # 玩家移动/采矿/射击 (class_name PlayerCtrl)
  world_view.gd   # _draw() 风格化几何绘制
  hud.gd          # 资源/波次/建造按钮 UI
  main.gd         # 根控制器，串联模拟循环
```

单一主场景 `scenes/main.tscn`：`Main` → `Camera2D` + `World` + `HUD`。  
地图约 40×30，格宽 32px；画面为深青工业风几何图形，无需外部贴图。

## 技术说明

- Godot 4 语法（`super()`、`@onready`、类型提示）
- 模拟在 `_process(delta)` 中推进
- 敌人使用 BFS / A* 朝核心寻路，会攻击墙与核心
- 本仓库为 Godot 栈版本，便于与其他引擎实现对照
