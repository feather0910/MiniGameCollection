use bevy::prelude::*;

use crate::components::*;
use crate::config::{tile_to_world, Dir, TILE};
use crate::map::{GameMap, GameOver, Inventory};

pub fn tick_conveyors(
    time: Res<Time>,
    over: Res<GameOver>,
    map: Res<GameMap>,
    mut inv: ResMut<Inventory>,
    mut belts: Query<(Entity, &Building, &mut Conveyor)>,
    mut turrets: Query<&mut Turret>,
    cores: Query<Entity, With<CoreTag>>,
    mut commands: Commands,
    visuals: Query<Entity, With<BeltItemVisual>>,
) {
    if over.done {
        return;
    }
    let dt = time.delta_secs();
    let speed = 4.2;

    // 推进
    let mut movers: Vec<(Entity, i32, i32, Dir, crate::config::ResourceKind, f32)> = Vec::new();
    for (e, b, mut belt) in belts.iter_mut() {
        if let Some(ref mut item) = belt.item {
            item.progress += speed * dt;
            if item.progress >= 1.0 {
                movers.push((e, b.tx, b.ty, b.rot, item.kind, item.progress));
            }
        }
    }

    movers.sort_by(|a, b| b.5.partial_cmp(&a.5).unwrap_or(std::cmp::Ordering::Equal));

    for (e, tx, ty, rot, kind, _) in movers {
        let (dx, dy) = rot.delta();
        let nx = tx + dx;
        let ny = ty + dy;
        let Some(tile) = map.get(nx, ny) else {
            if let Ok((_, _, mut belt)) = belts.get_mut(e) {
                if let Some(ref mut item) = belt.item {
                    item.progress = 0.99;
                }
            }
            continue;
        };
        let Some(target) = tile.building else {
            if let Ok((_, _, mut belt)) = belts.get_mut(e) {
                if let Some(ref mut item) = belt.item {
                    item.progress = 0.99;
                }
            }
            continue;
        };

        let mut moved = false;
        if cores.get(target).is_ok() {
            inv.add(kind, 1);
            moved = true;
        } else if let Ok((_, _, mut dest)) = belts.get_mut(target) {
            if dest.item.is_none() {
                dest.item = Some(BeltItem {
                    kind,
                    progress: 0.0,
                });
                moved = true;
            }
        } else if let Ok(mut turret) = turrets.get_mut(target) {
            if turret.ammo_type == kind {
                turret.ammo += 1;
                moved = true;
            }
        }

        if let Ok((_, _, mut belt)) = belts.get_mut(e) {
            if moved {
                belt.item = None;
            } else if let Some(ref mut item) = belt.item {
                item.progress = 0.99;
            }
        }
    }

    // 刷新物品视觉
    for v in visuals.iter() {
        commands.entity(v).despawn();
    }
    for (_, b, belt) in belts.iter() {
        if let Some(item) = belt.item {
            let (dx, dy) = b.rot.delta();
            let p = item.progress.clamp(0.0, 1.0);
            let base = tile_to_world(b.tx, b.ty, 1);
            let pos = base
                + Vec3::new(
                    (p - 0.5) * dx as f32 * TILE * 0.85,
                    (p - 0.5) * dy as f32 * TILE * 0.85,
                    3.0,
                );
            commands.spawn((
                Sprite {
                    color: item.kind.color(),
                    custom_size: Some(Vec2::splat(10.0)),
                    ..default()
                },
                Transform::from_translation(pos),
                BeltItemVisual,
            ));
        }
    }
}
