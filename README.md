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
├── games/
│   ├── snake/index.html    # 每个游戏均为独立的单文件页面
│   ├── lianliankan/index.html
│   ├── 2048/index.html
│   ├── tetris/index.html
│   ├── minesweeper/index.html
│   ├── breakout/index.html
│   ├── memory/index.html
│   ├── gomoku/index.html
│   ├── puzzle15/index.html
│   └── mole/index.html
└── .github/workflows/static.yml  # Pages 自动部署
```

## 特性

- 📱 移动端适配：触屏滑动、长按、虚拟按键
- 💾 本地存档：最高分 / 战绩保存在 `localStorage`
- 🎨 统一的深色主题 UI
- ⚡ 零依赖、零构建，打开即玩
