/** 网格世界：地形、矿脉、建筑占用 */

import { MAP_W, MAP_H } from './config.js';

export function createWorld() {
  const tiles = Array.from({ length: MAP_H }, () =>
    Array.from({ length: MAP_W }, () => ({
      ore: null, // 'copper' | 'lead' | null
      oreAmount: 0,
      building: null, // building instance ref
      occupied: false,
    }))
  );

  seedOres(tiles);
  return { w: MAP_W, h: MAP_H, tiles };
}

function seedOres(tiles) {
  // 确定性伪随机：围绕核心区分布铜/铅矿脉
  const patches = [
    { cx: 8, cy: 10, r: 3.2, ore: 'copper', dens: 0.85 },
    { cx: 12, cy: 18, r: 2.8, ore: 'copper', dens: 0.75 },
    { cx: 28, cy: 8, r: 3.0, ore: 'copper', dens: 0.8 },
    { cx: 30, cy: 20, r: 2.5, ore: 'lead', dens: 0.7 },
    { cx: 22, cy: 14, r: 2.2, ore: 'lead', dens: 0.65 },
    { cx: 6, cy: 22, r: 2.0, ore: 'lead', dens: 0.6 },
    { cx: 18, cy: 6, r: 2.4, ore: 'copper', dens: 0.7 },
  ];

  for (let y = 0; y < MAP_H; y++) {
    for (let x = 0; x < MAP_W; x++) {
      // 核心保护区不刷矿
      if (x >= 17 && x <= 22 && y >= 12 && y <= 17) continue;
      for (const p of patches) {
        const d = Math.hypot(x - p.cx, y - p.cy);
        if (d < p.r) {
          const edge = 1 - d / p.r;
          const n = hash2(x, y);
          if (n < p.dens * (0.45 + edge * 0.55)) {
            tiles[y][x].ore = p.ore;
            tiles[y][x].oreAmount = 9999;
            break;
          }
        }
      }
    }
  }
}

function hash2(x, y) {
  let h = x * 374761393 + y * 668265263;
  h = (h ^ (h >>> 13)) * 1274126177;
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

export function inBounds(world, x, y) {
  return x >= 0 && y >= 0 && x < world.w && y < world.h;
}

export function getTile(world, x, y) {
  if (!inBounds(world, x, y)) return null;
  return world.tiles[y][x];
}

/** 检查能否在 (tx,ty) 放置 size×size 建筑 */
export function canPlace(world, tx, ty, size, opts = {}) {
  const { requireOre = null, allowReplace = false } = opts;
  if (tx < 0 || ty < 0 || tx + size > world.w || ty + size > world.h) return false;

  let hasOre = false;
  for (let y = ty; y < ty + size; y++) {
    for (let x = tx; x < tx + size; x++) {
      const t = world.tiles[y][x];
      if (t.occupied && !allowReplace) return false;
      if (requireOre && t.ore === requireOre) hasOre = true;
      if (requireOre && t.ore && t.ore !== requireOre) return false;
    }
  }
  if (requireOre && !hasOre) return false;
  return true;
}

export function markOccupied(world, tx, ty, size, building) {
  for (let y = ty; y < ty + size; y++) {
    for (let x = tx; x < tx + size; x++) {
      const t = world.tiles[y][x];
      t.occupied = true;
      t.building = building;
    }
  }
}

export function clearOccupied(world, tx, ty, size) {
  for (let y = ty; y < ty + size; y++) {
    for (let x = tx; x < tx + size; x++) {
      if (!inBounds(world, x, y)) continue;
      const t = world.tiles[y][x];
      t.occupied = false;
      t.building = null;
    }
  }
}

/**
 * 简易 BFS：敌人寻路。
 * isBlocked(x,y) 为障碍；isGoal(x,y) 可选，用于多格目标（如核心）。
 */
export function findPath(world, sx, sy, gx, gy, isBlocked, isGoal = null) {
  const key = (x, y) => y * world.w + x;
  const start = { x: Math.floor(sx), y: Math.floor(sy) };
  const goal = { x: Math.floor(gx), y: Math.floor(gy) };
  if (!inBounds(world, start.x, start.y) || !inBounds(world, goal.x, goal.y)) return null;

  const goalFn = isGoal || ((x, y) => x === goal.x && y === goal.y);

  // 环形队列，避免 shift 的 O(n²)
  const q = [start];
  let qi = 0;
  const came = new Map();
  came.set(key(start.x, start.y), null);
  const dirs = [[1, 0], [-1, 0], [0, 1], [0, -1]];

  while (qi < q.length) {
    const cur = q[qi++];
    if (goalFn(cur.x, cur.y)) {
      const path = [];
      let c = cur;
      while (c) {
        path.push(c);
        c = came.get(key(c.x, c.y));
      }
      path.reverse();
      return path;
    }
    for (const [dx, dy] of dirs) {
      const nx = cur.x + dx;
      const ny = cur.y + dy;
      const k = key(nx, ny);
      if (!inBounds(world, nx, ny) || came.has(k)) continue;
      if (isBlocked(nx, ny) && !goalFn(nx, ny)) continue;
      came.set(k, cur);
      q.push({ x: nx, y: ny });
    }
  }
  return null;
}
