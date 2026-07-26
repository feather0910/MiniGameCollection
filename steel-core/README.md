# 钢核防线 (Steel Core)

Mindustry 风格的工厂建设 + 波次塔防，使用 **Rust + Bevy ECS** 实现。

## 为什么选这套技术栈

| 选项 | 结论 |
|------|------|
| H5 单文件 | 不适合复杂物流/战斗模拟，难维护 |
| Phaser / Pixi | 可用，但仍偏前端编排 |
| Godot | 很适合 2D，本环境未预装 |
| **Rust + Bevy** | ECS 天然对应钻头/传送带/炮塔/波次等独立系统，数据驱动、易扩展 |

核心思路：建筑与敌人都是 Entity，逻辑按 System 拆分；数值集中在 `config`，加建筑几乎只改定义表。

## 运行

依赖（Ubuntu/Debian）：

```bash
sudo apt-get install -y libasound2-dev libudev-dev libxkbcommon-x11-0 mesa-vulkan-drivers
```

```bash
cd steel-core
cargo run --release
```

无显示器 / CI（软件渲染）：

```bash
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json xvfb-run -a cargo run --release
```

```bash
cargo test
```

## 操作

| 按键 | 作用 |
|------|------|
| WASD / 方向键 | 移动 |
| 鼠标左键 | 建造（已选建筑时）/ 射击 |
| 鼠标右键 / X | 拆除 |
| 1–5 | 选择建筑 |
| Q / Esc | 取消建造 |
| R | 旋转传送带 |
| E | 靠近矿脉手挖 |
| Space | 射击 |

## 架构

```
src/
  main.rs           App 入口
  config.rs         地图尺寸、建筑/波次数值、配色
  components.rs     ECS 组件
  map.rs            网格、矿脉、占用
  systems/
    setup.rs        开局生成
    player.rs       玩家移动与手挖
    build.rs        建造/拆除
    extract.rs      钻头产出
    logistics.rs    传送带
    combat.rs       炮塔、子弹、敌人、波次
    camera.rs       镜头跟随
    ui.rs           HUD / 建造栏
```

## 胜利条件

守住核心，撑过第 15 波。
