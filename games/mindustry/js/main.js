/**
 * 钢核防线 — 入口与主循环
 * 架构：数据驱动建筑定义 + 分系统 tick（物流/战斗/玩家）+ Canvas 渲染
 */

import {
  BUILDINGS, BUILD_ORDER, TILE, COLORS, RESOURCES,
} from './config.js';
import { createWorld, canPlace, getTile } from './world.js';
import {
  createBuildingState, placeCore, placeBuilding, removeBuilding,
  buildingAt, rotateBuilding, tickExtractors,
  syncInventoryFromCore, syncCoreFromInventory, affordable,
} from './buildings.js';
import { tickConveyors } from './logistics.js';
import {
  createCombatState, tickWaves, tickEnemies, tickTurrets, tickBullets,
} from './combat.js';
import { createPlayer, tickPlayer } from './player.js';
import { createInput, consumeFlags, conveyorRotFromDrag } from './input.js';
import {
  createRenderer, resizeRenderer, render, updateCamera,
} from './renderer.js';

const canvas = document.getElementById('game');
const overlay = document.getElementById('overlay');
const ovTitle = document.getElementById('ov-title');
const ovMsg = document.getElementById('ov-msg');
const ovBtn = document.getElementById('ov-btn');

const ui = {
  copper: document.getElementById('res-copper'),
  lead: document.getElementById('res-lead'),
  wave: document.getElementById('stat-wave'),
  timer: document.getElementById('stat-timer'),
  coreHp: document.getElementById('stat-core'),
  buildBar: document.getElementById('build-bar'),
  hint: document.getElementById('hint'),
};

function createGame() {
  const world = createWorld();
  const buildings = createBuildingState();
  const core = placeCore(buildings, world, 18, 13);
  const inventory = {
    copper: core.storage.copper,
    lead: core.storage.lead,
  };
  const combat = createCombatState();
  const player = createPlayer(core.x + 1.5, core.y + 3.5);
  const input = createInput(canvas);
  const renderer = createRenderer(canvas);
  const cam = { x: 0, y: 0 };

  return {
    world,
    buildings,
    inventory,
    combat,
    player,
    input,
    renderer,
    cam,
    defs: BUILDINGS,
    buildRot: 0,
    lastPlaceTile: null,
    over: false,
    won: false,
    dt: 0,
    elapsed: 0,
    canPlaceGhost(type, x, y) {
      const def = BUILDINGS[type];
      if (!def) return false;
      if (!affordable(this.inventory, def.cost)) return false;
      if (type === 'drill') {
        if (!canPlace(this.world, x, y, def.size)) return false;
        const t = getTile(this.world, x, y);
        return !!(t && t.ore);
      }
      return canPlace(this.world, x, y, def.size);
    },
  };
}

let game = createGame();

function setupBuildBar() {
  ui.buildBar.innerHTML = '';
  BUILD_ORDER.forEach((id, i) => {
    const def = BUILDINGS[id];
    const btn = document.createElement('button');
    btn.className = 'build-btn';
    btn.dataset.id = id;
    btn.innerHTML = `
      <span class="swatch" style="background:${swatchColor(id)}"></span>
      <span class="name">${i + 1}. ${def.name}</span>
      <span class="cost">${costText(def.cost)}</span>
    `;
    btn.addEventListener('click', () => {
      game.input.selectBuild = id;
      game.input.buildIndex = i;
      refreshBuildBar();
    });
    ui.buildBar.appendChild(btn);
  });

  const cancel = document.createElement('button');
  cancel.className = 'build-btn ghost';
  cancel.textContent = 'Q 取消';
  cancel.addEventListener('click', () => {
    game.input.selectBuild = null;
    refreshBuildBar();
  });
  ui.buildBar.appendChild(cancel);
}

function swatchColor(id) {
  if (id === 'drill') return COLORS.drill;
  if (id === 'conveyor') return COLORS.conveyorArrow;
  if (id === 'wall') return COLORS.wall;
  if (id === 'duo') return COLORS.turretAccent;
  if (id === 'scatter') return COLORS.lead;
  return COLORS.core;
}

function costText(cost) {
  return Object.entries(cost)
    .map(([k, v]) => `${RESOURCES[k].name}${v}`)
    .join(' ');
}

function refreshBuildBar() {
  ui.buildBar.querySelectorAll('.build-btn').forEach((btn) => {
    btn.classList.toggle('active', btn.dataset.id === game.input.selectBuild);
  });
}

function layout() {
  const wrap = document.getElementById('canvas-wrap');
  const w = Math.min(wrap.clientWidth, 960);
  const h = Math.min(Math.max(360, window.innerHeight - 220), 640);
  resizeRenderer(game.renderer, w, h);
}

function restart() {
  game = createGame();
  overlay.classList.add('hidden');
  game.over = false;
  layout();
  refreshBuildBar();
  updateHint();
}

ovBtn.addEventListener('click', restart);
window.addEventListener('resize', layout);

setupBuildBar();
layout();
updateHint();

function updateHint() {
  const sel = game.input.selectBuild;
  if (sel) {
    const def = BUILDINGS[sel];
    ui.hint.innerHTML = `建造 <b>${def.name}</b>：${def.description} · 花费 ${costText(def.cost)} · R 旋转 · 右键/X 拆除`;
  } else {
    ui.hint.innerHTML = 'WASD 移动 · 鼠标瞄准 · 空格射击 · E 手挖 · 1-5 选建筑 · Tab 切换 · 拖拽可铺传送带';
  }
}

function handleBuildInput() {
  const flags = consumeFlags(game.input);
  game.input.cam = game.cam;

  if (flags.rotateRequest) {
    game.buildRot = (game.buildRot + 1) % 4;
    const hover = buildingAt(game.world, game.input.hoverTile.x, game.input.hoverTile.y);
    if (hover) rotateBuilding(hover);
  }

  if (flags.removeRequest) {
    const b = buildingAt(game.world, flags.removeRequest.x, flags.removeRequest.y);
    if (b && b.type !== 'core') {
      removeBuilding(game.buildings, game.world, game.inventory, b);
      syncCoreFromInventory(game.buildings, game.inventory);
    }
  }

  if (flags.placeRequest && game.input.selectBuild) {
    const type = game.input.selectBuild;
    const { x, y } = flags.placeRequest;
    let rot = game.buildRot;

    if (type === 'conveyor') {
      const auto = conveyorRotFromDrag(game.lastPlaceTile, { x, y });
      if (auto !== null) {
        rot = auto;
        game.buildRot = auto;
      }
    }

    // 避免同格重复放置
    if (!game.lastPlaceTile || game.lastPlaceTile.x !== x || game.lastPlaceTile.y !== y || type !== 'conveyor') {
      const placed = placeBuilding(game.buildings, game.world, game.inventory, type, x, y, rot);
      if (placed) {
        syncCoreFromInventory(game.buildings, game.inventory);
        game.lastPlaceTile = { x, y };
      }
    } else if (type === 'conveyor') {
      const placed = placeBuilding(game.buildings, game.world, game.inventory, type, x, y, rot);
      if (placed) {
        syncCoreFromInventory(game.buildings, game.inventory);
        game.lastPlaceTile = { x, y };
      }
    }
  }

  if (!game.input.mouse.down) game.lastPlaceTile = null;
}

function updateHUD() {
  syncInventoryFromCore(game.buildings, game.inventory);
  ui.copper.textContent = game.inventory.copper | 0;
  ui.lead.textContent = game.inventory.lead | 0;
  ui.wave.textContent = String(game.combat.wave);
  if (game.combat.betweenWaves) {
    ui.timer.textContent = Math.ceil(Math.max(0, game.combat.waveTimer)) + 's';
  } else {
    ui.timer.textContent = `敌${game.combat.enemies.length + game.combat.spawnQueue}`;
  }
  const core = game.buildings.core;
  ui.coreHp.textContent = core ? Math.max(0, Math.ceil(core.hp)) : 0;
  refreshBuildBar();
  updateHint();
}

function endGame(win) {
  game.over = true;
  game.won = win;
  overlay.classList.remove('hidden');
  ovTitle.textContent = win ? '防线稳固' : '核心被毁';
  ovMsg.textContent = win
    ? `你撑过了 ${game.combat.wave} 波进攻，工厂仍在运转。`
    : `坚持到第 ${game.combat.wave} 波。重建防线，再试一次！`;
  ovBtn.textContent = '重新开始';
}

let last = performance.now();
function frame(now) {
  const raw = Math.min(0.05, (now - last) / 1000);
  last = now;
  game.dt = raw;
  game.elapsed += raw;

  if (!game.over) {
    handleBuildInput();
    tickPlayer(game.player, game.world, game.buildings.core, game.inventory, game.combat, game.input, raw);
    syncCoreFromInventory(game.buildings, game.inventory);

    tickExtractors(game.buildings, game.world, raw);
    tickConveyors(game.buildings, game.world, raw);
    tickWaves(game.combat, game.buildings.core, raw);
    tickTurrets(game.buildings, game.combat, raw);
    tickBullets(game.combat, raw);
    const { coreDead } = tickEnemies(
      game.combat, game.buildings, game.world, game.inventory, game.buildings.core, raw
    );
    syncInventoryFromCore(game.buildings, game.inventory);

    if (coreDead || (game.buildings.core && game.buildings.core.hp <= 0)) {
      endGame(false);
    }
    // 轻量胜利条件：撑过 15 波
    if (game.combat.wave >= 15 && game.combat.betweenWaves && game.combat.enemies.length === 0) {
      endGame(true);
    }
  }

  updateCamera(game.cam, game.player, game.renderer.canvas);
  game.input.cam = game.cam;
  render(game.renderer, game);
  updateHUD();
  requestAnimationFrame(frame);
}

requestAnimationFrame(frame);
