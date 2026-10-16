//! The three stages of Mount Helico as courses: a height profile, the ground
//! under every cell, obstacles, aid stations, crowds, animals and props.
//! Each stage is scripted by hand and built the same way every race.

use crate::engine::rng::Rng;

/// Pixels per height sample.
pub const CELL: f32 = 4.0;
/// Where the runners line up.
pub const START_X: f32 = 200.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ground {
    Dirt,
    Grass,
    Rock,
    Mud,
    Scree,
    Snow,
    Water,
    Road,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObKind {
    Root,
    Rock,
    Log,
    Mushroom,
    /// sleeping marmot: jump it, or wake it up
    Marmot,
    SnowRock,
    /// overhead: duck under
    Branch,
    /// a fallen tree propped on a stump: duck under
    Trunk,
}

impl ObKind {
    pub fn width(self) -> f32 {
        match self {
            ObKind::Root => 12.0,
            ObKind::Rock => 12.0,
            ObKind::Log => 20.0,
            ObKind::Mushroom => 8.0,
            ObKind::Marmot => 12.0,
            ObKind::SnowRock => 14.0,
            ObKind::Branch => 16.0,
            ObKind::Trunk => 26.0,
        }
    }
    /// Height of a low obstacle above the trail.
    pub fn height(self) -> f32 {
        match self {
            ObKind::Root => 5.0,
            ObKind::Rock | ObKind::SnowRock => 9.0,
            ObKind::Log => 8.0,
            ObKind::Mushroom => 7.0,
            ObKind::Marmot => 6.0,
            ObKind::Branch | ObKind::Trunk => 0.0,
        }
    }
    pub fn overhead(self) -> bool {
        matches!(self, ObKind::Branch | ObKind::Trunk)
    }
}

/// Clearance under overhead obstacles: a ducking runner fits, a standing one not.
pub const OVERHEAD_CLEAR: f32 = 15.0;

#[derive(Clone, Copy, Debug)]
pub struct Obstacle {
    pub x: f32,
    pub kind: ObKind,
    pub var: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Item {
    TrailMix,
    Banana,
    Stroopwafel,
    Pickle,
}

impl Item {
    pub fn name(self) -> &'static str {
        match self {
            Item::TrailMix => "TRAIL MIX!",
            Item::Banana => "BANANA!",
            Item::Stroopwafel => "STROOPWAFEL!",
            Item::Pickle => "PICKLE JUICE!",
        }
    }
    pub fn stamina(self) -> f32 {
        match self {
            Item::TrailMix => 28.0,
            Item::Banana => 34.0,
            Item::Stroopwafel => 24.0,
            Item::Pickle => 45.0,
        }
    }
    /// HYPE frames it gives.
    pub fn hype(self) -> i32 {
        match self {
            Item::Stroopwafel => 120,
            Item::Pickle => 200,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Aid {
    pub x: f32,
    pub item: Item,
}

#[derive(Clone, Copy, Debug)]
pub struct Crowd {
    pub x: f32,
    pub n: u32,
    pub seed: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ZoneKind {
    /// the trail zig-zags: runners cover ground slowly, Gerald climbs straight up
    Switchback,
    Fog,
    /// deer leap across the trail here
    Deer,
}

#[derive(Clone, Copy, Debug)]
pub struct Zone {
    pub x0: f32,
    pub x1: f32,
    pub kind: ZoneKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AnimalKind {
    Cow,
    /// pops up from its hole and whistles when a runner comes near
    Marmot,
}

#[derive(Clone, Copy, Debug)]
pub struct Spawn {
    pub x: f32,
    pub kind: AnimalKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PropKind {
    Pine,
    Leafy,
    Birch,
    Fir,
    Bush,
    Fern,
    Flowers,
    Boulder,
    Fence,
    Ribbon,
    /// two lines of text on a wooden board
    Sign(u8),
    DeerSign,
    KmPost(u8),
    StartArch,
    FinishArch,
    SummitCross,
    Cairn,
    SnowPole,
    House,
    Church,
    Hut,
}

#[derive(Clone, Copy, Debug)]
pub struct Prop {
    pub x: f32,
    pub kind: PropKind,
    pub var: u8,
}

pub const SIGNS: [(&str, &str); 12] = [
    ("FOREST", "FLATS"),
    ("NO SHORTCUTS", "THIS MEANS YOU"),
    ("BEWARE OF", "MARMOTS"),
    ("COWS HAVE", "RIGHT OF WAY"),
    ("SUMMIT", "STRAIGHT UP"),
    ("FOG. GOOD", "LUCK."),
    ("PICKLE JUICE", "AHEAD"),
    ("ALPINE", "MEADOW"),
    ("SUMMIT", "PUSH"),
    ("HELICO", "VILLAGE"),
    ("DO NOT FEED", "THE GOAT"),
    ("SIT DOWN", "AND SLIDE"),
];

pub const STAGE_NAMES: [&str; 3] = ["FOREST FLATS", "ALPINE MEADOW", "SUMMIT PUSH"];

pub struct Course {
    pub stage: usize,
    pub finish: f32,
    pub end: f32,
    h: Vec<f32>,
    g: Vec<Ground>,
    pub obs: Vec<Obstacle>,
    pub aids: Vec<Aid>,
    pub crowds: Vec<Crowd>,
    pub zones: Vec<Zone>,
    pub spawns: Vec<Spawn>,
    pub props: Vec<Prop>,
    pub min_h: f32,
    pub max_h: f32,
}

impl Course {
    pub fn height(&self, x: f32) -> f32 {
        let u = (x / CELL).max(0.0);
        let i = (u as usize).min(self.h.len() - 2);
        let f = (u - i as f32).min(1.0);
        self.h[i] + (self.h[i + 1] - self.h[i]) * f
    }
    /// Rise over run; positive is uphill (to the right).
    pub fn slope(&self, x: f32) -> f32 {
        (self.height(x + 6.0) - self.height(x - 6.0)) / 12.0
    }
    pub fn ground(&self, x: f32) -> Ground {
        let i = ((x / CELL).max(0.0) as usize).min(self.g.len() - 1);
        self.g[i]
    }
    pub fn zone(&self, x: f32, kind: ZoneKind) -> Option<&Zone> {
        self.zones.iter().find(|z| z.kind == kind && x >= z.x0 && x < z.x1)
    }
    /// How foggy it is at x (0..1).
    pub fn fog(&self, x: f32) -> f32 {
        for z in &self.zones {
            if z.kind == ZoneKind::Fog && x > z.x0 - 300.0 && x < z.x1 + 300.0 {
                let a = ((x - (z.x0 - 300.0)) / 300.0).min(1.0);
                let b = (((z.x1 + 300.0) - x) / 300.0).min(1.0);
                return a.min(b).max(0.0);
            }
        }
        0.0
    }
    /// First index in a sorted-by-x list with x >= `x`.
    pub fn first_prop(&self, x: f32) -> usize {
        self.props.partition_point(|p| p.x < x)
    }
    pub fn first_ob(&self, x: f32) -> usize {
        self.obs.partition_point(|o| o.x < x)
    }
    pub fn cells(&self) -> usize {
        self.h.len()
    }
    pub fn progress(&self, x: f32) -> f32 {
        ((x - START_X) / (self.finish - START_X)).clamp(0.0, 1.0)
    }

    pub fn new(stage: usize) -> Course {
        let mut b = Builder::new(stage);
        match stage {
            0 => forest(&mut b),
            1 => meadow(&mut b),
            _ => summit(&mut b),
        }
        b.finish()
    }
}

// ---------------------------------------------------------------- builder

#[derive(Clone, Copy)]
enum Step {
    /// length, slope, ground, obstacle kinds, average gap (0 = none)
    Seg(f32, f32, Ground, &'static [ObKind], f32),
    Aid(Item),
    Crowd(u32),
    Stream(f32),
    Deer,
    Cow,
    Marmot,
    /// zig-zag climb: length, slope, ground
    Switch(f32, f32, Ground),
    FogOn,
    FogOff,
    Sign(u8),
    Summit,
}

struct Builder {
    c: Course,
    raw: Vec<f32>,
    x: f32,
    hc: f32,
    rng: Rng,
    streams: Vec<(f32, f32)>,
    fog_from: Option<f32>,
}

impl Builder {
    fn new(stage: usize) -> Self {
        Builder {
            c: Course {
                stage,
                finish: 0.0,
                end: 0.0,
                h: Vec::new(),
                g: Vec::new(),
                obs: Vec::new(),
                aids: Vec::new(),
                crowds: Vec::new(),
                zones: Vec::new(),
                spawns: Vec::new(),
                props: Vec::new(),
                min_h: 0.0,
                max_h: 0.0,
            },
            raw: Vec::new(),
            x: 0.0,
            hc: 0.0,
            rng: Rng(0x9E37_79B9 ^ (stage as u64 * 0x1234_5677 + 77)),
            streams: Vec::new(),
            fog_from: None,
        }
    }

    fn seg(&mut self, len: f32, slope: f32, g: Ground) {
        let n = (len / CELL) as usize;
        for _ in 0..n {
            self.raw.push(self.hc);
            self.c.g.push(g);
            self.hc += slope * CELL;
            self.x += CELL;
        }
    }

    fn step(&mut self, s: Step) {
        match s {
            Step::Seg(len, slope, g, kinds, gap) => {
                let x0 = self.x;
                self.seg(len, slope, g);
                if gap > 0.0 && !kinds.is_empty() {
                    let mut x = x0 + 50.0 + self.rng.range(0.0, gap * 0.5);
                    while x < x0 + len - 50.0 {
                        let k = kinds[self.rng.int(kinds.len() as i32) as usize];
                        let var = self.rng.int(4) as u8;
                        self.c.obs.push(Obstacle { x, kind: k, var });
                        x += gap * self.rng.range(0.75, 1.3) + k.width();
                    }
                }
            }
            Step::Aid(item) => {
                let g = self.flat_ground();
                self.seg(70.0, 0.0, g);
                self.c.aids.push(Aid { x: self.x, item });
                self.seg(110.0, 0.0, g);
            }
            Step::Crowd(n) => {
                let seed = self.rng.next();
                self.c.crowds.push(Crowd { x: self.x - 30.0, n, seed });
            }
            Step::Stream(w) => {
                let g = self.flat_ground();
                self.seg(60.0, 0.0, g);
                self.streams.push((self.x, w));
                self.seg(w, 0.0, Ground::Water);
                self.seg(60.0, 0.0, g);
            }
            Step::Deer => {
                let g = self.flat_ground();
                self.c.props.push(Prop { x: self.x + 10.0, kind: PropKind::DeerSign, var: 0 });
                self.seg(150.0, 0.0, g);
                self.c.zones.push(Zone { x0: self.x - 14.0, x1: self.x + 14.0, kind: ZoneKind::Deer });
                self.seg(110.0, 0.0, g);
            }
            Step::Cow => {
                let g = self.flat_ground();
                self.seg(90.0, 0.0, g);
                self.c.spawns.push(Spawn { x: self.x, kind: AnimalKind::Cow });
                self.seg(110.0, 0.0, g);
            }
            Step::Marmot => {
                let g = self.flat_ground();
                self.seg(80.0, 0.0, g);
                self.c.spawns.push(Spawn { x: self.x, kind: AnimalKind::Marmot });
                self.seg(80.0, 0.0, g);
            }
            Step::Switch(len, slope, g) => {
                let x0 = self.x;
                self.seg(len, slope, g);
                self.c.zones.push(Zone { x0, x1: self.x, kind: ZoneKind::Switchback });
            }
            Step::FogOn => self.fog_from = Some(self.x),
            Step::FogOff => {
                if let Some(x0) = self.fog_from.take() {
                    self.c.zones.push(Zone { x0, x1: self.x, kind: ZoneKind::Fog });
                }
            }
            Step::Sign(k) => self.c.props.push(Prop { x: self.x + 20.0, kind: PropKind::Sign(k), var: 0 }),
            Step::Summit => {
                self.c.props.push(Prop { x: self.x, kind: PropKind::SummitCross, var: 0 });
            }
        }
    }

    fn flat_ground(&self) -> Ground {
        match self.c.g.last() {
            Some(Ground::Scree) => Ground::Rock,
            Some(Ground::Water) | Some(Ground::Mud) => {
                if self.c.stage == 1 {
                    Ground::Grass
                } else {
                    Ground::Dirt
                }
            }
            Some(&g) => g,
            None => Ground::Dirt,
        }
    }

    fn finish(mut self) -> Course {
        // smooth the corners of the height profile
        let n = self.raw.len();
        let r = 6i32;
        let mut h = vec![0.0; n];
        for (i, out) in h.iter_mut().enumerate() {
            let mut s = 0.0;
            let mut k = 0.0;
            for d in -r..=r {
                let j = (i as i32 + d).clamp(0, n as i32 - 1) as usize;
                s += self.raw[j];
                k += 1.0;
            }
            *out = s / k;
        }
        // streams: a flat dip with stepping stones at trail level
        for &(x0, w) in &self.streams {
            let i0 = (x0 / CELL) as usize;
            let i1 = ((x0 + w) / CELL) as usize;
            let base = h[i0.saturating_sub(1)];
            let stones = ((w / 26.0).floor() as usize).max(1);
            for (k, hk) in h.iter_mut().enumerate().take(i1).skip(i0) {
                *hk = base - 4.0;
                self.c.g[k] = Ground::Water;
            }
            for s in 0..stones {
                let sx = x0 + w * (s as f32 + 1.0) / (stones as f32 + 1.0);
                let si = (sx / CELL) as usize;
                let (a, b) = (si.saturating_sub(1), (si + 1).min(i1 - 1));
                for (k, hk) in h.iter_mut().enumerate().take(b + 1).skip(a) {
                    *hk = base - 1.0;
                    self.c.g[k] = Ground::Rock;
                }
            }
        }
        self.c.min_h = h.iter().cloned().fold(f32::MAX, f32::min);
        self.c.max_h = h.iter().cloned().fold(f32::MIN, f32::max);
        self.c.h = h;
        self.c.end = self.x;
        self.decorate();
        self.c.obs.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        self.c.props.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        self.c
    }

    /// Trees, bushes, flowers and markers behind the trail.
    fn decorate(&mut self) {
        let stage = self.c.stage;
        let mut rng = Rng(0xC0FFEE ^ (stage as u64 * 977 + 5));
        self.c.props.push(Prop { x: START_X, kind: PropKind::StartArch, var: 0 });
        self.c.props.push(Prop { x: self.c.finish, kind: PropKind::FinishArch, var: 0 });
        let mut x = 20.0;
        while x < self.c.end - 10.0 {
            let g = self.c.ground(x);
            let near_arch = (x - START_X).abs() < 40.0 || (x - self.c.finish).abs() < 40.0;
            let fog = self.c.zones.iter().any(|z| z.kind == ZoneKind::Fog && x > z.x0 && x < z.x1);
            let village = x > self.c.finish - 500.0 || x < START_X - 40.0;
            let r = rng.f();
            let kind = if near_arch || g == Ground::Water {
                None
            } else if village && stage > 0 {
                Some(if r < 0.25 { PropKind::House } else if r < 0.35 && x > self.c.finish { PropKind::Church } else if r < 0.6 { PropKind::Fence } else if r < 0.8 { PropKind::Flowers } else { PropKind::Bush })
            } else {
                match stage {
                    0 => Some(if r < 0.3 { PropKind::Leafy } else if r < 0.5 { PropKind::Pine } else if r < 0.62 { PropKind::Birch } else if r < 0.78 { PropKind::Fern } else if r < 0.9 { PropKind::Bush } else { PropKind::Boulder }),
                    1 => match g {
                        Ground::Rock | Ground::Scree => Some(if r < 0.4 { PropKind::Boulder } else if r < 0.6 { PropKind::Fir } else { PropKind::Flowers }),
                        _ => Some(if r < 0.22 { PropKind::Fir } else if r < 0.55 { PropKind::Flowers } else if r < 0.68 { PropKind::Fence } else if r < 0.8 { PropKind::Boulder } else if r < 0.9 { PropKind::Bush } else { PropKind::Hut }),
                    },
                    _ => match g {
                        Ground::Snow => Some(if r < 0.25 { PropKind::SnowPole } else if r < 0.45 { PropKind::Boulder } else { PropKind::Cairn }),
                        Ground::Rock | Ground::Scree => Some(if fog { PropKind::Cairn } else if r < 0.5 { PropKind::Boulder } else if r < 0.7 { PropKind::Cairn } else { PropKind::Fir }),
                        _ => Some(if r < 0.45 { PropKind::Fir } else if r < 0.6 { PropKind::Pine } else if r < 0.8 { PropKind::Flowers } else { PropKind::Bush }),
                    },
                }
            };
            if let Some(k) = kind {
                // huts and houses are rare; don't stack them
                let big = matches!(k, PropKind::Hut | PropKind::House | PropKind::Church);
                if !big || rng.f() < 0.5 {
                    self.c.props.push(Prop { x, kind: k, var: rng.int(256) as u8 });
                }
            }
            x += match stage {
                0 => rng.range(18.0, 46.0),
                1 => rng.range(30.0, 70.0),
                _ => rng.range(40.0, 90.0),
            };
        }
        // ribbons and km posts along the trail
        let mut x = START_X + 300.0;
        while x < self.c.finish - 200.0 {
            self.c.props.push(Prop { x, kind: PropKind::Ribbon, var: 0 });
            x += 460.0;
        }
        let mut k = 1;
        while START_X + 2000.0 * (k as f32) < self.c.finish - 300.0 {
            self.c.props.push(Prop { x: START_X + 2000.0 * k as f32 + 15.0, kind: PropKind::KmPost(k as u8 * 2), var: 0 });
            k += 1;
        }
    }
}

// ---------------------------------------------------------------- stages

use Ground::*;
use ObKind as O;

fn run_steps(b: &mut Builder, steps: &[Step]) {
    for &s in steps {
        b.step(s);
    }
}

fn start(b: &mut Builder, g: Ground) {
    b.seg(START_X + 140.0, 0.0, g);
}

fn end(b: &mut Builder, g: Ground) {
    b.seg(260.0, 0.0, g);
    b.c.finish = b.x;
    b.seg(520.0, 0.0, g);
}

fn forest(b: &mut Builder) {
    start(b, Dirt);
    let low: &[ObKind] = &[O::Root, O::Rock];
    let mix: &[ObKind] = &[O::Root, O::Rock, O::Mushroom, O::Branch];
    let wood: &[ObKind] = &[O::Log, O::Branch, O::Root];
    run_steps(
        b,
        &[
            Step::Sign(0),
            Step::Seg(520.0, 0.0, Dirt, low, 170.0),
            Step::Crowd(4),
            Step::Seg(420.0, 0.1, Dirt, &[O::Root, O::Branch], 160.0),
            Step::Seg(420.0, -0.12, Dirt, &[O::Root, O::Mushroom, O::Rock], 150.0),
            Step::Seg(360.0, 0.0, Grass, wood, 160.0),
            Step::Seg(420.0, 0.0, Mud, &[O::Root], 200.0),
            Step::Deer,
            Step::Seg(560.0, 0.08, Dirt, mix, 140.0),
            Step::Sign(6),
            Step::Seg(260.0, 0.0, Dirt, &[], 0.0),
            Step::Aid(Item::TrailMix),
            Step::Crowd(5),
            Step::Seg(440.0, -0.1, Dirt, &[O::Log, O::Trunk], 170.0),
            Step::Seg(300.0, 0.0, Grass, &[O::Mushroom], 120.0),
            Step::Switch(420.0, 0.28, Rock),
            Step::Seg(240.0, 0.3, Rock, &[O::Rock], 200.0),
            Step::Crowd(3),
            Step::Seg(420.0, -0.2, Dirt, &[O::Root, O::Rock], 150.0),
            Step::Stream(44.0),
            Step::Seg(520.0, 0.0, Grass, &[O::Mushroom, O::Log, O::Branch], 140.0),
            Step::Seg(420.0, 0.14, Dirt, &[O::Root, O::Trunk], 150.0),
            Step::Deer,
            Step::Seg(520.0, -0.15, Dirt, &[O::Root, O::Rock, O::Log], 140.0),
            Step::Seg(400.0, 0.0, Mud, &[O::Root], 180.0),
            Step::Aid(Item::Banana),
            Step::Crowd(4),
            Step::Seg(600.0, 0.05, Dirt, mix, 125.0),
            Step::Seg(380.0, 0.18, Dirt, wood, 150.0),
            Step::Seg(460.0, -0.12, Grass, &[O::Log, O::Mushroom], 150.0),
            Step::Stream(54.0),
            Step::Seg(520.0, 0.0, Dirt, mix, 120.0),
            Step::Seg(400.0, -0.08, Grass, &[O::Log], 170.0),
            Step::Crowd(6),
        ],
    );
    end(b, Road);
}

fn meadow(b: &mut Builder) {
    start(b, Road);
    let rocks: &[ObKind] = &[O::Rock, O::Root, O::Marmot];
    run_steps(
        b,
        &[
            Step::Sign(7),
            Step::Seg(360.0, 0.05, Grass, &[O::Rock], 170.0),
            Step::Crowd(4),
            Step::Seg(480.0, 0.16, Grass, rocks, 160.0),
            Step::Cow,
            Step::Seg(420.0, 0.2, Grass, &[O::Rock, O::Marmot], 150.0),
            Step::Sign(1),
            Step::Switch(560.0, 0.32, Rock),
            Step::Seg(300.0, 0.05, Grass, &[], 0.0),
            Step::Marmot,
            Step::Stream(48.0),
            Step::Seg(400.0, 0.12, Grass, rocks, 140.0),
            Step::Aid(Item::Stroopwafel),
            Step::Crowd(5),
            Step::Sign(3),
            Step::Cow,
            Step::Seg(520.0, -0.18, Grass, &[O::Rock, O::Marmot, O::Root], 140.0),
            Step::Seg(340.0, -0.24, Rock, &[O::Rock], 150.0),
            Step::Stream(60.0),
            Step::Seg(300.0, 0.0, Grass, &[O::Marmot], 140.0),
            Step::Marmot,
            Step::Seg(420.0, 0.22, Grass, rocks, 150.0),
            Step::Switch(520.0, 0.34, Rock),
            Step::Seg(260.0, 0.1, Grass, &[], 0.0),
            Step::Cow,
            Step::Crowd(4),
            Step::Sign(6),
            Step::Seg(360.0, 0.08, Grass, &[O::Rock, O::Root], 150.0),
            Step::Aid(Item::Pickle),
            Step::Sign(2),
            Step::Marmot,
            Step::Seg(500.0, -0.14, Grass, rocks, 135.0),
            Step::Stream(44.0),
            Step::Seg(460.0, -0.2, Dirt, &[O::Root, O::Rock, O::Branch], 145.0),
            Step::Cow,
            Step::Seg(420.0, 0.0, Grass, &[O::Marmot, O::Rock], 140.0),
            Step::Crowd(6),
        ],
    );
    end(b, Road);
}

fn summit(b: &mut Builder) {
    start(b, Dirt);
    run_steps(
        b,
        &[
            Step::Sign(8),
            Step::Seg(360.0, 0.1, Dirt, &[O::Root, O::Rock], 160.0),
            Step::Crowd(4),
            Step::Seg(460.0, 0.26, Rock, &[O::Rock], 170.0),
            Step::Switch(520.0, 0.38, Scree),
            Step::Seg(300.0, 0.12, Rock, &[O::Rock], 150.0),
            Step::Aid(Item::Banana),
            Step::Crowd(3),
            Step::Seg(400.0, 0.3, Scree, &[O::Rock], 160.0),
            Step::Sign(5),
            Step::FogOn,
            Step::Seg(420.0, 0.2, Snow, &[O::SnowRock], 170.0),
            Step::Switch(480.0, 0.4, Snow),
            Step::Seg(360.0, 0.24, Rock, &[O::Rock], 150.0),
            Step::Seg(300.0, 0.32, Snow, &[O::SnowRock], 160.0),
            Step::FogOff,
            Step::Seg(160.0, 0.0, Rock, &[], 0.0),
            Step::Summit,
            Step::Crowd(5),
            Step::Seg(120.0, 0.0, Rock, &[], 0.0),
            Step::Sign(11),
            Step::Seg(640.0, -0.3, Snow, &[O::SnowRock], 210.0),
            Step::Seg(520.0, -0.36, Snow, &[O::SnowRock], 200.0),
            Step::Seg(200.0, -0.1, Rock, &[], 0.0),
            Step::Aid(Item::Pickle),
            Step::Seg(460.0, -0.3, Scree, &[O::Rock], 150.0),
            Step::Seg(420.0, -0.22, Rock, &[O::Rock, O::Root], 140.0),
            Step::Crowd(4),
            Step::Seg(500.0, -0.16, Dirt, &[O::Root, O::Branch, O::Log], 135.0),
            Step::Stream(50.0),
            Step::Seg(440.0, -0.1, Grass, &[O::Root, O::Trunk], 140.0),
            Step::Aid(Item::Stroopwafel),
            Step::Seg(420.0, 0.06, Dirt, &[O::Root, O::Rock, O::Branch], 130.0),
            Step::Sign(9),
            Step::Seg(380.0, -0.06, Grass, &[O::Log], 160.0),
            Step::Crowd(6),
            Step::Seg(300.0, 0.0, Road, &[], 0.0),
            Step::Crowd(8),
        ],
    );
    end(b, Road);
}
