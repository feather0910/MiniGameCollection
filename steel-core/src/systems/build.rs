use bevy::prelude::*;

use crate::components::*;
use crate::config::{world_to_tile, BuildingKind, Dir};
use crate::map::{BuildMode, GameMap, GameOver, HoverTile, Inventory};
use crate::systems::setup::spawn_building;

pub fn update_hover(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut hover: ResMut<HoverTile>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok((camera, cam_tf)) = camera_q.get_single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(world) = camera.viewport_to_world_2d(cam_tf, cursor) else {
        return;
    };
    hover.world = world;
    let (x, y) = world_to_tile(world);
    hover.x = x;
    hover.y = y;
}

pub fn handle_build_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<BuildMode>,
    mut conveyors: Query<&mut Building, With<Conveyor>>,
    hover: Res<HoverTile>,
    map: Res<GameMap>,
) {
    let kinds = BuildingKind::selectable();
    for (i, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
    ]
    .iter()
    .enumerate()
    {
        if keys.just_pressed(*key) {
            if let Some(k) = kinds.get(i) {
                mode.selected = Some(*k);
            }
        }
    }
    if keys.just_pressed(KeyCode::KeyQ) || keys.just_pressed(KeyCode::Escape) {
        mode.selected = None;
    }
    if keys.just_pressed(KeyCode::Tab) {
        let idx = mode
            .selected
            .and_then(|s| kinds.iter().position(|k| *k == s))
            .map(|i| (i + 1) % kinds.len())
            .unwrap_or(0);
        mode.selected = Some(kinds[idx]);
    }
    if keys.just_pressed(KeyCode::KeyR) {
        mode.rot = mode.rot.next();
        // 旋转指向的传送带
        if let Some(tile) = map.get(hover.x, hover.y) {
            if let Some(e) = tile.building {
                if let Ok(mut b) = conveyors.get_mut(e) {
                    if b.kind == BuildingKind::Conveyor {
                        b.rot = b.rot.next();
                    }
                }
            }
        }
    }
}

pub fn handle_build_click(
    mouse: Res<ButtonInput<MouseButton>>,
    mut mode: ResMut<BuildMode>,
    mut map: ResMut<GameMap>,
    mut inv: ResMut<Inventory>,
    hover: Res<HoverTile>,
    over: Res<GameOver>,
    mut commands: Commands,
) {
    if over.done {
        return;
    }
    let Some(kind) = mode.selected else {
        mode.last_place = None;
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        mode.last_place = None;
        return;
    }

    let (x, y) = (hover.x, hover.y);
    if mode.last_place == Some((x, y)) {
        return;
    }

    let mut rot = mode.rot;
    if kind == BuildingKind::Conveyor {
        if let Some((lx, ly)) = mode.last_place {
            if let Some(d) = Dir::from_delta(x - lx, y - ly) {
                rot = d;
                mode.rot = d;
            }
        }
    }

    let require_ore = kind == BuildingKind::Drill;
    if !map.can_place(x, y, kind.size(), require_ore) {
        return;
    }
    if kind == BuildingKind::Drill && map.get(x, y).and_then(|t| t.ore).is_none() {
        return;
    }
    if !inv.pay(kind) {
        return;
    }

    spawn_building(&mut commands, &mut map, kind, x, y, rot);
    mode.last_place = Some((x, y));
}

pub fn handle_demolish(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    hover: Res<HoverTile>,
    mut map: ResMut<GameMap>,
    mut inv: ResMut<Inventory>,
    over: Res<GameOver>,
    mut commands: Commands,
    buildings: Query<&Building>,
    bars: Query<(Entity, &HpBar)>,
) {
    if over.done {
        return;
    }
    let want = mouse.just_pressed(MouseButton::Right)
        || keys.just_pressed(KeyCode::KeyX)
        || keys.just_pressed(KeyCode::Backspace);
    if !want {
        return;
    }
    let Some(tile) = map.get(hover.x, hover.y).cloned() else {
        return;
    };
    let Some(e) = tile.building else {
        return;
    };
    let Ok(b) = buildings.get(e) else {
        return;
    };
    if b.kind == BuildingKind::Core {
        return;
    }
    let kind = b.kind;
    let (tx, ty, size) = (b.tx, b.ty, kind.size());
    map.clear(tx, ty, size);
    inv.refund_half(kind);
    // 清血条
    for (bar_e, bar) in bars.iter() {
        if bar.owner == e {
            commands.entity(bar_e).despawn();
        }
    }
    commands.entity(e).despawn_recursive();
}

pub fn update_ghost(
    mode: Res<BuildMode>,
    hover: Res<HoverTile>,
    map: Res<GameMap>,
    inv: Res<Inventory>,
    mut commands: Commands,
    ghosts: Query<Entity, With<GhostPreview>>,
) {
    for e in ghosts.iter() {
        commands.entity(e).despawn_recursive();
    }
    let Some(kind) = mode.selected else {
        return;
    };
    let require_ore = kind == BuildingKind::Drill;
    let ok = map.can_place(hover.x, hover.y, kind.size(), require_ore)
        && inv.can_afford(kind)
        && (!require_ore || map.get(hover.x, hover.y).and_then(|t| t.ore).is_some());

    let pos = crate::config::tile_to_world(hover.x, hover.y, kind.size());
    let color = if ok {
        Color::srgba(0.24, 0.84, 0.78, 0.35)
    } else {
        Color::srgba(0.91, 0.36, 0.36, 0.35)
    };
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::splat(kind.size() as f32 * crate::config::TILE - 2.0)),
            ..default()
        },
        Transform::from_translation(pos + Vec3::new(0.0, 0.0, 4.0)),
        GhostPreview,
    ));
}
