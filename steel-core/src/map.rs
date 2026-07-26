use bevy::prelude::*;

use crate::config::{BuildingKind, ResourceKind, MAP_H, MAP_W};

#[derive(Clone, Debug)]
pub struct Tile {
    pub ore: Option<ResourceKind>,
    pub building: Option<Entity>,
    pub occupied: bool,
    pub solid: bool,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            ore: None,
            building: None,
            occupied: false,
            solid: false,
        }
    }
}

#[derive(Resource)]
pub struct GameMap {
    pub tiles: Vec<Tile>,
    pub w: i32,
    pub h: i32,
}

impl GameMap {
    pub fn new() -> Self {
        let mut map = Self {
            tiles: vec![Tile::default(); (MAP_W * MAP_H) as usize],
            w: MAP_W,
            h: MAP_H,
        };
        map.seed_ores();
        map
    }

    fn idx(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            None
        } else {
            Some((y * self.w + x) as usize)
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&Tile> {
        self.idx(x, y).map(|i| &self.tiles[i])
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut Tile> {
        self.idx(x, y).map(|i| &mut self.tiles[i])
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        self.idx(x, y).is_some()
    }

    pub fn can_place(&self, tx: i32, ty: i32, size: i32, require_ore: bool) -> bool {
        if tx < 0 || ty < 0 || tx + size > self.w || ty + size > self.h {
            return false;
        }
        let mut has_ore = false;
        for y in ty..ty + size {
            for x in tx..tx + size {
                let t = self.get(x, y).unwrap();
                if t.occupied {
                    return false;
                }
                if t.ore.is_some() {
                    has_ore = true;
                }
            }
        }
        if require_ore && !has_ore {
            return false;
        }
        true
    }

    pub fn mark(&mut self, tx: i32, ty: i32, size: i32, entity: Entity, solid: bool) {
        for y in ty..ty + size {
            for x in tx..tx + size {
                if let Some(t) = self.get_mut(x, y) {
                    t.occupied = true;
                    t.building = Some(entity);
                    t.solid = solid;
                }
            }
        }
    }

    pub fn clear(&mut self, tx: i32, ty: i32, size: i32) {
        for y in ty..ty + size {
            for x in tx..tx + size {
                if let Some(t) = self.get_mut(x, y) {
                    t.occupied = false;
                    t.building = None;
                    t.solid = false;
                }
            }
        }
    }

    fn seed_ores(&mut self) {
        let patches = [
            (8, 10, 3.2_f32, ResourceKind::Copper, 0.85_f32),
            (12, 18, 2.8, ResourceKind::Copper, 0.75),
            (28, 8, 3.0, ResourceKind::Copper, 0.8),
            (30, 20, 2.5, ResourceKind::Lead, 0.7),
            (22, 14, 2.2, ResourceKind::Lead, 0.65),
            (6, 22, 2.0, ResourceKind::Lead, 0.6),
            (18, 6, 2.4, ResourceKind::Copper, 0.7),
        ];

        for y in 0..self.h {
            for x in 0..self.w {
                // 核心保护区
                if (17..=22).contains(&x) && (12..=17).contains(&y) {
                    continue;
                }
                for &(cx, cy, r, ore, dens) in &patches {
                    let d = ((x - cx) as f32).hypot((y - cy) as f32);
                    if d < r {
                        let edge = 1.0 - d / r;
                        let n = hash2(x, y);
                        if n < dens * (0.45 + edge * 0.55) {
                            if let Some(t) = self.get_mut(x, y) {
                                t.ore = Some(ore);
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    /// BFS 寻路。核心格不阻挡，便于冲向多格核心。
    pub fn find_path(&self, sx: i32, sy: i32, goals: &[(i32, i32)]) -> Option<Vec<(i32, i32)>> {
        if goals.is_empty() {
            return None;
        }
        let goal_set: std::collections::HashSet<(i32, i32)> = goals.iter().copied().collect();
        let start = (sx, sy);
        if !self.in_bounds(sx, sy) {
            return None;
        }

        let mut q = vec![start];
        let mut qi = 0;
        let mut came: std::collections::HashMap<(i32, i32), Option<(i32, i32)>> =
            std::collections::HashMap::new();
        came.insert(start, None);
        let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];

        while qi < q.len() {
            let cur = q[qi];
            qi += 1;
            if goal_set.contains(&cur) {
                let mut path = vec![cur];
                let mut c = cur;
                while let Some(Some(prev)) = came.get(&c).cloned() {
                    path.push(prev);
                    c = prev;
                }
                path.reverse();
                return Some(path);
            }
            for (dx, dy) in dirs {
                let nx = cur.0 + dx;
                let ny = cur.1 + dy;
                if !self.in_bounds(nx, ny) || came.contains_key(&(nx, ny)) {
                    continue;
                }
                let blocked = self
                    .get(nx, ny)
                    .map(|t| t.solid && !goal_set.contains(&(nx, ny)))
                    .unwrap_or(true);
                if blocked {
                    continue;
                }
                came.insert((nx, ny), Some(cur));
                q.push((nx, ny));
            }
        }
        None
    }
}

fn hash2(x: i32, y: i32) -> f32 {
    let mut h = x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    let u = (h ^ (h >> 16)) as u32;
    (u as f32) / 4294967296.0
}

#[derive(Resource)]
pub struct Inventory {
    pub copper: u32,
    pub lead: u32,
}

impl Inventory {
    pub fn can_afford(&self, kind: BuildingKind) -> bool {
        let (c, l) = kind.cost();
        self.copper >= c && self.lead >= l
    }

    pub fn pay(&mut self, kind: BuildingKind) -> bool {
        if !self.can_afford(kind) {
            return false;
        }
        let (c, l) = kind.cost();
        self.copper -= c;
        self.lead -= l;
        true
    }

    pub fn refund_half(&mut self, kind: BuildingKind) {
        let (c, l) = kind.cost();
        self.copper += c / 2;
        self.lead += l / 2;
    }

    pub fn add(&mut self, res: ResourceKind, n: u32) {
        match res {
            ResourceKind::Copper => self.copper += n,
            ResourceKind::Lead => self.lead += n,
        }
    }
}

#[derive(Resource)]
pub struct BuildMode {
    pub selected: Option<BuildingKind>,
    pub rot: crate::config::Dir,
    pub last_place: Option<(i32, i32)>,
}

#[derive(Resource)]
pub struct WaveState {
    pub wave: u32,
    pub timer: f32,
    pub between: bool,
    pub spawn_queue: i32,
    pub spawn_acc: f32,
}

#[derive(Resource)]
pub struct GameOver {
    pub done: bool,
    pub won: bool,
}

#[derive(Resource, Default)]
pub struct HoverTile {
    pub x: i32,
    pub y: i32,
    pub world: Vec2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ores_seeded() {
        let map = GameMap::new();
        let ores = map.tiles.iter().filter(|t| t.ore.is_some()).count();
        assert!(ores > 40, "expected ore veins, got {ores}");
    }

    #[test]
    fn path_to_core_tiles() {
        let mut map = GameMap::new();
        // 模拟 3x3 核心占用但不阻挡寻路目标
        let core = Entity::from_raw(1);
        map.mark(18, 13, 3, core, true);
        // 核心格 solid=true，但 find_path 把 goals 当作可走
        let goals: Vec<(i32, i32)> = (13..16)
            .flat_map(|y| (18..21).map(move |x| (x, y)))
            .collect();
        let path = map.find_path(1, 1, &goals);
        assert!(path.is_some(), "should path to multi-tile core");
        assert!(path.unwrap().len() > 5);
    }

    #[test]
    fn drill_requires_ore() {
        let map = GameMap::new();
        // 找一个无矿空地
        let empty = (0..MAP_W)
            .flat_map(|x| (0..MAP_H).map(move |y| (x, y)))
            .find(|&(x, y)| map.get(x, y).map(|t| t.ore.is_none() && !t.occupied).unwrap_or(false))
            .unwrap();
        assert!(!map.can_place(empty.0, empty.1, 1, true));
        let ore_tile = (0..MAP_W)
            .flat_map(|x| (0..MAP_H).map(move |y| (x, y)))
            .find(|&(x, y)| map.get(x, y).map(|t| t.ore.is_some()).unwrap_or(false))
            .unwrap();
        assert!(map.can_place(ore_tile.0, ore_tile.1, 1, true));
    }
}
