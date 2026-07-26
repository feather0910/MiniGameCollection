/** 风格化 Canvas 渲染：几何工业风，高辨识度色块 */

import { TILE, COLORS, DIRS, RESOURCES, MAP_W, MAP_H } from './config.js';

export function createRenderer(canvas) {
  const ctx = canvas.getContext('2d');
  return { canvas, ctx, time: 0 };
}

export function resizeRenderer(renderer, cssW, cssH) {
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  renderer.canvas.width = Math.floor(cssW * dpr);
  renderer.canvas.height = Math.floor(cssH * dpr);
  renderer.canvas.style.width = cssW + 'px';
  renderer.canvas.style.height = cssH + 'px';
  renderer.dpr = dpr;
}

export function render(renderer, game) {
  const { ctx, canvas } = renderer;
  const { world, buildings, combat, player, input, cam } = game;
  renderer.time += game.dt;

  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = COLORS.bg;
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.save();
  ctx.translate(-cam.x, -cam.y);

  drawFloor(ctx, world, cam, canvas);
  drawOres(ctx, world);
  drawBuildings(ctx, buildings, renderer.time);
  drawItems(ctx, buildings);
  drawPlayer(ctx, player, renderer.time);
  drawEnemies(ctx, combat.enemies, renderer.time);
  drawBullets(ctx, combat.bullets);
  drawGhost(ctx, game);
  drawHover(ctx, input.hoverTile);

  ctx.restore();

  drawTouchControls(ctx, canvas, input);
}

function drawFloor(ctx, world, cam, canvas) {
  const x0 = Math.max(0, Math.floor(cam.x / TILE) - 1);
  const y0 = Math.max(0, Math.floor(cam.y / TILE) - 1);
  const x1 = Math.min(world.w, Math.ceil((cam.x + canvas.width) / TILE) + 1);
  const y1 = Math.min(world.h, Math.ceil((cam.y + canvas.height) / TILE) + 1);

  for (let y = y0; y < y1; y++) {
    for (let x = x0; x < x1; x++) {
      const px = x * TILE;
      const py = y * TILE;
      ctx.fillStyle = (x + y) % 2 === 0 ? COLORS.floor : COLORS.floorAlt;
      ctx.fillRect(px, py, TILE, TILE);
    }
  }

  // 细网格
  ctx.strokeStyle = COLORS.gridLine;
  ctx.lineWidth = 1;
  ctx.beginPath();
  for (let x = x0; x <= x1; x++) {
    ctx.moveTo(x * TILE, y0 * TILE);
    ctx.lineTo(x * TILE, y1 * TILE);
  }
  for (let y = y0; y <= y1; y++) {
    ctx.moveTo(x0 * TILE, y * TILE);
    ctx.lineTo(x1 * TILE, y * TILE);
  }
  ctx.stroke();
}

function drawOres(ctx, world) {
  for (let y = 0; y < world.h; y++) {
    for (let x = 0; x < world.w; x++) {
      const ore = world.tiles[y][x].ore;
      if (!ore) continue;
      const c = RESOURCES[ore].color;
      const px = x * TILE;
      const py = y * TILE;
      // 矿脉底色
      ctx.globalAlpha = 0.35;
      ctx.fillStyle = c;
      ctx.fillRect(px + 2, py + 2, TILE - 4, TILE - 4);
      ctx.globalAlpha = 1;
      // 几何碎块
      ctx.fillStyle = c;
      const dots = [[8, 10], [20, 8], [14, 20], [24, 18], [10, 22]];
      for (const [dx, dy] of dots) {
        ctx.beginPath();
        ctx.moveTo(px + dx, py + dy - 3);
        ctx.lineTo(px + dx + 3, py + dy);
        ctx.lineTo(px + dx, py + dy + 3);
        ctx.lineTo(px + dx - 3, py + dy);
        ctx.closePath();
        ctx.fill();
      }
    }
  }
}

function drawBuildings(ctx, buildings, time) {
  for (const b of buildings.list) {
    const px = b.x * TILE;
    const py = b.y * TILE;
    const s = b.def.size * TILE;

    if (b.type === 'core') drawCore(ctx, px, py, s, time, b);
    else if (b.type === 'drill') drawDrill(ctx, px, py, time, b);
    else if (b.type === 'conveyor') drawConveyor(ctx, px, py, b, time);
    else if (b.type === 'wall') drawWall(ctx, px, py);
    else if (b.type === 'duo' || b.type === 'scatter') drawTurret(ctx, px, py, b);

    if (b.hp < b.maxHp) drawHpBar(ctx, px, py, s, b.hp / b.maxHp);
  }
}

function drawCore(ctx, px, py, s, time, b) {
  const cx = px + s / 2;
  const cy = py + s / 2;
  // 外环脉冲
  const pulse = 0.5 + 0.5 * Math.sin(time * 2.5);
  ctx.strokeStyle = COLORS.core;
  ctx.globalAlpha = 0.25 + pulse * 0.25;
  ctx.lineWidth = 3;
  hexPath(ctx, cx, cy, s * 0.48);
  ctx.stroke();
  ctx.globalAlpha = 1;

  // 主体六边形
  ctx.fillStyle = COLORS.coreDark;
  hexPath(ctx, cx, cy, s * 0.4);
  ctx.fill();
  ctx.fillStyle = COLORS.core;
  hexPath(ctx, cx, cy, s * 0.28);
  ctx.fill();

  // 内核菱形
  ctx.fillStyle = '#e8fffb';
  ctx.beginPath();
  ctx.moveTo(cx, cy - 8);
  ctx.lineTo(cx + 8, cy);
  ctx.lineTo(cx, cy + 8);
  ctx.lineTo(cx - 8, cy);
  ctx.closePath();
  ctx.fill();

  // 装甲角
  ctx.fillStyle = '#2a9e92';
  for (let i = 0; i < 6; i++) {
    const a = (Math.PI / 3) * i + time * 0.3;
    const r = s * 0.42;
    ctx.fillRect(cx + Math.cos(a) * r - 3, cy + Math.sin(a) * r - 3, 6, 6);
  }
}

function hexPath(ctx, cx, cy, r) {
  ctx.beginPath();
  for (let i = 0; i < 6; i++) {
    const a = (Math.PI / 3) * i - Math.PI / 6;
    const x = cx + Math.cos(a) * r;
    const y = cy + Math.sin(a) * r;
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  }
  ctx.closePath();
}

function drawDrill(ctx, px, py, time, b) {
  ctx.fillStyle = COLORS.drill;
  roundRect(ctx, px + 3, py + 3, TILE - 6, TILE - 6, 4);
  ctx.fill();
  ctx.fillStyle = '#1e3a44';
  ctx.fillRect(px + 8, py + 8, TILE - 16, TILE - 16);

  // 旋转钻头
  const cx = px + TILE / 2;
  const cy = py + TILE / 2;
  ctx.save();
  ctx.translate(cx, cy);
  ctx.rotate(time * 4);
  ctx.strokeStyle = RESOURCES[b.oreType]?.color || COLORS.copper;
  ctx.lineWidth = 3;
  ctx.beginPath();
  ctx.moveTo(-8, 0);
  ctx.lineTo(8, 0);
  ctx.moveTo(0, -8);
  ctx.lineTo(0, 8);
  ctx.stroke();
  ctx.fillStyle = '#c8e6ef';
  ctx.beginPath();
  ctx.arc(0, 0, 3, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();
}

function drawConveyor(ctx, px, py, b, time) {
  ctx.fillStyle = COLORS.conveyor;
  ctx.fillRect(px + 2, py + 2, TILE - 4, TILE - 4);

  const d = DIRS[b.rot];
  const cx = px + TILE / 2;
  const cy = py + TILE / 2;
  const anim = ((time * b.def.speed) % 1);

  ctx.save();
  ctx.translate(cx, cy);
  ctx.rotate(Math.atan2(d.dy, d.dx));
  ctx.fillStyle = COLORS.conveyorArrow;
  // 流动箭头
  for (let i = -1; i <= 1; i++) {
    const ox = (i + anim) * 10 - 5;
    ctx.globalAlpha = 0.45 + (i === 0 ? 0.35 : 0);
    ctx.beginPath();
    ctx.moveTo(ox - 4, -5);
    ctx.lineTo(ox + 4, 0);
    ctx.lineTo(ox - 4, 5);
    ctx.closePath();
    ctx.fill();
  }
  ctx.restore();
  ctx.globalAlpha = 1;

  // 有弹药缓冲提示色边
  ctx.strokeStyle = '#3a6a72';
  ctx.lineWidth = 1;
  ctx.strokeRect(px + 2.5, py + 2.5, TILE - 5, TILE - 5);
}

function drawWall(ctx, px, py) {
  ctx.fillStyle = COLORS.wall;
  roundRect(ctx, px + 2, py + 2, TILE - 4, TILE - 4, 3);
  ctx.fill();
  ctx.fillStyle = '#6a7e8c';
  ctx.fillRect(px + 6, py + 6, 8, 8);
  ctx.fillRect(px + 18, py + 18, 8, 8);
  ctx.fillStyle = '#3a4650';
  ctx.fillRect(px + 18, py + 6, 8, 8);
  ctx.fillRect(px + 6, py + 18, 8, 8);
}

function drawTurret(ctx, px, py, b) {
  ctx.fillStyle = COLORS.turret;
  roundRect(ctx, px + 3, py + 3, TILE - 6, TILE - 6, 5);
  ctx.fill();

  // 底座环
  const cx = px + TILE / 2;
  const cy = py + TILE / 2;
  ctx.fillStyle = '#2a343c';
  ctx.beginPath();
  ctx.arc(cx, cy, 8, 0, Math.PI * 2);
  ctx.fill();

  // 炮管
  ctx.save();
  ctx.translate(cx, cy);
  ctx.rotate(b.aim || 0);
  ctx.fillStyle = b.type === 'scatter' ? COLORS.lead : COLORS.turretAccent;
  ctx.fillRect(2, -3, 14, 6);
  if (b.type === 'scatter') {
    ctx.fillRect(8, -7, 8, 3);
    ctx.fillRect(8, 4, 8, 3);
  }
  ctx.fillStyle = '#dfe7ee';
  ctx.beginPath();
  ctx.arc(0, 0, 4, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();

  // 缺弹闪烁
  if ((b.ammoBuffer || 0) < b.def.ammoPerShot) {
    ctx.strokeStyle = COLORS.enemy;
    ctx.globalAlpha = 0.5 + 0.5 * Math.sin(performance.now() / 120);
    ctx.lineWidth = 2;
    ctx.strokeRect(px + 4, py + 4, TILE - 8, TILE - 8);
    ctx.globalAlpha = 1;
  }
}

function drawItems(ctx, buildings) {
  for (const b of buildings.list) {
    if (b.type !== 'conveyor' || !b.item) continue;
    const d = DIRS[b.rot];
    const p = Math.min(1, b.item.progress);
    const cx = (b.x + 0.5 + (p - 0.5) * d.dx * 0.85) * TILE;
    const cy = (b.y + 0.5 + (p - 0.5) * d.dy * 0.85) * TILE;
    ctx.fillStyle = RESOURCES[b.item.type]?.color || '#fff';
    ctx.beginPath();
    ctx.arc(cx, cy, 5, 0, Math.PI * 2);
    ctx.fill();
    ctx.strokeStyle = 'rgba(0,0,0,0.35)';
    ctx.stroke();
  }
}

function drawPlayer(ctx, player, time) {
  const px = player.x * TILE;
  const py = player.y * TILE;
  // 影子
  ctx.fillStyle = 'rgba(0,0,0,0.35)';
  ctx.beginPath();
  ctx.ellipse(px, py + 6, 10, 4, 0, 0, Math.PI * 2);
  ctx.fill();

  // 机体 — 菱形机甲
  ctx.save();
  ctx.translate(px, py);
  ctx.rotate(player.aim);
  ctx.fillStyle = COLORS.player;
  ctx.beginPath();
  ctx.moveTo(12, 0);
  ctx.lineTo(0, 8);
  ctx.lineTo(-10, 0);
  ctx.lineTo(0, -8);
  ctx.closePath();
  ctx.fill();
  ctx.fillStyle = '#d8f7ff';
  ctx.fillRect(2, -2, 8, 4);
  ctx.restore();

  // 手持资源
  if (player.carrying) {
    ctx.fillStyle = RESOURCES[player.carrying.type].color;
    ctx.beginPath();
    ctx.arc(px + 10, py - 12, 5, 0, Math.PI * 2);
    ctx.fill();
  }

  drawHpBar(ctx, px - 16, py - 22, 32, player.hp / player.maxHp);
}

function drawEnemies(ctx, enemies, time) {
  for (const e of enemies) {
    const px = e.x * TILE;
    const py = e.y * TILE;
    ctx.save();
    ctx.translate(px, py);
    const ang = e.path && e.path[e.pathIdx + 1]
      ? Math.atan2(e.path[e.pathIdx + 1].y + 0.5 - e.y, e.path[e.pathIdx + 1].x + 0.5 - e.x)
      : time;
    ctx.rotate(ang);

    if (e.kind === 'tank') {
      ctx.fillStyle = COLORS.enemyDark;
      roundRect(ctx, -12, -10, 24, 20, 4);
      ctx.fill();
      ctx.fillStyle = COLORS.enemy;
      ctx.fillRect(4, -4, 12, 8);
    } else {
      // 爬行虫：三角 + 节肢
      ctx.fillStyle = COLORS.enemy;
      ctx.beginPath();
      ctx.moveTo(11, 0);
      ctx.lineTo(-9, 8);
      ctx.lineTo(-5, 0);
      ctx.lineTo(-9, -8);
      ctx.closePath();
      ctx.fill();
      ctx.strokeStyle = COLORS.enemyDark;
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(-2, -7);
      ctx.lineTo(-8, -12);
      ctx.moveTo(-2, 7);
      ctx.lineTo(-8, 12);
      ctx.stroke();
    }
    ctx.restore();
    drawHpBar(ctx, px - 14, py - 16, 28, e.hp / e.maxHp);
  }
}

function drawBullets(ctx, bullets) {
  for (const b of bullets) {
    ctx.fillStyle = b.from === 'player' ? COLORS.player : COLORS.bullet;
    ctx.beginPath();
    ctx.arc(b.x * TILE, b.y * TILE, 3, 0, Math.PI * 2);
    ctx.fill();
  }
}

function drawGhost(ctx, game) {
  const type = game.input.selectBuild;
  if (!type) return;
  const { x, y } = game.input.hoverTile;
  const def = game.defs[type];
  if (!def) return;
  const ok = game.canPlaceGhost(type, x, y);
  ctx.fillStyle = ok ? COLORS.ghostOk : COLORS.ghostBad;
  ctx.fillRect(x * TILE, y * TILE, def.size * TILE, def.size * TILE);

  if (type === 'conveyor') {
    const d = DIRS[game.buildRot];
    const cx = (x + 0.5) * TILE;
    const cy = (y + 0.5) * TILE;
    ctx.fillStyle = '#fff';
    ctx.beginPath();
    ctx.moveTo(cx + d.dx * 8 - d.dy * 5, cy + d.dy * 8 + d.dx * 5);
    ctx.lineTo(cx + d.dx * 14, cy + d.dy * 14);
    ctx.lineTo(cx + d.dx * 8 + d.dy * 5, cy + d.dy * 8 - d.dx * 5);
    ctx.closePath();
    ctx.fill();
  }

  if (type === 'duo' || type === 'scatter') {
    ctx.strokeStyle = ok ? 'rgba(240,193,74,0.35)' : 'rgba(232,93,93,0.35)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.arc((x + 0.5) * TILE, (y + 0.5) * TILE, def.range * TILE, 0, Math.PI * 2);
    ctx.stroke();
  }
}

function drawHover(ctx, tile) {
  ctx.strokeStyle = 'rgba(126, 224, 255, 0.35)';
  ctx.lineWidth = 1;
  ctx.strokeRect(tile.x * TILE + 1, tile.y * TILE + 1, TILE - 2, TILE - 2);
}

function drawHpBar(ctx, x, y, w, ratio) {
  const h = 3;
  ctx.fillStyle = 'rgba(0,0,0,0.55)';
  ctx.fillRect(x, y, w, h);
  ctx.fillStyle = ratio > 0.35 ? COLORS.hpGood : COLORS.hpBad;
  ctx.fillRect(x, y, w * Math.max(0, ratio), h);
}

function drawTouchControls(ctx, canvas, input) {
  // 仅在粗指针/窄屏时画提示圈 — 用 canvas 宽判断
  if (canvas.width / (input._dpr || 1) > 820) return;
  const w = canvas.width;
  const h = canvas.height;
  ctx.save();
  ctx.globalAlpha = 0.25;
  ctx.strokeStyle = '#fff';
  ctx.lineWidth = 3;
  // 摇杆
  ctx.beginPath();
  ctx.arc(w * 0.18, h * 0.78, w * 0.1, 0, Math.PI * 2);
  ctx.stroke();
  if (input.stick.active) {
    ctx.globalAlpha = 0.4;
    ctx.beginPath();
    ctx.arc(
      w * 0.18 + input.stick.x * w * 0.06,
      h * 0.78 + input.stick.y * w * 0.06,
      w * 0.035,
      0,
      Math.PI * 2
    );
    ctx.fillStyle = COLORS.player;
    ctx.fill();
  }
  // 动作键
  ctx.globalAlpha = 0.25;
  ctx.beginPath();
  ctx.arc(w * 0.82, h * 0.78, w * 0.08, 0, Math.PI * 2);
  ctx.stroke();
  ctx.restore();
}

function roundRect(ctx, x, y, w, h, r) {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.arcTo(x + w, y, x + w, y + h, r);
  ctx.arcTo(x + w, y + h, x, y + h, r);
  ctx.arcTo(x, y + h, x, y, r);
  ctx.arcTo(x, y, x + w, y, r);
  ctx.closePath();
}

export function updateCamera(cam, player, canvas) {
  const targetX = player.x * TILE - canvas.width / 2;
  const targetY = player.y * TILE - canvas.height / 2;
  const maxX = MAP_W * TILE - canvas.width;
  const maxY = MAP_H * TILE - canvas.height;
  cam.x += (clamp(targetX, 0, Math.max(0, maxX)) - cam.x) * 0.12;
  cam.y += (clamp(targetY, 0, Math.max(0, maxY)) - cam.y) * 0.12;
}

function clamp(v, a, b) {
  return Math.max(a, Math.min(b, v));
}
