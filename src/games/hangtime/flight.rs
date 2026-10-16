//! The race: pilots driven by pads (the CPU pilots get their pads from
//! `ai`), the flight model, launch, landing, scoring and the wildlife.

use super::ai::{Brain, Style};
use super::course::*;
use super::pilots::PILOTS;
use crate::engine::gfx::rgb;
use crate::engine::input::{edge, Pad};
use crate::engine::rng::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Sfx {
    Vario1,
    Vario2,
    Vario3,
    Sink,
    Stall,
    Whoosh,
    Step,
    Thud,
    Crash,
    Splash,
    Honk,
    Baa,
    Ding,
    Cheer,
    Blip,
    Confirm,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ctrl {
    Human(usize),
    Cpu,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Landing {
    Perfect,
    Soft,
    Hard,
    Belly,
    Crash,
    Splash,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    /// in line on the hilltop
    Wait,
    Run,
    /// launched too early: a hop on the ramp
    Hop(i32),
    /// ran off the edge without launching
    Wobble(i32),
    Fly,
    Stall(i32),
    /// inside the storm cloud
    Suck(i32),
    Flare(i32),
    Landed(Landing, i32),
    Paddle(i32),
    Walk(i32),
    Done,
}

// flight model (px and px/frame at 60 Hz)
pub const TRIM: f32 = 0.80;
const MIN_SINK_V: f32 = 0.66;
pub const ENERGY: f32 = 13.0;
/// Seconds after which there is no more lift ("sunset").
pub const SUNSET_S: u32 = 210;
/// Time bonus: `TIME_PTS` per second under this, times the share of gates made.
pub const TIME_PAR_S: i32 = 240;
pub const GATE_PTS: i32 = 250;
pub const HONK_PTS: i32 = 25;
pub const TIME_PTS: i32 = 10;
/// Landing rings around the target cross: (radius, points); 50 in the field.
pub const RINGS: [(f32, i32); 3] = [(9.0, 500), (22.0, 300), (40.0, 150)];
pub const FLARE_MAX_AGL: f32 = 32.0;

pub fn polar_sink(v: f32) -> f32 {
    0.075 + 0.30 * (v - MIN_SINK_V) * (v - MIN_SINK_V)
}

#[derive(Clone, Debug)]
pub struct Pilot {
    pub def: usize,
    pub ctrl: Ctrl,
    /// which view's banners and sounds this pilot drives
    pub view: Option<usize>,
    pub brain: Option<Brain>,
    pub state: State,
    pub x: f32,
    pub y: f32,
    pub v: f32,
    pub face: f32,
    pub pitch: f32,
    pub turn: i32,
    pub turn_len: i32,
    pub vy: f32,
    pub vario: f32,
    pub tuck: bool,
    pub run: f32,
    pub step: i8,
    pub anim: f32,
    pub start_at: u32,
    /// frames since start (stops at touchdown)
    pub clock: u32,
    pub gates: u32,
    pub gate_pts: i32,
    pub landing_pts: i32,
    pub bonus: i32,
    pub penalty: i32,
    pub time_bonus: i32,
    pub walk_frames: u32,
    pub landing: Option<Landing>,
    pub land_x: f32,
    pub sheep_hit: bool,
    pub dazed: i32,
    pub sunset: bool,
    pub honk: i32,
    pub ufo: bool,
    pub prev: Pad,
    pub vario_t: i32,
    pub hop_y: f32,
    pub slide: f32,
    pub say: Option<(&'static str, i32)>,
    pub said_t: u32,
    pub missed_note: u32,
    pub stalls: u32,
    pub turns: u32,
    pub max_y: f32,
    pub lift_frames: u32,
    pub launch_v: f32,
    /// frames until the storm can swallow this pilot again
    pub storm_cool: i32,
}

impl Pilot {
    pub fn airborne(&self) -> bool {
        matches!(self.state, State::Wobble(_) | State::Fly | State::Stall(_) | State::Suck(_) | State::Flare(_))
    }
    pub fn on_ground(&self) -> bool {
        matches!(self.state, State::Landed(..) | State::Paddle(_) | State::Walk(_) | State::Done)
    }
    pub fn total(&self) -> i32 {
        (self.gate_pts + self.time_bonus + self.landing_pts + self.bonus - self.penalty).max(0)
    }
    pub fn gates_made(&self) -> u32 {
        self.gates.count_ones()
    }
    /// Shown on the clock: flight time plus any walking.
    pub fn shown_time(&self) -> u32 {
        let walk = match self.state {
            State::Walk(t) => (self.walk_frames as f32 * (t as f32 / WALK_ANIM as f32).min(1.0)) as u32,
            State::Done => self.walk_frames,
            _ => 0,
        };
        let paddle = match self.state {
            State::Paddle(t) => PADDLE_TIME * (PADDLE_ANIM - t).max(0) as u32 / PADDLE_ANIM as u32,
            _ => 0,
        };
        self.clock + walk + paddle
    }
    /// Height above the floor (ground or water).
    pub fn agl(&self, c: &Course) -> f32 {
        self.y - c.floor_at(self.x)
    }
    /// Turn progress 0..1 while turning.
    pub fn turn_phase(&self) -> f32 {
        if self.turn > 0 { 1.0 - self.turn as f32 / self.turn_len as f32 } else { 0.0 }
    }
}

const WALK_ANIM: i32 = 110;
const PADDLE_ANIM: i32 = 150;
const PADDLE_TIME: u32 = 20 * 60;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FxKind {
    Dust,
    Drop,
    Star,
    Wool,
    Spark,
    Feather,
}

#[derive(Clone, Copy, Debug)]
pub struct Fx {
    pub x: f32,
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
    pub text: String,
    pub color: u32,
    pub life: i32,
}

#[derive(Clone, Debug)]
pub struct Banner {
    pub text: String,
    pub t: i32,
    pub color: u32,
    pub life: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SheepMode {
    Graze,
    Walk,
    Run,
    Fly,
    Dizzy,
}

#[derive(Clone, Copy, Debug)]
pub struct Sheep {
    pub x: f32,
    pub h: f32,
    pub vx: f32,
    pub vh: f32,
    pub face: f32,
    pub t: i32,
    pub mode: SheepMode,
    pub spin: f32,
}

/// A V of geese. Ride in the V for free lift.
#[derive(Clone, Copy, Debug)]
pub struct Flock {
    pub x0: f32,
    pub y: f32,
    pub t0: u32,
}

pub const GOOSE_SPEED: f32 = 0.62;

impl Flock {
    /// Lead goose position, or None when not in the sky.
    pub fn pos(&self, t: u32, wind: f32) -> Option<(f32, f32)> {
        if t < self.t0 {
            return None;
        }
        let dt = (t - self.t0) as f32;
        let x = self.x0 + (GOOSE_SPEED + wind) * dt;
        if x > LEN as f32 + 80.0 {
            return None;
        }
        Some((x, self.y + (dt * 0.012).sin() * 8.0))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ufo {
    pub x: f32,
    pub y: f32,
    pub t: i32,
}

impl Ufo {
    pub fn beaming(&self) -> bool {
        self.t > 60 && self.t < 620
    }
}

pub const UFO_LIFE: i32 = 720;

#[derive(Clone, Copy, Default, Debug)]
pub struct Stats {
    pub honks: u32,
    pub ufo_rides: u32,
    pub cloudsucks: u32,
    pub sheep: u32,
}

pub struct Race {
    pub course: Course,
    pub pilots: Vec<Pilot>,
    pub t: u32,
    pub sfx: Vec<Sfx>,
    pub fx: Vec<Fx>,
    pub popups: Vec<Popup>,
    pub banners: [Option<Banner>; 2],
    pub sheep: Vec<Sheep>,
    pub flocks: Vec<Flock>,
    pub ufo: Option<Ufo>,
    pub ufo_at: Option<u32>,
    pub stats: Stats,
    /// lightning flash frames
    pub flash: i32,
    pub rng: Rng,
}

/// One entry per pilot: (pilot def, control, CPU skill, CPU style).
pub type Entry = (usize, Ctrl, f32, Style);

impl Race {
    pub fn new(venue: Venue, entries: &[Entry], seed: u64) -> Self {
        let course = Course::new(venue, seed ^ 0x9E37);
        let mut rng = Rng(seed | 1);
        let l = course.launch;
        let mut pilots = Vec::new();
        let gap = if entries.iter().filter(|e| matches!(e.1, Ctrl::Human(_))).count() > 1 { 100 } else { 150 };
        for (k, &(def, ctrl, skill, style)) in entries.iter().enumerate() {
            let view = match ctrl {
                Ctrl::Human(i) => Some(i),
                Ctrl::Cpu => None,
            };
            let brain = matches!(ctrl, Ctrl::Cpu).then(|| Brain::new(skill, style, seed.wrapping_add(k as u64 * 7919)));
            pilots.push(Pilot {
                def,
                ctrl,
                view,
                brain,
                state: State::Wait,
                x: l.x0 - 26.0 * k as f32,
                y: l.y0,
                v: 0.0,
                face: 1.0,
                pitch: 0.0,
                turn: 0,
                turn_len: 1,
                vy: 0.0,
                vario: 0.0,
                tuck: false,
                run: 0.0,
                step: 0,
                anim: 0.0,
                start_at: 90 + gap * k as u32,
                clock: 0,
                gates: 0,
                gate_pts: 0,
                landing_pts: 0,
                bonus: 0,
                penalty: 0,
                time_bonus: 0,
                walk_frames: 0,
                landing: None,
                land_x: 0.0,
                sheep_hit: false,
                dazed: 0,
                sunset: false,
                honk: -1,
                ufo: false,
                prev: Pad::default(),
                vario_t: 0,
                hop_y: 0.0,
                slide: 0.0,
                say: None,
                said_t: 0,
                missed_note: 0,
                stalls: 0,
                turns: 0,
                max_y: 0.0,
                lift_frames: 0,
                launch_v: 0.0,
                storm_cool: 0,
            });
        }
        // sheep graze on the landing field; one likes the bullseye
        let mut sheep = Vec::new();
        let (f0, f1) = course.field;
        for k in 0..7 {
            let x = if k == 0 { course.target + rng.range(-20.0, 20.0) } else { rng.range(f0 + 16.0, f1 - 16.0) };
            sheep.push(Sheep { x, h: 0.0, vx: 0.0, vh: 0.0, face: if rng.f() < 0.5 { 1.0 } else { -1.0 }, t: rng.int(200), mode: SheepMode::Graze, spin: 0.0 });
        }
        let (ylo, yhi) = match venue {
            Venue::Dunes => (190.0, 260.0),
            Venue::Alps => (320.0, 400.0),
            Venue::Canyon => (360.0, 440.0),
        };
        let mut flocks = Vec::new();
        for k in 0..7 {
            flocks.push(Flock { x0: 500.0 + k as f32 * 640.0 + rng.range(0.0, 200.0), y: rng.range(ylo, yhi), t0: 500 + k * 1300 + rng.int(300) as u32 });
        }
        let ufo_at = if rng.f() < 0.3 { Some(1800 + rng.int(4800) as u32) } else { None };
        Race {
            course,
            pilots,
            t: 0,
            sfx: Vec::new(),
            fx: Vec::new(),
            popups: Vec::new(),
            banners: [None, None],
            sheep,
            flocks,
            ufo: None,
            ufo_at,
            stats: Stats::default(),
            flash: 0,
            rng,
        }
    }

    pub fn finished(&self) -> bool {
        self.pilots.iter().all(|p| p.state == State::Done)
    }

    /// Ranking: pilot indexes, best total first.
    pub fn ranking(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.pilots.len()).collect();
        idx.sort_by(|&a, &b| self.pilots[b].total().cmp(&self.pilots[a].total()).then(a.cmp(&b)));
        idx
    }

    /// Lift from the geese and the UFO at a point: (lift, flock index, ufo).
    pub fn extra_lift(&self, x: f32, y: f32) -> (f32, Option<usize>, bool) {
        let mut l = 0.0;
        let mut flock = None;
        for (k, fl) in self.flocks.iter().enumerate() {
            if let Some((lx, ly)) = fl.pos(self.t, self.course.wind) {
                if x > lx - 70.0 && x < lx - 4.0 && (y - ly).abs() < 18.0 {
                    l += 0.40;
                    flock = Some(k);
                }
            }
        }
        let mut ufo = false;
        if let Some(u) = &self.ufo {
            if u.beaming() && (x - u.x).abs() < 14.0 && y < u.y - 4.0 && y > u.y - 170.0 {
                l += 0.65;
                ufo = true;
            }
        }
        (l, flock, ufo)
    }

    /// Total vertical air movement a given pilot feels at a point.
    pub fn air(&self, def: usize, x: f32, y: f32) -> f32 {
        let d = &PILOTS[def];
        let c = &self.course;
        let natural = c.thermal_lift(x, y, self.t) + c.ridge_lift(x, y);
        let natural = if natural > 0.0 { natural * d.lift_mul() } else { natural };
        natural - c.sink_at(x) + c.storm_lift(x, y) + self.extra_lift(x, y).0
    }

    pub fn update(&mut self, pads: &[Pad; 2]) {
        self.t += 1;
        // CPU pilots think first, on the world as it is
        let mut cmd = vec![Pad::default(); self.pilots.len()];
        for i in 0..self.pilots.len() {
            cmd[i] = match self.pilots[i].ctrl {
                Ctrl::Human(k) => pads[k.min(1)],
                Ctrl::Cpu => {
                    let mut b = self.pilots[i].brain.take().expect("cpu pilot has a brain");
                    let pad = b.think(self, i);
                    self.pilots[i].brain = Some(b);
                    pad
                }
            };
        }
        for (i, pad) in cmd.into_iter().enumerate() {
            self.step_pilot(i, pad);
        }
        self.update_world();
    }

    fn banner(&mut self, view: Option<usize>, text: &str, color: u32) {
        if let Some(v) = view {
            self.banners[v] = Some(Banner { text: text.to_string(), t: 0, color, life: 100 });
        }
    }

    fn sound(&mut self, view: Option<usize>, s: Sfx) {
        if view.is_some() {
            self.sfx.push(s);
        }
    }

    fn popup(&mut self, x: f32, y: f32, text: &str, color: u32) {
        self.popups.push(Popup { x, y, text: text.to_string(), color, life: 70 });
    }

    fn burst(&mut self, x: f32, y: f32, kind: FxKind, n: usize, color: u32, spread: f32) {
        for _ in 0..n {
            let a = self.rng.range(0.0, std::f32::consts::TAU);
            let s = self.rng.range(0.3, 1.0) * spread;
            let life = 20 + self.rng.int(25);
            self.fx.push(Fx { x, y, vx: a.cos() * s, vy: a.sin().abs() * s + 0.2, life, max: life, kind, color });
        }
    }

    fn step_pilot(&mut self, i: usize, pad: Pad) {
        let e = edge(pad, self.pilots[i].prev);
        self.pilots[i].prev = pad;
        let view = self.pilots[i].view;
        let def = &PILOTS[self.pilots[i].def];
        let st = self.pilots[i].state;
        if !matches!(st, State::Wait | State::Landed(..) | State::Paddle(_) | State::Walk(_) | State::Done) {
            self.pilots[i].clock += 1;
        }
        if let Some((_, t)) = &mut self.pilots[i].say {
            *t -= 1;
            if *t <= 0 {
                self.pilots[i].say = None;
            }
        }
        match st {
            State::Wait => self.step_wait(i),
            State::Run => self.step_run(i, e),
            State::Hop(t) => {
                let l = self.course.launch;
                let p = &mut self.pilots[i];
                p.x += p.run;
                p.anim += 0.1;
                let k = (24 - t) as f32 / 24.0;
                p.hop_y = (k * std::f32::consts::PI).sin() * 7.0;
                p.y = l.y_at(p.x) + p.hop_y;
                if p.x >= l.x1 {
                    self.tumble_off(i);
                } else if t <= 1 {
                    p.hop_y = 0.0;
                    p.state = State::Run;
                } else {
                    p.state = State::Hop(t - 1);
                }
            }
            State::Wobble(t) => {
                let p = &mut self.pilots[i];
                p.v += (0.72 - p.v) * 0.04;
                p.vy = -0.36 - 0.1 * ((t as f32) * 0.3).sin();
                p.y += p.vy;
                p.x += p.face * p.v + self.course.wind;
                p.pitch = ((t as f32) * 0.35).sin();
                p.state = if t <= 1 { State::Fly } else { State::Wobble(t - 1) };
                if t <= 1 {
                    p.pitch = 0.0;
                }
                self.check_ground(i);
            }
            State::Fly => self.step_fly(i, pad, e),
            State::Stall(t) => {
                let p = &mut self.pilots[i];
                p.pitch += (-1.0 - p.pitch) * 0.2;
                p.v += (0.75 - p.v) * 0.05;
                p.vy = -0.62 + 0.4 * (1.0 - t as f32 / 50.0).max(0.0);
                if p.turn > 0 {
                    p.turn = 0;
                    p.face = -p.face;
                }
                p.y += p.vy;
                p.x += p.face * p.v * 0.6 + self.course.wind;
                p.vario += (p.vy - p.vario) * 0.1;
                if t <= 1 {
                    p.state = State::Fly;
                    p.v = p.v.max(0.72);
                    p.pitch = -0.3;
                } else {
                    p.state = State::Stall(t - 1);
                }
                self.check_gates(i);
                self.check_ground(i);
            }
            State::Suck(t) => {
                let st = self.course.storm.expect("storm");
                let p = &mut self.pilots[i];
                let k = 1.0 - t as f32 / 100.0;
                p.x += (st.x - p.x) * 0.05;
                p.y += (st.top - 30.0 - p.y) * 0.05;
                p.vy = 1.0;
                p.vario = 1.0;
                p.pitch = (k * 40.0).sin();
                if t % 23 == 0 {
                    self.flash = 6;
                }
                if t <= 1 {
                    p.state = State::Fly;
                    p.x = st.x + st.w + 70.0;
                    p.y = st.top + 8.0;
                    p.face = 1.0;
                    p.turn = 0;
                    p.v = 0.95;
                    p.pitch = 0.0;
                    p.dazed = 150 - 12 * def.nerve;
                    p.storm_cool = 60 * 25;
                    let (x, y) = (p.x, p.y);
                    self.banner(view, "PTOOEY!", rgb(160, 232, 120));
                    self.burst(x, y, FxKind::Star, 8, rgb(248, 248, 120), 1.2);
                    self.sound(view, Sfx::Whoosh);
                } else {
                    p.state = State::Suck(t - 1);
                }
            }
            State::Flare(t) => {
                let p = &mut self.pilots[i];
                p.v *= 0.935;
                p.vy = (p.vy - def.flare_grav()).max(-2.2);
                p.pitch += (1.0 - p.pitch) * 0.3;
                p.y += p.vy;
                p.x += p.face * p.v + self.course.wind * 0.4;
                p.vario += (p.vy - p.vario) * 0.1;
                p.state = State::Flare(t + 1);
                self.check_ground(i);
            }
            State::Landed(kind, t) => self.step_landed(i, kind, t),
            State::Paddle(t) => {
                let p = &mut self.pilots[i];
                let target = self.course.target;
                let dir = if target > p.x { 1.0 } else { -1.0 };
                p.face = dir;
                p.x += dir * 0.25;
                p.anim += 0.12;
                p.y = self.course.floor_at(p.x);
                if t <= 1 || self.course.water_at(p.x).is_none() {
                    p.walk_frames += PADDLE_TIME;
                    if self.course.in_field(p.x) {
                        self.finish(i);
                    } else {
                        self.start_walk(i);
                    }
                } else {
                    p.state = State::Paddle(t - 1);
                }
            }
            State::Walk(t) => {
                let p = &mut self.pilots[i];
                let dir = if self.course.target > p.x { 1.0 } else { -1.0 };
                p.face = dir;
                p.x += dir * 0.3;
                p.anim += 0.08;
                p.y = self.course.floor_at(p.x);
                if t >= WALK_ANIM {
                    self.finish(i);
                } else {
                    p.state = State::Walk(t + 1);
                }
            }
            State::Done => {
                let p = &mut self.pilots[i];
                p.anim += 0.05;
            }
        }
        let p = &mut self.pilots[i];
        p.max_y = p.max_y.max(p.y);
    }

    fn step_wait(&mut self, i: usize) {
        let l = self.course.launch;
        let ahead = (0..i).filter(|&k| self.pilots[k].state == State::Wait).count();
        let p = &mut self.pilots[i];
        let goal = l.x0 - 26.0 * ahead as f32;
        if p.x < goal - 0.5 {
            p.x += 0.35;
            p.anim += 0.06;
        }
        p.y = self.course.ground_at(p.x).max(l.y_at(p.x));
        if p.x >= l.x0 - 0.5 {
            p.y = l.y0;
        }
        if self.t >= p.start_at && ahead == 0 {
            p.state = State::Run;
            p.x = p.x.max(l.x0);
            p.run = 0.08;
            p.step = 0;
            self.banner(view_of(&self.pilots[i]), "GO!", rgb(120, 232, 96));
            self.sound(view_of(&self.pilots[i]), Sfx::Confirm);
        }
    }

    fn step_run(&mut self, i: usize, e: Pad) {
        let l = self.course.launch;
        let view = self.pilots[i].view;
        let p = &mut self.pilots[i];
        let dir = if e.left { -1 } else if e.right { 1 } else { 0 };
        let mut stepped = false;
        if dir != 0 {
            if dir != p.step {
                p.run = (p.run + 0.13).min(1.05);
                stepped = true;
            } else {
                p.run *= 0.85;
            }
            p.step = dir;
        }
        p.run = (p.run * 0.993).max(0.12);
        p.x += p.run;
        p.anim += 0.05 + p.run * 0.12;
        p.y = l.y_at(p.x);
        let (z0, _) = l.zone();
        if stepped {
            self.sound(view, Sfx::Step);
        }
        let p = &mut self.pilots[i];
        if e.b {
            if p.x >= z0 {
                p.state = State::Fly;
                p.v = 0.42 + p.run * 0.46;
                p.launch_v = p.v;
                p.vy = 0.0;
                p.pitch = 0.0;
                let (x, y, v) = (p.x, p.y, p.v);
                self.sound(view, Sfx::Whoosh);
                if v > 0.84 {
                    self.pilots[i].bonus += 50;
                    self.banner(view, "SUPER LAUNCH!", rgb(248, 216, 48));
                    self.popup(x, y + 20.0, "+50", rgb(248, 232, 96));
                } else {
                    self.banner(view, "AIRBORNE!", rgb(96, 224, 248));
                }
            } else {
                p.state = State::Hop(24);
                p.run *= 0.6;
                let (x, y) = (p.x, p.y);
                self.popup(x, y + 26.0, "TOO EARLY!", rgb(248, 152, 64));
                self.sound(view, Sfx::Step);
            }
            return;
        }
        if p.x >= l.x1 {
            self.tumble_off(i);
        }
    }

    fn tumble_off(&mut self, i: usize) {
        let view = self.pilots[i].view;
        let p = &mut self.pilots[i];
        p.state = State::Wobble(45);
        p.v = 0.3;
        p.launch_v = 0.3;
        p.hop_y = 0.0;
        self.banner(view, "WHOOPS!", rgb(248, 120, 72));
        self.sound(view, Sfx::Stall);
    }

    fn step_fly(&mut self, i: usize, pad: Pad, e: Pad) {
        let view = self.pilots[i].view;
        let d = &PILOTS[self.pilots[i].def];
        let (x0, y0) = (self.pilots[i].x, self.pilots[i].y);
        let (extra, flock, ufo) = self.extra_lift(x0, y0);
        let c = &self.course;
        let natural = c.thermal_lift(x0, y0, self.t) + c.ridge_lift(x0, y0);
        let natural = if natural > 0.0 { natural * d.lift_mul() } else { natural };
        let storm = if self.pilots[i].storm_cool > 0 { c.storm_lift(x0, y0).min(0.2) } else { c.storm_lift(x0, y0) };
        let mut air = natural - c.sink_at(x0) + storm + extra;
        let wind = c.wind;
        let storm_def = c.storm;
        let agl = self.pilots[i].agl(c);
        let p = &mut self.pilots[i];
        if p.sunset {
            air = -0.16;
            if p.clock > (SUNSET_S + 150) * 60 {
                air = -1.0;
            }
        } else if p.clock > SUNSET_S * 60 {
            p.sunset = true;
            self.banner(view, "SUNSET! LAND!", rgb(248, 136, 64));
            self.sound(view, Sfx::Stall);
            return;
        }
        let p = &mut self.pilots[i];
        if air > 0.1 {
            p.lift_frames += 1;
        }

        // controls
        let mut pin = if pad.up {
            1.0
        } else if pad.down {
            -1.0
        } else {
            0.0
        };
        let mut want_turn = (e.left && p.face > 0.0) || (e.right && p.face < 0.0);
        if p.dazed > 0 {
            p.dazed -= 1;
            pin = ((p.dazed as f32) * 0.21).sin() * 0.8;
            want_turn = false;
        }
        p.pitch += (pin - p.pitch) * 0.07;
        p.tuck = pad.c && p.pitch < 0.25 && p.dazed == 0;
        let mut vt = if p.pitch >= 0.0 { TRIM - p.pitch * 0.46 } else { TRIM - p.pitch * 0.42 };
        if p.tuck {
            vt = d.tuck_speed();
        }
        let v0 = p.v;
        p.v += (vt - p.v) * 0.03;
        let zoom = (v0 * v0 - p.v * p.v) * ENERGY;

        // turning
        if want_turn && p.turn == 0 {
            p.turn_len = d.turn_frames();
            p.turn = p.turn_len;
            p.turns += 1;
        }
        let mut turn_sink = 0.0;
        let heading = if p.turn > 0 {
            let k = 1.0 - p.turn as f32 / p.turn_len as f32;
            p.turn -= 1;
            turn_sink = 0.08 + 0.015 * (5 - d.handling) as f32;
            p.v *= 0.997;
            let h = (k * std::f32::consts::PI).cos() * p.face;
            if p.turn == 0 {
                p.face = -p.face;
            }
            h
        } else {
            p.face
        };

        let sink = polar_sink(p.v) * d.sink_mul();
        p.vy = -sink + air + zoom - turn_sink;
        p.y += p.vy;
        p.x += heading * p.v + wind;
        p.vario += (p.vy - zoom - p.vario) * 0.08;

        // course edges
        if p.x < 40.0 && p.face < 0.0 && p.turn == 0 {
            p.turn_len = d.turn_frames();
            p.turn = p.turn_len;
        }
        p.x = p.x.clamp(10.0, LEN as f32 - 10.0);

        // stall
        if p.v < d.stall_speed() {
            p.state = State::Stall(50);
            p.stalls += 1;
            p.tuck = false;
            self.banner(view, "STALL!", rgb(248, 72, 72));
            self.sound(view, Sfx::Stall);
            return;
        }

        // flare
        if e.b && agl < FLARE_MAX_AGL && p.dazed == 0 {
            p.state = State::Flare(0);
            p.vy = p.vy.max(-0.25) + 0.2;
            p.turn = 0;
            self.sound(view, Sfx::Whoosh);
            return;
        }

        // vario sounds
        let p = &mut self.pilots[i];
        p.vario_t -= 1;
        if p.vario_t <= 0 {
            let v = p.vario;
            if v > 0.03 {
                let s = if v < 0.12 {
                    Sfx::Vario1
                } else if v < 0.26 {
                    Sfx::Vario2
                } else {
                    Sfx::Vario3
                };
                p.vario_t = (30.0 - v * 70.0).clamp(7.0, 30.0) as i32;
                self.sound(view, s);
            } else if v < -0.24 {
                p.vario_t = 55;
                self.sound(view, Sfx::Sink);
            } else {
                p.vario_t = 6;
            }
        }

        // geese and the UFO
        let p = &mut self.pilots[i];
        if let Some(k) = flock {
            if p.honk != k as i32 {
                p.honk = k as i32;
                p.bonus += HONK_PTS;
                let (x, y) = (p.x, p.y);
                self.stats.honks += 1;
                self.banner(view, "HONK HONK!", rgb(248, 248, 248));
                self.popup(x, y + 22.0, &format!("+{HONK_PTS}"), rgb(248, 232, 96));
                self.sound(view, Sfx::Honk);
            } else if self.t % 50 == 0 {
                self.sound(view, Sfx::Honk);
            }
        }
        let p = &mut self.pilots[i];
        if ufo && !p.ufo {
            p.ufo = true;
            p.bonus += 100;
            let (x, y) = (p.x, p.y);
            self.stats.ufo_rides += 1;
            self.banner(view, "CLOSE ENCOUNTER!", rgb(120, 248, 160));
            self.popup(x, y + 22.0, "+100", rgb(120, 248, 160));
            self.sound(view, Sfx::Ding);
        }

        // the storm cloud swallows you (once in a while)
        let p = &mut self.pilots[i];
        p.storm_cool -= 1;
        if let Some(st) = storm_def {
            if p.storm_cool <= 0 && !p.sunset && (p.x - st.x).abs() < st.w + 6.0 && p.y > st.base - 2.0 && p.y < st.top {
                p.state = State::Suck(100);
                p.turn = 0;
                self.stats.cloudsucks += 1;
                self.banner(view, "CLOUDSUCK!", rgb(184, 160, 248));
                self.sound(view, Sfx::Crash);
                self.flash = 8;
                return;
            }
        }
        self.check_gates(i);
        self.check_heckle(i);
        self.check_ground(i);
    }

    fn check_gates(&mut self, i: usize) {
        let view = self.pilots[i].view;
        let (x, y, gates) = (self.pilots[i].x, self.pilots[i].y, self.pilots[i].gates);
        let px = x - self.pilots[i].face * self.pilots[i].v - self.course.wind;
        for k in 0..self.course.gates.len() {
            if gates & (1 << k) != 0 {
                continue;
            }
            let g = self.course.gates[k];
            if (px - g.x) * (x - g.x) > 0.0 {
                continue;
            }
            if y >= g.lo && y <= g.hi {
                let p = &mut self.pilots[i];
                p.gates |= 1 << k;
                p.gate_pts += GATE_PTS;
                self.popup(g.x, y + 24.0, "+250", rgb(248, 232, 96));
                self.burst(g.x, y, FxKind::Spark, 14, rgb(248, 232, 96), 1.4);
                self.banner(view, &format!("GATE {}!", k + 1), rgb(248, 216, 48));
                self.sound(view, Sfx::Ding);
            } else if self.pilots[i].missed_note & (1 << k) == 0 {
                self.pilots[i].missed_note |= 1 << k;
                if view.is_some() {
                    let txt = if y < g.lo { "TOO LOW!" } else { "TOO HIGH!" };
                    let yy = y.clamp(g.lo - 20.0, g.hi + 20.0);
                    self.popup(g.x, yy + 18.0, txt, rgb(248, 120, 96));
                    self.sound(view, Sfx::Sink);
                }
            }
        }
    }

    /// CPU pilots heckle you when they fly past.
    fn check_heckle(&mut self, i: usize) {
        if self.pilots[i].ctrl != Ctrl::Cpu || self.t < self.pilots[i].said_t + 1200 {
            return;
        }
        let (x, y, face) = (self.pilots[i].x, self.pilots[i].y, self.pilots[i].face);
        let passed = self.pilots.iter().any(|h| {
            h.view.is_some() && h.ctrl != Ctrl::Cpu && h.airborne() && (h.x - x).abs() < 3.0 && (h.y - y).abs() < 60.0 && (h.face > 0.0) == (face > 0.0)
        });
        if passed {
            let def = self.pilots[i].def;
            self.pilots[i].say = Some((PILOTS[def].heckle, 150));
            self.pilots[i].said_t = self.t;
        }
    }

    fn check_ground(&mut self, i: usize) {
        let c = &self.course;
        let p = &self.pilots[i];
        let floor = c.floor_at(p.x);
        if p.y > floor {
            return;
        }
        let wet = c.water_at(p.x).map(|l| l >= c.ground_at(p.x)).unwrap_or(false);
        let (vy, v) = (p.vy, p.v);
        let flaring = matches!(p.state, State::Flare(_));
        let kind = if wet {
            Landing::Splash
        } else if flaring {
            if vy <= -1.45 {
                Landing::Crash
            } else if vy <= -0.75 {
                Landing::Hard
            } else if v < 0.62 {
                if vy > -0.45 { Landing::Perfect } else { Landing::Soft }
            } else {
                Landing::Belly
            }
        } else if matches!(p.state, State::Stall(_)) || v > 1.12 || vy < -1.0 {
            Landing::Crash
        } else {
            Landing::Belly
        };
        self.touchdown(i, kind, floor);
    }

    fn touchdown(&mut self, i: usize, kind: Landing, floor: f32) {
        let view = self.pilots[i].view;
        let c = &self.course;
        let p = &mut self.pilots[i];
        p.y = floor;
        p.landing = Some(kind);
        p.land_x = p.x;
        p.state = State::Landed(kind, 0);
        p.turn = 0;
        p.tuck = false;
        p.slide = match kind {
            Landing::Perfect | Landing::Soft => p.v * 0.8,
            Landing::Belly => p.v * 0.9,
            Landing::Hard => 0.3,
            Landing::Crash => 0.6,
            Landing::Splash => 0.2,
        };
        let in_field = c.in_field(p.x);
        let (x, y) = (p.x, p.y);
        let d = (p.x - c.target).abs();
        let ring = if !in_field { 0 } else { RINGS.iter().find(|r| d < r.0).map(|r| r.1).unwrap_or(50) };
        let mut pen = 0;
        let mut ring_pts = ring;
        let (txt, col, snd) = match kind {
            Landing::Perfect => ("PERFECT FLARE!", rgb(120, 232, 96), Sfx::Thud),
            Landing::Soft => ("NICE LANDING!", rgb(120, 232, 96), Sfx::Thud),
            Landing::Hard => {
                pen = 50;
                ("THUD!", rgb(248, 176, 64), Sfx::Thud)
            }
            Landing::Belly => {
                pen = 50;
                ("BELLY FLOP!", rgb(248, 176, 64), Sfx::Thud)
            }
            Landing::Crash => {
                pen = 200;
                ring_pts = 0;
                ("CRASH!", rgb(248, 72, 72), Sfx::Crash)
            }
            Landing::Splash => {
                pen = 100;
                ring_pts = 0;
                ("SPLASH!", rgb(96, 200, 248), Sfx::Splash)
            }
        };
        p.landing_pts = ring_pts;
        p.penalty += pen;
        // a sheep where you land
        let mut baa = false;
        if kind != Landing::Splash {
            if let Some(s) = self.sheep.iter_mut().find(|s| (s.x - x).abs() < 8.0 && s.mode != SheepMode::Fly) {
                s.mode = SheepMode::Fly;
                s.vh = 2.6;
                s.vx = if x < s.x { 1.0 } else { -1.0 } * 0.9;
                s.t = 0;
                baa = true;
            }
        }
        if baa {
            let p = &mut self.pilots[i];
            p.penalty += 100;
            p.sheep_hit = true;
            self.stats.sheep += 1;
            self.banner(view, "BAAAD LANDING!", rgb(248, 248, 248));
            self.popup(x, y + 30.0, "-100", rgb(248, 96, 96));
            self.sound(view, Sfx::Baa);
            self.burst(x, y + 6.0, FxKind::Wool, 12, rgb(248, 248, 248), 1.2);
        } else if ring == 500 && ring_pts > 0 {
            self.banner(view, "BULLSEYE!", rgb(248, 216, 48));
            self.sound(view, Sfx::Cheer);
        } else {
            self.banner(view, txt, col);
        }
        if ring_pts > 0 {
            self.popup(x, y + 22.0, &format!("+{}", ring_pts), rgb(248, 232, 96));
        }
        if pen > 0 && !baa {
            self.popup(x, y + 34.0, &format!("-{}", pen), rgb(248, 96, 96));
        }
        self.sound(view, snd);
        match kind {
            Landing::Splash => self.burst(x, y, FxKind::Drop, 16, rgb(160, 216, 248), 1.6),
            Landing::Crash => {
                self.burst(x, y + 2.0, FxKind::Dust, 12, rgb(216, 192, 152), 1.0);
                self.burst(x, y + 8.0, FxKind::Star, 6, rgb(248, 248, 120), 0.8);
            }
            _ => self.burst(x, y + 1.0, FxKind::Dust, 6, rgb(216, 192, 152), 0.6),
        }
    }

    fn step_landed(&mut self, i: usize, kind: Landing, t: i32) {
        let p = &mut self.pilots[i];
        p.x += p.face * p.slide;
        p.slide *= if kind == Landing::Belly { 0.95 } else { 0.9 };
        p.anim += p.slide * 0.15;
        p.y = self.course.floor_at(p.x);
        p.vario *= 0.9;
        if t < 70 {
            p.state = State::Landed(kind, t + 1);
            return;
        }
        if kind == Landing::Splash {
            p.state = State::Paddle(PADDLE_ANIM);
        } else if self.course.in_field(p.land_x) {
            self.finish(i);
        } else {
            self.start_walk(i);
        }
    }

    fn start_walk(&mut self, i: usize) {
        let view = self.pilots[i].view;
        let c = &self.course;
        let p = &mut self.pilots[i];
        let dist = if p.x < c.field.0 {
            c.field.0 - p.x
        } else if p.x > c.field.1 {
            p.x - c.field.1
        } else {
            0.0
        };
        // walking with a glider on your back: half a pixel per frame, sadly
        p.walk_frames += (dist / 0.5) as u32;
        p.state = State::Walk(0);
        if dist > 0.0 {
            self.banner(view, "WALK OF SHAME", rgb(200, 200, 232));
        }
    }

    fn finish(&mut self, i: usize) {
        let n = self.course.gates.len() as i32;
        let p = &mut self.pilots[i];
        let secs = ((p.clock + p.walk_frames) / 60) as i32;
        p.time_bonus = (TIME_PAR_S - secs).max(0) * TIME_PTS * p.gates_made() as i32 / n.max(1);
        p.state = State::Done;
    }

    fn update_world(&mut self) {
        // fx
        for f in self.fx.iter_mut() {
            f.x += f.vx;
            f.y += f.vy;
            f.life -= 1;
            match f.kind {
                FxKind::Drop | FxKind::Dust => f.vy -= 0.05,
                FxKind::Wool | FxKind::Feather => {
                    f.vy -= 0.01;
                    f.vx *= 0.97;
                }
                _ => f.vy -= 0.02,
            }
        }
        self.fx.retain(|f| f.life > 0);
        for p in self.popups.iter_mut() {
            p.y += 0.25;
            p.life -= 1;
        }
        self.popups.retain(|p| p.life > 0);
        for b in self.banners.iter_mut() {
            if let Some(bn) = b {
                bn.t += 1;
                if bn.t > bn.life {
                    *b = None;
                }
            }
        }
        if self.flash > 0 {
            self.flash -= 1;
        }
        if let Some(c) = self.course.storm {
            if self.t % 400 == 130 {
                self.flash = 5;
                let _ = c;
            }
        }
        // the UFO
        if Some(self.t) == self.ufo_at {
            let lead = self.pilots.iter().filter(|p| p.airborne()).map(|p| (p.view.is_some(), p.x, p.y)).max_by(|a, b| a.0.cmp(&b.0));
            if let Some((_, x, y)) = lead {
                self.ufo = Some(Ufo { x: x + 140.0, y: y + 60.0, t: 0 });
            }
        }
        if let Some(u) = &mut self.ufo {
            u.t += 1;
            u.x += (u.t as f32 * 0.02).sin() * 0.3 + 0.08;
            if u.t > 620 {
                u.y += 3.0;
                u.x += 2.0;
            }
            if u.t > UFO_LIFE {
                self.ufo = None;
            }
        }
        self.update_sheep();
    }

    fn update_sheep(&mut self) {
        let lows: Vec<(f32, f32)> =
            self.pilots.iter().filter(|p| p.airborne() || matches!(p.state, State::Landed(..))).map(|p| (p.x, p.y - self.course.ground_at(p.x))).collect();
        let (f0, f1) = self.course.field;
        for k in 0..self.sheep.len() {
            let r = self.rng.f();
            let r2 = self.rng.f();
            let s = &mut self.sheep[k];
            s.t += 1;
            match s.mode {
                SheepMode::Fly => {
                    s.x += s.vx;
                    s.h += s.vh;
                    s.vh -= 0.08;
                    s.spin += 0.3;
                    if s.h <= 0.0 && s.vh < 0.0 {
                        s.h = 0.0;
                        s.mode = SheepMode::Dizzy;
                        s.t = 0;
                    }
                }
                SheepMode::Dizzy => {
                    if s.t > 240 {
                        s.mode = SheepMode::Graze;
                    }
                }
                _ => {
                    let scare = lows.iter().find(|(x, h)| (x - s.x).abs() < 36.0 && *h < 26.0);
                    if let Some((x, _)) = scare {
                        s.mode = SheepMode::Run;
                        s.face = if *x < s.x { 1.0 } else { -1.0 };
                        s.t = 0;
                    }
                    match s.mode {
                        SheepMode::Run => {
                            s.x += s.face * 0.9;
                            if s.t > 50 {
                                s.mode = SheepMode::Graze;
                            }
                        }
                        SheepMode::Walk => {
                            s.x += s.face * 0.15;
                            if s.t > 90 && r < 0.02 {
                                s.mode = SheepMode::Graze;
                                s.t = 0;
                            }
                        }
                        _ => {
                            if s.t > 120 && r < 0.01 {
                                s.mode = SheepMode::Walk;
                                s.face = if r2 < 0.5 { 1.0 } else { -1.0 };
                                s.t = 0;
                            }
                        }
                    }
                    if s.x < f0 + 8.0 {
                        s.face = 1.0;
                    }
                    if s.x > f1 - 8.0 {
                        s.face = -1.0;
                    }
                    s.x = s.x.clamp(f0, f1);
                }
            }
        }
    }
}

fn view_of(p: &Pilot) -> Option<usize> {
    p.view
}
