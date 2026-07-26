# 钢核防线 — 三技术栈对照

同一套 Mindustry 风格玩法（工厂物流 + 波次塔防），用三种技术栈各实现一份，便于对比维护方式与扩展成本。

| 技术栈 | 目录 | 运行方式 |
|--------|------|----------|
| **H5**（Canvas 2D + ES Modules） | [`games/mindustry/`](games/mindustry/) | 打开大厅卡片，或 `python3 -m http.server` 后访问 `/games/mindustry/` |
| **Rust + Bevy ECS** | [`steel-core/`](steel-core/) | `cd steel-core && cargo run --release` |
| **Godot 4.3 GDScript** | [`godot-steel-core/`](godot-steel-core/) | Godot 打开工程后 F5，或 `godot --path godot-steel-core` |

## 共同玩法

- 钻头开采铜 / 铅 → 传送带运回核心或装填炮塔
- 铜墙、双管炮、散射炮防守
- 敌人从地图边缘寻路进攻核心
- 撑过第 15 波胜利；核心被毁失败
- 操作：WASD 移动 · E 手挖 · 空格射击 · 1–5 建造 · R 旋转 · 右键/X 拆除

## 为何三套都要

| 栈 | 适合场景 |
|----|----------|
| H5 | 零安装、GitHub Pages 即玩、前端同学易改 |
| Bevy | 复杂模拟系统用 ECS 拆分清晰、性能与类型安全 |
| Godot | 场景/节点可视化编辑、GDScript 迭代快、导出多端方便 |

## 风格

工业青绿底 + 铜橙 / 铅紫矿脉 + 几何色块建筑（风格化、高辨识、少贴图）。
