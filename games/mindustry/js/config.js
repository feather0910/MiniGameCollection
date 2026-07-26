/** 钢核防线 — 配置与建筑定义（数据驱动，便于扩展） */

export const TILE = 32;
export const MAP_W = 40;
export const MAP_H = 30;
export const TICK_RATE = 60;

/** 风格化配色：工业青绿 + 铜橙，辨识度高 */
export const COLORS = {
  bg: '#0b1418',
  grid: '#142228',
  gridLine: '#1a2e36',
  floor: '#163038',
  floorAlt: '#1a3840',
  copper: '#e08a3c',
  copperDark: '#a85f24',
  lead: '#8b7bb8',
  leadDark: '#5e4f82',
  core: '#3dd6c6',
  coreDark: '#1f8a7e',
  turret: '#5a6b78',
  turretAccent: '#f0c14a',
  wall: '#4a5a66',
  conveyor: '#2d4a52',
  conveyorArrow: '#6ec9b8',
  drill: '#3d6a78',
  player: '#7ee0ff',
  enemy: '#e85d5d',
  enemyDark: '#9a2e2e',
  bullet: '#ffe08a',
  hpGood: '#3dd68c',
  hpBad: '#e85d5d',
  ghostOk: 'rgba(61, 214, 198, 0.35)',
  ghostBad: 'rgba(232, 93, 93, 0.35)',
  oreGlow: 0.55,
};

export const RESOURCES = {
  copper: { id: 'copper', name: '铜', color: COLORS.copper },
  lead: { id: 'lead', name: '铅', color: COLORS.lead },
};

/**
 * 建筑定义：cost / size / category / tick 行为由系统读取
 * category: extract | logistics | defense | core
 */
export const BUILDINGS = {
  core: {
    id: 'core',
    name: '核心',
    size: 3,
    category: 'core',
    hp: 1200,
    cost: {},
    solid: true,
    description: '必须守护的基地，接收资源',
  },
  drill: {
    id: 'drill',
    name: '钻头',
    size: 1,
    category: 'extract',
    hp: 80,
    cost: { copper: 12 },
    solid: true,
    mineRate: 0.35, // 每秒产出
    description: '在矿脉上开采资源',
  },
  conveyor: {
    id: 'conveyor',
    name: '传送带',
    size: 1,
    category: 'logistics',
    hp: 40,
    cost: { copper: 1 },
    solid: false,
    speed: 4.2, // 格/秒
    rotatable: true,
    description: '运输资源到核心',
  },
  wall: {
    id: 'wall',
    name: '铜墙',
    size: 1,
    category: 'defense',
    hp: 320,
    cost: { copper: 6 },
    solid: true,
    description: '阻挡敌人前进',
  },
  duo: {
    id: 'duo',
    name: '双管炮',
    size: 1,
    category: 'defense',
    hp: 160,
    cost: { copper: 35 },
    solid: true,
    range: 6.5,
    fireRate: 2.2,
    damage: 12,
    ammoPerShot: 1,
    ammoType: 'copper',
    description: '消耗铜弹药射击敌人',
  },
  scatter: {
    id: 'scatter',
    name: '散射炮',
    size: 1,
    category: 'defense',
    hp: 200,
    cost: { copper: 40, lead: 25 },
    solid: true,
    range: 8,
    fireRate: 1.1,
    damage: 9,
    pellets: 3,
    ammoPerShot: 2,
    ammoType: 'lead',
    description: '消耗铅弹药，散射打击',
  },
};

/** 建造栏顺序（不含核心） */
export const BUILD_ORDER = ['drill', 'conveyor', 'wall', 'duo', 'scatter'];

export const PLAYER = {
  speed: 5.2,
  radius: 0.32,
  mineRange: 1.6,
  mineRate: 2.5,
  hp: 200,
  shootRange: 5,
  fireRate: 3.5,
  damage: 8,
};

export const WAVE = {
  firstDelay: 30,
  interval: 40,
  baseCount: 4,
  countGrowth: 2,
  hpBase: 40,
  hpGrowth: 14,
  speedBase: 1.15,
  speedGrowth: 0.04,
  damage: 8,
  attackInterval: 0.7,
};

export const DIRS = [
  { dx: 1, dy: 0, name: 'right' },
  { dx: 0, dy: 1, name: 'down' },
  { dx: -1, dy: 0, name: 'left' },
  { dx: 0, dy: -1, name: 'up' },
];
