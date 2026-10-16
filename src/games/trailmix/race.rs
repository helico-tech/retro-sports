//! One race up (and down) Mount Helico. Pure simulation at 60 Hz: no
//! drawing, no I/O. The course runs along x (pixels, to the right), the
//! height profile gives the ground; runners have a pace, stamina and a state.

use super::course::*;
use super::runners::{GERALD, RUNNERS};
use crate::engine::gfx::rgb;
pub use crate::engine::input::{edge, Pad};
pub use crate::engine::rng::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Sfx {
    Step,
    Jump,
    Splash,
    Cowbell,
    Baa,
    Whistle,
    Crowd,
    Gulp,
    Trip,
    Slide,
    Ding,
    Heart,
    Blip,
    Confirm,
    Horn,
    Moo,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Pace {
    Walk,
    Jog,
    Run,
    Sprint,
}

pub const PACES: [Pace; 4] = [Pace::Walk, Pace::Jog, Pace::Run, Pace::Sprint];
pub const PACE_NAMES: [&str; 4] = ["WALK", "JOG", "RUN", "SPRINT"];
/// Speed on the flat, pixels per frame.
const PACE_V: [f32; 4] = [0.72, 1.12, 1.5, 1.95];
/// Stamina used per second on the flat (negative: recovers).
const PACE_DRAIN: [f32; 4] = [-8.0, -3.5, 1.2, 10.0];
/// Extra stamina per second per unit of uphill slope.
const PACE_UP: [f32; 4] = [5.0, 11.0, 19.0, 28.0];

const GRAV: f32 = 0.16;
/// Slowest forward speed in the air.
pub const AIR_MIN: f32 = 1.1;
/// Horizontal margin around obstacles for feet.
const FOOT: f32 = 2.0;
/// Slope from which poles help (power hike).
pub const STEEP: f32 = 0.18;
/// Stamina needed to shake off a bonk.
const UNBONK: f32 = 50.0;
pub const RACE_CAP: u32 = 60 * 60 * 6;

impl Pace {
    pub fn idx(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Strategy {
    /// paces by terrain and stamina, hikes, grabs food
    Smart,
    /// always RUN
    Steady,
    /// always SPRINT
    Sprinter,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Ctrl {
    Human(usize),
    Cpu(Strategy, f32),
    /// Gerald, with his speed factor
    Goat(f32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RState {
    Ready,
    Run,
    Air,
    Tumble,
    /// face in the dirt
    Down,
    GetUp,
    /// bumped: head on a branch, a cow, a deer
    Stun,
    Glissade,
    Finished,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Intent {
    pub faster: bool,
    pub slower: bool,
    pub jump: bool,
    /// held: duck, brake, glissade
    pub down: bool,
    /// edges: grab / high five / pole, and pole
    pub a: bool,
    pub c: bool,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct RunStats {
    pub falls: u32,
    /// falls from going too fast (the rest are obstacles)
    pub wipeouts: u32,
    pub stuns: u32,
    pub clean: u32,
    pub combo: u32,
    pub best_combo: u32,
    pub fives: u32,
    pub aid: u32,
    pub bonks: u32,
    pub splashes: u32,
    pub limbo: u32,
    pub glissades: u32,
    pub style: i32,
    /// frames spent at each pace while running
    pub pace_frames: [u32; 4],
    pub hike_frames: u32,
}

#[derive(Clone, Copy, Default, Debug)]
struct Ai {
    decided: Option<usize>,
    ok: bool,
    lead: f32,
    pole_t: i32,
    pole_next: bool,
    grab_try: u32,
    five_try: u32,
}

#[derive(Clone, Debug)]
pub struct Runner {
    pub def: usize,
    pub ctrl: Ctrl,
    /// depth on the trail's top face (0 back .. 9 front)
    pub lane: i32,
    pub x: f32,
    pub v: f32,
    /// absolute height of the feet
    pub alt: f32,
    pub vz: f32,
    pub state: RState,
    pub st_t: i32,
    pub pace: Pace,
    pub stamina: f32,
    pub bonk: bool,
    pub duck: bool,
    pub hike: f32,
    last_pole: u8,
    pub hype: i32,
    pub muddy: i32,
    pub wet: i32,
    pub pickle: i32,
    pub wobble: f32,
    pub anim: f32,
    pub finished: Option<u32>,
    pub place: usize,
    pub grab_t: i32,
    pub five_t: i32,
    pub flash: i32,
    /// frames after a landing before the next jump
    pub land_t: i32,
    /// how far the camera keeps this runner from the left edge: less when
    /// fast, so you see further ahead (smoothed)
    pub look: f32,
    pub say: Option<(&'static str, i32)>,
    pub stats: RunStats,
    next_ob: usize,
    aid_done: u32,
    five_done: u32,
    in_water: bool,
    in_mud: bool,
    warned: bool,
    squeeze: bool,
    snack: i32,
    snack_cd: i32,
    ai: Ai,
    prev_pad: Pad,
    steps: u32,
    /// x when the shortcut started (Gerald)
    pub shortcut: bool,
}

impl Runner {
    pub fn goat(&self) -> bool {
        self.def == GERALD
    }
    pub fn human(&self) -> Option<usize> {
        match self.ctrl {
            Ctrl::Human(p) => Some(p),
            _ => None,
        }
    }
    pub fn grounded(&self) -> bool {
        self.state != RState::Air
    }
    pub fn hiking(&self, slope: f32) -> bool {
        self.hike > 0.08 && slope > STEEP && self.state == RState::Run
    }
}

#[derive(Clone, Debug)]
pub struct Animal {
    pub kind: AnimalKind,
    pub x: f32,
    pub home: f32,
    pub t: i32,
    /// marmot: 0 hidden, 1 up, 2 flattened; cow: 0 grazing, 1 shuffling
    pub state: u8,
    pub face: f32,
    pub rang: i32,
}

impl Animal {
    pub fn marmot_up(&self) -> bool {
        self.kind == AnimalKind::Marmot && self.state == 1 && self.t > 6
    }
}

/// A deer family crossing: one leap every `DEER_PERIOD` frames while runners are near.
#[derive(Clone, Debug)]
pub struct DeerX {
    pub x: f32,
    pub t: i32,
    pub active: bool,
    pub n: u32,
}

pub const DEER_PERIOD: i32 = 150;
/// Frames of the cycle in which the deer is over the trail.
pub const DEER_CROSS: (i32, i32) = (70, 84);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FxKind {
    Dust,
    Splash,
    Mud,
    Spore,
    Sweat,
    Star,
    Snow,
    Note,
}

#[derive(Clone, Copy, Debug)]
pub struct Fx {
    pub x: f32,
    pub alt: f32,
    pub vx: f32,
    pub vz: f32,
    pub life: i32,
    pub max: i32,
    pub kind: FxKind,
}

#[derive(Clone, Copy, Debug)]
pub struct Popup {
    pub x: f32,
    pub alt: f32,
    pub text: &'static str,
    pub color: u32,
    pub life: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Banner {
    pub text: &'static str,
    pub t: i32,
    pub dur: i32,
    pub color: u32,
    /// pop in place instead of sliding in
    pub pop: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Countdown,
    Race,
    Done,
}

pub const COUNTDOWN: i32 = 200;

pub struct Race {
    pub course: Course,
    pub runners: Vec<Runner>,
    pub animals: Vec<Animal>,
    pub deer: Vec<DeerX>,
    pub phase: Phase,
    pub phase_t: i32,
    /// frames since GO
    pub t: u32,
    pub fx: Vec<Fx>,
    pub popups: Vec<Popup>,
    /// one banner slot per view (player)
    pub banners: [Option<Banner>; 2],
    pub sfx: Vec<Sfx>,
    /// runner index shown in each view
    pub views: Vec<usize>,
    pub rng: Rng,
    heart_t: i32,
    /// demo / sim: stop when the leader is done
    pub cpu_only: bool,
    leader_done: Option<u32>,
    /// picks the rare sightings of this race (yeti, balloon, cow cloud)
    pub rare: u32,
}

const YELLOW: u32 = rgb(248, 232, 64);
const WHITE: u32 = rgb(248, 248, 248);
const RED: u32 = rgb(248, 72, 72);
const GREEN: u32 = rgb(120, 232, 96);
const CYAN: u32 = rgb(96, 224, 248);
const ORANGE: u32 = rgb(248, 152, 48);
const PINK: u32 = rgb(248, 120, 200);

const GOAT_TAUNTS: [&str; 8] = [
    "BAAAH!",
    "SEE YOU AT THE TOP!",
    "GOATS DON'T BONK!",
    "NICE FACEPLANT!",
    "TOO SLOW! BAAAH!",
    "I HAVE FOUR LEGS!",
    "SHORTCUT! BAAAH!",
    "MUNCH MUNCH",
];

impl Race {
    /// `defs`: runner index per racer (GERALD for the goat), with controls.
    pub fn new(stage: usize, racers: &[(usize, Ctrl)], seed: u64) -> Race {
        let course = Course::new(stage);
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let n = racers.len();
        let mut runners = Vec::new();
        for (k, &(def, ctrl)) in racers.iter().enumerate() {
            // humans in the front lanes, so they are never hidden
            let lane = match ctrl {
                Ctrl::Human(p) => 8 - p as i32 * 3,
                _ => 1 + (k as i32 * 2) % 6,
            };
            // a little queue behind the line, humans at the front
            let slot = match ctrl {
                Ctrl::Human(p) => p,
                _ => 2 + k,
            };
            let x = START_X - 6.0 - slot as f32 * 13.0;
            let _ = n;
            runners.push(Runner {
                def,
                ctrl,
                lane,
                x,
                v: 0.0,
                alt: course.height(x),
                vz: 0.0,
                state: RState::Ready,
                st_t: 0,
                pace: Pace::Jog,
                stamina: 100.0,
                bonk: false,
                duck: false,
                hike: 0.0,
                last_pole: 0,
                hype: 0,
                muddy: 0,
                wet: 0,
                pickle: 0,
                wobble: 0.0,
                anim: rng.range(0.0, 4.0),
                finished: None,
                place: 0,
                grab_t: 0,
                five_t: 0,
                flash: 0,
                land_t: 0,
                look: 96.0,
                say: None,
                stats: RunStats::default(),
                next_ob: 0,
                aid_done: 0,
                five_done: 0,
                in_water: false,
                in_mud: false,
                warned: false,
                squeeze: false,
                snack: 0,
                snack_cd: 600,
                ai: Ai::default(),
                prev_pad: Pad::default(),
                steps: 0,
                shortcut: false,
            });
        }
        let mut views: Vec<(usize, usize)> = runners.iter().enumerate().filter_map(|(i, r)| r.human().map(|p| (p, i))).collect();
        views.sort();
        let mut views: Vec<usize> = views.into_iter().map(|(_, i)| i).collect();
        let cpu_only = views.is_empty();
        if views.is_empty() {
            // demo: follow the first non-goat runner
            views.push(runners.iter().position(|r| !r.goat()).unwrap_or(0));
        }
        let animals = course
            .spawns
            .iter()
            .map(|s| Animal { kind: s.kind, x: s.x, home: s.x, t: rng.int(200), state: 0, face: if rng.f() < 0.5 { 1.0 } else { -1.0 }, rang: 0 })
            .collect();
        let deer = course.zones.iter().filter(|z| z.kind == ZoneKind::Deer).map(|z| DeerX { x: (z.x0 + z.x1) / 2.0, t: 0, active: false, n: 0 }).collect();
        let rare = rng.next();
        let mut r = Race {
            rare,
            course,
            runners,
            animals,
            deer,
            phase: Phase::Countdown,
            phase_t: 0,
            t: 0,
            fx: Vec::with_capacity(256),
            popups: Vec::with_capacity(32),
            banners: [None, None],
            sfx: Vec::new(),
            views,
            rng,
            heart_t: 0,
            cpu_only,
            leader_done: None,
        };
        if let Some(g) = r.runners.iter_mut().find(|r| r.goat()) {
            g.say = Some((GOAT_TAUNTS[1], 150));
        }
        r
    }

    pub fn finished(&self) -> bool {
        self.phase == Phase::Done
    }

    fn banner(&mut self, view: Option<usize>, text: &'static str, color: u32, pop: bool) {
        let b = Banner { text, t: 0, dur: if pop { 50 } else { 80 }, color, pop };
        match view {
            Some(v) => {
                if v < 2 {
                    self.banners[v] = Some(b)
                }
            }
            None => self.banners = [Some(b), Some(b)],
        }
    }

    /// Which view (if any) shows runner `i` as its player.
    fn view_of(&self, i: usize) -> Option<usize> {
        if self.cpu_only {
            return None;
        }
        self.views.iter().position(|&v| v == i)
    }

    fn near_view(&self, x: f32) -> bool {
        self.views.iter().any(|&v| (self.runners[v].x - x).abs() < 220.0)
    }

    fn cue(&mut self, s: Sfx, x: f32) {
        if self.near_view(x) && !self.sfx.contains(&s) {
            self.sfx.push(s);
        }
    }

    fn popup(&mut self, x: f32, alt: f32, text: &'static str, color: u32) {
        if self.near_view(x) {
            self.popups.push(Popup { x, alt, text, color, life: 50 });
        }
    }

    fn puff(&mut self, x: f32, alt: f32, kind: FxKind, n: usize, spread: f32) {
        if !self.near_view(x) || self.fx.len() > 240 {
            return;
        }
        for _ in 0..n {
            let vx = self.rng.range(-spread, spread);
            let vz = self.rng.range(0.4, 1.4) * if kind == FxKind::Splash { 1.4 } else { 1.0 };
            let life = 16 + self.rng.int(12);
            self.fx.push(Fx { x, alt, vx, vz, life, max: life, kind });
        }
    }

    fn goat_say(&mut self, k: usize, life: i32) {
        if let Some(g) = self.runners.iter_mut().find(|r| r.goat()) {
            g.say = Some((GOAT_TAUNTS[k], life));
        }
        if let Some(g) = self.runners.iter().find(|r| r.goat()) {
            let x = g.x;
            self.cue(Sfx::Baa, x);
        }
    }

    // ------------------------------------------------------------ update

    pub fn update(&mut self, pads: &[Pad; 2]) {
        self.phase_t += 1;
        for b in self.banners.iter_mut() {
            if let Some(bn) = b {
                bn.t += 1;
                if bn.t > bn.dur {
                    *b = None;
                }
            }
        }
        match self.phase {
            Phase::Countdown => {
                let t = self.phase_t;
                if t == 20 || t == 80 || t == 140 {
                    let s = ["3", "2", "1"][(t / 60) as usize];
                    self.banner(None, s, YELLOW, true);
                    self.sfx.push(Sfx::Blip);
                }
                if t >= COUNTDOWN {
                    self.phase = Phase::Race;
                    self.phase_t = 0;
                    self.banner(None, "GO!", GREEN, true);
                    self.sfx.push(Sfx::Horn);
                    for r in self.runners.iter_mut() {
                        r.state = RState::Run;
                    }
                }
                // humans may set their pace before the start
                for i in 0..self.runners.len() {
                    if let Ctrl::Human(p) = self.runners[i].ctrl {
                        let e = edge(pads[p], self.runners[i].prev_pad);
                        self.runners[i].prev_pad = pads[p];
                        self.change_pace(i, e.right, e.left);
                    }
                }
                self.ambient();
                return;
            }
            Phase::Race => {
                self.t += 1;
            }
            Phase::Done => {
                self.ambient();
                for i in 0..self.runners.len() {
                    if self.runners[i].state == RState::Finished {
                        self.finish_walk(i);
                    }
                }
                return;
            }
        }

        for i in 0..self.runners.len() {
            let intent = match self.runners[i].ctrl {
                Ctrl::Human(p) => {
                    let pad = pads[p];
                    let e = edge(pad, self.runners[i].prev_pad);
                    self.runners[i].prev_pad = pad;
                    Intent { faster: e.right, slower: e.left, jump: e.b, down: pad.down, a: e.a, c: e.c }
                }
                Ctrl::Cpu(st, skill) => self.ai(i, st, skill),
                Ctrl::Goat(_) => self.goat_ai(i),
            };
            self.step_runner(i, intent);
        }
        self.animals_update();
        self.deer_update();
        self.ambient();
        self.places();
        self.end_check();
    }

    fn ambient(&mut self) {
        for f in self.fx.iter_mut() {
            f.x += f.vx;
            f.alt += f.vz;
            f.vz -= match f.kind {
                FxKind::Spore | FxKind::Note | FxKind::Sweat => 0.01,
                FxKind::Star => 0.0,
                _ => 0.08,
            };
            f.life -= 1;
        }
        self.fx.retain(|f| f.life > 0);
        for p in self.popups.iter_mut() {
            p.alt += 0.5;
            p.life -= 1;
        }
        self.popups.retain(|p| p.life > 0);
        for r in self.runners.iter_mut() {
            if let Some((_, l)) = &mut r.say {
                *l -= 1;
                if *l <= 0 {
                    r.say = None;
                }
            }
        }
    }

    fn change_pace(&mut self, i: usize, faster: bool, slower: bool) {
        let r = &mut self.runners[i];
        if faster && r.pace != Pace::Sprint {
            if r.bonk {
                let (x, alt) = (r.x, r.alt);
                self.popup(x, alt + 50.0, "NO LEGS!", RED);
                return;
            }
            r.pace = PACES[r.pace.idx() + 1];
            if r.human().is_some() {
                self.sfx.push(Sfx::Blip);
            }
        } else if slower && r.pace != Pace::Walk {
            r.pace = PACES[r.pace.idx() - 1];
            if r.human().is_some() {
                self.sfx.push(Sfx::Blip);
            }
        }
    }

    /// Target speed on the current ground, before zone effects.
    pub fn target_speed(&self, i: usize) -> f32 {
        let r = &self.runners[i];
        let c = &self.course;
        let s = c.slope(r.x);
        let g = c.ground(r.x);
        let up = s.max(0.0);
        let down = (-s).clamp(0.0, 0.45);
        if r.goat() {
            let fac = match r.ctrl {
                Ctrl::Goat(f) => f,
                _ => 1.0,
            };
            let mut v = 1.32 * fac * (1.0 - 0.62 * up) * (1.0 + 0.25 * down);
            if g == Ground::Mud || g == Ground::Water {
                v *= 0.85;
            }
            return v;
        }
        let d = &RUNNERS[r.def];
        let mut v = if r.bonk { 0.5 } else { PACE_V[r.pace.idx()] * d.speed };
        let up_pen = 1.55 - 0.12 * d.climb as f32;
        v *= (1.0 - up_pen * up).max(0.22);
        v *= 1.0 + (0.45 + 0.13 * d.descend as f32) * down;
        v *= match g {
            Ground::Mud => 0.72,
            Ground::Snow if s > 0.0 => 0.85,
            Ground::Snow => 0.92,
            Ground::Scree if s > 0.0 => 0.82,
            Ground::Scree => 1.08,
            Ground::Water => 0.45,
            Ground::Road => 1.03,
            Ground::Rock => 0.97,
            _ => 1.0,
        };
        if r.hike > 0.08 && up > STEEP && !r.bonk {
            // power hiking: the poles set the speed, not the pace
            v = (0.42 + 0.72 * r.hike.min(1.0)) * (1.0 - 0.3 * up) * (0.84 + 0.06 * d.climb as f32);
        }
        if r.duck {
            v *= 0.68;
        }
        if r.hype > 0 {
            v *= 1.14;
        }
        v
    }

    fn stamina_use(&self, i: usize) -> f32 {
        let r = &self.runners[i];
        let s = self.course.slope(r.x);
        let up = s.max(0.0);
        let down = (-s).clamp(0.0, 0.4);
        let d = &RUNNERS[r.def];
        let p = if r.bonk { 0 } else { r.pace.idx() };
        let mut use_ = if r.hiking(s) {
            1.5 + 9.0 * up
        } else {
            PACE_DRAIN[p] + PACE_UP[p] * up - 7.0 * down
        };
        if r.duck {
            use_ = use_.min(0.5);
        }
        if r.state == RState::Glissade {
            use_ = -7.0;
        }
        if r.bonk {
            // bonked legs come back slowly
            use_ = -3.2 - 0.25 * d.endurance as f32;
        } else if use_ > 0.0 {
            use_ *= 1.35 - 0.12 * d.endurance as f32;
        } else {
            use_ *= 0.8 + 0.08 * d.endurance as f32;
        }
        if r.hype > 0 {
            use_ = use_.min(0.0);
        }
        use_ / 60.0
    }

    fn step_runner(&mut self, i: usize, it: Intent) {
        let human_view = self.view_of(i);
        {
            let r = &mut self.runners[i];
            r.st_t += 1;
            r.hype = (r.hype - 1).max(0);
            r.muddy = (r.muddy - 1).max(0);
            r.wet = (r.wet - 1).max(0);
            r.pickle = (r.pickle - 1).max(0);
            r.grab_t = (r.grab_t - 1).max(0);
            r.five_t = (r.five_t - 1).max(0);
            r.flash = (r.flash - 1).max(0);
            r.land_t = (r.land_t - 1).max(0);
            let want = (96.0 - (r.v - 1.4) * 40.0).clamp(60.0, 100.0);
            r.look += (want - r.look) * 0.03;
            r.hike *= 0.97;
            if r.hike < 0.02 {
                r.hike = 0.0;
            }
        }
        if self.runners[i].state == RState::Finished {
            self.finish_walk(i);
            return;
        }
        if !self.runners[i].goat() {
            self.change_pace(i, it.faster, it.slower);
        }
        let x = self.runners[i].x;
        let s = self.course.slope(x);
        let g = self.course.ground(x);
        let state = self.runners[i].state;

        // ---- actions
        let can_act = matches!(state, RState::Run | RState::Glissade);
        if can_act {
            self.runners[i].duck = it.down && state == RState::Run;
            if it.a || it.c {
                self.poles_and_grabs(i, it, s);
            }
            if it.jump {
                self.jump(i);
            }
        } else {
            self.runners[i].duck = false;
        }
        // glissade: sit down on a snowy descent
        let state = self.runners[i].state;
        if state == RState::Run && it.down && g == Ground::Snow && s < -0.12 && !self.runners[i].goat() {
            let r = &mut self.runners[i];
            r.state = RState::Glissade;
            r.st_t = 0;
            r.duck = false;
            r.stats.glissades += 1;
            r.stats.style += 30;
            let alt = r.alt;
            self.popup(x, alt + 46.0, "WHEEE!", CYAN);
            self.cue(Sfx::Slide, x);
        } else if state == RState::Glissade && (!it.down || g != Ground::Snow || s > -0.04) {
            let r = &mut self.runners[i];
            r.state = RState::Run;
            r.st_t = 0;
        }

        // ---- speed
        let state = self.runners[i].state;
        let mut target = self.target_speed(i);
        match state {
            RState::Glissade => target = (1.7 + 3.2 * (-s)).min(3.0),
            RState::Tumble => target = 0.35,
            RState::Down | RState::GetUp | RState::Ready => target = 0.0,
            RState::Stun => target = 0.0,
            _ => {}
        }
        if self.runners[i].squeeze {
            target = target.min(0.5);
        }
        {
            let r = &mut self.runners[i];
            if state != RState::Air {
                let acc = if target > r.v { if state == RState::Glissade { 0.04 } else { 0.05 } } else { 0.08 };
                r.v += (target - r.v) * acc;
            }
        }
        // stamina
        if !self.runners[i].goat() && matches!(state, RState::Run | RState::Air | RState::Glissade | RState::Tumble) {
            let use_ = self.stamina_use(i);
            let r = &mut self.runners[i];
            r.stamina = (r.stamina - use_).clamp(0.0, 100.0);
            if r.stamina <= 0.0 && !r.bonk {
                r.bonk = true;
                r.pace = Pace::Walk;
                r.stats.bonks += 1;
                let alt = r.alt;
                self.popup(x, alt + 50.0, "BONK!", RED);
                self.banner(human_view, "BONK!", RED, false);
                if human_view.is_some() && self.goat_near(x, 300.0) {
                    self.goat_say(2, 120);
                }
            } else if r.bonk && r.stamina >= UNBONK {
                r.bonk = false;
                r.pace = Pace::Jog;
                let alt = r.alt;
                self.popup(x, alt + 50.0, "LEGS ARE BACK!", GREEN);
            }
            let r = &self.runners[i];
            if r.state == RState::Run {
                let p = r.pace.idx();
                let hik = r.hiking(s);
                let r = &mut self.runners[i];
                r.stats.pace_frames[p] += 1;
                if hik {
                    r.stats.hike_frames += 1;
                }
            }
        }

        // ---- move
        let zone_f = if self.course.zone(x, ZoneKind::Switchback).is_some() && !self.runners[i].goat() { 0.62 } else { 1.0 };
        {
            let r = &mut self.runners[i];
            // a jump always carries you forward a bit, even from a slow hike
            let vx = if r.state == RState::Air { r.v.max(AIR_MIN) } else { r.v };
            r.x += vx * zone_f;
            if r.goat() {
                r.shortcut = zone_f == 1.0 && self.course.zone(r.x, ZoneKind::Switchback).is_some();
            }
            let stride = if r.state == RState::Glissade { 0.0 } else { r.v * zone_f * 0.11 + if r.hiking(s) { 0.05 } else { 0.0 } };
            let before = r.anim as u32;
            r.anim += stride;
            if r.anim as u32 != before && r.state == RState::Run {
                r.steps += 1;
            }
        }
        // footsteps
        if human_view.is_some() && self.runners[i].state == RState::Run && self.runners[i].steps.is_multiple_of(2) && self.runners[i].anim.fract() < self.runners[i].v * 0.11 {
            self.sfx.push(Sfx::Step);
        }

        // ---- vertical
        let nx = self.runners[i].x;
        let ground_h = self.course.height(nx);
        if self.runners[i].state == RState::Air {
            let r = &mut self.runners[i];
            r.vz -= GRAV;
            r.alt += r.vz;
            if r.alt <= ground_h {
                r.alt = ground_h;
                r.vz = 0.0;
                r.state = RState::Run;
                r.st_t = 0;
                self.land(i);
            }
        } else {
            self.runners[i].alt = ground_h;
        }

        self.wobble(i, s);
        self.surface(i);
        self.collide(i);
        self.timers(i);
        self.heart(i);

        let r = &self.runners[i];
        if r.x >= self.course.finish && r.finished.is_none() {
            let t = self.t;
            let r = &mut self.runners[i];
            r.finished = Some(t);
            r.state = RState::Finished;
            r.st_t = 0;
            r.duck = false;
            let goat = r.goat();
            let view = self.view_of(i);
            let first = self.runners.iter().filter(|q| q.finished.is_some()).count() == 1;
            if view.is_some() {
                self.banner(view, if first { "WINNER!" } else { "FINISH!" }, if first { YELLOW } else { WHITE }, false);
                self.sfx.push(Sfx::Horn);
                self.sfx.push(Sfx::Crowd);
            } else if goat && first && !self.cpu_only {
                for v in 0..self.views.len() {
                    if self.runners[self.views[v]].finished.is_none() {
                        self.banner(Some(v), "GERALD WINS?!", RED, false);
                    }
                }
                self.goat_say(0, 120);
            }
        }
    }

    fn poles_and_grabs(&mut self, i: usize, it: Intent, s: f32) {
        let x = self.runners[i].x;
        let alt = self.runners[i].alt;
        let is_view = self.view_of(i).is_some();
        if it.a {
            // aid station: grab what the volunteer holds out
            for k in 0..self.course.aids.len() {
                let aid = self.course.aids[k];
                let d = (x - aid.x).abs();
                let bit = 1u32 << k;
                if d < 15.0 && self.runners[i].aid_done & bit == 0 {
                    let item = aid.item;
                    let r = &mut self.runners[i];
                    r.aid_done |= bit;
                    r.stamina = (r.stamina + item.stamina()).min(100.0);
                    r.hype = r.hype.max(item.hype());
                    r.grab_t = 20;
                    r.stats.aid += 1;
                    r.stats.style += 20;
                    if r.bonk && r.stamina >= UNBONK {
                        r.bonk = false;
                        r.pace = Pace::Jog;
                    }
                    if item == Item::Pickle {
                        r.pickle = 110;
                    }
                    self.popup(x, alt + 50.0, item.name(), if item == Item::Pickle { GREEN } else { YELLOW });
                    if is_view {
                        self.sfx.push(Sfx::Gulp);
                        self.sfx.push(Sfx::Ding);
                        if item == Item::Pickle {
                            let v = self.view_of(i);
                            self.banner(v, "PICKLE POWER!", GREEN, false);
                        }
                    }
                    return;
                }
                if d < 40.0 && self.runners[i].aid_done & bit == 0 && is_view {
                    self.popup(x, alt + 50.0, if x < aid.x { "TOO EARLY!" } else { "TOO LATE!" }, ORANGE);
                }
            }
            // cowbell crowd: high five for HYPE
            for k in 0..self.course.crowds.len() {
                let cr = self.course.crowds[k];
                let bit = 1u32 << (k % 32);
                if (x - cr.x).abs() < 18.0 && self.runners[i].five_done & bit == 0 {
                    let r = &mut self.runners[i];
                    r.five_done |= bit;
                    r.hype = r.hype.max(150);
                    r.five_t = 16;
                    r.stats.fives += 1;
                    r.stats.style += 50;
                    self.popup(x, alt + 50.0, "HIGH FIVE!", PINK);
                    if is_view {
                        self.sfx.push(Sfx::Crowd);
                        self.sfx.push(Sfx::Cowbell);
                    }
                    return;
                }
            }
        }
        // poles: alternate A and C on steep climbs
        if s > STEEP && self.runners[i].state == RState::Run && !self.runners[i].bonk {
            let b = if it.a { 1 } else { 2 };
            let r = &mut self.runners[i];
            let alternate = r.last_pole != b;
            let gain = if alternate { 0.3 } else { 0.04 };
            r.hike = (r.hike + gain).min(1.0);
            r.last_pole = b;
            if alternate && is_view {
                self.sfx.push(Sfx::Step);
            }
        }
    }

    fn jump(&mut self, i: usize) {
        let r = &mut self.runners[i];
        if !matches!(r.state, RState::Run | RState::Glissade) || r.land_t > 0 {
            return;
        }
        let ag = if r.goat() { 6.0 } else { RUNNERS[r.def].agility as f32 };
        r.state = RState::Air;
        r.st_t = 0;
        r.vz = if r.goat() { 3.3 } else { 2.5 + 0.08 * ag };
        r.duck = false;
        if !r.goat() {
            r.stamina = (r.stamina - 1.0).max(0.0);
        }
        let x = r.x;
        if self.view_of(i).is_some() {
            self.sfx.push(Sfx::Jump);
        }
        let a = self.runners[i].alt;
        self.puff(x, a, FxKind::Dust, 3, 0.6);
    }

    fn land(&mut self, i: usize) {
        // landing costs a little speed, and you need a moment before the next jump
        if !self.runners[i].goat() {
            self.runners[i].land_t = 9;
            self.runners[i].v *= 0.93;
        }
        let x = self.runners[i].x;
        let alt = self.runners[i].alt;
        match self.course.ground(x) {
            Ground::Water => {
                self.puff(x, alt, FxKind::Splash, 10, 1.0);
                self.cue(Sfx::Splash, x);
            }
            Ground::Mud => {
                self.runners[i].muddy = self.runners[i].muddy.max(200);
                self.puff(x, alt, FxKind::Mud, 8, 1.0);
                self.popup(x, alt + 46.0, "SPLAT!", ORANGE);
            }
            Ground::Snow => self.puff(x, alt, FxKind::Snow, 5, 0.8),
            _ => self.puff(x, alt, FxKind::Dust, 4, 0.8),
        }
    }

    /// Going too fast on technical ground makes you wobble, then trip.
    fn wobble(&mut self, i: usize, s: f32) {
        let r = &self.runners[i];
        if r.goat() || r.state != RState::Run {
            let r = &mut self.runners[i];
            r.wobble = (r.wobble - 0.03).max(0.0);
            return;
        }
        let d = &RUNNERS[r.def];
        let g = self.course.ground(r.x);
        let thr = match g {
            Ground::Scree if s < -0.12 => 1.65 + 0.1 * d.descend as f32,
            Ground::Rock if s < -0.15 => 1.8 + 0.1 * d.descend as f32,
            Ground::Snow if s < -0.15 => 1.75 + 0.1 * d.descend as f32,
            Ground::Mud => 1.22 + 0.05 * d.agility as f32,
            _ => 99.0,
        };
        let mut w = r.wobble;
        if r.v > thr && !r.duck {
            w += (r.v - thr) * 0.07 + 0.008;
        } else {
            w -= 0.025;
        }
        w = w.max(0.0);
        let (x, alt) = (r.x, r.alt);
        let warned = r.warned;
        self.runners[i].wobble = w;
        if w > 0.3 && !warned {
            self.runners[i].warned = true;
            if self.view_of(i).is_some() {
                self.popup(x, alt + 50.0, if g == Ground::Mud { "SLIPPERY!" } else { "TOO FAST!" }, ORANGE);
            }
        }
        if w < 0.1 {
            self.runners[i].warned = false;
        }
        if w >= 1.0 {
            self.runners[i].stats.wipeouts += 1;
            self.trip(i, if g == Ground::Mud { "FACEPLANT!" } else { "WIPEOUT!" });
        }
    }

    fn surface(&mut self, i: usize) {
        let r = &self.runners[i];
        let x = r.x;
        let alt = r.alt;
        let g = self.course.ground(x);
        let grounded = r.grounded();
        let water = grounded && g == Ground::Water;
        let mud = grounded && g == Ground::Mud;
        if water && !r.in_water && !r.goat() {
            self.runners[i].wet = 240;
            self.runners[i].stats.splashes += 1;
            self.puff(x, alt, FxKind::Splash, 12, 1.2);
            self.popup(x, alt + 46.0, "SPLASH!", CYAN);
            self.cue(Sfx::Splash, x);
            self.runners[i].stats.combo = 0;
        }
        if mud && self.runners[i].v > 0.6 && self.t.is_multiple_of(9) {
            self.puff(x, alt, FxKind::Mud, 1, 0.6);
        }
        if grounded && g == Ground::Snow && self.runners[i].state == RState::Glissade && self.t.is_multiple_of(3) {
            self.puff(x - 6.0, alt, FxKind::Snow, 1, 0.5);
        }
        let r = &mut self.runners[i];
        r.in_water = water;
        r.in_mud = mud;
        if mud {
            r.muddy = r.muddy.max(60);
        }
    }

    fn trip(&mut self, i: usize, text: &'static str) {
        let r = &mut self.runners[i];
        if matches!(r.state, RState::Tumble | RState::Down | RState::GetUp | RState::Finished) {
            return;
        }
        r.state = RState::Tumble;
        r.st_t = 0;
        r.vz = 0.0;
        r.duck = false;
        r.wobble = 0.0;
        r.stats.falls += 1;
        r.stats.combo = 0;
        r.hike = 0.0;
        r.stamina = (r.stamina - 4.0).max(0.0);
        let (x, alt) = (r.x, self.course.height(r.x));
        r.alt = alt;
        let mud = self.course.ground(x) == Ground::Mud;
        if mud || self.rng.f() < 0.3 {
            self.runners[i].muddy = 900;
        }
        let view = self.view_of(i);
        self.puff(x, alt, if mud { FxKind::Mud } else { FxKind::Dust }, 10, 1.2);
        self.cue(Sfx::Trip, x);
        if view.is_some() {
            self.banner(view, text, ORANGE, false);
            if self.goat_near(x, 260.0) {
                self.goat_say(3, 110);
            }
        } else {
            self.popup(x, alt + 46.0, "OOF!", ORANGE);
        }
    }

    fn stun(&mut self, i: usize, text: &'static str, bounce: f32) {
        let r = &mut self.runners[i];
        if matches!(r.state, RState::Tumble | RState::Down | RState::GetUp | RState::Stun | RState::Finished) {
            return;
        }
        if r.state == RState::Air {
            r.alt = self.course.height(r.x);
            r.vz = 0.0;
        }
        r.state = RState::Stun;
        r.stats.stuns += 1;
        r.st_t = 0;
        r.v = bounce;
        r.duck = false;
        r.stats.combo = 0;
        let (x, alt) = (r.x, r.alt);
        self.popup(x, alt + 50.0, text, ORANGE);
        self.puff(x, alt + 24.0, FxKind::Star, 5, 0.9);
        self.cue(Sfx::Trip, x);
    }

    fn goat_near(&self, x: f32, d: f32) -> bool {
        self.runners.iter().any(|r| r.goat() && (r.x - x).abs() < d)
    }

    fn timers(&mut self, i: usize) {
        let ag = RUNNERS[self.runners[i].def.min(3)].agility;
        let r = &mut self.runners[i];
        match r.state {
            RState::Tumble => {
                // rolling: keep low on the ground
                if r.st_t > 46 - ag * 4 {
                    r.state = RState::Down;
                    r.st_t = 0;
                }
            }
            RState::Down => {
                if r.st_t > 34 {
                    r.state = RState::GetUp;
                    r.st_t = 0;
                }
            }
            RState::GetUp => {
                if r.st_t > 14 {
                    r.state = RState::Run;
                    r.st_t = 0;
                }
            }
            RState::Stun
                if r.st_t > 32 => {
                    r.state = RState::Run;
                    r.st_t = 0;
                }
            _ => {}
        }
    }

    fn heart(&mut self, i: usize) {
        if self.view_of(i).is_none() {
            return;
        }
        let r = &self.runners[i];
        if r.stamina < 20.0 && r.state != RState::Finished {
            self.heart_t -= 1;
            if self.heart_t <= 0 {
                self.heart_t = if r.stamina < 8.0 { 28 } else { 44 };
                self.sfx.push(Sfx::Heart);
                let (x, alt) = (r.x, r.alt);
                self.puff(x, alt + 26.0, FxKind::Sweat, 1, 0.4);
            }
        }
    }

    // ------------------------------------------------------------ collisions

    fn collide(&mut self, i: usize) {
        let r = &self.runners[i];
        if r.goat() || matches!(r.state, RState::Tumble | RState::Down | RState::GetUp | RState::Finished | RState::Ready) {
            self.skip_passed(i);
            return;
        }
        let x = r.x;
        let foot = r.alt - self.course.height(x);
        let head = foot + if r.duck || r.state == RState::Glissade { 12.0 } else { 26.0 };
        let mut k = r.next_ob;
        while k < self.course.obs.len() {
            let o = self.course.obs[k];
            let half = o.kind.width() / 2.0;
            if o.x - half - FOOT > x {
                break;
            }
            if o.x + half + FOOT < x {
                // passed it cleanly
                self.runners[i].next_ob = k + 1;
                self.cleared(i, o.kind);
                k += 1;
                continue;
            }
            let hit = if o.kind.overhead() { head > OVERHEAD_CLEAR } else { foot < o.kind.height() - 1.0 };
            if hit {
                self.runners[i].next_ob = k + 1;
                self.hit(i, o);
                return;
            }
            k += 1;
        }
        // standing marmots and cows
        for a in 0..self.animals.len() {
            let an = &self.animals[a];
            let d = (an.x - x).abs();
            match an.kind {
                AnimalKind::Marmot => {
                    if an.marmot_up() && d < 8.0 && foot < 9.0 {
                        let an = &mut self.animals[a];
                        an.state = 2;
                        an.t = 0;
                        self.stun(i, "EEEP!", 0.3);
                        self.cue(Sfx::Whistle, x);
                        return;
                    }
                }
                AnimalKind::Cow => {
                    let r = &self.runners[i];
                    if d < 16.0 && foot < 18.0 {
                        if r.v > 0.95 && !r.squeeze {
                            self.stun(i, "MOO!", -0.7);
                            self.cue(Sfx::Moo, x);
                            self.cue(Sfx::Cowbell, x);
                            self.animals[a].rang = 30;
                        } else if !self.runners[i].squeeze {
                            self.runners[i].squeeze = true;
                            let alt = self.runners[i].alt;
                            self.popup(x, alt + 50.0, "EXCUSE ME!", WHITE);
                            self.animals[a].rang = 30;
                            self.cue(Sfx::Cowbell, x);
                        }
                    } else if d >= 16.0 && self.runners[i].squeeze && x > an.x {
                        self.runners[i].squeeze = false;
                    }
                }
            }
        }
        // deer crossings: duck and the deer leaps over you
        for d in 0..self.deer.len() {
            let dx = &self.deer[d];
            if dx.active && (dx.x - x).abs() < 12.0 && (DEER_CROSS.0..DEER_CROSS.1).contains(&dx.t) {
                let r = &self.runners[i];
                if !(r.duck && r.grounded()) && r.state != RState::Stun {
                    self.stun(i, "OH DEER!", 0.0);
                }
            }
        }
    }

    /// Obstacles behind a runner who can't hit them (Gerald, a tumbler).
    fn skip_passed(&mut self, i: usize) {
        let x = self.runners[i].x;
        let r = &mut self.runners[i];
        while r.next_ob < self.course.obs.len() {
            let o = self.course.obs[r.next_ob];
            if o.x + o.kind.width() / 2.0 + FOOT < x {
                r.next_ob += 1;
            } else {
                break;
            }
        }
    }

    fn cleared(&mut self, i: usize, kind: ObKind) {
        let r = &mut self.runners[i];
        let x = r.x;
        let alt = r.alt;
        if kind.overhead() {
            r.stats.limbo += 1;
            r.stats.style += 15;
            r.stats.combo += 1;
            r.stats.best_combo = r.stats.best_combo.max(r.stats.combo);
            if self.view_of(i).is_some() {
                self.popup(x, alt + 46.0, "LIMBO!", GREEN);
            }
        } else if r.state == RState::Air {
            r.stats.clean += 1;
            r.stats.combo += 1;
            r.stats.best_combo = r.stats.best_combo.max(r.stats.combo);
            r.stats.style += 10 * r.stats.combo.min(10) as i32;
            let c = r.stats.combo;
            if self.view_of(i).is_some() {
                let txt = match c {
                    1 => "NICE!",
                    2 => "NICE! X2",
                    3 => "NICE! X3",
                    4 => "SMOOTH! X4",
                    5 => "SMOOTH! X5",
                    6..=7 => "GOATLIKE!",
                    _ => "TRAIL GOD!",
                };
                self.popup(x, alt + 50.0, txt, GREEN);
            }
        }
    }

    fn hit(&mut self, i: usize, o: Obstacle) {
        let x = self.runners[i].x;
        let alt = self.runners[i].alt;
        match o.kind {
            ObKind::Mushroom => {
                self.runners[i].v *= 0.8;
                self.runners[i].stats.combo = 0;
                self.popup(x, alt + 46.0, "SQUISH!", PINK);
                self.puff(o.x, alt + 4.0, FxKind::Spore, 8, 0.6);
            }
            ObKind::Marmot => {
                self.stun(i, "EEEP!", 0.3);
                self.cue(Sfx::Whistle, x);
            }
            ObKind::Branch => self.stun(i, "THWACK!", 0.1),
            ObKind::Trunk => self.stun(i, "TIMBER!", 0.1),
            _ => {
                let txt = if self.rng.f() < 0.5 { "FACEPLANT!" } else { "WIPEOUT!" };
                self.trip(i, txt)
            }
        }
    }

    // ------------------------------------------------------------ world

    fn animals_update(&mut self) {
        for a in 0..self.animals.len() {
            let ax = self.animals[a].x;
            // nearest runner coming up behind it
            let near = self.runners.iter().any(|r| r.x < ax - 18.0 && r.x > ax - 95.0 && r.finished.is_none());
            let close = self.runners.iter().any(|r| (r.x - ax).abs() < 70.0);
            let an = &mut self.animals[a];
            an.t += 1;
            an.rang = (an.rang - 1).max(0);
            match an.kind {
                AnimalKind::Marmot => match an.state {
                    0 => {
                        if near && an.t > 60 {
                            an.state = 1;
                            an.t = 0;
                            self.cue(Sfx::Whistle, ax);
                        }
                    }
                    1 => {
                        if an.t > 100 {
                            an.state = 0;
                            an.t = 0;
                        }
                    }
                    _ => {
                        if an.t > 50 {
                            an.state = 0;
                            an.t = 0;
                        }
                    }
                },
                AnimalKind::Cow => {
                    // shuffles around its spot, bell clanking
                    if an.state == 0 && an.t > 200 && self.rng.f() < 0.01 {
                        an.state = 1;
                        an.t = 0;
                        an.face = if an.x > an.home { -1.0 } else { 1.0 };
                    }
                    if an.state == 1 {
                        an.x += an.face * 0.12;
                        if an.t > 90 {
                            an.state = 0;
                            an.t = 0;
                        }
                    }
                    if close && an.t % 50 == 0 {
                        an.rang = 20;
                        self.cue(Sfx::Cowbell, ax);
                    }
                }
            }
        }
    }

    fn deer_update(&mut self) {
        for d in 0..self.deer.len() {
            let x = self.deer[d].x;
            let near = self.runners.iter().any(|r| r.x > x - 260.0 && r.x < x + 40.0);
            let dx = &mut self.deer[d];
            if dx.active {
                dx.t += 1;
                if dx.t >= DEER_PERIOD {
                    dx.t = 0;
                    dx.n += 1;
                    if !near {
                        dx.active = false;
                    }
                }
            } else if near {
                dx.active = true;
                dx.t = 0;
            }
        }
    }

    fn places(&mut self) {
        let mut order: Vec<usize> = (0..self.runners.len()).collect();
        let key = |r: &Runner| match r.finished {
            Some(t) => (0, t as f32),
            None => (1, -r.x),
        };
        order.sort_by(|&a, &b| {
            let (ka, kb) = (key(&self.runners[a]), key(&self.runners[b]));
            ka.0.cmp(&kb.0).then(ka.1.partial_cmp(&kb.1).unwrap())
        });
        // Gerald heckles when he passes a player
        let goat = self.runners.iter().position(|r| r.goat());
        for (p, &i) in order.iter().enumerate() {
            let old = self.runners[i].place;
            self.runners[i].place = p + 1;
            if let (Some(g), Some(view)) = (goat, self.view_of(i)) {
                let gp = self.runners[g].place;
                if old != 0 && old < p + 1 && gp == p && self.runners[g].finished.is_none() && self.t > 60 {
                    self.banner(Some(view), "BAAAH!", WHITE, true);
                    let k = [0, 4, 5][(self.t / 7 % 3) as usize];
                    self.goat_say(k, 100);
                }
            }
        }
    }

    fn end_check(&mut self) {
        let humans_done = self.views.iter().all(|&v| self.runners[v].finished.is_some());
        if self.leader_done.is_none() && self.runners.iter().any(|r| r.finished.is_some()) {
            self.leader_done = Some(self.t);
        }
        let all_done = self.runners.iter().all(|r| r.finished.is_some());
        let done = if self.cpu_only {
            all_done || self.leader_done.map(|t| self.t > t + 60 * 20).unwrap_or(false)
        } else {
            humans_done && (all_done || self.phase_t > 0 && self.views.iter().all(|&v| self.runners[v].st_t > 150))
        };
        if done || self.t >= RACE_CAP {
            self.phase = Phase::Done;
            self.phase_t = 0;
        }
    }

    /// After the line: jog out and celebrate.
    fn finish_walk(&mut self, i: usize) {
        let fin = self.course.finish;
        let r = &mut self.runners[i];
        r.st_t += 1;
        r.v = (r.v * 0.96).max(if r.x < fin + 40.0 + r.lane as f32 * 6.0 { 0.4 } else { 0.0 });
        if r.x > fin + 70.0 + r.lane as f32 * 8.0 {
            r.v = 0.0;
        }
        r.x += r.v;
        r.anim += r.v * 0.11;
        r.alt = self.course.height(r.x);
    }

    /// Finishing time (frames) per runner; unfinished runners get a projected time.
    pub fn result_time(&self, i: usize) -> u32 {
        let r = &self.runners[i];
        match r.finished {
            Some(t) => t,
            None => {
                let done = (r.x - START_X).max(1.0);
                let avg = (done / self.t.max(1) as f32).max(0.6);
                self.t + ((self.course.finish - r.x).max(0.0) / avg) as u32
            }
        }
    }

    /// Runner indexes by finishing order (projected for the unfinished).
    pub fn ranking(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.runners.len()).collect();
        order.sort_by_key(|&i| (self.result_time(i), i));
        order
    }

    // ------------------------------------------------------------ CPU

    fn ai(&mut self, i: usize, st: Strategy, skill: f32) -> Intent {
        let mut it = Intent::default();
        let r = self.runners[i].clone();
        if !matches!(r.state, RState::Run | RState::Air | RState::Glissade) {
            return it;
        }
        let x = r.x;
        let s = self.course.slope(x);
        let g = self.course.ground(x);
        let ahead = (self.course.height(x + 120.0) - self.course.height(x)) / 120.0;
        let remain = self.course.finish - x;
        let d = &RUNNERS[r.def];
        let v = r.v.max(0.5);

        // ---- pace
        let want = match st {
            Strategy::Sprinter => Pace::Sprint,
            Strategy::Steady => Pace::Run,
            Strategy::Smart => {
                let st_ = r.stamina;
                // enough legs left to sprint home?
                let home = remain < 300.0 + 700.0 * skill && st_ > remain * 0.09 + 5.0;
                let tech = matches!(g, Ground::Scree | Ground::Rock | Ground::Mud);
                if r.bonk {
                    Pace::Walk
                } else if home {
                    Pace::Sprint
                } else if s > STEEP {
                    // the poles do the work; keep the pace sensible for when they stop
                    if st_ > 25.0 { Pace::Run } else if st_ > 10.0 { Pace::Jog } else { Pace::Walk }
                } else if s > 0.08 || ahead > 0.2 {
                    if st_ > 45.0 { Pace::Run } else if st_ > 12.0 { Pace::Jog } else { Pace::Walk }
                } else if s < -0.1 {
                    if !tech && st_ > 40.0 && d.descend >= 3 { Pace::Sprint } else { Pace::Run }
                } else if g == Ground::Mud {
                    Pace::Jog
                } else if st_ > 20.0 {
                    Pace::Run
                } else {
                    Pace::Jog
                }
            }
        };
        if want > r.pace {
            it.faster = true;
        } else if want < r.pace {
            it.slower = true;
        }

        // ---- poles on steep climbs
        if st == Strategy::Smart && s > STEEP && r.state == RState::Run && r.stamina > 6.0 {
            let rate = 26 - (skill * 12.0) as i32;
            let jitter = self.rng.int(5) - 2;
            let ai = &mut self.runners[i].ai;
            ai.pole_t -= 1;
            if ai.pole_t <= 0 {
                ai.pole_t = rate + jitter;
                ai.pole_next = !ai.pole_next;
                if ai.pole_next {
                    it.c = true;
                } else {
                    it.a = true;
                }
            }
        }

        // ---- obstacles
        let obs = &self.course.obs;
        let mut k = r.next_ob;
        while k < obs.len() && obs[k].x + obs[k].kind.width() / 2.0 + FOOT + 1.0 < x {
            k += 1;
        }
        if k < obs.len() {
            let o = obs[k];
            let dist = o.x - o.kind.width() / 2.0 - x;
            if self.runners[i].ai.decided != Some(k) && dist < v * 40.0 {
                let ok = self.rng.f() < 0.8 + 0.19 * skill;
                let lead = self.rng.range(-4.0, 4.0) * (1.2 - skill);
                let ai = &mut self.runners[i].ai;
                ai.decided = Some(k);
                ai.ok = ok;
                ai.lead = lead;
            }
            let ai = self.runners[i].ai;
            if ai.decided == Some(k) && ai.ok {
                if o.kind.overhead() {
                    if dist < 26.0 + ai.lead {
                        it.down = true;
                    }
                } else {
                    // take off so the top of the arc is over the obstacle
                    let va = v.max(AIR_MIN);
                    let vz = 2.5 + 0.08 * d.agility as f32;
                    let take = va * (vz - s.max(0.0) * va) / GRAV + ai.lead;
                    if o.x - x <= take && r.state != RState::Air {
                        it.jump = true;
                    }
                }
            }
        }
        // marmots popping up: jump them; cows: squeeze past slowly
        for a in 0..self.animals.len() {
            let (kind, ax, up) = (self.animals[a].kind, self.animals[a].x, self.animals[a].marmot_up());
            let dist = ax - x;
            if kind == AnimalKind::Marmot && up && dist > 0.0 && dist < v * 14.0 && r.state == RState::Run && self.rng.f() < 0.2 + 0.7 * skill {
                it.jump = true;
            }
            if kind == AnimalKind::Cow && dist > -12.0 && dist < 80.0 && skill > 0.35 {
                it.down = true;
                if r.pace > Pace::Walk {
                    it.slower = true;
                    it.faster = false;
                }
            }
        }
        // deer: duck through the crossing when one is coming
        for k in 0..self.deer.len() {
            let (dx, active, t) = (self.deer[k].x, self.deer[k].active, self.deer[k].t);
            let dist = dx - x;
            if active && dist > -10.0 && dist < 40.0 && t > DEER_CROSS.0 - 40 && t < DEER_CROSS.1 && self.rng.f() < 0.3 + 0.7 * skill {
                it.down = true;
            }
        }
        // braking when it gets wobbly; glissade on snow
        if r.wobble > 0.25 + 0.3 * (1.0 - skill) {
            it.down = true;
        }
        if g == Ground::Snow && s < -0.12 && skill > 0.25 && st == Strategy::Smart {
            it.down = true;
        }
        if it.down && it.jump {
            it.down = false;
        }
        // food and high fives
        for k in 0..self.course.aids.len() {
            let ax = self.course.aids[k].x;
            let bit = 1u32 << k;
            let ai = self.runners[i].ai;
            if (x - ax).abs() < 9.0 && r.aid_done & bit == 0 && ai.grab_try & bit == 0 {
                let want = r.stamina < 90.0 && self.rng.f() < 0.45 + 0.5 * skill;
                self.runners[i].ai.grab_try |= bit;
                if want {
                    it.a = true;
                }
            }
        }
        for k in 0..self.course.crowds.len() {
            let cx = self.course.crowds[k].x;
            let bit = 1u32 << (k % 32);
            let ai = self.runners[i].ai;
            if (x - cx).abs() < 12.0 && ai.five_try & bit == 0 {
                self.runners[i].ai.five_try |= bit;
                if self.rng.f() < 0.25 + 0.5 * skill {
                    it.a = true;
                }
            }
        }
        it
    }

    fn goat_ai(&mut self, i: usize) -> Intent {
        let mut it = Intent::default();
        let r = &self.runners[i];
        if r.state != RState::Run {
            return it;
        }
        let x = r.x;
        let c = &self.course;
        // leap anything ahead
        let ob = c.obs[r.next_ob.min(c.obs.len() - 1)..].iter().find(|o| o.x + o.kind.width() / 2.0 > x).map(|o| o.x - o.kind.width() / 2.0 - x);
        let mut danger = ob.map(|d| d > 0.0 && d < 22.0).unwrap_or(false);
        for an in &self.animals {
            let d = an.x - x;
            if d > 0.0 && d < 26.0 {
                danger = true;
            }
        }
        if c.ground(x + 20.0) == Ground::Water && c.ground(x) != Ground::Water {
            danger = true;
        }
        if danger {
            it.jump = true;
        }
        // snacks: stops to munch now and then, more when far ahead of the players
        let lead = self.views.iter().map(|&v| x - self.runners[v].x).fold(f32::MIN, f32::max);
        let aid = c.aids.iter().position(|a| (a.x - x).abs() < 3.0).filter(|&k| self.runners[i].aid_done & (1 << k) == 0);
        let near_aid = aid.is_some();
        let r = &mut self.runners[i];
        if let Some(k) = aid {
            r.aid_done |= 1 << k;
        }
        r.snack_cd -= 1;
        if r.snack > 0 {
            r.snack -= 1;
            r.v *= 0.8;
            if r.snack == 0 {
                r.snack_cd = 700 + self.rng.int(400);
            }
        } else if !self.cpu_only && (near_aid || (r.snack_cd <= 0 && c.slope(x).abs() < 0.1 && self.rng.f() < if lead > 250.0 { 0.03 } else { 0.004 })) {
            r.snack = 70 + self.rng.int(60);
            r.say = Some((GOAT_TAUNTS[7], 90));
            if near_aid {
                r.snack = 120;
            }
        }
        // a little rubber band so the race stays close
        let r = &self.runners[i];
        if !self.cpu_only && r.snack == 0 && r.state == RState::Run {
            let f = if lead > 300.0 { 0.995 } else if lead < -300.0 { 1.004 } else { 1.0 };
            self.runners[i].v *= f;
        }
        if self.runners[i].shortcut && self.t.is_multiple_of(240) {
            self.goat_say(6, 90);
        }
        it
    }
}
