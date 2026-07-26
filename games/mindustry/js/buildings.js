/** 建筑放置、拆除与逻辑 tick */

import { BUILDINGS, DIRS } from './config.js';
import { canPlace, markOccupied, clearOccupied, getTile } from './world.js';

let nextId = 1;

export function createBuildingState() {
  return {
    list: [],
    core: null,
  };
}

export function affordable(inventory, cost) {
  for (const [k, v] of Object.entries(cost)) {
    if ((inventory[k] || 0) < v) return false;
  }
  return true;
}

export function pay(inventory, cost) {
  for (const [k, v] of Object.entries(cost)) {
    inventory[k] = (inventory[k] || 0) - v;
  }
}

export function refund(inventory, cost, ratio = 0.5) {
  for (const [k, v] of Object.entries(cost)) {
    inventory[k] = (inventory[k] || 0) + Math.floor(v * ratio);
  }
}

export function placeBuilding(state, world, inventory, typeId, tx, ty, rot = 0) {
  const def = BUILDINGS[typeId];
  if (!def || typeId === 'core') return null;

  const opts = {};
  if (typeId === 'drill') {
    // 钻头需要任意矿脉
    if (!canPlaceOnOre(world, tx, ty, def.size)) return null;
  } else if (!canPlace(world, tx, ty, def.size)) {
    return null;
  }

  if (!affordable(inventory, def.cost)) return null;

  pay(inventory, def.cost);
  const b = makeBuilding(def, tx, ty, rot);
  if (typeId === 'drill') {
    const tile = getTile(world, tx, ty);
    b.oreType = tile?.ore || 'copper';
  }
  markOccupied(world, tx, ty, def.size, b);
  state.list.push(b);
  return b;
}

function canPlaceOnOre(world, tx, ty, size) {
  if (!canPlace(world, tx, ty, size)) return false;
  for (let y = ty; y < ty + size; y++) {
    for (let x = tx; x < tx + size; x++) {
      if (world.tiles[y][x].ore) return true;
    }
  }
  return false;
}

export function placeCore(state, world, cx, cy) {
  const def = BUILDINGS.core;
  const tx = cx;
  const ty = cy;
  const b = makeBuilding(def, tx, ty, 0);
  b.storage = { copper: 100, lead: 30 };
  markOccupied(world, tx, ty, def.size, b);
  state.list.push(b);
  state.core = b;
  return b;
}

function makeBuilding(def, tx, ty, rot) {
  return {
    id: nextId++,
    type: def.id,
    def,
    x: tx,
    y: ty,
    rot: rot % 4,
    hp: def.hp,
    maxHp: def.hp,
    // extract
    oreType: null,
    mineAcc: 0,
    // logistics
    item: null, // { type, progress 0..1 } on conveyor
    // defense
    cool: 0,
    aim: 0,
    ammoBuffer: 0,
  };
}

export function removeBuilding(state, world, inventory, building) {
  if (!building || building.type === 'core') return false;
  clearOccupied(world, building.x, building.y, building.def.size);
  const idx = state.list.indexOf(building);
  if (idx >= 0) state.list.splice(idx, 1);
  refund(inventory, building.def.cost, 0.5);
  return true;
}

export function buildingAt(world, tx, ty) {
  const t = getTile(world, tx, ty);
  return t?.building || null;
}

export function rotateBuilding(building) {
  if (!building || !building.def.rotatable) return;
  building.rot = (building.rot + 1) % 4;
}

/** 提取类建筑产出到自身缓冲，再尝试吐到相邻传送带/核心 */
export function tickExtractors(state, world, dt) {
  for (const b of state.list) {
    if (b.type !== 'drill') continue;
    if (!b.oreType) continue;
    b.mineAcc += b.def.mineRate * dt;
    while (b.mineAcc >= 1) {
      b.mineAcc -= 1;
      tryOutput(world, b, b.oreType);
    }
  }
}

function tryOutput(world, from, itemType) {
  // 优先四个方向的传送带入口或核心
  for (let i = 0; i < 4; i++) {
    const d = DIRS[i];
    const nx = from.x + d.dx;
    const ny = from.y + d.dy;
    const nb = buildingAt(world, nx, ny);
    if (!nb) continue;
    if (nb.type === 'conveyor' && !nb.item) {
      nb.item = { type: itemType, progress: 0 };
      return true;
    }
    if (nb.type === 'core') {
      nb.storage[itemType] = (nb.storage[itemType] || 0) + 1;
      return true;
    }
    if ((nb.type === 'duo' || nb.type === 'scatter') && nb.def.ammoType === itemType) {
      nb.ammoBuffer = (nb.ammoBuffer || 0) + 1;
      return true;
    }
  }
  return false;
}

export function syncInventoryFromCore(state, inventory) {
  const core = state.core;
  if (!core) return;
  inventory.copper = core.storage.copper || 0;
  inventory.lead = core.storage.lead || 0;
}

export function syncCoreFromInventory(state, inventory) {
  const core = state.core;
  if (!core) return;
  core.storage.copper = inventory.copper || 0;
  core.storage.lead = inventory.lead || 0;
}

export function isSolidBuilding(b) {
  return !!(b && b.def.solid);
}

export function damageBuilding(state, world, inventory, building, dmg) {
  if (!building) return false;
  building.hp -= dmg;
  if (building.hp <= 0) {
    if (building.type === 'core') {
      building.hp = 0;
      return true; // core destroyed
    }
    removeBuilding(state, world, inventory, building);
    return false;
  }
  return false;
}
