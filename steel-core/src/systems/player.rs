use bevy::prelude::*;

use crate::components::*;
use crate::config::{world_to_tile, PlayerStats, TILE};
use crate::map::{BuildMode, GameMap, GameOver, HoverTile, Inventory};

pub fn player_move(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    map: Res<GameMap>,
    over: Res<GameOver>,
    mut q: Query<&mut Transform, With<Player>>,
) {
    if over.done {
        return;
    }
    let Ok(mut tf) = q.get_single_mut() else {
        return;
    };

    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    if dir == Vec2::ZERO {
        return;
    }
    let step = dir.normalize() * PlayerStats::SPEED * time.delta_secs();
    let next = tf.translation.truncate() + step;

    // 固体碰撞（核心可穿过边缘）
    let (tx, ty) = world_to_tile(next);
    let blocked = map.get(tx, ty).map(|t| t.solid).unwrap_or(true);
    if !blocked {
        tf.translation.x = next.x;
        tf.translation.y = next.y;
    } else {
        // 轴分离滑动
        let nx = Vec2::new(tf.translation.x + step.x, tf.translation.y);
        let (ax, ay) = world_to_tile(nx);
        if !map.get(ax, ay).map(|t| t.solid).unwrap_or(true) {
            tf.translation.x = nx.x;
        }
        let ny = Vec2::new(tf.translation.x, tf.translation.y + step.y);
        let (bx, by) = world_to_tile(ny);
        if !map.get(bx, by).map(|t| t.solid).unwrap_or(true) {
            tf.translation.y = ny.y;
        }
    }
}

pub fn player_mine(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    map: Res<GameMap>,
    mut inv: ResMut<Inventory>,
    over: Res<GameOver>,
    mut q: Query<(&Transform, &mut Player)>,
    core_q: Query<&Transform, With<CoreTag>>,
) {
    if over.done || !keys.pressed(KeyCode::KeyE) {
        return;
    }
    let Ok((tf, mut player)) = q.get_single_mut() else {
        return;
    };
    if player.carrying.is_some() {
        return;
    }

    let pos = tf.translation.truncate();
    let (px, py) = world_to_tile(pos);
    let mut best = None;
    let mut best_d = PlayerStats::MINE_RANGE;
    for dy in -2..=2 {
        for dx in -2..=2 {
            let x = px + dx;
            let y = py + dy;
            if let Some(ore) = map.get(x, y).and_then(|t| t.ore) {
                let wp = crate::config::tile_to_world(x, y, 1).truncate();
                let d = wp.distance(pos);
                if d < best_d {
                    best_d = d;
                    best = Some(ore);
                }
            }
        }
    }
    let Some(ore) = best else {
        return;
    };

    player.mine_acc += PlayerStats::MINE_RATE * time.delta_secs();
    if player.mine_acc >= 1.0 {
        player.mine_acc = 0.0;
        let near_core = core_q
            .iter()
            .any(|c| c.translation.truncate().distance(pos) < 5.0 * TILE);
        if near_core {
            inv.add(ore, 1);
        } else {
            player.carrying = Some(ore);
        }
    }

    // 上交
    if let Some(res) = player.carrying {
        if core_q
            .iter()
            .any(|c| c.translation.truncate().distance(pos) < 2.2 * TILE)
        {
            inv.add(res, 1);
            player.carrying = None;
        }
    }
}

pub fn player_shoot(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    hover: Res<HoverTile>,
    build: Res<BuildMode>,
    over: Res<GameOver>,
    mut commands: Commands,
    mut q: Query<(&Transform, &mut Player)>,
) {
    if over.done {
        return;
    }
    let Ok((tf, mut player)) = q.get_single_mut() else {
        return;
    };
    player.shoot_cd = (player.shoot_cd - time.delta_secs()).max(0.0);

    let shooting = keys.pressed(KeyCode::Space)
        || (mouse.pressed(MouseButton::Left) && build.selected.is_none());
    if !shooting || player.shoot_cd > 0.0 {
        return;
    }

    let origin = tf.translation.truncate();
    let aim = (hover.world - origin).normalize_or_zero();
    if aim == Vec2::ZERO {
        return;
    }
    player.shoot_cd = 1.0 / PlayerStats::FIRE_RATE;

    let spawn = origin + aim * 14.0;
    commands.spawn((
        Sprite {
            color: crate::config::colors::BULLET,
            custom_size: Some(Vec2::splat(6.0)),
            ..default()
        },
        Transform::from_translation(spawn.extend(6.0)),
        Bullet {
            damage: PlayerStats::DAMAGE,
            life: 0.55,
        },
        LinearVelocity(aim * 420.0),
    ));
}
