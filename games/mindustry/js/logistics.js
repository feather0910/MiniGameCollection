/** 传送带物流：物品在格间流动并汇入核心/弹药库 */

import { DIRS } from './config.js';
import { buildingAt } from './buildings.js';
import { inBounds } from './world.js';

/**
 * 简化 Mindustry 传送带模型：
 * - 每格最多 1 个物品
 * - progress 0→1 后尝试进入前方格
 */
export function tickConveyors(state, world, dt) {
  // 两阶段：先推进进度，再尝试移交，避免同帧双吃
  const belts = state.list.filter((b) => b.type === 'conveyor' && b.item);

  for (const b of belts) {
    const speed = b.def.speed;
    b.item.progress += speed * dt;
  }

  // 按朝向分批移交，减少堵塞抖动
  belts.sort((a, b) => b.item.progress - a.item.progress);

  for (const b of belts) {
    if (!b.item || b.item.progress < 1) continue;
    const d = DIRS[b.rot];
    const nx = b.x + d.dx;
    const ny = b.y + d.dy;
    if (!inBounds(world, nx, ny)) {
      b.item.progress = 0.99;
      continue;
    }
    const target = buildingAt(world, nx, ny);
    if (!target) {
      b.item.progress = 0.99;
      continue;
    }

    if (target.type === 'conveyor') {
      if (!target.item) {
        target.item = { type: b.item.type, progress: 0 };
        b.item = null;
      } else {
        b.item.progress = 0.99;
      }
      continue;
    }

    if (target.type === 'core') {
      target.storage[b.item.type] = (target.storage[b.item.type] || 0) + 1;
      b.item = null;
      continue;
    }

    if ((target.type === 'duo' || target.type === 'scatter') && target.def.ammoType === b.item.type) {
      target.ammoBuffer = (target.ammoBuffer || 0) + 1;
      b.item = null;
      continue;
    }

    b.item.progress = 0.99;
  }
}
