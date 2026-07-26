use bevy::prelude::*;

use crate::components::*;
use crate::config::{colors, tile_to_world, BuildingKind, Dir, TILE, MAP_H, MAP_W};
use crate::map::GameMap;

pub fn setup_world(mut commands: Commands, mut map: ResMut<GameMap>) {
    commands.spawn((Camera2d, MainCamera));

    // 地板棋盘
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let color = if (x + y) % 2 == 0 {
                colors::FLOOR_A
            } else {
                colors::FLOOR_B
            };
            let pos = tile_to_world(x, y, 1);
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(TILE - 1.0)),
                    ..default()
                },
                Transform::from_translation(pos + Vec3::new(0.0, 0.0, -10.0)),
                FloorTile,
            ));

            if let Some(ore) = map.get(x, y).and_then(|t| t.ore) {
                spawn_ore_visual(&mut commands, x, y, ore);
            }
        }
    }

    // 背景色
    commands.insert_resource(ClearColor(colors::BG));

    // 核心
    let core_tx = 18;
    let core_ty = 13;
    let core_e = spawn_building(
        &mut commands,
        &mut map,
        BuildingKind::Core,
        core_tx,
        core_ty,
        Dir::Right,
    );

    // 玩家
    let p = tile_to_world(core_tx + 1, core_ty - 1, 1);
    commands.spawn((
        Sprite {
            color: colors::PLAYER,
            custom_size: Some(Vec2::new(20.0, 16.0)),
            ..default()
        },
        Transform::from_translation(p + Vec3::new(0.0, 0.0, 5.0)),
        Player {
            hp: crate::config::PlayerStats::HP,
            max_hp: crate::config::PlayerStats::HP,
            mine_acc: 0.0,
            shoot_cd: 0.0,
            carrying: None,
        },
    ));

    // 核心血条
    spawn_hp_bar(&mut commands, core_e, 3.0 * TILE);
}

fn spawn_ore_visual(commands: &mut Commands, tx: i32, ty: i32, ore: crate::config::ResourceKind) {
    let pos = tile_to_world(tx, ty, 1);
    let base = ore.color().darker(0.25);
    commands.spawn((
        Sprite {
            color: base.with_alpha(0.7),
            custom_size: Some(Vec2::splat(TILE - 4.0)),
            ..default()
        },
        Transform::from_translation(pos + Vec3::new(0.0, 0.0, -8.0)),
        OreVein {
            kind: ore,
            tx,
            ty,
        },
    ));
    // 碎晶点
    for (ox, oy, s) in [(-6.0, -4.0, 4.0), (5.0, 3.0, 3.0), (-2.0, 6.0, 3.0), (7.0, -6.0, 4.0)] {
        commands.spawn((
            Sprite {
                color: ore.color(),
                custom_size: Some(Vec2::splat(s)),
                ..default()
            },
            Transform::from_translation(pos + Vec3::new(ox, oy, -7.5)),
        ));
    }
}

pub fn spawn_building(
    commands: &mut Commands,
    map: &mut GameMap,
    kind: BuildingKind,
    tx: i32,
    ty: i32,
    rot: Dir,
) -> Entity {
    let size = kind.size();
    let pos = tile_to_world(tx, ty, size);
    let z = match kind {
        BuildingKind::Conveyor => 0.0,
        BuildingKind::Core => 2.0,
        _ => 1.0,
    };

    let entity = commands
        .spawn((
            Sprite {
                color: kind.color(),
                custom_size: Some(Vec2::splat(size as f32 * TILE - 4.0)),
                ..default()
            },
            Transform::from_translation(pos + Vec3::new(0.0, 0.0, z)),
            Building {
                kind,
                tx,
                ty,
                rot,
                hp: kind.max_hp(),
                max_hp: kind.max_hp(),
            },
        ))
        .id();

    match kind {
        BuildingKind::Core => {
            commands.entity(entity).insert(CoreTag).with_children(|c| {
                c.spawn((
                    Sprite {
                        color: Color::srgb(0.91, 1.0, 0.98),
                        custom_size: Some(Vec2::splat(18.0)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(0.0, 0.0, 0.1))
                        .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
                ));
            });
        }
        BuildingKind::Drill => {
            let ore = map
                .get(tx, ty)
                .and_then(|t| t.ore)
                .unwrap_or(crate::config::ResourceKind::Copper);
            commands.entity(entity).insert(Drill { ore, acc: 0.0 });
        }
        BuildingKind::Conveyor => {
            let (dx, dy) = rot.delta();
            commands
                .entity(entity)
                .insert(Conveyor { item: None })
                .with_children(|c| {
                    c.spawn((
                        Sprite {
                            color: Color::srgb(0.43, 0.79, 0.72),
                            custom_size: Some(Vec2::new(10.0, 6.0)),
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(
                            dx as f32 * 6.0,
                            dy as f32 * 6.0,
                            0.1,
                        )),
                    ));
                });
        }
        BuildingKind::Duo => {
            commands.entity(entity).insert(Turret {
                cool: 0.0,
                aim: 0.0,
                ammo: 0,
                range: 6.5 * TILE,
                fire_rate: 2.2,
                damage: 12.0,
                ammo_cost: 1,
                ammo_type: crate::config::ResourceKind::Copper,
                pellets: 1,
            });
        }
        BuildingKind::Scatter => {
            commands.entity(entity).insert(Turret {
                cool: 0.0,
                aim: 0.0,
                ammo: 0,
                range: 8.0 * TILE,
                fire_rate: 1.1,
                damage: 9.0,
                ammo_cost: 2,
                ammo_type: crate::config::ResourceKind::Lead,
                pellets: 3,
            });
        }
        BuildingKind::Wall => {}
    }

    map.mark(tx, ty, size, entity, kind.solid());
    if kind != BuildingKind::Core {
        spawn_hp_bar(commands, entity, size as f32 * TILE);
    }
    entity
}

pub fn spawn_hp_bar(commands: &mut Commands, owner: Entity, width: f32) {
    commands.spawn((
        Sprite {
            color: colors::HP_GOOD,
            custom_size: Some(Vec2::new(width - 6.0, 3.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 8.0)),
        HpBar { owner, width: width - 6.0 },
    ));
}

/// Color helper
trait ColorExt {
    fn darker(self, amount: f32) -> Color;
    fn with_alpha(self, a: f32) -> Color;
}

impl ColorExt for Color {
    fn darker(self, amount: f32) -> Color {
        let c = self.to_srgba();
        Color::srgba(
            (c.red * (1.0 - amount)).clamp(0.0, 1.0),
            (c.green * (1.0 - amount)).clamp(0.0, 1.0),
            (c.blue * (1.0 - amount)).clamp(0.0, 1.0),
            c.alpha,
        )
    }

    fn with_alpha(self, a: f32) -> Color {
        let c = self.to_srgba();
        Color::srgba(c.red, c.green, c.blue, a)
    }
}
