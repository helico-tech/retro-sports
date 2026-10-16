//! One race: up to four riders on a two-lane bike path, against the wind.
//! Pure simulation (no drawing, no I/O), stepped at 60 Hz.
//!
//! World: `x` along the route in pixels (finish at `length`), `y` the lane
//! position (0 = far lane, 1 = near lane, the canal beyond 1.6), `z` the
//! hop height. Speed `v` is in px/frame; km/h on the HUD is `v * 12`.

use super::data::*;
use crate::engine::gfx::rgb;
pub use crate::engine::input::Pad;
pub use crate::engine::rng::Rng;

pub const GRAV: f32 = 0.25;
pub const P_STROKE: f32 = 0.088;
pub const C_ROLL: f32 = 0.0011;
pub const C_AIR: f32 = 0.00095;
/// Hitting something solid faster than this sends you into the canal.
pub const CRASH_V: f32 = 1.3;
pub const CANAL_Y: f32 = 1.6;
pub const HEDGE_Y: f32 = -0.6;
pub const START_LINE: f32 = 60.0;
pub const MAX_V: f32 = 4.5;
pub const COUNTDOWN: i32 = 200;
pub const SHEEP_VY: f32 = 0.02;

/// Screen y of the ground at lane position `y` (full-screen layout).
pub fn ground_y(y: f32) -> f32 {
    158.0 + y * 20.0
}

pub fn kmh(v: f32) -> i32 {
    (v * 12.0).round() as i32
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Sfx {
    Bell,
    Tick,
    Gust,
    Splash,
    Moo,
    Honk,
    Crash,
    Hop,
    Rattle,
    Chomp,
    Whistle,
    TramBell,
    BridgeAlarm,
    Cheer,
    Blip,
    Confirm,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ctrl {
    Human(usize),
    Cpu,
}

/// What a rider does this frame: pedal presses are edges, the rest held.
#[derive(Clone, Copy, Default, Debug)]
pub struct Ctl {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub up_e: bool,
    pub down_e: bool,
    pub hop: bool,
    pub bell: bool,
    pub tuck: bool,
    /// both pedal keys held: brake (after a short moment)
    pub brake: bool,
}

pub fn ctl_from_pad(now: Pad, prev: Pad) -> Ctl {
    Ctl {
        left: now.left && !prev.left,
        right: now.right && !prev.right,
        up: now.up,
        down: now.down,
        up_e: now.up && !prev.up,
        down_e: now.down && !prev.down,
        hop: now.b && !prev.b,
        bell: now.a && !prev.a,
        tuck: now.c,
        brake: now.left && now.right,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RState {
    Ride,
    /// over the handlebars, on the way to the canal
    Fly,
    Swim,
    Climb,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stroke {
    None,
    Perfect,
    Good,
    Late,
    Wasted,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct RStats {
    pub perfect: u32,
    pub good: u32,
    pub wasted: u32,
    pub hops: u32,
    pub clean_hops: u32,
    pub crashes: u32,
    pub bumps: u32,
    pub eaten: u32,
    pub draft_frames: u32,
    pub tuck_frames: u32,
    pub head_frames: u32,
    pub tuck_head_frames: u32,
    pub bonks: u32,
    pub rings: u32,
    pub cargo_lost: u32,
    pub overtakes: u32,
    pub wait_frames: u32,
    pub bites: u32,
    /// collisions per obstacle kind (`OKind::idx`)
    pub hits: [u16; 12],
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Ai {
    next_gap: i32,
    lane_cd: i32,
    hop_cd: i32,
    rung: i32,
    lazy: i32,
}

#[derive(Clone, Debug)]
pub struct Rider {
    pub who: Who,
    pub bike: Bike,
    pub ctrl: Ctrl,
    pub skill: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vz: f32,
    pub v: f32,
    pub lane: i32,
    pub energy: f32,
    pub bonk: bool,
    pub last_side: i8,
    pub since: i32,
    pub clean: bool,
    /// pedal crank, in turns (8 animation frames per turn)
    pub crank: f32,
    crank_to: f32,
    pub spin: i32,
    pub last_stroke: Stroke,
    pub stroke_t: i32,
    pub tuck: bool,
    pub leaning: bool,
    pub state: RState,
    pub st: i32,
    pub crash_x: f32,
    pub crash_y: f32,
    pub cargo: i32,
    pub cargo_max: i32,
    pub finish: Option<u32>,
    pub dnf: bool,
    pub draft: f32,
    pub bell_t: i32,
    pub wobble: i32,
    pub drip: i32,
    pub hammer: i32,
    pub say: Option<(&'static str, i32)>,
    pub wheel: f32,
    pub waiting: bool,
    pub warn: i32,
    /// frames of grace after a bump (no chain bumps through a flock)
    pub bump_cd: i32,
    pub brake_t: i32,
    pub place: usize,
    pub stats: RStats,
    pub ai: Ai,
    tip_cd: i32,
}

impl Rider {
    pub fn riding(&self) -> bool {
        self.state == RState::Ride
    }
    pub fn crank_frame(&self) -> usize {
        ((self.crank * 8.0) as i32).rem_euclid(8) as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Food {
    Stroopwafel,
    Kroket,
    Haring,
    Kaas,
}

impl Food {
    pub fn energy(self) -> f32 {
        match self {
            Food::Stroopwafel => 22.0,
            Food::Kaas => 25.0,
            Food::Haring => 30.0,
            Food::Kroket => 45.0,
        }
    }
    pub fn shout(self) -> &'static str {
        match self {
            Food::Stroopwafel => "STROOPWAFEL!",
            Food::Kaas => "KAAS!",
            Food::Haring => "HARING!",
            Food::Kroket => "KROKET!",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OKind {
    Pothole,
    Bollard,
    Herring,
    Cat,
    Goose,
    Sheep,
    Cow,
    Tourist,
    Rails,
    Tractor,
    Sofa,
    Food(Food),
}

impl OKind {
    /// Height a hop must clear (99 = cannot be hopped).
    pub fn height(self) -> f32 {
        match self {
            OKind::Pothole | OKind::Rails => 0.5,
            OKind::Herring => 3.0,
            OKind::Cat => 7.0,
            OKind::Goose => 10.0,
            OKind::Sheep => 13.0,
            _ => 99.0,
        }
    }
    pub fn width(self) -> f32 {
        match self {
            OKind::Pothole => 10.0,
            OKind::Bollard => 6.0,
            OKind::Herring => 10.0,
            OKind::Cat => 10.0,
            OKind::Goose => 10.0,
            OKind::Sheep => 14.0,
            OKind::Cow => 24.0,
            OKind::Tourist => 8.0,
            OKind::Rails => 16.0,
            OKind::Tractor => 30.0,
            OKind::Sofa => 26.0,
            OKind::Food(_) => 10.0,
        }
    }
    /// Covers both lanes.
    pub fn wide(self) -> bool {
        self == OKind::Rails
    }
    pub fn idx(self) -> usize {
        match self {
            OKind::Pothole => 0,
            OKind::Bollard => 1,
            OKind::Herring => 2,
            OKind::Cat => 3,
            OKind::Goose => 4,
            OKind::Sheep => 5,
            OKind::Cow => 6,
            OKind::Tourist => 7,
            OKind::Rails => 8,
            OKind::Tractor => 9,
            OKind::Sofa => 10,
            OKind::Food(_) => 11,
        }
    }
    pub fn slow_rider(self) -> bool {
        matches!(self, OKind::Tractor | OKind::Sofa)
    }
}

#[derive(Clone, Debug)]
pub struct Obj {
    pub kind: OKind,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub t: i32,
    /// 0 idle, 1 alerted (fleeing, chasing, crossing), 2 done
    pub state: u8,
    pub hit: u8,
    pub target: usize,
    pub gone: bool,
    pub face: i8,
    pub var: u8,
}

impl Obj {
    fn new(kind: OKind, x: f32, y: f32, var: u8) -> Self {
        Obj { kind, x, y, z: 0.0, vx: 0.0, vy: 0.0, vz: 0.0, t: 0, state: 0, hit: 0, target: 0, gone: false, face: -1, var }
    }
    /// Can it hurt a rider right now?
    pub fn blocking(&self) -> bool {
        if self.gone {
            return false;
        }
        match self.kind {
            OKind::Tourist | OKind::Cat => self.state == 0,
            OKind::Goose => self.state != 2 || (HEDGE_Y..CANAL_Y).contains(&self.y),
            OKind::Food(_) => false,
            _ => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BState {
    Closed,
    Warn,
    Open,
    Closing,
    Done,
}

#[derive(Clone, Debug)]
pub struct Bridge {
    pub x: f32,
    pub state: BState,
    pub t: i32,
    /// 0 = deck down, 1 = fully raised
    pub lift: f32,
}

pub const BRIDGE_WARN: i32 = 150;
pub const BRIDGE_OPEN: i32 = 240;
pub const BRIDGE_CLOSE: i32 = 70;

impl Bridge {
    /// The barrier riders stop at.
    pub fn barrier(&self) -> f32 {
        self.x - 40.0
    }
    pub fn blocking(&self) -> bool {
        match self.state {
            BState::Warn => self.t > 100,
            BState::Open | BState::Closing => true,
            _ => false,
        }
    }
    pub fn lights(&self) -> bool {
        matches!(self.state, BState::Warn | BState::Open | BState::Closing)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FxKind {
    Drop,
    Dust,
    Feather,
    Wool,
    Crumb,
    Sparkle,
    Spark,
}

#[derive(Clone, Debug)]
pub struct Fx {
    /// world x
    pub x: f32,
    /// screen y in the full-screen layout
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: i32,
    pub max: i32,
    pub kind: FxKind,
    pub color: u32,
}

#[derive(Clone, Debug)]
pub struct Popup {
    pub x: f32,
    pub y: f32,
    pub text: &'static str,
    pub color: u32,
    pub life: i32,
}

#[derive(Clone, Debug)]
pub struct Banner {
    pub text: String,
    pub t: i32,
    pub dur: i32,
    pub color: u32,
    /// screen y of the text top in the view (None = middle)
    pub y: Option<i32>,
    pub scale: i32,
    pub pop: bool,
}

/// A cargo piece bobbing in the canal.
#[derive(Clone, Debug)]
pub struct Floater {
    pub x: f32,
    pub cargo: Cargo,
    pub t: i32,
    pub y: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Countdown,
    Race,
    Over,
}

pub struct Race {
    pub stage: usize,
    pub riders: Vec<Rider>,
    pub objs: Vec<Obj>,
    pub bridge: Option<Bridge>,
    pub floaters: Vec<Floater>,
    pub length: f32,
    pub phase: Phase,
    pub phase_t: i32,
    pub frame: u32,
    /// race clock in frames since the start signal
    pub clock: u32,
    pub rng: Rng,
    pub sfx: Vec<Sfx>,
    pub fx: Vec<Fx>,
    pub popups: Vec<Popup>,
    /// one banner per view (player 1, player 2)
    pub banners: [Option<Banner>; 2],
    pub shake: [i32; 2],
    pub gust: f32,
    gust_t: i32,
    gust_len: i32,
    pub gust_on: bool,
    /// which rider each view follows
    pub views: Vec<usize>,
    pub tram: Option<f32>,
    tram_t: i32,
    moo_cd: i32,
    alarm_cd: i32,
    rattle_cd: i32,
    pub first_cpu_finish: Option<u32>,
    pub swept: bool,
    pub section: Vec<usize>,
    hup: Vec<bool>,
    say_cd: i32,
}

pub fn stage(i: usize) -> &'static StageDef {
    &STAGES[i.min(STAGES.len() - 1)]
}

impl Race {
    /// `field`: (who, bike, control, skill) per rider, at most 4.
    pub fn new(stage_i: usize, field: &[(Who, Bike, Ctrl, f32)], seed: u64) -> Self {
        let st = stage(stage_i);
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let grid = [(20.0, 1), (20.0, 0), (46.0, 1), (46.0, 0)];
        let riders: Vec<Rider> = field
            .iter()
            .enumerate()
            .map(|(k, &(who, bike, ctrl, skill))| {
                let (x, lane) = grid[k % 4];
                let n = bike.def().cargo_n;
                Rider {
                    who,
                    bike,
                    ctrl,
                    skill,
                    x,
                    y: lane as f32,
                    z: 0.0,
                    vz: 0.0,
                    v: 0.0,
                    lane,
                    energy: 100.0,
                    bonk: false,
                    last_side: 0,
                    since: 99,
                    clean: false,
                    crank: rng.f(),
                    crank_to: 0.0,
                    spin: 0,
                    last_stroke: Stroke::None,
                    stroke_t: 99,
                    tuck: false,
                    leaning: false,
                    state: RState::Ride,
                    st: 0,
                    crash_x: 0.0,
                    crash_y: 0.0,
                    cargo: n,
                    cargo_max: n,
                    finish: None,
                    dnf: false,
                    draft: 0.0,
                    bell_t: 0,
                    wobble: 0,
                    drip: 0,
                    hammer: 0,
                    say: None,
                    wheel: 0.0,
                    waiting: false,
                    warn: 0,
                    bump_cd: 0,
                    brake_t: 0,
                    place: k,
                    stats: RStats::default(),
                    ai: Ai::default(),
                    tip_cd: 0,
                }
            })
            .collect();
        let mut riders = riders;
        for r in riders.iter_mut() {
            r.crank_to = r.crank;
        }
        let mut views: Vec<usize> = Vec::new();
        for p in 0..2 {
            if let Some(i) = riders.iter().position(|r| r.ctrl == Ctrl::Human(p)) {
                views.push(i);
            }
        }
        if views.is_empty() {
            views.push(0);
        }
        let n = riders.len();
        let mut race = Race {
            stage: stage_i,
            riders,
            objs: Vec::new(),
            bridge: st.bridge.map(|x| Bridge { x, state: BState::Closed, t: 0, lift: 0.0 }),
            floaters: Vec::new(),
            length: st.length(),
            phase: Phase::Countdown,
            phase_t: 0,
            frame: 0,
            clock: 0,
            rng,
            sfx: Vec::new(),
            fx: Vec::new(),
            popups: Vec::new(),
            banners: [None, None],
            shake: [0; 2],
            gust: 1.0,
            gust_t: 0,
            gust_len: 0,
            gust_on: false,
            views,
            tram: None,
            tram_t: 400,
            moo_cd: 0,
            alarm_cd: 0,
            rattle_cd: 0,
            first_cpu_finish: None,
            swept: false,
            section: vec![0; n],
            hup: vec![false; n],
            say_cd: 240,
        };
        race.build_course();
        race
    }

    pub fn def(&self) -> &'static StageDef {
        stage(self.stage)
    }

    pub fn humans(&self) -> impl Iterator<Item = usize> + '_ {
        self.riders.iter().enumerate().filter(|(_, r)| matches!(r.ctrl, Ctrl::Human(_))).map(|(i, _)| i)
    }

    /// The view (0 or 1) that follows a human rider.
    pub fn view_of(&self, i: usize) -> Option<usize> {
        match self.riders[i].ctrl {
            Ctrl::Human(p) => Some(p.min(self.views.len() - 1)),
            Ctrl::Cpu => {
                if self.humans().next().is_none() && self.views.first() == Some(&i) {
                    Some(0)
                } else {
                    None
                }
            }
        }
    }

    pub fn finished(&self) -> bool {
        self.phase == Phase::Over && self.phase_t > 170
    }

    // ------------------------------------------------------------ course

    fn build_course(&mut self) {
        let st = self.def();
        let len = self.length;
        let total: i32 = st.spawns.iter().map(|s| s.1).sum();
        let mut x = 520.0 + self.rng.int(120) as f32;
        let mut last_lane = 0;
        while x < len - 420.0 {
            let near_special = st.bridge.map(|b| (x - b).abs() < 230.0).unwrap_or(false) || st.snackbar.map(|b| (x - b).abs() < 160.0).unwrap_or(false);
            if !near_special {
                let mut pick = self.rng.int(total);
                let mut kind = st.spawns[0].0;
                for &(k, w) in st.spawns {
                    if pick < w {
                        kind = k;
                        break;
                    }
                    pick -= w;
                }
                // alternate lanes more often than not, so both lanes see action
                let lane = if self.rng.f() < 0.65 { 1 - last_lane } else { last_lane };
                last_lane = lane;
                self.spawn(kind, x, lane);
            }
            x += (st.spacing.0 + self.rng.int(st.spacing.1 - st.spacing.0)) as f32;
        }
        if let Some(sx) = st.snackbar {
            self.objs.push(Obj::new(OKind::Food(Food::Kroket), sx + 30.0, 0.0, 0));
            self.objs.push(Obj::new(OKind::Food(Food::Kroket), sx + 30.0, 1.0, 0));
        }
        self.objs.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    }

    fn spawn(&mut self, kind: Spawn, x: f32, lane: i32) {
        let y = lane as f32;
        let venue = self.def().venue;
        let var = self.rng.int(4) as u8;
        match kind {
            Spawn::Pothole => self.objs.push(Obj::new(OKind::Pothole, x, y, var)),
            Spawn::Bollard => self.objs.push(Obj::new(OKind::Bollard, x, y, var)),
            Spawn::Herring => self.objs.push(Obj::new(OKind::Herring, x, y, var)),
            Spawn::Cat => self.objs.push(Obj::new(OKind::Cat, x, y, var)),
            Spawn::Cow => self.objs.push(Obj::new(OKind::Cow, x, y, var)),
            Spawn::Rails => self.objs.push(Obj::new(OKind::Rails, x, 0.5, var)),
            Spawn::Geese => {
                let n = 2 + self.rng.int(2);
                for k in 0..n {
                    let mut o = Obj::new(OKind::Goose, x + k as f32 * 13.0, y + self.rng.range(-0.15, 0.15), var);
                    o.t = self.rng.int(60);
                    self.objs.push(o);
                }
            }
            Spawn::Sheep => {
                let n = 4 + self.rng.int(3);
                for k in 0..n {
                    let mut o = Obj::new(OKind::Sheep, x + k as f32 * 9.0 + self.rng.range(-4.0, 4.0), -0.9 - k as f32 * 0.4, (k % 4) as u8);
                    o.t = self.rng.int(40);
                    o.face = 1;
                    self.objs.push(o);
                }
            }
            Spawn::Tourist => {
                let mut o = Obj::new(OKind::Tourist, x, y, var);
                o.vx = if self.rng.f() < 0.5 { 0.25 } else { -0.2 };
                o.face = if o.vx > 0.0 { 1 } else { -1 };
                self.objs.push(o);
                if self.rng.f() < 0.4 {
                    let mut o2 = Obj::new(OKind::Tourist, x + 90.0, 1.0 - y, (var + 1) % 4);
                    o2.vx = 0.2;
                    o2.face = 1;
                    self.objs.push(o2);
                }
            }
            Spawn::Tractor => {
                if x > 900.0 {
                    let mut o = Obj::new(OKind::Tractor, x, y, var);
                    o.vx = 1.45;
                    o.face = 1;
                    self.objs.push(o);
                }
            }
            Spawn::Sofa => {
                if x > 900.0 {
                    let mut o = Obj::new(OKind::Sofa, x, y, var);
                    o.vx = 1.2;
                    o.face = 1;
                    self.objs.push(o);
                }
            }
            Spawn::Food => {
                let food = match (venue, self.rng.int(3)) {
                    (Venue::Polder, 0) => Food::Kaas,
                    (Venue::Amsterdam, 0) | (Venue::Dijk, 0) => Food::Haring,
                    _ => Food::Stroopwafel,
                };
                self.objs.push(Obj::new(OKind::Food(food), x, y, var));
            }
        }
    }

    // ------------------------------------------------------------ helpers

    pub fn wind_for(&self, i: usize) -> (Wind, usize) {
        self.def().wind_at(self.riders[i].x)
    }

    /// Frames between strokes before a push is wasted (the cadence limit).
    pub fn gap_for(&self, i: usize) -> i32 {
        let r = &self.riders[i];
        let mut g = r.bike.def().gap;
        if r.v < 1.0 {
            // low gear while getting going
            g -= ((1.0 - r.v) * 2.5) as i32;
        }
        if r.bonk {
            g += 7;
        }
        g
    }

    pub fn progress(&self, i: usize) -> f32 {
        (self.riders[i].x / self.length).clamp(0.0, 1.0)
    }

    fn banner(&mut self, view: Option<usize>, text: impl Into<String>, dur: i32, color: u32) {
        let text = text.into();
        let b = Banner { text, t: 0, dur, color, y: None, scale: 0, pop: false };
        match view {
            Some(v) => self.banners[v.min(1)] = Some(b),
            None => {
                self.banners[0] = Some(b.clone());
                self.banners[1] = Some(b);
            }
        }
    }

    fn banner_rider(&mut self, i: usize, text: impl Into<String>, dur: i32, color: u32) {
        if let Some(v) = self.view_of(i) {
            self.banner(Some(v), text, dur, color);
        }
    }

    fn popup(&mut self, x: f32, y: f32, text: &'static str, color: u32) {
        self.popups.push(Popup { x, y, text, color, life: 50 });
    }

    fn popup_rider(&mut self, i: usize, text: &'static str, color: u32) {
        let r = &self.riders[i];
        let (x, y) = (r.x, ground_y(r.y) - 40.0 - r.z);
        self.popup(x, y, text, color);
    }

    fn human_sfx(&mut self, i: usize, s: Sfx) {
        if self.view_of(i).is_some() {
            self.sfx.push(s);
        }
    }

    fn near_view(&self, x: f32) -> bool {
        self.views.iter().any(|&v| (self.riders[v].x - x).abs() < 170.0)
    }

    fn drops(&mut self, x: f32, y: f32, n: usize, kind: FxKind, color: u32) {
        for _ in 0..n {
            let vx = self.rng.range(-1.2, 1.2);
            let vy = self.rng.range(-2.6, -0.6);
            let life = 20 + self.rng.int(16);
            self.fx.push(Fx { x, y, vx, vy, life, max: life, kind, color });
        }
    }

    // ------------------------------------------------------------ update

    pub fn update(&mut self, pads: &[Pad; 2], prev: &[Pad; 2]) {
        self.frame = self.frame.wrapping_add(1);
        self.phase_t += 1;
        match self.phase {
            Phase::Countdown => {
                let t = self.phase_t;
                for (k, s) in [(20, "3"), (70, "2"), (120, "1")] {
                    if t == k {
                        self.banner(None, s, 40, rgb(248, 248, 248));
                        if let Some(b) = self.banners[0].as_mut() {
                            b.pop = true;
                            b.scale = 6;
                        }
                        if let Some(b) = self.banners[1].as_mut() {
                            b.pop = true;
                            b.scale = 6;
                        }
                        self.sfx.push(Sfx::Blip);
                    }
                }
                if t >= COUNTDOWN {
                    self.phase = Phase::Race;
                    self.phase_t = 0;
                    self.banner(None, "FIETSEN!", 60, rgb(248, 136, 24));
                    self.sfx.push(Sfx::Whistle);
                    // everyone starts with their legs ready
                    for r in self.riders.iter_mut() {
                        r.since = 99;
                    }
                }
            }
            Phase::Race | Phase::Over => {
                if self.phase == Phase::Race {
                    self.clock += 1;
                }
                self.step_world();
                let ctls: Vec<Ctl> = (0..self.riders.len())
                    .map(|i| match self.riders[i].ctrl {
                        Ctrl::Human(p) => {
                            if self.riders[i].finish.is_some() || self.riders[i].dnf {
                                Ctl::default()
                            } else {
                                ctl_from_pad(pads[p.min(1)], prev[p.min(1)])
                            }
                        }
                        Ctrl::Cpu => Ctl::default(),
                    })
                    .collect();
                let mut order: Vec<usize> = (0..self.riders.len()).collect();
                order.sort_by(|&a, &b| self.riders[b].x.partial_cmp(&self.riders[a].x).unwrap());
                for &i in &order {
                    let ctl = if self.riders[i].ctrl == Ctrl::Cpu { self.ai_ctl(i) } else { ctls[i] };
                    self.step_rider(i, ctl);
                }
                self.block_riders(&order);
                self.check_finish();
                self.rank();
                self.chatter();
            }
        }
        self.step_fx();
    }

    fn step_world(&mut self) {
        // gusts: a slow swell plus bursts every few seconds
        self.gust_t -= 1;
        if self.gust_t <= 0 {
            if self.gust_on {
                self.gust_on = false;
                self.gust_t = 240 + self.rng.int(420);
            } else {
                self.gust_on = true;
                self.gust_len = 80 + self.rng.int(60);
                self.gust_t = self.gust_len;
                let strong = self.views.iter().any(|&v| self.def().wind_at(self.riders[v].x).0.strength() > 0.45);
                if strong && self.phase == Phase::Race {
                    self.sfx.push(Sfx::Gust);
                }
            }
        }
        let burst = if self.gust_on {
            let k = 1.0 - self.gust_t as f32 / self.gust_len.max(1) as f32;
            0.45 * (k * std::f32::consts::PI).sin()
        } else {
            0.0
        };
        self.gust = 1.0 + 0.12 * (self.frame as f32 * 0.013).sin() + burst;

        // bridge
        let trigger = self.trigger_rider();
        if let Some(b) = self.bridge.as_mut() {
            b.t += 1;
            match b.state {
                BState::Closed => {
                    if let Some(tx) = trigger {
                        if tx > b.barrier() - 200.0 && tx < b.barrier() {
                            b.state = BState::Warn;
                            b.t = 0;
                        }
                    }
                }
                BState::Warn => {
                    if b.t >= BRIDGE_WARN {
                        b.state = BState::Open;
                        b.t = 0;
                    }
                }
                BState::Open => {
                    b.lift = (b.lift + 0.02).min(1.0);
                    if b.t >= BRIDGE_OPEN {
                        b.state = BState::Closing;
                        b.t = 0;
                    }
                }
                BState::Closing => {
                    b.lift = (b.lift - 0.02).max(0.0);
                    if b.t >= BRIDGE_CLOSE && b.lift <= 0.0 {
                        b.state = BState::Done;
                    }
                }
                BState::Done => {}
            }
        }
        if let Some((state, t, x, lights)) = self.bridge.as_ref().map(|b| (b.state, b.t, b.x, b.lights())) {
            if state == BState::Warn && t == 1 && self.near_view(x - 200.0) {
                self.banner(None, "BRUG OPEN!", 80, rgb(248, 72, 72));
            }
            if lights && self.near_view(x) {
                self.alarm_cd -= 1;
                if self.alarm_cd <= 0 {
                    self.alarm_cd = 48;
                    self.sfx.push(Sfx::BridgeAlarm);
                }
            }
        }

        // tram in the background (Amsterdam)
        if self.def().venue == Venue::Amsterdam {
            match self.tram {
                Some(tx) => {
                    let nx = tx - 2.2;
                    self.tram = if nx < -140.0 { None } else { Some(nx) };
                }
                None => {
                    self.tram_t -= 1;
                    if self.tram_t <= 0 {
                        self.tram = Some(300.0);
                        self.tram_t = 700 + self.rng.int(600);
                        if self.phase == Phase::Race {
                            self.sfx.push(Sfx::TramBell);
                        }
                    }
                }
            }
        }

        // animals and people
        let racing = self.phase == Phase::Race;
        let mut sounds = Vec::new();
        let mut pops = Vec::new();
        let mut bites = Vec::new();
        self.moo_cd -= 1;
        for k in 0..self.objs.len() {
            let o = &mut self.objs[k];
            if o.gone {
                continue;
            }
            o.t += 1;
            match o.kind {
                OKind::Sheep => {
                    if o.state == 0 && self.riders.iter().any(|r| o.x - r.x < 190.0 && o.x - r.x > 0.0) {
                        o.state = 1;
                    }
                    if o.state == 1 {
                        o.y += SHEEP_VY;
                        if o.y > 1.95 {
                            o.state = 2;
                        }
                    }
                }
                OKind::Tourist => {
                    if o.state == 0 {
                        o.x += o.vx;
                        // turn around now and then
                        if o.t % 300 == 299 {
                            o.vx = -o.vx;
                            o.face = -o.face;
                        }
                    } else if o.state == 1 {
                        o.y += (-1.15 - o.y).clamp(-0.07, 0.07);
                        if o.y <= -1.1 {
                            o.state = 2;
                        }
                    }
                }
                OKind::Cat => {
                    if o.state == 1 {
                        o.y -= 0.07;
                        o.z += o.vz;
                        o.vz -= 0.3;
                        if o.z < 0.0 {
                            o.z = 0.0;
                            o.vz = 1.8;
                        }
                        if o.y < -1.5 {
                            o.gone = true;
                        }
                    }
                }
                OKind::Goose => {
                    match o.state {
                        0 => {
                            o.x += ((o.t as f32) * 0.05).sin() * 0.1;
                        }
                        1 => {
                            let tr = &self.riders[o.target];
                            o.x += 2.35;
                            o.face = 1;
                            o.y += (tr.y - o.y).clamp(-0.03, 0.03);
                            if o.t % 30 == 0 && (tr.x - o.x).abs() < 160.0 {
                                sounds.push((o.target, Sfx::Honk));
                            }
                            if tr.riding() && tr.x - o.x < 8.0 && tr.x - o.x > -10.0 && (tr.y - o.y).abs() < 0.4 && tr.z < 6.0 {
                                bites.push((o.target, k));
                                o.state = 2;
                                o.t = 0;
                            } else if o.t > 220 || !tr.riding() {
                                o.state = 2;
                                o.t = 0;
                            }
                        }
                        _ => {
                            // waddle proudly off the path
                            o.y += (-1.0 - o.y).clamp(-0.02, 0.02);
                            o.face = -1;
                        }
                    }
                }
                OKind::Tractor | OKind::Sofa => {
                    if racing {
                        o.x += o.vx;
                    }
                }
                OKind::Cow => {
                    if self.moo_cd <= 0 && o.state == 0 {
                        for &v in &self.views {
                            if (self.riders[v].x - o.x).abs() < 110.0 {
                                o.state = 1;
                                self.moo_cd = 120;
                                sounds.push((v, Sfx::Moo));
                                pops.push((o.x, ground_y(o.y) - 30.0, "BOE!"));
                                break;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        for (i, s) in sounds {
            self.human_sfx(i, s);
        }
        for (x, y, s) in pops {
            self.popup(x, y, s, rgb(248, 248, 248));
        }
        for (i, _k) in bites {
            let r = &mut self.riders[i];
            r.v *= 0.55;
            r.energy = (r.energy - 8.0).max(0.0);
            r.wobble = 30;
            r.stats.bites += 1;
            self.popup_rider(i, "AU!", rgb(248, 72, 72));
            self.human_sfx(i, Sfx::Honk);
            let r = &self.riders[i];
            let (x, y) = (r.x - 6.0, ground_y(r.y) - 10.0);
            self.drops(x, y, 6, FxKind::Feather, rgb(248, 248, 248));
        }
        for f in self.floaters.iter_mut() {
            f.t += 1;
        }
    }

    /// The rider whose approach opens the drawbridge: the leading human, or
    /// the leader when nobody plays.
    fn trigger_rider(&self) -> Option<f32> {
        if self.phase != Phase::Race {
            return None;
        }
        let humans: Vec<f32> = self.humans().filter(|&i| self.riders[i].finish.is_none()).map(|i| self.riders[i].x).collect();
        if humans.is_empty() {
            if self.humans().next().is_some() {
                return None;
            }
            return self.riders.iter().map(|r| r.x).fold(None, |a: Option<f32>, x| Some(a.map_or(x, |a| a.max(x))));
        }
        humans.into_iter().fold(None, |a: Option<f32>, x| Some(a.map_or(x, |a| a.max(x))))
    }

    // ------------------------------------------------------------ riders

    fn step_rider(&mut self, i: usize, ctl: Ctl) {
        let racing = self.phase == Phase::Race || self.phase == Phase::Over;
        {
            let r = &mut self.riders[i];
            r.since = (r.since + 1).min(999);
            r.stroke_t += 1;
            r.bell_t = (r.bell_t - 1).max(0);
            r.wobble = (r.wobble - 1).max(0);
            r.drip = (r.drip - 1).max(0);
            r.hammer = (r.hammer - 1).max(0);
            r.spin = (r.spin - 1).max(0);
            r.warn = (r.warn - 1).max(0);
            r.bump_cd = (r.bump_cd - 1).max(0);
            r.tip_cd = (r.tip_cd - 1).max(0);
            if let Some((_, t)) = r.say.as_mut() {
                *t -= 1;
                if *t <= 0 {
                    r.say = None;
                }
            }
        }
        match self.riders[i].state {
            RState::Fly => {
                let r = &mut self.riders[i];
                r.st += 1;
                let dur = if r.crash_y > 1.3 && r.v < 0.01 { 18 } else { 34 };
                if r.st >= dur {
                    r.state = RState::Swim;
                    r.st = 0;
                    let x = r.crash_x + if dur == 18 { 6.0 } else { 34.0 };
                    self.drops(x, 200.0, 14, FxKind::Drop, rgb(160, 216, 248));
                    self.popup(x, 176.0, "PLONS!", rgb(96, 200, 248));
                    self.human_sfx(i, Sfx::Splash);
                    if self.view_of(i).is_some() {
                        let v = self.view_of(i).unwrap();
                        self.shake[v] = 10;
                    }
                }
                return;
            }
            RState::Swim => {
                let r = &mut self.riders[i];
                r.st += 1;
                if r.st >= 66 {
                    r.state = RState::Climb;
                    r.st = 0;
                }
                return;
            }
            RState::Climb => {
                let r = &mut self.riders[i];
                r.st += 1;
                if r.st >= 26 {
                    r.state = RState::Ride;
                    r.st = 0;
                    r.x = r.crash_x;
                    r.y = 1.0;
                    r.lane = 1;
                    r.v = 0.0;
                    r.z = 0.0;
                    r.vz = 0.0;
                    r.drip = 300;
                    r.since = 99;
                    r.wobble = 20;
                }
                return;
            }
            RState::Ride => {}
        }
        if !racing {
            return;
        }
        let st = self.def();
        let (wind, sec) = self.wind_for(i);
        let d = self.riders[i].who.def();
        let b = self.riders[i].bike.def();
        let done = self.riders[i].finish.is_some() || self.riders[i].dnf;

        // section change: show the wind
        if sec != self.section[i] {
            self.section[i] = sec;
            if !done {
                let col = match wind {
                    Wind::Head(_) => rgb(248, 96, 72),
                    Wind::Tail(_) => rgb(120, 232, 96),
                    Wind::Side(_) => rgb(248, 216, 48),
                };
                self.banner_rider(i, format!("{}!", wind.name()), 55, col);
            }
        }

        // tuck, lean, bell, hop
        {
            let r = &mut self.riders[i];
            r.tuck = ctl.tuck && r.z <= 0.0 && !done;
            r.leaning = false;
        }
        if ctl.bell && !self.riders[i].tuck {
            self.ring(i);
        }
        if ctl.hop && !self.riders[i].tuck && self.riders[i].z <= 0.0 && !done {
            let r = &mut self.riders[i];
            r.vz = b.hop;
            r.z = 0.1;
            r.energy = (r.energy - 1.2).max(0.0);
            r.stats.hops += 1;
            self.human_sfx(i, Sfx::Hop);
        }

        // brake: both pedal keys held for a moment (a quick overlap is not a brake)
        {
            let r = &mut self.riders[i];
            r.brake_t = if ctl.brake { r.brake_t + 1 } else { 0 };
            if r.brake_t > 10 && r.z <= 0.0 {
                r.v = (r.v - 0.05).max(0.0);
            }
        }

        // pedal strokes
        if !done && self.riders[i].z <= 0.0 && !self.riders[i].waiting {
            if ctl.left && !ctl.right {
                self.stroke(i, -1);
            } else if ctl.right && !ctl.left {
                self.stroke(i, 1);
            }
        }

        // lanes
        {
            let r = &mut self.riders[i];
            if ctl.up_e && r.lane > 0 {
                r.lane = 0;
            }
            if ctl.down_e && r.lane < 1 {
                r.lane = 1;
            }
        }
        let gust = self.gust;
        let side = wind.side() * gust * d.wind_feel;
        {
            let bike = self.riders[i].bike;
            let r = &mut self.riders[i];
            let k = if r.tuck { 0.05 } else { 0.1 };
            let mut drift = side * 0.028 * if r.tuck { 1.3 } else { 1.0 } * match bike {
                Bike::Lig => 0.6,
                Bike::Bak => 0.8,
                _ => 1.0,
            };
            if ctl.up && side > 0.0 {
                drift *= 0.35;
                r.leaning = true;
            }
            if r.v < 0.3 {
                drift *= r.v / 0.3;
            }
            let mv = ((r.lane as f32 - r.y) * k).clamp(-0.07, 0.07);
            r.y += mv + drift;
            if r.y < HEDGE_Y {
                r.y = HEDGE_Y;
                r.v *= 0.97;
            }
        }
        if self.riders[i].y > 1.32 && self.riders[i].tip_cd == 0 && !done {
            self.riders[i].tip_cd = 120;
            self.riders[i].warn = 40;
            self.popup_rider(i, "PAS OP!", rgb(248, 216, 48));
        }
        if self.riders[i].y > CANAL_Y {
            self.crash(i, true);
            return;
        }

        // drag
        let draft = self.draft_for(i);
        let mut landed = false;
        let surface = if st.on_cobbles(self.riders[i].x) { 2.4 } else { 1.0 };
        {
            let r = &mut self.riders[i];
            r.draft = draft.1;
            let head = wind.head() * gust * d.wind_feel;
            let air = r.v + head;
            let mut cd = C_AIR * b.drag;
            if r.tuck {
                cd *= b.tuck;
            }
            cd *= draft.0;
            let drag = cd * air * air.abs() + C_ROLL * b.roll * surface * r.v;
            r.v = (r.v - drag).clamp(0.0, MAX_V);
            if done {
                // roll out past the crowd, then stop and wave
                r.v = if r.x < self.length + 150.0 { r.v.max(1.0) * 0.995 } else { r.v * 0.96 };
            }
            if r.draft > 0.3 && !done {
                r.stats.draft_frames += 1;
                r.energy = (r.energy + 0.012).min(100.0);
            }
            if wind.head() > 0.45 {
                r.stats.head_frames += 1;
                if r.tuck {
                    r.stats.tuck_head_frames += 1;
                }
            }
            if r.tuck {
                r.stats.tuck_frames += 1;
            }
            // energy comes back while you roll
            let rest = if r.bonk { 0.1 } else if r.since > 30 { 0.03 } else { 0.004 };
            r.energy = (r.energy + rest).min(100.0);
            if r.bonk && r.energy >= 25.0 {
                r.bonk = false;
            }
            // hop
            if r.z > 0.0 || r.vz > 0.0 {
                r.z += r.vz;
                r.vz -= GRAV;
                if r.z <= 0.0 {
                    r.z = 0.0;
                    r.vz = 0.0;
                    landed = true;
                }
            }
            // crank and wheels
            if r.spin > 0 {
                r.crank += 0.07;
                r.crank_to = r.crank;
            } else {
                r.crank += (r.crank_to - r.crank) * 0.22;
            }
            r.wheel += r.v;
        }
        // landing with a racing bike shakes cargo loose
        if landed && self.riders[i].bike == Bike::Race && self.rng.f() < 0.3 {
            self.shed_cargo(i, 1, 0.0);
        }
        let r = &mut self.riders[i];
        let dx = r.v;
        r.x += dx;
        if surface > 1.0 && r.v > 0.5 && self.view_of(i).is_some() {
            let v = self.view_of(i).unwrap();
            self.shake[v] = self.shake[v].max(2);
            self.rattle_cd -= 1;
            if self.rattle_cd <= 0 {
                self.rattle_cd = 26;
                self.sfx.push(Sfx::Rattle);
            }
        }

        // bridge
        if let Some(bx) = self.bridge.as_ref().map(|b| (b.barrier(), b.blocking())) {
            let r = &mut self.riders[i];
            let front = r.x + 12.0;
            r.waiting = false;
            if bx.1 && front > bx.0 - 4.0 && r.x < bx.0 + 2.0 {
                if front > bx.0 + 2.0 {
                    // already on the barrier line when it came down: squeezes through
                } else {
                    r.x = r.x.min(bx.0 - 12.0);
                    r.v = 0.0;
                    r.waiting = true;
                    r.stats.wait_frames += 1;
                }
            }
        }

        if !done {
            self.collide(i);
        }

        // bonk
        let r = &mut self.riders[i];
        if r.energy < 0.5 && !r.bonk {
            if d.never_bonks {
                r.energy = 5.0;
            } else {
                r.bonk = true;
                r.hammer = 150;
                r.stats.bonks += 1;
                self.banner_rider(i, "DE MAN MET DE HAMER!", 100, rgb(200, 120, 248));
                self.popup_rider(i, "BONK!", rgb(248, 248, 248));
            }
        }

        // final kilometre
        if !self.hup[i] && self.progress(i) > 0.86 && !done {
            self.hup[i] = true;
            self.banner_rider(i, "HUP HOLLAND HUP!", 80, rgb(248, 136, 24));
        }
    }

    fn stroke(&mut self, i: usize, side: i8) {
        let g = self.gap_for(i);
        let (wind, _) = self.wind_for(i);
        let view = self.view_of(i);
        let r = &mut self.riders[i];
        if r.last_side == side {
            // same foot twice: the pedal is already down
            return;
        }
        r.last_side = side;
        r.stroke_t = 0;
        if r.since < g {
            r.stats.wasted += 1;
            if !r.bonk {
                r.energy = (r.energy - 0.22).max(0.0);
            }
            r.v *= 0.99;
            r.spin = 10;
            r.clean = false;
            r.last_stroke = Stroke::Wasted;
            if view.is_some() && r.tip_cd == 0 {
                r.tip_cd = 100;
                self.popup_rider(i, "TE SNEL!", rgb(248, 96, 72));
            }
            return;
        }
        let d = r.who.def();
        let b = r.bike.def();
        let (q, mult, cost) = if r.clean && r.since <= g + 3 {
            (Stroke::Perfect, 1.2, 0.16)
        } else if r.since <= g + 14 {
            (Stroke::Good, 1.0, 0.11)
        } else if r.since <= 45 {
            (Stroke::Late, 0.85, 0.08)
        } else {
            (Stroke::Late, 0.7, 0.08)
        };
        let wind_pow = if wind.head() > 0.45 {
            d.head_power
        } else if wind.head() < -0.3 {
            d.tail_power
        } else {
            1.0
        };
        let skill_pow = if r.ctrl == Ctrl::Cpu { 0.9 + 0.12 * r.skill } else { 1.0 };
        let mut p = P_STROKE * b.power * d.power * mult * wind_pow * skill_pow;
        if r.tuck {
            p *= 0.92;
        }
        if r.bonk {
            p *= 0.75;
        }
        r.v += p;
        r.since = 0;
        r.clean = true;
        r.last_stroke = q;
        r.energy = (r.energy - cost).max(0.0);
        r.crank_to += 0.5;
        match q {
            Stroke::Perfect => r.stats.perfect += 1,
            _ => r.stats.good += 1,
        }
        if view.is_some() {
            self.sfx.push(Sfx::Tick);
        }
    }

    fn ring(&mut self, i: usize) {
        let r = &mut self.riders[i];
        if r.bell_t > 8 {
            return;
        }
        r.bell_t = 24;
        r.stats.rings += 1;
        let (x, y) = (r.x, r.y);
        self.human_sfx(i, Sfx::Bell);
        let mut pops = Vec::new();
        for o in self.objs.iter_mut() {
            if o.gone || o.state != 0 {
                continue;
            }
            let dx = o.x - x;
            if !(-10.0..130.0).contains(&dx) || (o.y - y).abs() > 0.6 {
                continue;
            }
            match o.kind {
                OKind::Tourist => {
                    o.state = 1;
                    pops.push((o.x, ground_y(o.y) - 34.0, ["SORRY!", "OH!", "EXCUSE!", "WOW!"][o.var as usize % 4]));
                }
                OKind::Cat => {
                    o.state = 1;
                    o.vz = 2.0;
                    pops.push((o.x, ground_y(o.y) - 16.0, "MIAUW!"));
                }
                OKind::Goose if dx < 90.0 => {
                    pops.push((o.x, ground_y(o.y) - 22.0, "SSSS!"));
                }
                OKind::Sheep if dx < 90.0 => {
                    pops.push((o.x, ground_y(o.y) - 22.0, "BEEEH?"));
                }
                _ => {}
            }
        }
        for (x, y, s) in pops {
            self.popup(x, y, s, rgb(248, 248, 248));
        }
    }

    /// (drag multiplier, draft strength 0..1)
    fn draft_for(&self, i: usize) -> (f32, f32) {
        let r = &self.riders[i];
        let d = r.who.def();
        let mut best = 0.0f32;
        for (j, o) in self.riders.iter().enumerate() {
            if j == i || !o.riding() || o.finish.is_some() {
                continue;
            }
            let dx = o.x - r.x;
            if (o.y - r.y).abs() < 0.38 && dx > 10.0 && dx < 54.0 {
                let s = if dx < 28.0 { 1.0 } else { 1.0 - (dx - 28.0) / 26.0 };
                best = best.max(s);
            }
        }
        let mut f = 1.0 - (1.0 - d.draft) * best;
        for o in &self.objs {
            if !o.kind.slow_rider() || o.gone {
                continue;
            }
            let dx = o.x - r.x;
            if (o.y - r.y).abs() < 0.4 && dx > 14.0 && dx < 70.0 {
                let s = if dx < 40.0 { 1.0 } else { 1.0 - (dx - 40.0) / 30.0 };
                best = best.max(s);
                let big = if o.kind == OKind::Tractor { 0.3 } else { 0.5 };
                f = f.min(1.0 - (1.0 - big) * s);
            }
        }
        (f, best)
    }

    fn collide(&mut self, i: usize) {
        let (rx, ry, rz, rv) = {
            let r = &self.riders[i];
            (r.x, r.y, r.z, r.v)
        };
        let bit = 1u8 << i;
        let mut hit: Option<usize> = None;
        let mut eat: Option<usize> = None;
        for (k, o) in self.objs.iter().enumerate() {
            if o.gone || o.hit & bit != 0 {
                continue;
            }
            let half = o.kind.width() / 2.0;
            // low things only matter under the wheels' middle; tall ones hit the whole bike
            let reach = if o.kind.height() < 50.0 { 6.0 } else { 13.0 };
            if o.x + half < rx - reach || o.x - half > rx + reach {
                continue;
            }
            let lane_hit = if o.kind.wide() { true } else { (o.y - ry).abs() < 0.45 };
            if !lane_hit {
                continue;
            }
            if let OKind::Food(_) = o.kind {
                if rz < 14.0 && (o.x - rx).abs() < 10.0 {
                    eat = Some(k);
                }
                continue;
            }
            if o.kind.slow_rider() {
                continue;
            }
            if !o.blocking() {
                continue;
            }
            if rz > o.kind.height() {
                continue;
            }
            if self.riders[i].bump_cd > 0 && o.kind.height() < 50.0 {
                continue;
            }
            hit = Some(k);
            break;
        }
        if let Some(k) = eat {
            let OKind::Food(food) = self.objs[k].kind else { return };
            self.objs[k].gone = true;
            let r = &mut self.riders[i];
            r.energy = (r.energy + food.energy()).min(100.0);
            r.stats.eaten += 1;
            self.popup_rider(i, food.shout(), rgb(248, 216, 96));
            self.human_sfx(i, Sfx::Chomp);
            let (x, y) = (rx + 8.0, ground_y(ry) - 22.0);
            self.drops(x, y, 5, FxKind::Crumb, rgb(216, 152, 64));
        }
        // slow riders (tractor, the lady with the sofa) block from behind
        for k in 0..self.objs.len() {
            let o = &self.objs[k];
            if !o.kind.slow_rider() || o.gone {
                continue;
            }
            let gap = o.kind.width() / 2.0 + 13.0;
            if (o.y - ry).abs() < 0.45 && o.x - rx < gap && o.x - rx > 0.0 {
                let (ox, ovx) = (o.x, o.vx);
                let r = &mut self.riders[i];
                r.x = r.x.min(ox - gap);
                r.v = r.v.min(ovx);
            }
        }
        let Some(k) = hit else { return };
        self.objs[k].hit |= bit;
        let kind = self.objs[k].kind;
        self.riders[i].stats.hits[kind.idx()] += 1;
        let (ox, oy) = (self.objs[k].x, self.objs[k].y);
        match kind {
            OKind::Pothole => {
                self.bump(i, 0.72, "AU!");
            }
            OKind::Rails => {
                if rv > 2.1 {
                    self.popup(rx, ground_y(ry) - 44.0, "TRAMRAILS!", rgb(248, 216, 48));
                    self.crash(i, false);
                } else {
                    self.bump(i, 0.75, "OEPS!");
                }
            }
            OKind::Herring => {
                self.bump(i, 0.8, "GLIBBERIG!");
                self.riders[i].y += 0.28;
                self.riders[i].wobble = 34;
            }
            OKind::Cat => {
                self.objs[k].state = 1;
                self.objs[k].vz = 2.2;
                self.popup(ox, ground_y(oy) - 16.0, "MIAUW!", rgb(248, 248, 248));
                self.bump(i, 0.75, "OEPS!");
            }
            OKind::Goose => {
                if self.objs[k].state == 0 {
                    self.objs[k].state = 1;
                    self.objs[k].target = i;
                    self.objs[k].t = 0;
                }
                self.drops(ox, ground_y(oy) - 8.0, 6, FxKind::Feather, rgb(248, 248, 248));
                self.human_sfx(i, Sfx::Honk);
                self.bump(i, 0.6, "GAK!");
            }
            OKind::Sheep => {
                self.objs[k].vz = 0.0;
                self.drops(ox, ground_y(oy) - 10.0, 6, FxKind::Wool, rgb(248, 248, 240));
                self.popup(ox, ground_y(oy) - 22.0, "BEEEH!", rgb(248, 248, 248));
                self.bump(i, 0.35, "");
            }
            OKind::Bollard | OKind::Cow | OKind::Tourist => {
                if kind == OKind::Tourist {
                    self.objs[k].state = 1;
                    self.popup(ox, ground_y(oy) - 34.0, "HELP!", rgb(248, 248, 248));
                }
                if kind == OKind::Cow {
                    self.human_sfx(i, Sfx::Moo);
                    self.popup(ox, ground_y(oy) - 30.0, "BOE!", rgb(248, 248, 248));
                }
                let limit = if self.riders[i].bike == Bike::Bak { CRASH_V * 1.3 } else { CRASH_V };
                if rv > limit {
                    self.crash(i, false);
                } else {
                    self.bump(i, 0.2, "DOE NORMAAL!");
                    self.riders[i].x -= 4.0;
                }
            }
            _ => {}
        }
    }

    fn bump(&mut self, i: usize, keep: f32, text: &'static str) {
        let r = &mut self.riders[i];
        // the heavy bakfiets ploughs on
        let keep = if r.bike == Bike::Bak { (keep + 0.15).min(1.0) } else { keep };
        r.v *= keep;
        r.wobble = r.wobble.max(22);
        r.bump_cd = 40;
        r.stats.bumps += 1;
        if !text.is_empty() {
            self.popup_rider(i, text, rgb(248, 248, 248));
        }
        self.human_sfx(i, Sfx::Rattle);
        if let Some(v) = self.view_of(i) {
            self.shake[v] = self.shake[v].max(6);
        }
        self.shed_cargo(i, 1, 1.0);
    }

    /// Knock up to `n` pieces off the rack; `luck` scales the bike's grip.
    fn shed_cargo(&mut self, i: usize, n: i32, luck: f32) {
        let cargo = self.def().cargo;
        let mut lost = 0;
        for _ in 0..n {
            let r = &self.riders[i];
            if r.cargo <= 0 {
                break;
            }
            if self.rng.f() < r.bike.def().cargo_grip * luck {
                continue;
            }
            let r = &mut self.riders[i];
            r.cargo -= 1;
            r.stats.cargo_lost += 1;
            lost += 1;
            let x = r.x - 10.0 + self.rng.range(0.0, 30.0);
            self.floaters.push(Floater { x, cargo, t: 0, y: self.rng.range(0.0, 1.0) });
        }
        if lost > 0 {
            self.popup_rider(i, "JAMMER!", rgb(248, 176, 48));
        }
    }

    fn crash(&mut self, i: usize, sideways: bool) {
        let r = &mut self.riders[i];
        r.state = RState::Fly;
        r.st = 0;
        r.crash_x = r.x;
        r.crash_y = r.y;
        if sideways {
            r.v = 0.0;
        }
        r.z = 0.0;
        r.vz = 0.0;
        r.tuck = false;
        r.stats.crashes += 1;
        r.last_side = 0;
        r.clean = false;
        self.human_sfx(i, Sfx::Crash);
        let words = ["DOE NORMAAL!", "OEPS!", "HELAAS!"];
        let w = words[self.rng.int(3) as usize];
        self.banner_rider(i, w, 70, rgb(248, 72, 72));
        if let Some(v) = self.view_of(i) {
            self.shake[v] = 12;
        }
        self.shed_cargo(i, 2, 0.5);
    }

    fn block_riders(&mut self, order: &[usize]) {
        for (k, &i) in order.iter().enumerate() {
            if !self.riders[i].riding() {
                continue;
            }
            for &j in &order[..k] {
                let (fx, fy, fv, ok) = {
                    let f = &self.riders[j];
                    // riders who finished are out of the way
                    (f.x, f.y, f.v, f.riding() && f.finish.is_none() && !f.dnf)
                };
                if !ok {
                    continue;
                }
                let r = &mut self.riders[i];
                if (fy - r.y).abs() < 0.42 && fx - r.x < 22.0 && fx >= r.x {
                    r.x = r.x.min(fx - 22.0).max(r.x - 2.0);
                    r.v = r.v.min(fv);
                }
            }
        }
    }

    fn check_finish(&mut self) {
        let len = self.length;
        for i in 0..self.riders.len() {
            let r = &self.riders[i];
            if r.finish.is_none() && !r.dnf && r.x >= len {
                let place = self.riders.iter().filter(|o| o.finish.is_some()).count();
                self.riders[i].finish = Some(self.clock);
                if self.riders[i].ctrl == Ctrl::Cpu && self.first_cpu_finish.is_none() {
                    self.first_cpu_finish = Some(self.clock);
                }
                if self.view_of(i).is_some() {
                    let txt = format!("FINISH! {}", ordinal(place));
                    let col = if place < 2 { rgb(248, 216, 48) } else { rgb(200, 200, 232) };
                    self.banner_rider(i, txt, 150, col);
                    self.human_sfx(i, Sfx::Cheer);
                    self.human_sfx(i, Sfx::Whistle);
                }
            }
        }
        if self.phase != Phase::Race {
            return;
        }
        let humans: Vec<usize> = self.humans().collect();
        let all_done = self.riders.iter().all(|r| r.finish.is_some() || r.dnf);
        let humans_done = !humans.is_empty() && humans.iter().all(|&h| self.riders[h].finish.is_some());
        let first = self.riders.iter().filter_map(|r| r.finish).min();
        let mut over = all_done;
        if humans_done {
            let last_h = humans.iter().filter_map(|&h| self.riders[h].finish).max().unwrap_or(0);
            if self.clock > last_h + 60 * 8 {
                over = true;
            }
        }
        if humans.is_empty() {
            if let Some(f) = first {
                if self.clock > f + 60 * 30 {
                    over = true;
                }
            }
        } else if let Some(f) = self.first_cpu_finish {
            // the sweeper van collects whoever is still out there
            let cpus_done = self.riders.iter().all(|r| r.ctrl != Ctrl::Cpu || r.finish.is_some());
            let last_cpu = self.riders.iter().filter(|r| r.ctrl == Ctrl::Cpu).filter_map(|r| r.finish).max().unwrap_or(f);
            if (cpus_done && self.clock > last_cpu + 60 * 45) || self.clock > f + 60 * 90 {
                for &h in &humans {
                    if self.riders[h].finish.is_none() {
                        self.riders[h].dnf = true;
                        self.swept = true;
                        self.banner_rider(h, "BEZEMWAGEN!", 150, rgb(200, 200, 232));
                    }
                }
                over = true;
            }
        }
        if over {
            self.phase = Phase::Over;
            self.phase_t = 0;
        }
    }

    /// Finish times (or estimates for riders still out there), best first:
    /// (rider, frames, dnf).
    pub fn results(&self) -> Vec<(usize, u32, bool)> {
        let mut v: Vec<(usize, u32, bool)> = self
            .riders
            .iter()
            .enumerate()
            .map(|(i, r)| {
                if r.dnf {
                    (i, u32::MAX - 10 + i as u32, true)
                } else if let Some(f) = r.finish {
                    (i, f, false)
                } else {
                    let left = (self.length - r.x).max(0.0);
                    (i, self.clock + (left / r.v.max(1.4)) as u32 + 1, false)
                }
            })
            .collect();
        v.sort_by_key(|e| e.1);
        v
    }

    fn rank(&mut self) {
        let mut order: Vec<usize> = (0..self.riders.len()).collect();
        order.sort_by(|&a, &b| {
            let ra = &self.riders[a];
            let rb = &self.riders[b];
            match (ra.finish, rb.finish) {
                (Some(x), Some(y)) => x.cmp(&y),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                _ => rb.x.partial_cmp(&ra.x).unwrap(),
            }
        });
        for (place, &i) in order.iter().enumerate() {
            let old = self.riders[i].place;
            self.riders[i].place = place;
            if place < old && self.phase == Phase::Race && self.clock > 120 && self.riders[i].finish.is_none() {
                self.riders[i].stats.overtakes += 1;
                if self.view_of(i).is_some() && self.rng.f() < 0.35 {
                    self.popup_rider(i, "LEKKER BEZIG!", rgb(120, 232, 96));
                }
                // the rider who got passed may have something to say
                if let Some(&j) = order.get(place + 1) {
                    if self.riders[j].ctrl == Ctrl::Cpu && self.riders[j].say.is_none() && self.rng.f() < 0.4 {
                        let h = self.riders[j].who.def().heckles;
                        let s = h[self.rng.int(h.len() as i32) as usize];
                        self.riders[j].say = Some((s, 90));
                    }
                }
            }
        }
    }

    /// Rivals talk now and then (only near a camera, so you see it).
    fn chatter(&mut self) {
        if self.phase != Phase::Race {
            return;
        }
        self.say_cd -= 1;
        if self.say_cd > 0 {
            return;
        }
        self.say_cd = 360 + self.rng.int(480);
        let cands: Vec<usize> = (0..self.riders.len())
            .filter(|&i| self.riders[i].ctrl == Ctrl::Cpu && self.riders[i].riding() && self.riders[i].say.is_none() && self.near_view(self.riders[i].x))
            .collect();
        if cands.is_empty() {
            return;
        }
        let i = cands[self.rng.int(cands.len() as i32) as usize];
        let (wind, _) = self.wind_for(i);
        let who = self.riders[i].who;
        let s = match who {
            Who::Dennis if wind.head() > 0.45 => ["PFFF, WIND...", "THIS IS UNFAIR!", "MY WATTS!"][self.rng.int(3) as usize],
            Who::Dennis if wind.head() < -0.3 => ["WHOOSH!", "AERO, BABY!", "STRAVA SEGMENT!"][self.rng.int(3) as usize],
            _ => {
                let h = who.def().heckles;
                h[self.rng.int(h.len() as i32) as usize]
            }
        };
        self.riders[i].say = Some((s, 110));
    }

    fn step_fx(&mut self) {
        for f in self.fx.iter_mut() {
            f.x += f.vx;
            f.y += f.vy;
            f.vy += match f.kind {
                FxKind::Feather | FxKind::Wool => 0.03,
                FxKind::Sparkle => 0.0,
                _ => 0.15,
            };
            if matches!(f.kind, FxKind::Feather | FxKind::Wool) {
                f.vx *= 0.95;
                f.vy = f.vy.min(0.5);
            }
            f.life -= 1;
        }
        self.fx.retain(|f| f.life > 0);
        for p in self.popups.iter_mut() {
            p.y -= 0.4;
            p.life -= 1;
        }
        self.popups.retain(|p| p.life > 0);
        if self.popups.len() > 12 {
            let n = self.popups.len() - 12;
            self.popups.drain(..n);
        }
        for b in self.banners.iter_mut() {
            if let Some(bn) = b {
                bn.t += 1;
                if bn.t > bn.dur {
                    *b = None;
                }
            }
        }
        for s in self.shake.iter_mut() {
            *s = (*s - 1).max(0);
        }
        // drips from wet riders and sparkles on perfect strokes
        for i in 0..self.riders.len() {
            let r = &self.riders[i];
            if r.drip > 0 && r.riding() && self.frame % 7 == 0 {
                let (x, y) = (r.x + self.rng.range(-6.0, 6.0), ground_y(r.y) - 20.0 - r.z);
                self.fx.push(Fx { x, y, vx: 0.0, vy: 0.5, life: 14, max: 14, kind: FxKind::Drop, color: rgb(160, 216, 248) });
            }
            let r = &self.riders[i];
            if r.last_stroke == Stroke::Perfect && r.stroke_t == 0 && self.view_of(i).is_some() {
                let (x, y) = (r.x + 1.0, ground_y(r.y) - 5.0);
                self.fx.push(Fx { x, y, vx: 0.0, vy: -0.4, life: 10, max: 10, kind: FxKind::Sparkle, color: rgb(248, 248, 160) });
            }
        }
        if self.fx.len() > 220 {
            let n = self.fx.len() - 220;
            self.fx.drain(..n);
        }
    }

    // ------------------------------------------------------------ CPU

    /// Is lane `lane` free of trouble from `x - back` to `x + ahead`?
    fn lane_clear(&self, i: usize, lane: i32, ahead: f32) -> bool {
        let r = &self.riders[i];
        let ly = lane as f32;
        for o in &self.objs {
            if o.gone || matches!(o.kind, OKind::Food(_)) || o.kind.wide() {
                continue;
            }
            if !o.blocking() && !o.kind.slow_rider() {
                continue;
            }
            let dx = o.x - r.x;
            if dx > -24.0 && dx < ahead && (o.y - ly).abs() < 0.6 {
                return false;
            }
            if o.kind == OKind::Sheep && o.state == 1 && dx > -10.0 && dx < ahead && o.y < ly + 0.6 && o.y > ly - 1.2 {
                return false;
            }
        }
        for (j, o) in self.riders.iter().enumerate() {
            if j != i && o.riding() && (o.y - ly).abs() < 0.5 && (o.x - r.x).abs() < 24.0 {
                return false;
            }
        }
        true
    }

    fn ai_ctl(&mut self, i: usize) -> Ctl {
        let mut c = Ctl::default();
        let r = &self.riders[i];
        if !r.riding() || r.finish.is_some() || self.phase != Phase::Race {
            return c;
        }
        let d = r.who.def();
        let b = r.bike.def();
        let skill = r.skill;
        let (wind, _) = self.wind_for(i);
        let g = self.gap_for(i);
        let mut ai = r.ai;
        ai.lane_cd -= 1;
        ai.hop_cd -= 1;
        ai.rung -= 1;
        let look = (50.0 + r.v * 30.0) * (0.7 + 0.5 * skill);

        // the first thing in my lane
        let mut hazard: Option<(f32, OKind, f32, f32)> = None;
        for o in &self.objs {
            if o.gone || matches!(o.kind, OKind::Food(_)) {
                continue;
            }
            let dx = o.x - o.kind.width() / 2.0 - (r.x + 13.0);
            if dx < -6.0 || dx > look {
                continue;
            }
            let in_lane = if o.kind.wide() {
                true
            } else if o.kind == OKind::Sheep && o.state == 1 {
                // will the sheep be in my lane when I get there?
                let t = (dx / r.v.max(0.5)).min(120.0);
                ((o.y + SHEEP_VY * t) - r.y).abs() < 0.6
            } else {
                (o.y - r.y).abs() < 0.5
            };
            if !in_lane {
                continue;
            }
            if o.kind.slow_rider() {
                if dx < 40.0 {
                    hazard = Some((dx, o.kind, o.y, o.x));
                    break;
                }
                continue;
            }
            if !o.blocking() {
                continue;
            }
            hazard = Some((dx, o.kind, o.y, o.x));
            break;
        }
        let other = 1 - r.lane;
        let mut coast = false;
        if let Some((dx, kind, _, ox)) = hazard {
            let h = kind.height();
            let can_hop = hop_clears(b.hop, h, kind.width(), r.v) && !kind.slow_rider();
            if kind == OKind::Tourist && dx < 110.0 && ai.rung <= 0 {
                c.bell = true;
                ai.rung = 50;
            }
            let switch_ok = ai.lane_cd <= 0 && !kind.wide() && self.lane_clear(i, other, look.max(60.0));
            let prefer_switch = !can_hop || kind == OKind::Sheep || skill > 0.6 && kind != OKind::Pothole;
            if kind == OKind::Tourist && dx > 50.0 {
                // ring first, they usually step aside
            } else if switch_ok && prefer_switch {
                c.up_e = other == 0;
                c.down_e = other == 1;
                ai.lane_cd = 40;
            } else if can_hop && ai.hop_cd <= 0 {
                // time the hop so the top of the jump is over the obstacle
                let noise = (1.0 - skill) * 5.0 * (self.rng.f() - 0.5);
                let lead = (b.hop / GRAV + noise) * r.v;
                if ox - r.x <= lead {
                    c.hop = true;
                    ai.hop_cd = 30;
                }
            } else if !can_hop && switch_ok {
                c.up_e = other == 0;
                c.down_e = other == 1;
                ai.lane_cd = 40;
            } else if !can_hop && r.v > CRASH_V * 0.8 && dx < 40.0 + r.v * 22.0 && !kind.slow_rider() {
                coast = true;
                c.brake = true;
            }
        } else if ai.lane_cd <= 0 {
            // no trouble: side wind, food, a wheel to follow, a slow rider to pass
            let side = wind.side() > 0.3;
            let mut want: Option<i32> = None;
            if side && r.lane == 1 {
                want = Some(0);
            }
            if want.is_none() && r.energy < 75.0 {
                for o in &self.objs {
                    if let OKind::Food(_) = o.kind {
                        let dx = o.x - r.x;
                        if !o.gone && dx > 20.0 && dx < look + 40.0 && (o.y - other as f32).abs() < 0.3 {
                            want = Some(other);
                            break;
                        }
                    }
                }
            }
            if want.is_none() && !(side && other == 1) {
                let mut best: Option<f32> = None;
                for (j, o) in self.riders.iter().enumerate() {
                    if j == i || !o.riding() || o.finish.is_some() {
                        continue;
                    }
                    let dx = o.x - r.x;
                    if dx > 16.0 && dx < 90.0 && (o.y - other as f32).abs() < 0.3 && best.is_none_or(|b| dx < b) {
                        best = Some(dx);
                    }
                }
                if best.is_some() && self.rng.f() < d.draft_seek * 0.05 * (0.4 + skill) {
                    want = Some(other);
                }
                // stuck behind someone slower: go round
                for (j, o) in self.riders.iter().enumerate() {
                    if j != i && o.riding() && (o.y - r.y).abs() < 0.3 && o.x - r.x > 0.0 && o.x - r.x < 26.0 && r.since > 6 && self.rng.f() < 0.04 + 0.04 * skill {
                        want = Some(other);
                    }
                }
            }
            if let Some(l) = want {
                if l != r.lane && self.lane_clear(i, l, look.max(70.0)) {
                    c.up_e = l == 0;
                    c.down_e = l == 1;
                    ai.lane_cd = 50;
                }
            }
        }
        // lean into side wind
        if wind.side() > 0.0 && r.y > r.lane as f32 + 0.08 {
            c.up = true;
        }

        // tuck in a headwind when nothing is coming
        let head = wind.head() * d.wind_feel;
        let near_hazard = hazard.map(|h| h.0 < look).unwrap_or(false);
        let safe_side = !(wind.side() > 0.3 && r.y > 0.6);
        if d.tuck_seek > 0.0 && head > 0.4 + (1.0 - d.tuck_seek) * 0.5 && !near_hazard && safe_side && skill > 0.25 {
            c.tuck = true;
        }

        // pedal in rhythm
        if !coast {
            if ai.next_gap <= 0 {
                ai.next_gap = g + 2;
            }
            if r.since >= ai.next_gap && r.z <= 0.0 {
                if r.last_side >= 0 {
                    c.left = true;
                } else {
                    c.right = true;
                }
                let mut delay = if d.never_bonks {
                    // Oma Riek: the same beat, always; a little slower on easy stages
                    1.0 + (1.0 - skill) * 3.0 + if self.rng.f() < 0.15 { 1.0 } else { 0.0 }
                } else {
                    (1.0 - skill) * 5.0 + self.rng.f() * (1.0 - skill) * 4.0
                };
                let near_end = self.progress(i) > 0.88;
                if r.energy < 28.0 && !near_end && !d.never_bonks {
                    delay += 5.0;
                }
                if r.who == Who::Dennis && head > 0.45 {
                    // Dennis sulks in a headwind
                    ai.lazy += 1;
                    if ai.lazy % 3 == 0 {
                        delay += 3.0;
                    }
                }
                ai.next_gap = g + delay.round() as i32;
            }
        }
        self.riders[i].ai = ai;
        c
    }
}

/// Frames above height `h` for a hop with take-off speed `vz`.
pub fn air_above(vz: f32, h: f32) -> f32 {
    let disc = vz * vz - 2.0 * GRAV * h;
    if disc <= 0.0 { 0.0 } else { 2.0 * disc.sqrt() / GRAV }
}

/// Can a hop at speed `v` carry the bike's middle over an obstacle?
pub fn hop_clears(vz: f32, h: f32, w: f32, v: f32) -> bool {
    h < 50.0 && air_above(vz, h) * v > 12.0 + w + 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wind_lookup() {
        let st = &STAGES[0];
        assert_eq!(st.wind_at(10.0).1, 0);
        assert!(st.length() > 10000.0);
    }
}
