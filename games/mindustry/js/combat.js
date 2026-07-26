/** 波次、敌人、炮塔射击与子弹 */

import { WAVE, MAP_W, MAP_H } from './config.js';
import { findPath, getTile } from './world.js';
import { isSolidBuilding, damageBuilding, buildingAt } from './buildings.js';

let enemyId = 1;
let bulletId = 1;

export function createCombatState() {
  return {
    enemies: [],
    bullets: [],
    wave: 0,
    waveTimer: WAVE.firstDelay,
    betweenWaves: true,
    spawnQueue: 0,
    spawnAcc: 0,
    pathCacheTimer: 0,
  };
}

function isBlocked(world, x, y) {
  const t = getTile(world, x, y);
  if (!t) return true;
  return isSolidBuilding(t.building);
}

function spawnPoint() {
  // 从地图边缘随机刷怪
  const edge = Math.floor(Math.random() * 4);
  if (edge === 0) return { x: Math.random() * MAP_W, y: 0.5 };
  if (edge === 1) return { x: Math.random() * MAP_W, y: MAP_H - 0.5 };
  if (edge === 2) return { x: 0.5, y: Math.random() * MAP_H };
  return { x: MAP_W - 0.5, y: Math.random() * MAP_H };
}

export function tickWaves(combat, core, dt) {
  if (!core || core.hp <= 0) return;

  if (combat.spawnQueue > 0) {
    combat.spawnAcc += dt;
    if (combat.spawnAcc >= 0.45) {
      combat.spawnAcc = 0;
      combat.spawnQueue--;
      spawnEnemy(combat, core);
    }
  }

  if (!combat.betweenWaves) {
    if (combat.spawnQueue <= 0 && combat.enemies.length === 0) {
      combat.betweenWaves = true;
      combat.waveTimer = WAVE.interval;
    }
    return;
  }

  combat.waveTimer -= dt;
  if (combat.waveTimer <= 0) {
    combat.wave++;
    combat.betweenWaves = false;
    combat.spawnQueue = WAVE.baseCount + (combat.wave - 1) * WAVE.countGrowth;
    combat.spawnAcc = 0;
  }
}

function spawnEnemy(combat, core) {
  const p = spawnPoint();
  const w = combat.wave;
  const hp = WAVE.hpBase + (w - 1) * WAVE.hpGrowth;
  const speed = WAVE.speedBase + (w - 1) * WAVE.speedGrowth;
  combat.enemies.push({
    id: enemyId++,
    x: p.x,
    y: p.y,
    hp,
    maxHp: hp,
    speed: Math.min(speed, 2.4),
    damage: WAVE.damage + Math.floor(w / 3),
    attackAcc: 0,
    path: null,
    pathIdx: 0,
    retarget: 0,
    kind: w >= 5 && Math.random() < 0.25 ? 'tank' : 'crawler',
  });

  // tank 变体更肉
  const e = combat.enemies[combat.enemies.length - 1];
  if (e.kind === 'tank') {
    e.hp *= 2.2;
    e.maxHp = e.hp;
    e.speed *= 0.65;
    e.damage *= 1.4;
  }
}

export function tickEnemies(combat, state, world, inventory, core, dt) {
  if (!core) return { coreDead: false };

  combat.pathCacheTimer -= dt;
  let coreDead = false;

  for (let i = combat.enemies.length - 1; i >= 0; i--) {
    const e = combat.enemies[i];
    if (e.hp <= 0) {
      combat.enemies.splice(i, 1);
      // 击杀奖励
      core.storage.copper = (core.storage.copper || 0) + 2;
      if (Math.random() < 0.35) core.storage.lead = (core.storage.lead || 0) + 1;
      continue;
    }

    e.retarget -= dt;
    const cx = core.x + core.def.size / 2;
    const cy = core.y + core.def.size / 2;

    // 近战攻击核心或挡路建筑
    const distCore = Math.hypot(e.x - cx, e.y - cy);
    if (distCore < core.def.size * 0.65 + 0.35) {
      e.attackAcc += dt;
      if (e.attackAcc >= WAVE.attackInterval) {
        e.attackAcc = 0;
        coreDead = damageBuilding(state, world, inventory, core, e.damage) || coreDead;
      }
      continue;
    }

    // 攻击挡路固体建筑
    const fx = Math.floor(e.x);
    const fy = Math.floor(e.y);
    const ahead = sampleBlockAhead(world, e, cx, cy);
    if (ahead) {
      e.attackAcc += dt;
      if (e.attackAcc >= WAVE.attackInterval) {
        e.attackAcc = 0;
        damageBuilding(state, world, inventory, ahead, e.damage);
        e.path = null;
      }
      continue;
    }

    if (!e.path || e.retarget <= 0) {
      e.path = findPath(world, e.x, e.y, cx, cy, (x, y) => isBlocked(world, x, y));
      e.pathIdx = 0;
      e.retarget = 1.2 + Math.random() * 0.6;
      // 无路则直线冲向核心（拆墙）
      if (!e.path) {
        const ang = Math.atan2(cy - e.y, cx - e.x);
        e.x += Math.cos(ang) * e.speed * dt;
        e.y += Math.sin(ang) * e.speed * dt;
        continue;
      }
    }

    // 沿路径移动
    while (e.path && e.pathIdx < e.path.length - 1) {
      const node = e.path[e.pathIdx + 1];
      const tx = node.x + 0.5;
      const ty = node.y + 0.5;
      const dx = tx - e.x;
      const dy = ty - e.y;
      const dist = Math.hypot(dx, dy);
      if (dist < 0.08) {
        e.pathIdx++;
        continue;
      }
      const step = e.speed * dt;
      if (step >= dist) {
        e.x = tx;
        e.y = ty;
        e.pathIdx++;
      } else {
        e.x += (dx / dist) * step;
        e.y += (dy / dist) * step;
      }
      break;
    }

    // 轻微推挤，避免重叠
    for (let j = 0; j < i; j++) {
      const o = combat.enemies[j];
      const ddx = e.x - o.x;
      const ddy = e.y - o.y;
      const d = Math.hypot(ddx, ddy);
      if (d > 0 && d < 0.55) {
        const push = (0.55 - d) * 0.5;
        e.x += (ddx / d) * push;
        e.y += (ddy / d) * push;
      }
    }
  }

  return { coreDead };
}

function sampleBlockAhead(world, e, cx, cy) {
  const ang = Math.atan2(cy - e.y, cx - e.x);
  const px = Math.floor(e.x + Math.cos(ang) * 0.55);
  const py = Math.floor(e.y + Math.sin(ang) * 0.55);
  const b = buildingAt(world, px, py);
  if (b && isSolidBuilding(b) && b.type !== 'core') return b;
  // 脚下固体也拆
  const under = buildingAt(world, Math.floor(e.x), Math.floor(e.y));
  if (under && isSolidBuilding(under) && under.type !== 'core') return under;
  return null;
}

export function tickTurrets(state, combat, dt) {
  for (const b of state.list) {
    if (b.type !== 'duo' && b.type !== 'scatter') continue;
    b.cool = Math.max(0, b.cool - dt);

    const bx = b.x + 0.5;
    const by = b.y + 0.5;
    const range = b.def.range;
    let target = null;
    let best = range;

    for (const e of combat.enemies) {
      const d = Math.hypot(e.x - bx, e.y - by);
      if (d <= best) {
        best = d;
        target = e;
      }
    }
    if (!target) continue;

    b.aim = Math.atan2(target.y - by, target.x - bx);
    if (b.cool > 0) continue;
    if ((b.ammoBuffer || 0) < b.def.ammoPerShot) continue;

    b.ammoBuffer -= b.def.ammoPerShot;
    b.cool = 1 / b.def.fireRate;

    const pellets = b.def.pellets || 1;
    for (let i = 0; i < pellets; i++) {
      const spread = pellets > 1 ? (i - (pellets - 1) / 2) * 0.18 : 0;
      const ang = b.aim + spread;
      combat.bullets.push({
        id: bulletId++,
        x: bx + Math.cos(ang) * 0.35,
        y: by + Math.sin(ang) * 0.35,
        vx: Math.cos(ang) * 14,
        vy: Math.sin(ang) * 14,
        dmg: b.def.damage,
        life: 0.7,
        from: 'turret',
      });
    }
  }
}

export function tickBullets(combat, dt) {
  for (let i = combat.bullets.length - 1; i >= 0; i--) {
    const b = combat.bullets[i];
    b.x += b.vx * dt;
    b.y += b.vy * dt;
    b.life -= dt;
    if (b.life <= 0) {
      combat.bullets.splice(i, 1);
      continue;
    }

    let hit = false;
    for (const e of combat.enemies) {
      if (Math.hypot(e.x - b.x, e.y - b.y) < 0.38) {
        e.hp -= b.dmg;
        hit = true;
        break;
      }
    }
    if (hit) combat.bullets.splice(i, 1);
  }
}

export function playerShoot(combat, player, aimAng) {
  combat.bullets.push({
    id: bulletId++,
    x: player.x + Math.cos(aimAng) * 0.4,
    y: player.y + Math.sin(aimAng) * 0.4,
    vx: Math.cos(aimAng) * 12,
    vy: Math.sin(aimAng) * 12,
    dmg: player.damage,
    life: 0.55,
    from: 'player',
  });
}
