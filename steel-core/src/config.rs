//! 数据驱动配置：改数值 / 加建筑主要改这里

use bevy::prelude::*;

pub const TILE: f32 = 32.0;
pub const MAP_W: i32 = 40;
pub const MAP_H: i32 = 30;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResourceKind {
    Copper,
    Lead,
}

impl ResourceKind {
    pub fn color(self) -> Color {
        match self {
            Self::Copper => Color::srgb(0.88, 0.54, 0.24),
            Self::Lead => Color::srgb(0.55, 0.48, 0.72),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Copper => "铜",
            Self::Lead => "铅",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BuildingKind {
    Core,
    Drill,
    Conveyor,
    Wall,
    Duo,
    Scatter,
}

impl BuildingKind {
    pub fn selectable() -> &'static [BuildingKind] {
        &[
            Self::Drill,
            Self::Conveyor,
            Self::Wall,
            Self::Duo,
            Self::Scatter,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Core => "核心",
            Self::Drill => "钻头",
            Self::Conveyor => "传送带",
            Self::Wall => "铜墙",
            Self::Duo => "双管炮",
            Self::Scatter => "散射炮",
        }
    }

    pub fn size(self) -> i32 {
        match self {
            Self::Core => 3,
            _ => 1,
        }
    }

    pub fn solid(self) -> bool {
        !matches!(self, Self::Conveyor)
    }

    pub fn max_hp(self) -> f32 {
        match self {
            Self::Core => 1200.0,
            Self::Drill => 80.0,
            Self::Conveyor => 40.0,
            Self::Wall => 320.0,
            Self::Duo => 160.0,
            Self::Scatter => 200.0,
        }
    }

    pub fn cost(self) -> (u32, u32) {
        // (copper, lead)
        match self {
            Self::Core => (0, 0),
            Self::Drill => (12, 0),
            Self::Conveyor => (1, 0),
            Self::Wall => (6, 0),
            Self::Duo => (35, 0),
            Self::Scatter => (40, 25),
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Core => Color::srgb(0.24, 0.84, 0.78),
            Self::Drill => Color::srgb(0.24, 0.42, 0.47),
            Self::Conveyor => Color::srgb(0.18, 0.29, 0.32),
            Self::Wall => Color::srgb(0.29, 0.35, 0.40),
            Self::Duo => Color::srgb(0.35, 0.42, 0.47),
            Self::Scatter => Color::srgb(0.42, 0.38, 0.55),
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Core => "必须守护的基地",
            Self::Drill => "在矿脉上开采资源",
            Self::Conveyor => "运输资源到核心/炮塔",
            Self::Wall => "阻挡敌人前进",
            Self::Duo => "消耗铜弹药射击",
            Self::Scatter => "消耗铅弹药散射",
            }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    Right = 0,
    Down = 1,
    Left = 2,
    Up = 3,
}

impl Dir {
    pub fn from_index(i: u8) -> Self {
        match i % 4 {
            0 => Self::Right,
            1 => Self::Down,
            2 => Self::Left,
            _ => Self::Up,
        }
    }

    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn delta(self) -> (i32, i32) {
        match self {
            Self::Right => (1, 0),
            Self::Down => (0, -1), // Bevy Y-up：屏幕下方为 -Y，网格 y 向上增加时需统一
            Self::Left => (-1, 0),
            Self::Up => (0, 1),
        }
    }

    pub fn next(self) -> Self {
        Self::from_index(self.index() + 1)
    }

    pub fn from_delta(dx: i32, dy: i32) -> Option<Self> {
        if dx == 0 && dy == 0 {
            return None;
        }
        if dx.abs() >= dy.abs() {
            Some(if dx > 0 { Self::Right } else { Self::Left })
        } else {
            Some(if dy > 0 { Self::Up } else { Self::Down })
        }
    }
}

/// 网格坐标：x 向右，y 向上（与 Bevy 2D 一致）
pub fn tile_to_world(tx: i32, ty: i32, size: i32) -> Vec3 {
    let cx = (tx as f32 + size as f32 * 0.5) * TILE - (MAP_W as f32 * TILE * 0.5);
    let cy = (ty as f32 + size as f32 * 0.5) * TILE - (MAP_H as f32 * TILE * 0.5);
    Vec3::new(cx, cy, 0.0)
}

pub fn world_to_tile(pos: Vec2) -> (i32, i32) {
    let x = ((pos.x + MAP_W as f32 * TILE * 0.5) / TILE).floor() as i32;
    let y = ((pos.y + MAP_H as f32 * TILE * 0.5) / TILE).floor() as i32;
    (x, y)
}

pub mod colors {
    use bevy::prelude::*;
    pub const BG: Color = Color::srgb(0.043, 0.078, 0.094);
    pub const FLOOR_A: Color = Color::srgb(0.086, 0.188, 0.220);
    pub const FLOOR_B: Color = Color::srgb(0.102, 0.220, 0.251);
    pub const PLAYER: Color = Color::srgb(0.49, 0.88, 1.0);
    pub const ENEMY: Color = Color::srgb(0.91, 0.36, 0.36);
    pub const ENEMY_TANK: Color = Color::srgb(0.60, 0.18, 0.18);
    pub const BULLET: Color = Color::srgb(1.0, 0.88, 0.54);
    pub const HP_GOOD: Color = Color::srgb(0.24, 0.84, 0.55);
    pub const HP_BAD: Color = Color::srgb(0.91, 0.36, 0.36);
}

pub struct PlayerStats;
impl PlayerStats {
    pub const SPEED: f32 = 165.0;
    pub const RADIUS: f32 = 10.0;
    pub const HP: f32 = 200.0;
    pub const MINE_RANGE: f32 = 52.0;
    pub const MINE_RATE: f32 = 2.5;
    pub const FIRE_RATE: f32 = 3.5;
    pub const DAMAGE: f32 = 8.0;
}

pub struct WaveStats;
impl WaveStats {
    pub const FIRST_DELAY: f32 = 30.0;
    pub const INTERVAL: f32 = 40.0;
    pub const BASE_COUNT: i32 = 4;
    pub const COUNT_GROWTH: i32 = 2;
    pub const HP_BASE: f32 = 40.0;
    pub const HP_GROWTH: f32 = 14.0;
    pub const SPEED_BASE: f32 = 40.0;
    pub const SPEED_GROWTH: f32 = 1.5;
    pub const DAMAGE: f32 = 8.0;
    pub const ATTACK_INTERVAL: f32 = 0.7;
    pub const WIN_WAVE: u32 = 15;
}
