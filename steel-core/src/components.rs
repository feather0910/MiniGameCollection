use bevy::prelude::*;

use crate::config::{BuildingKind, Dir, ResourceKind};

#[derive(Component)]
pub struct Player {
    pub hp: f32,
    pub max_hp: f32,
    pub mine_acc: f32,
    pub shoot_cd: f32,
    pub carrying: Option<ResourceKind>,
}

#[derive(Component)]
pub struct Building {
    pub kind: BuildingKind,
    pub tx: i32,
    pub ty: i32,
    pub rot: Dir,
    pub hp: f32,
    pub max_hp: f32,
}

#[derive(Component)]
pub struct CoreTag;

#[derive(Component)]
pub struct Drill {
    pub ore: ResourceKind,
    pub acc: f32,
}

#[derive(Component)]
pub struct Conveyor {
    pub item: Option<BeltItem>,
}

#[derive(Clone, Copy, Debug)]
pub struct BeltItem {
    pub kind: ResourceKind,
    pub progress: f32,
}

#[derive(Component)]
pub struct Turret {
    pub cool: f32,
    pub aim: f32,
    pub ammo: u32,
    pub range: f32,
    pub fire_rate: f32,
    pub damage: f32,
    pub ammo_cost: u32,
    pub ammo_type: ResourceKind,
    pub pellets: u32,
}

#[derive(Component)]
pub struct Enemy {
    pub hp: f32,
    pub max_hp: f32,
    pub speed: f32,
    pub damage: f32,
    pub attack_acc: f32,
    pub path: Vec<(i32, i32)>,
    pub path_idx: usize,
    pub retarget: f32,
    pub is_tank: bool,
}

#[derive(Component)]
pub struct Bullet {
    pub damage: f32,
    pub life: f32,
}

#[derive(Component)]
pub struct OreVein {
    pub kind: ResourceKind,
    pub tx: i32,
    pub ty: i32,
}

#[derive(Component)]
pub struct FloorTile;

#[derive(Component)]
pub struct HpBar {
    pub owner: Entity,
    pub width: f32,
}

#[derive(Component)]
pub struct BeltItemVisual;

#[derive(Component)]
pub struct GhostPreview;

#[derive(Component)]
pub struct MainCamera;

/// 简易速度（不引入物理插件）
#[derive(Component, Clone, Copy)]
pub struct LinearVelocity(pub Vec2);
