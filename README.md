# 🎮 H5 小游戏合集（MiniGameCollection）

纯 HTML5 + CSS + JavaScript 打造的小游戏合集，无任何构建工具与外部依赖，支持电脑和手机浏览器，可直接通过 GitHub Pages 在线游玩。

**在线地址（启用 Pages 后）：** <https://feather0910.github.io/MiniGameCollection/>

## 游戏列表

| 游戏 | 目录 | 玩法 |
|------|------|------|
| 🐍 贪吃蛇 | `games/snake/` | 方向键 / WASD / 滑动控制，吃食物变长 |
| 🀄 连连看 | `games/lianliankan/` | 两次转折内连通相同图案即可消除 |
| 🔢 2048 | `games/2048/` | 滑动合并相同数字，冲击 2048 |
| 🧱 俄罗斯方块 | `games/tetris/` | 经典落块消行，带幽灵块与下一块预览 |
| 💣 扫雷 | `games/minesweeper/` | 三档难度，首点安全，手机长按插旗 |
| 🎯 打砖块 | `games/breakout/` | 鼠标 / 触屏 / 键盘控制挡板，多关卡 |
| 🃏 记忆翻牌 | `games/memory/` | 3D 翻牌动画，记步数计时 |
| ⚫ 五子棋 | `games/gomoku/` | 与启发式 AI 对战，支持悔棋 |
| 🧩 数字华容道 | `games/puzzle15/` | 15 拼图，随机打乱保证有解 |
| 🔨 打地鼠 | `games/mole/` | 30 秒限时，金鼠加分炸弹扣分 |
| ✈️ 飞机大战 | `games/plane/` | 竖版射击，自动开火，大型敌机更耐打 |
| 🐦 像素鸟 | `games/flappy/` | 点按飞行，穿过管道间隙 |
| 🦖 恐龙跑酷 | `games/dino/` | 无尽跑酷，跳过仙人掌和飞鸟 |
| ⚪ 黑白棋 | `games/reversi/` | 与位置权重 AI 对战翻转棋 |
| 🧮 数独 | `games/sudoku/` | 随机生成、保证唯一解，三档难度 |
| 🎹 别踩白块 | `games/pianotiles/` | 音游式手速挑战，支持 DFJK 键 |
| 🫧 泡泡龙 | `games/bubble/` | 六边形网格消除射击，悬空泡泡掉落 |
| 🏓 乒乓对战 | `games/pong/` | 与 AI 对打，先得 7 分获胜 |
| 🗼 汉诺塔 | `games/hanoi/` | 3–8 层可选，挑战最少步数 |
| 🎲 猜数字 1A2B | `games/guess1a2b/` | 经典逻辑推理，猜中 4A 获胜 |
| 💎 宝石消消乐 | `games/match3/` | 三消玩法，连锁反应，死局自动重排 |
| 🍉 合成大西瓜 | `games/watermelon/` | 圆形物理碰撞，同类水果合成升级 |
| 🥷 切水果 | `games/fruitninja/` | 刀光轨迹判定，连击加分，炸弹终结 |
| 🦘 涂鸦跳跃 | `games/doodlejump/` | 无尽向上跳，移动/易碎平台 |
| 🕳️ 下100层 | `games/down100/` | 尖刺与传送带，血量制无尽下落 |
| 🚗 公路飞车 | `games/racing/` | 三车道躲避，速度渐增 |
| 🎯 跳一跳 | `games/jumpjump/` | 蓄力起跳，落点中心连击加分 |
| 🔴 四子棋 | `games/connect4/` | alpha-beta 剪枝 AI 对战 |
| 🃏 纸牌接龙 | `games/solitaire/` | 经典 Klondike，支持撤销 |
| ♟️ 中国象棋 | `games/chess/` | 完整规则 + 极大极小搜索 AI |
| 💡 点灯谜题 | `games/lightsout/` | Lights Out，三档难度保证可解 |
| 🎨 数织 | `games/nonogram/` | 8 幅像素图案，涂色/标记双模式 |
| 🐴 华容道 | `games/klotski/` | 三个布局（经 BFS 验证可解） |
| 🎵 记忆序列 | `games/simon/` | Simon Says，带 WebAudio 音效 |
| 🧱 推箱子 | `games/sokoban/` | 7 个关卡（经 BFS 验证可解），支持撤销 |
| ⚙️ 钢核防线 · H5 | `games/mindustry/` | Mindustry 风格工厂塔防（Canvas + ES Modules） |

## 钢核防线 — 三技术栈

同一玩法（工厂物流 + 波次塔防）用三套技术栈各实现一份，对照说明见 [`STEEL_CORE.md`](STEEL_CORE.md)。

| 技术栈 | 目录 | 运行 |
|--------|------|------|
| **H5** Canvas + ES Modules | [`games/mindustry/`](games/mindustry/) | 大厅进入，或静态服务器打开该目录 |
| **Rust + Bevy ECS** | [`steel-core/`](steel-core/) | `cd steel-core && cargo run --release` |
| **Godot 4.3 GDScript** | [`godot-steel-core/`](godot-steel-core/) | Godot 打开工程 F5，或 `godot --path godot-steel-core` |

## 本地运行

无需安装依赖，任选一种方式启动静态服务器：

```bash
# Python
python3 -m http.server 8000

# 或 Node.js
npx serve .
```

然后浏览器打开 <http://localhost:8000> 即可。

## 部署到 GitHub Pages

Pages 已启用（Source 为 GitHub Actions），自动部署工作流为 `.github/workflows/static.yml`。每次推送到 `main` 分支都会自动发布，访问 <https://feather0910.github.io/MiniGameCollection/>。

## 项目结构

```
├── index.html              # 游戏大厅首页
├── assets/common.css       # 共享样式（主题色、HUD、弹窗等）
├── games/                  # 每个游戏均为独立的单文件页面
│   ├── snake/  lianliankan/  2048/  tetris/  minesweeper/
│   ├── breakout/  memory/  gomoku/  puzzle15/  mole/
│   ├── plane/  flappy/  dino/  reversi/  sudoku/
│   ├── pianotiles/  bubble/  pong/  hanoi/  guess1a2b/
│   ├── match3/  watermelon/  fruitninja/  doodlejump/  down100/
│   ├── racing/  jumpjump/  connect4/  solitaire/  chess/
│   ├── lightsout/  nonogram/  klotski/  simon/  sokoban/
│   └── mindustry/              # 钢核防线 · H5
├── steel-core/             # 钢核防线 · Rust + Bevy ECS
├── godot-steel-core/       # 钢核防线 · Godot 4.3 GDScript
├── STEEL_CORE.md           # 三技术栈对照说明
└── .github/workflows/static.yml  # Pages 自动部署
```

## 特性

- 📱 移动端适配：触屏滑动、长按、虚拟按键
- 💾 本地存档：最高分 / 战绩保存在 `localStorage`
- 🎨 统一的深色主题 UI
- ⚡ 零依赖、零构建，打开即玩
