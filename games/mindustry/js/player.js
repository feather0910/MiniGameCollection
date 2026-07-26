/** 玩家单位：移动、近距离手挖、随身射击 */

import { PLAYER, MAP_W, MAP_H } from './config.js';
import { getTile } from './world.js';
import { isSolidBuilding } from './buildings.js';
import { playerShoot } from './combat.js';

export function createPlayer(x, y) {
  return {
    x,
    y,
    hp: PLAYER.hp,
    maxHp: PLAYER.hp,
    speed: PLAYER.speed,
    radius: PLAYER.radius,
    mineAcc: 0,
    shootCool: 0,
    damage: PLAYER.damage,
    aim: 0,
    carrying: null, // { type } 手持一件资源时可走近核心上交
  };
}

export function tickPlayer(player, world, core, inventory, combat, input, dt) {
  // 移动
  let mx = 0;
  let my = 0;
  if (input.keys.has('KeyW') || input.keys.has('ArrowUp')) my -= 1;
  if (input.keys.has('KeyS') || input.keys.has('ArrowDown')) my += 1;
  if (input.keys.has('KeyA') || input.keys.has('ArrowLeft')) mx -= 1;
  if (input.keys.has('KeyD') || input.keys.has('ArrowRight')) mx += 1;

  // 虚拟摇杆
  if (input.stick.active) {
    mx += input.stick.x;
    my += input.stick.y;
  }

  if (mx || my) {
    const len = Math.hypot(mx, my) || 1;
    const nx = player.x + (mx / len) * player.speed * dt;
    const ny = player.y + (my / len) * player.speed * dt;
    player.x = collideMove(world, player, nx, player.y).x;
    player.y = collideMove(world, player, player.x, ny).y;
  }

  player.x = clamp(player.x, 0.3, MAP_W - 0.3);
  player.y = clamp(player.y, 0.3, MAP_H - 0.3);

  // 瞄准
  if (input.worldAim) {
    player.aim = Math.atan2(input.worldAim.y - player.y, input.worldAim.x - player.x);
  }

  // 手挖矿（按住 E / 挖矿按钮）
  if (input.mining) {
    tryMine(player, world, core, inventory, dt);
  }

  // 靠近核心自动上交手持资源
  if (player.carrying && core) {
    const cx = core.x + core.def.size / 2;
    const cy = core.y + core.def.size / 2;
    if (Math.hypot(player.x - cx, player.y - cy) < 2.2) {
      core.storage[player.carrying.type] = (core.storage[player.carrying.type] || 0) + 1;
      inventory[player.carrying.type] = (inventory[player.carrying.type] || 0) + 1;
      player.carrying = null;
    }
  }

  // 射击（空格或触屏射击）
  player.shootCool = Math.max(0, player.shootCool - dt);
  if (input.shooting && player.shootCool <= 0) {
    player.shootCool = 1 / PLAYER.fireRate;
    playerShoot(combat, player, player.aim);
  }
}

function tryMine(player, world, core, inventory, dt) {
  if (player.carrying) return;
  let best = null;
  let bestD = PLAYER.mineRange;
  const x0 = Math.floor(player.x - 2);
  const y0 = Math.floor(player.y - 2);
  const x1 = Math.floor(player.x + 2);
  const y1 = Math.floor(player.y + 2);
  for (let y = y0; y <= y1; y++) {
    for (let x = x0; x <= x1; x++) {
      const t = getTile(world, x, y);
      if (!t || !t.ore) continue;
      const d = Math.hypot(x + 0.5 - player.x, y + 0.5 - player.y);
      if (d < bestD) {
        bestD = d;
        best = t;
      }
    }
  }
  if (!best) return;
  player.mineAcc += PLAYER.mineRate * dt;
  if (player.mineAcc >= 1) {
    player.mineAcc = 0;
    // 直接进核心库存（早期上手友好）；远离核心则手持
    if (core) {
      const cx = core.x + core.def.size / 2;
      const cy = core.y + core.def.size / 2;
      if (Math.hypot(player.x - cx, player.y - cy) < 5) {
        core.storage[best.ore] = (core.storage[best.ore] || 0) + 1;
        inventory[best.ore] = (inventory[best.ore] || 0) + 1;
      } else {
        player.carrying = { type: best.ore };
      }
    }
  }
}

function collideMove(world, player, nx, ny) {
  const r = player.radius;
  // 采样四周固体
  const samples = [
    [nx - r, ny - r],
    [nx + r, ny - r],
    [nx - r, ny + r],
    [nx + r, ny + r],
    [nx, ny],
  ];
  for (const [sx, sy] of samples) {
    const t = getTile(world, Math.floor(sx), Math.floor(sy));
    if (t && isSolidBuilding(t.building)) {
      // 核心允许穿过边缘靠近
      if (t.building.type === 'core') continue;
      return { x: player.x, y: player.y };
    }
  }
  return { x: nx, y: ny };
}

function clamp(v, a, b) {
  return Math.max(a, Math.min(b, v));
}
