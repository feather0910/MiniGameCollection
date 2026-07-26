use bevy::prelude::*;

use crate::components::*;
use crate::config::Dir;
use crate::map::{GameMap, GameOver, Inventory};

pub fn tick_extractors(
    time: Res<Time>,
    over: Res<GameOver>,
    map: Res<GameMap>,
    mut inv: ResMut<Inventory>,
    mut drills: Query<(&Building, &mut Drill)>,
    mut conveyors: Query<&mut Conveyor>,
    mut turrets: Query<&mut Turret>,
    cores: Query<Entity, With<CoreTag>>,
) {
    if over.done {
        return;
    }
    let dt = time.delta_secs();
    for (b, mut drill) in drills.iter_mut() {
        drill.acc += 0.35 * dt;
        while drill.acc >= 1.0 {
            drill.acc -= 1.0;
            try_output(
                &map,
                b,
                drill.ore,
                &mut inv,
                &mut conveyors,
                &mut turrets,
                &cores,
            );
        }
    }
}

fn try_output(
    map: &GameMap,
    from: &Building,
    item: crate::config::ResourceKind,
    inv: &mut Inventory,
    conveyors: &mut Query<&mut Conveyor>,
    turrets: &mut Query<&mut Turret>,
    cores: &Query<Entity, With<CoreTag>>,
) -> bool {
    for i in 0..4 {
        let d = Dir::from_index(i);
        let (dx, dy) = d.delta();
        let nx = from.tx + dx;
        let ny = from.ty + dy;
        let Some(tile) = map.get(nx, ny) else {
            continue;
        };
        let Some(e) = tile.building else {
            continue;
        };
        if cores.get(e).is_ok() {
            inv.add(item, 1);
            return true;
        }
        if let Ok(mut belt) = conveyors.get_mut(e) {
            if belt.item.is_none() {
                belt.item = Some(crate::components::BeltItem {
                    kind: item,
                    progress: 0.0,
                });
                return true;
            }
        }
        if let Ok(mut turret) = turrets.get_mut(e) {
            if turret.ammo_type == item {
                turret.ammo += 1;
                return true;
            }
        }
    }
    false
}
