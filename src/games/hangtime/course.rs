//! The three venues: terrain height map, lift (thermals, ridge lift, sink),
//! gates under balloons, the launch ramp and the landing field.
//!
//! World coordinates: x to the right (the course runs left to right, the
//! wind blows to the right), y is altitude in pixels (up is positive).

use crate::engine::rng::Rng;

/// Width of every course in world pixels.
pub const LEN: i32 = 5200;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Venue {
    Dunes,
    Alps,
    Canyon,
}

pub const VENUES: [Venue; 3] = [Venue::Dunes, Venue::Alps, Venue::Canyon];

impl Venue {
    pub fn name(self) -> &'static str {
        match self {
            Venue::Dunes => "HELICO DUNES",
            Venue::Alps => "ALPINE VALLEY",
            Venue::Canyon => "THUNDER CANYON",
        }
    }
    pub fn tagline(self) -> &'static str {
        match self {
            Venue::Dunes => "THE SEA BREEZE IS FREE. THE FRIES ARE NOT.",
            Venue::Alps => "MIND THE COWS. THE COWS DO NOT MIND YOU.",
            Venue::Canyon => "THAT CLOUD LOOKS HUNGRY.",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Surf {
    Sand,
    Grass,
    Rock,
    Field,
    Tulips,
    Snow,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Marker {
    /// circling storks
    Storks,
    /// rising leaves or dune fluff
    Leaves,
    /// a dust devil
    Dust,
}

#[derive(Clone, Copy, Debug)]
pub struct Thermal {
    pub x: f32,
    /// half width of the rising core
    pub w: f32,
    /// lift at the core (px/frame)
    pub s: f32,
    /// cloud base: lift fades out just below it
    pub top: f32,
    pub phase: f32,
    pub marker: Marker,
}

#[derive(Clone, Copy, Debug)]
pub struct Gate {
    pub x: f32,
    pub lo: f32,
    pub hi: f32,
}

impl Gate {
    pub fn mid(&self) -> f32 {
        (self.lo + self.hi) / 2.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Launch {
    /// top of the ramp (start of the run)
    pub x0: f32,
    /// the edge
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
}

impl Launch {
    pub fn y_at(&self, x: f32) -> f32 {
        let t = ((x - self.x0) / (self.x1 - self.x0)).clamp(0.0, 1.0);
        self.y0 + (self.y1 - self.y0) * t
    }
    /// The zone where B launches cleanly.
    pub fn zone(&self) -> (f32, f32) {
        (self.x1 - 22.0, self.x1)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Storm {
    pub x: f32,
    pub base: f32,
    pub top: f32,
    /// half width of the suction column
    pub w: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PropKind {
    Lighthouse,
    Windmill,
    BeachHut,
    Pine,
    RoundTree,
    Poplar,
    Cactus,
    Chapel,
    House,
    Greenhouse,
    Cow,
    KiteSurfer,
    Windsock,
    Crowd,
    Rock,
    Mesa,
    Sign,
    Boat,
    Cabin,
}

#[derive(Clone, Copy, Debug)]
pub struct Prop {
    pub kind: PropKind,
    pub x: f32,
    pub y: f32,
    pub var: u32,
}

pub struct Course {
    pub venue: Venue,
    pub ground: Vec<f32>,
    pub surf: Vec<Surf>,
    /// lakes and sea: (x0, x1, water level)
    pub water: Vec<(f32, f32, f32)>,
    pub thermals: Vec<Thermal>,
    /// sink zones: (x0, x1, strength)
    pub sinks: Vec<(f32, f32, f32)>,
    pub wind: f32,
    pub gates: Vec<Gate>,
    pub launch: Launch,
    /// landing field: (x0, x1) and the target cross
    pub field: (f32, f32),
    pub target: f32,
    pub storm: Option<Storm>,
    pub props: Vec<Prop>,
    /// precomputed ridge lift per x: (factor, reference height)
    ridge_up: Vec<(f32, f32)>,
    /// precomputed lee sink per x
    lee: Vec<f32>,
    ridge_k: f32,
}

fn smooth(a: f32, b: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let s = t * t * (3.0 - 2.0 * t);
    a + (b - a) * s
}

struct Spec {
    pts: &'static [(i32, i32)],
    surf: &'static [(i32, Surf)],
    water: &'static [(i32, i32, i32)],
    thermals: &'static [(i32, i32, f32, i32, Marker)],
    sinks: &'static [(i32, i32, f32)],
    gates: &'static [(i32, i32, i32)],
    launch: (i32, i32, i32, i32),
    field: (i32, i32, i32),
    wind: f32,
    ridge: f32,
    bumps: f32,
}

const DUNES: Spec = Spec {
    pts: &[
        (0, -14),
        (150, -6),
        (196, 10),
        (260, 16),
        (300, 30),
        (430, 200),
        (452, 206),
        (470, 196),
        (500, 160),
        (540, 110),
        (600, 72),
        (700, 46),
        (800, 40),
        (1000, 152),
        (1030, 150),
        (1090, 60),
        (1200, 18),
        (1500, 14),
        (1900, 16),
        (2200, 20),
        (2440, 118),
        (2470, 116),
        (2560, 30),
        (2700, 16),
        (3300, 14),
        (3600, 18),
        (4000, 14),
        (4560, 14),
        (4960, 14),
        (5060, 42),
        (5200, 44),
    ],
    surf: &[
        (0, Surf::Sand),
        (1200, Surf::Grass),
        (1500, Surf::Tulips),
        (1660, Surf::Grass),
        (2200, Surf::Sand),
        (2600, Surf::Grass),
        (3200, Surf::Tulips),
        (3380, Surf::Grass),
        (3800, Surf::Tulips),
        (3920, Surf::Grass),
        (4560, Surf::Field),
        (4960, Surf::Grass),
    ],
    water: &[(-200, 176, 6), (2860, 2940, 12), (3460, 3520, 10)],
    thermals: &[
        (1360, 40, 0.34, 330, Marker::Storks),
        (1760, 36, 0.30, 300, Marker::Leaves),
        (2120, 44, 0.36, 340, Marker::Storks),
        (2640, 40, 0.34, 330, Marker::Leaves),
        (3120, 44, 0.36, 340, Marker::Storks),
        (3560, 38, 0.32, 320, Marker::Leaves),
        (4020, 42, 0.34, 320, Marker::Storks),
        (4380, 36, 0.30, 300, Marker::Leaves),
    ],
    sinks: &[],
    gates: &[(1160, 150, 246), (2000, 146, 240), (2900, 156, 250), (3880, 136, 230)],
    launch: (452, 512, 212, 204),
    field: (4560, 4960, 4760),
    wind: 0.10,
    ridge: 0.55,
    bumps: 2.0,
};

const ALPS: Spec = Spec {
    pts: &[
        (0, 560),
        (180, 470),
        (280, 424),
        (360, 420),
        (380, 404),
        (420, 330),
        (480, 270),
        (560, 220),
        (650, 184),
        (800, 140),
        (1000, 104),
        (1150, 110),
        (1300, 168),
        (1360, 176),
        (1460, 140),
        (1620, 90),
        (1720, 56),
        (2380, 56),
        (2480, 90),
        (2800, 110),
        (3000, 160),
        (3300, 262),
        (3360, 270),
        (3480, 170),
        (3700, 104),
        (4000, 92),
        (4380, 88),
        (4780, 88),
        (4900, 110),
        (5060, 230),
        (5200, 420),
    ],
    surf: &[(0, Surf::Snow), (240, Surf::Grass), (440, Surf::Rock), (700, Surf::Grass), (4380, Surf::Field), (4780, Surf::Grass), (5000, Surf::Rock)],
    water: &[(1740, 2360, 70)],
    thermals: &[
        (860, 42, 0.42, 480, Marker::Storks),
        (1320, 46, 0.46, 500, Marker::Leaves),
        (1600, 40, 0.38, 460, Marker::Storks),
        (2620, 44, 0.44, 500, Marker::Leaves),
        (2920, 40, 0.42, 500, Marker::Storks),
        (3240, 46, 0.48, 520, Marker::Leaves),
        (3800, 42, 0.44, 480, Marker::Storks),
        (4200, 40, 0.40, 460, Marker::Leaves),
    ],
    sinks: &[(1740, 2360, 0.12)],
    gates: &[(1040, 300, 396), (2060, 284, 380), (3020, 330, 426), (3960, 280, 376)],
    launch: (300, 372, 426, 418),
    field: (4380, 4780, 4580),
    wind: 0.06,
    ridge: 0.38,
    bumps: 3.0,
};

const CANYON: Spec = Spec {
    pts: &[
        (0, 344),
        (200, 336),
        (320, 334),
        (334, 326),
        (360, 150),
        (390, 40),
        (410, 22),
        (660, 22),
        (700, 40),
        (760, 250),
        (790, 262),
        (1000, 266),
        (1030, 250),
        (1070, 60),
        (1100, 50),
        (1480, 54),
        (1520, 70),
        (1580, 296),
        (1610, 306),
        (1880, 304),
        (1910, 290),
        (1960, 80),
        (2000, 70),
        (2880, 76),
        (2930, 90),
        (2990, 276),
        (3020, 284),
        (3280, 282),
        (3310, 268),
        (3360, 80),
        (3400, 64),
        (4420, 60),
        (4820, 60),
        (4980, 74),
        (5040, 240),
        (5200, 256),
    ],
    surf: &[(0, Surf::Rock), (4420, Surf::Field), (4820, Surf::Rock)],
    water: &[(430, 640, 30)],
    thermals: &[
        (900, 40, 0.55, 520, Marker::Dust),
        (1720, 44, 0.60, 560, Marker::Storks),
        (2160, 38, 0.58, 540, Marker::Dust),
        (2440, 40, 0.56, 540, Marker::Storks),
        (3140, 40, 0.58, 560, Marker::Dust),
        (3620, 36, 0.50, 520, Marker::Storks),
        (4080, 44, 0.58, 540, Marker::Dust),
    ],
    sinks: &[(1100, 1460, 0.20), (3420, 3540, 0.18), (3720, 3860, 0.18)],
    gates: &[(1260, 300, 404), (2240, 340, 444), (3160, 360, 464), (4000, 300, 404)],
    launch: (250, 330, 344, 336),
    field: (4420, 4820, 4620),
    wind: 0.08,
    ridge: 0.50,
    bumps: 2.5,
};

impl Course {
    pub fn new(venue: Venue, seed: u64) -> Self {
        let spec = match venue {
            Venue::Dunes => &DUNES,
            Venue::Alps => &ALPS,
            Venue::Canyon => &CANYON,
        };
        let mut rng = Rng(seed | 1);
        let n = (LEN + 1) as usize;
        let mut ground = vec![0.0f32; n];
        let (p1, p2) = (rng.range(0.0, 6.28), rng.range(0.0, 6.28));
        let launch = Launch {
            x0: spec.launch.0 as f32,
            x1: spec.launch.1 as f32,
            y0: spec.launch.2 as f32,
            y1: spec.launch.3 as f32,
        };
        let field = (spec.field.0 as f32, spec.field.1 as f32);
        let mut k = 0;
        for (x, g) in ground.iter_mut().enumerate() {
            let xf = x as f32;
            while k + 2 < spec.pts.len() && spec.pts[k + 1].0 as f32 <= xf {
                k += 1;
            }
            let (a, b) = (spec.pts[k], spec.pts[k + 1]);
            let t = (xf - a.0 as f32) / (b.0 - a.0) as f32;
            let mut h = smooth(a.1 as f32, b.1 as f32, t);
            // small bumps, but flat on the field, the ramp hilltop and under water
            let flat = (xf >= field.0 - 20.0 && xf <= field.1 + 20.0) || (xf >= launch.x0 - 90.0 && xf <= launch.x0 + 10.0);
            if !flat {
                h += spec.bumps * ((xf * 0.031 + p1).sin() * 0.6 + (xf * 0.083 + p2).sin() * 0.4);
            }
            *g = h;
        }
        let mut surf = vec![Surf::Grass; n];
        for (i, &(x0, s)) in spec.surf.iter().enumerate() {
            let x1 = spec.surf.get(i + 1).map(|v| v.0).unwrap_or(LEN + 1);
            for x in x0.max(0)..x1.min(LEN + 1) {
                surf[x as usize] = s;
            }
        }
        let thermals = spec
            .thermals
            .iter()
            .map(|&(x, w, s, top, marker)| Thermal { x: x as f32, w: w as f32, s, top: top as f32, phase: rng.range(0.0, 6.28), marker })
            .collect();
        let gates = spec.gates.iter().map(|&(x, lo, hi)| Gate { x: x as f32, lo: lo as f32, hi: hi as f32 }).collect();
        let storm = (venue == Venue::Canyon).then_some(Storm { x: 2700.0, base: 470.0, top: 610.0, w: 60.0 });
        let mut c = Course {
            venue,
            ground,
            surf,
            water: spec.water.iter().map(|&(a, b, l)| (a as f32, b as f32, l as f32)).collect(),
            thermals,
            sinks: spec.sinks.iter().map(|&(a, b, s)| (a as f32, b as f32, s)).collect(),
            wind: spec.wind,
            gates,
            launch,
            field,
            target: spec.field.2 as f32,
            storm,
            props: Vec::new(),
            ridge_up: Vec::new(),
            lee: Vec::new(),
            ridge_k: spec.ridge,
        };
        c.precompute_ridge();
        c.place_props(&mut rng);
        c
    }

    fn precompute_ridge(&mut self) {
        let n = self.ground.len();
        let g = |x: i32| self.ground[x.clamp(0, n as i32 - 1) as usize];
        let mut up = vec![(0.0, 0.0); n];
        let mut lee = vec![0.0; n];
        for x in 0..n as i32 {
            let here = g(x);
            // windward: the ground rises ahead (wind blows to +x)
            let mut crest = here;
            for d in (0..=70).step_by(5) {
                crest = crest.max(g(x + d));
            }
            let rise = crest - here;
            let local = (g(x + 6) - g(x - 6)) / 12.0;
            let f = if rise > 4.0 || local > 0.08 { ((rise / 60.0).max(local * 1.2)).min(1.4) } else { 0.0 };
            up[x as usize] = (f, crest);
            // lee: the ground fell away just upwind
            let mut back = here;
            for d in (10..=70).step_by(5) {
                back = back.max(g(x - d));
            }
            let drop = back - here;
            lee[x as usize] = if drop > 20.0 { (drop / 80.0).min(1.0) } else { 0.0 };
        }
        // smooth so the edges of the lift band are soft
        let mut sm = up.clone();
        for x in 0..n {
            let (mut s, mut c) = (0.0, 0.0);
            for d in -8i32..=8 {
                let i = (x as i32 + d).clamp(0, n as i32 - 1) as usize;
                s += up[i].0;
                c += 1.0;
            }
            sm[x].0 = s / c;
        }
        self.ridge_up = sm;
        self.lee = lee;
    }

    fn place_props(&mut self, rng: &mut Rng) {
        let add = |c: &mut Course, kind: PropKind, x: f32, var: u32| {
            let y = c.ground_at(x);
            c.props.push(Prop { kind, x, y, var });
        };
        let l = self.launch;
        add(self, PropKind::Windsock, l.x0 - 70.0, 0);
        add(self, PropKind::Sign, l.x0 - 30.0, 0);
        let (f0, f1) = self.field;
        add(self, PropKind::Windsock, f0 + 30.0, 1);
        add(self, PropKind::Crowd, f1 - 70.0, 0);
        match self.venue {
            Venue::Dunes => {
                for (x, v) in [(40.0, 0), (110.0, 1), (-40.0, 2)] {
                    self.props.push(Prop { kind: PropKind::KiteSurfer, x, y: 6.0, var: v });
                }
                for x in [214.0, 236.0, 258.0] {
                    add(self, PropKind::BeachHut, x, (x as u32 / 22) % 3);
                }
                add(self, PropKind::Lighthouse, 1018.0, 0);
                for x in [1300.0, 1460.0, 1980.0, 2300.0, 3060.0, 3640.0, 4140.0] {
                    add(self, PropKind::Greenhouse, x, rng.next() % 3);
                }
                for x in [1700.0, 2800.0, 3330.0, 4430.0] {
                    add(self, PropKind::House, x, rng.next() % 4);
                }
                add(self, PropKind::Windmill, 4500.0, 0);
                add(self, PropKind::Windmill, 2760.0, 1);
                let mut x = 1220.0;
                while x < 4540.0 {
                    if rng.f() < 0.5 && self.water_at(x).is_none() {
                        add(self, PropKind::Poplar, x, rng.next() % 4);
                    }
                    x += 60.0 + rng.range(0.0, 80.0);
                }
                for x in [5000.0, 5040.0, 5090.0, 5140.0] {
                    add(self, PropKind::RoundTree, x, rng.next() % 4);
                }
                for x in [1840.0, 2380.0, 3420.0] {
                    add(self, PropKind::Cow, x, rng.next() % 4);
                }
            }
            Venue::Alps => {
                add(self, PropKind::Cabin, 230.0, 0);
                add(self, PropKind::Chapel, 1352.0, 0);
                add(self, PropKind::Boat, 2050.0, 0);
                let mut x = 480.0;
                while x < 5200.0 {
                    let inside = (1700.0..2400.0).contains(&x) || (4340.0..4820.0).contains(&x) || (1300.0..1400.0).contains(&x);
                    if !inside && rng.f() < 0.75 {
                        add(self, PropKind::Pine, x, rng.next() % 4);
                    }
                    x += 26.0 + rng.range(0.0, 50.0);
                }
                for x in [1120.0, 1500.0, 2560.0, 2700.0, 3780.0, 4140.0] {
                    add(self, PropKind::Cow, x, rng.next() % 4);
                }
                for x in [4300.0, 4330.0, 4840.0, 4880.0] {
                    add(self, PropKind::House, x, rng.next() % 4);
                }
            }
            Venue::Canyon => {
                let mut x = 120.0;
                while x < 5100.0 {
                    let wet = self.water_at(x).is_some();
                    let field = (4400.0..4840.0).contains(&x);
                    if !wet && !field && rng.f() < 0.45 {
                        let k = if rng.f() < 0.6 { PropKind::Cactus } else { PropKind::Rock };
                        add(self, k, x, rng.next() % 4);
                    }
                    x += 50.0 + rng.range(0.0, 70.0);
                }
                for x in [4440.0, 4800.0] {
                    add(self, PropKind::Cactus, x, 3);
                }
            }
        }
        // the ramp area stays clear
        let (a, b) = (l.x0 - 100.0, l.x1 + 40.0);
        self.props.retain(|p| !(p.x > a && p.x < b && !matches!(p.kind, PropKind::Windsock | PropKind::Sign)));
    }

    pub fn ground_at(&self, x: f32) -> f32 {
        let n = self.ground.len() as i32;
        let xi = x.floor() as i32;
        let t = x - xi as f32;
        let a = self.ground[xi.clamp(0, n - 1) as usize];
        let b = self.ground[(xi + 1).clamp(0, n - 1) as usize];
        a + (b - a) * t
    }

    pub fn surf_at(&self, x: f32) -> Surf {
        self.surf[(x as i32).clamp(0, LEN) as usize]
    }

    /// Water level if x is over a lake or the sea.
    pub fn water_at(&self, x: f32) -> Option<f32> {
        self.water.iter().find(|w| x >= w.0 && x < w.1).map(|w| w.2)
    }

    /// The surface a pilot stands on: ground or water.
    pub fn floor_at(&self, x: f32) -> f32 {
        let g = self.ground_at(x);
        match self.water_at(x) {
            Some(l) => g.max(l),
            None => g,
        }
    }

    pub fn in_field(&self, x: f32) -> bool {
        x >= self.field.0 && x <= self.field.1
    }

    /// Thermal lift only (for markers and the AI's eyes).
    pub fn thermal_lift(&self, x: f32, y: f32, t: u32) -> f32 {
        let agl = y - self.ground_at(x);
        let mut l = 0.0;
        for th in &self.thermals {
            let dx = (x - th.x).abs();
            if dx > th.w * 1.8 {
                continue;
            }
            let vert = ((th.top - y) / 50.0).clamp(0.0, 1.0) * (agl / 30.0).clamp(0.35, 1.0);
            if vert <= 0.0 {
                continue;
            }
            let breathe = 0.88 + 0.12 * (t as f32 * 0.0045 + th.phase).sin();
            let u = dx / th.w;
            let core = if u < 1.0 { 1.0 - u * u } else { -0.22 * (1.0 - (u - 1.0) / 0.8) };
            l += th.s * core * vert * breathe;
        }
        l
    }

    /// Ridge lift (positive) or lee sink (negative).
    pub fn ridge_lift(&self, x: f32, y: f32) -> f32 {
        let i = (x as i32).clamp(0, LEN) as usize;
        let (f, crest) = self.ridge_up[i];
        let g = self.ground[i];
        let mut l = 0.0;
        if f > 0.0 {
            let above = y - crest;
            let fall = if above < 0.0 { 1.0 } else { (1.0 - above / 90.0).max(0.0) };
            l += self.ridge_k * f * fall * (self.wind / 0.1);
        }
        let lee = self.lee[i];
        if lee > 0.0 {
            let agl = y - g;
            l -= self.ridge_k * 0.45 * lee * (1.0 - agl / 90.0).max(0.0);
        }
        l
    }

    /// Steady sink over lakes and canyon floors.
    pub fn sink_at(&self, x: f32) -> f32 {
        let mut s = 0.0;
        for &(a, b, k) in &self.sinks {
            if x > a - 30.0 && x < b + 30.0 {
                let edge = ((x - (a - 30.0)) / 60.0).min(((b + 30.0) - x) / 60.0).clamp(0.0, 1.0);
                s += k * edge;
            }
        }
        s
    }

    /// The storm cloud: strong suction under it and a downdraft around it.
    pub fn storm_lift(&self, x: f32, y: f32) -> f32 {
        let Some(st) = self.storm else { return 0.0 };
        let dx = (x - st.x).abs();
        if y > st.top {
            return 0.0;
        }
        if dx < st.w {
            let near = ((y - (st.base - 220.0)) / 220.0).clamp(0.0, 1.0);
            0.35 + 0.75 * near * near
        } else if dx < st.w + 110.0 && y < st.base + 20.0 {
            -0.42 * (1.0 - (dx - st.w) / 110.0)
        } else {
            0.0
        }
    }

    /// All air movement at a point (vertical, px/frame).
    pub fn lift(&self, x: f32, y: f32, t: u32) -> f32 {
        self.thermal_lift(x, y, t) + self.ridge_lift(x, y) - self.sink_at(x) + self.storm_lift(x, y)
    }

    /// Index of the thermal whose core contains x, if any.
    pub fn thermal_near(&self, x: f32) -> Option<usize> {
        self.thermals.iter().position(|th| (x - th.x).abs() < th.w)
    }
}
