use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::config::{colors, tile_to_world, world_to_tile, BuildingKind, WaveStats, TILE, MAP_H, MAP_W};
use crate::map::{GameMap, GameOver, Inventory, WaveState};

pub fn tick_waves(
    time: Res<Time>,
    over: Res<GameOver>,
    mut wave: ResMut<WaveState>,
    mut commands: Commands,
    core_q: Query<&Building, With<CoreTag>>,
    enemies: Query<Entity, With<Enemy>>,
) {
    if over.done || core_q.is_empty() {
        return;
    }
    let dt = time.delta_secs();

    if wave.spawn_queue > 0 {
        wave.spawn_acc += dt;
        if wave.spawn_acc >= 0.45 {
            wave.spawn_acc = 0.0;
            wave.spawn_queue -= 1;
            spawn_enemy(&mut commands, wave.wave);
        }
    }

    if !wave.between {
        if wave.spawn_queue <= 0 && enemies.iter().count() == 0 {
            wave.between = true;
            wave.timer = WaveStats::INTERVAL;
        }
        return;
    }

    wave.timer -= dt;
    if wave.timer <= 0.0 {
        wave.wave += 1;
        wave.between = false;
        wave.spawn_queue = WaveStats::BASE_COUNT + (wave.wave as i32 - 1) * WaveStats::COUNT_GROWTH;
        wave.spawn_acc = 0.0;
    }
}

fn spawn_enemy(commands: &mut Commands, wave: u32) {
    let mut rng = rand::thread_rng();
    let edge = rng.gen_range(0..4);
    let (tx, ty) = match edge {
        0 => (rng.gen_range(0..MAP_W), MAP_H - 1),
        1 => (rng.gen_range(0..MAP_W), 0),
        2 => (0, rng.gen_range(0..MAP_H)),
        _ => (MAP_W - 1, rng.gen_range(0..MAP_H)),
    };
    let is_tank = wave >= 5 && rng.gen_bool(0.25);
    let mut hp = WaveStats::HP_BASE + (wave.saturating_sub(1) as f32) * WaveStats::HP_GROWTH;
    let mut speed = WaveStats::SPEED_BASE + (wave.saturating_sub(1) as f32) * WaveStats::SPEED_GROWTH;
    let mut damage = WaveStats::DAMAGE + (wave / 3) as f32;
    if is_tank {
        hp *= 2.2;
        speed *= 0.65;
        damage *= 1.4;
    }
    let pos = tile_to_world(tx, ty, 1);
    let color = if is_tank {
        colors::ENEMY_TANK
    } else {
        colors::ENEMY
    };
    let size = if is_tank {
        Vec2::new(22.0, 18.0)
    } else {
        Vec2::new(16.0, 14.0)
    };

    let e = commands
        .spawn((
            Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            Transform::from_translation(pos + Vec3::new(0.0, 0.0, 4.5)),
            Enemy {
                hp,
                max_hp: hp,
                speed,
                damage,
                attack_acc: 0.0,
                path: Vec::new(),
                path_idx: 0,
                retarget: 0.0,
                is_tank,
            },
        ))
        .id();
    crate::systems::setup::spawn_hp_bar(commands, e, size.x + 8.0);
}

pub fn tick_enemies(
    time: Res<Time>,
    over: Res<GameOver>,
    mut map: ResMut<GameMap>,
    mut inv: ResMut<Inventory>,
    mut commands: Commands,
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy)>,
    mut buildings: Query<&mut Building>,
    cores: Query<Entity, With<CoreTag>>,
) {
    if over.done {
        return;
    }
    let dt = time.delta_secs();
    let Ok(core_e) = cores.get_single() else {
        return;
    };
    let Ok(core_b) = buildings.get(core_e) else {
        return;
    };
    let goals: Vec<(i32, i32)> = {
        let s = core_b.kind.size();
        let mut g = Vec::new();
        for y in core_b.ty..core_b.ty + s {
            for x in core_b.tx..core_b.tx + s {
                g.push((x, y));
            }
        }
        g
    };
    let core_pos = tile_to_world(core_b.tx, core_b.ty, core_b.kind.size()).truncate();
    let core_size = core_b.kind.size();

    let mut killed = Vec::new();
    let mut core_hits = 0.0_f32;
    let mut building_damage: Vec<(Entity, f32)> = Vec::new();

    for (e, mut tf, mut enemy) in enemies.iter_mut() {
        if enemy.hp <= 0.0 {
            killed.push(e);
            continue;
        }

        let pos = tf.translation.truncate();
        if pos.distance(core_pos) < core_size as f32 * TILE * 0.55 {
            enemy.attack_acc += dt;
            if enemy.attack_acc >= WaveStats::ATTACK_INTERVAL {
                enemy.attack_acc = 0.0;
                core_hits += enemy.damage;
            }
            continue;
        }

        // 拆挡路建筑
        let (ex, ey) = world_to_tile(pos);
        let ang = (core_pos - pos).normalize_or_zero();
        let ahead = world_to_tile(pos + ang * (TILE * 0.6));
        let block = [ahead, (ex, ey)]
            .into_iter()
            .find_map(|(x, y)| {
                map.get(x, y).and_then(|t| {
                    t.building.filter(|_| t.solid).filter(|&b| b != core_e)
                })
            });
        if let Some(be) = block {
            enemy.attack_acc += dt;
            if enemy.attack_acc >= WaveStats::ATTACK_INTERVAL {
                enemy.attack_acc = 0.0;
                building_damage.push((be, enemy.damage));
                enemy.path.clear();
            }
            continue;
        }

        enemy.retarget -= dt;
        if enemy.path.is_empty() || enemy.retarget <= 0.0 {
            let path = map.find_path(ex, ey, &goals).unwrap_or_default();
            enemy.path = path;
            enemy.path_idx = 0;
            enemy.retarget = 1.2;
            if enemy.path.is_empty() {
                tf.translation.x += ang.x * enemy.speed * dt;
                tf.translation.y += ang.y * enemy.speed * dt;
                continue;
            }
        }

        if enemy.path_idx + 1 < enemy.path.len() {
            let (nx, ny) = enemy.path[enemy.path_idx + 1];
            let target = tile_to_world(nx, ny, 1).truncate();
            let delta = target - pos;
            let dist = delta.length();
            if dist < 4.0 {
                enemy.path_idx += 1;
            } else {
                let step = enemy.speed * dt;
                let dir = delta / dist;
                tf.translation.x += dir.x * step;
                tf.translation.y += dir.y * step;
            }
        }
    }

    for e in killed {
        inv.copper += 2;
        if rand::random::<f32>() < 0.35 {
            inv.lead += 1;
        }
        commands.entity(e).despawn_recursive();
    }

    if core_hits > 0.0 {
        if let Ok(mut b) = buildings.get_mut(core_e) {
            b.hp -= core_hits;
        }
    }
    for (be, dmg) in building_damage {
        if let Ok(mut b) = buildings.get_mut(be) {
            b.hp -= dmg;
            if b.hp <= 0.0 && b.kind != BuildingKind::Core {
                let (tx, ty, size) = (b.tx, b.ty, b.kind.size());
                map.clear(tx, ty, size);
                commands.entity(be).despawn_recursive();
            }
        }
    }
}

pub fn tick_turrets(
    time: Res<Time>,
    over: Res<GameOver>,
    mut commands: Commands,
    mut turrets: Query<(&Transform, &mut Turret, &Building)>,
    enemies: Query<&Transform, With<Enemy>>,
) {
    if over.done {
        return;
    }
    let dt = time.delta_secs();
    for (tf, mut turret, _) in turrets.iter_mut() {
        turret.cool = (turret.cool - dt).max(0.0);
        let origin = tf.translation.truncate();
        let mut best: Option<(Vec2, f32)> = None;
        for et in enemies.iter() {
            let p = et.translation.truncate();
            let d = p.distance(origin);
            if d <= turret.range {
                if best.map(|(_, bd)| d < bd).unwrap_or(true) {
                    best = Some((p, d));
                }
            }
        }
        let Some((target, _)) = best else {
            continue;
        };
        let aim = (target - origin).normalize_or_zero();
        turret.aim = aim.y.atan2(aim.x);
        if turret.cool > 0.0 || turret.ammo < turret.ammo_cost {
            continue;
        }
        turret.ammo -= turret.ammo_cost;
        turret.cool = 1.0 / turret.fire_rate;

        for i in 0..turret.pellets {
            let spread = if turret.pellets > 1 {
                (i as f32 - (turret.pellets as f32 - 1.0) * 0.5) * 0.18
            } else {
                0.0
            };
            let a = turret.aim + spread;
            let dir = Vec2::new(a.cos(), a.sin());
            let spawn = origin + dir * 12.0;
            commands.spawn((
                Sprite {
                    color: colors::BULLET,
                    custom_size: Some(Vec2::splat(5.0)),
                    ..default()
                },
                Transform::from_translation(spawn.extend(6.0)),
                Bullet {
                    damage: turret.damage,
                    life: 0.7,
                },
                LinearVelocity(dir * 480.0),
            ));
        }
    }
}

pub fn tick_bullets(
    time: Res<Time>,
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut Transform, &mut Bullet, &LinearVelocity)>,
    mut enemies: Query<(Entity, &Transform, &mut Enemy), Without<Bullet>>,
) {
    let dt = time.delta_secs();
    for (e, mut tf, mut bullet, vel) in bullets.iter_mut() {
        tf.translation.x += vel.0.x * dt;
        tf.translation.y += vel.0.y * dt;
        bullet.life -= dt;
        if bullet.life <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        let bp = tf.translation.truncate();
        for (ee, et, mut enemy) in enemies.iter_mut() {
            if et.translation.truncate().distance(bp) < 12.0 {
                enemy.hp -= bullet.damage;
                commands.entity(e).despawn();
                if enemy.hp <= 0.0 {
                    // 击杀掉落在 tick_enemies 处理；此处仅扣血
                    let _ = ee;
                }
                break;
            }
        }
    }
}

pub fn update_hp_bars(
    mut bars: Query<(&HpBar, &mut Transform, &mut Sprite)>,
    targets: Query<(Entity, &Transform, Option<&Building>, Option<&Enemy>, Option<&Player>), Without<HpBar>>,
) {
    for (e, tf, building, enemy, player) in targets.iter() {
        let ratio = if let Some(b) = building {
            b.hp / b.max_hp
        } else if let Some(en) = enemy {
            en.hp / en.max_hp
        } else if let Some(p) = player {
            p.hp / p.max_hp
        } else {
            continue;
        };

        for (bar, mut btf, mut sprite) in bars.iter_mut() {
            if bar.owner != e {
                continue;
            }
            btf.translation.x = tf.translation.x;
            btf.translation.y = tf.translation.y + 18.0;
            btf.translation.z = 8.0;
            sprite.custom_size = Some(Vec2::new(bar.width * ratio.clamp(0.0, 1.0), 3.0));
            sprite.color = if ratio > 0.35 {
                colors::HP_GOOD
            } else {
                colors::HP_BAD
            };
        }
    }
}

pub fn check_game_over(
    mut over: ResMut<GameOver>,
    wave: Res<WaveState>,
    cores: Query<&Building, With<CoreTag>>,
    enemies: Query<Entity, With<Enemy>>,
) {
    if over.done {
        return;
    }
    if let Ok(core) = cores.get_single() {
        if core.hp <= 0.0 {
            over.done = true;
            over.won = false;
        }
    } else {
        over.done = true;
        over.won = false;
    }
    if wave.wave >= WaveStats::WIN_WAVE && wave.between && enemies.iter().count() == 0 {
        over.done = true;
        over.won = true;
    }
}
