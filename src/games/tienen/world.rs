//! TIENEN! simulation at 60 Hz: the pitch, the ball, four kids and the
//! rules of tienen. World coordinates: x along the pitch (the goal line is
//! at `GOAL_X`, the goal on the left), y depth (0 = far side, `DEPTH` =
//! near side), z height. `proj` maps them to the screen.

use super::bot::{self, Brain, BotKind};
use super::data::*;
use crate::engine::input::{edge, Pad};
pub use crate::engine::rng::Rng;

pub const GOAL_X: f32 = 30.0;
/// the posts (depth)
pub const G_Y0: f32 = 18.0;
pub const G_Y1: f32 = 54.0;
pub const BAR: f32 = 30.0;
/// back of the net
pub const NET_X: f32 = 16.0;
/// the wall behind the goal
pub const WALL_X: f32 = 6.0;
/// the keeper's area
pub const AREA_X: f32 = 66.0;
pub const AREA_Y0: f32 = 6.0;
pub const AREA_Y1: f32 = 66.0;
pub const FIELD_X1: f32 = 238.0;
pub const DEPTH: f32 = 72.0;
pub const BALL_R: f32 = 2.5;
/// ball gravity
pub const G: f32 = 0.16;
pub const KID_G: f32 = 0.22;
pub const JUMP_V: f32 = 3.1;
/// screen
pub const MID: f32 = 128.0;
pub const TOP: f32 = 118.0;
/// frames a swing can meet the ball
pub const WINDOW: u32 = 9;
/// latjetrap and billenbrand spot
pub const SPOT_X: f32 = 118.0;
/// After this long the mums call everybody in: the kid with the fewest
/// points loses.
pub const MAX_GAME: u32 = 60 * 60 * 5;

pub fn goal_mid() -> f32 {
    (G_Y0 + G_Y1) / 2.0
}

/// World to screen.
pub fn proj(x: f32, y: f32, z: f32) -> (i32, i32) {
    let s = 0.82 + 0.18 * (y / DEPTH);
    ((MID + (x - MID) * s).round() as i32, (TOP + y - z).round() as i32)
}

pub fn in_area(x: f32, y: f32) -> bool {
    x <= AREA_X && (AREA_Y0..=AREA_Y1).contains(&y)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Sfx {
    Blip,
    Confirm,
    Kick,
    Header,
    Bounce,
    Post,
    Net,
    Catch,
    Punch,
    Whistle,
    Cheer,
    Groan,
    Honk,
    Bark,
    Ouch,
    Whiff,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Ctrl {
    Human(usize),
    Cpu,
    Probe(BotKind),
}

/// What a kid wants this frame (from a pad or a brain). `a`, `b`, `c` are
/// presses, not holds.
#[derive(Clone, Copy, Default, Debug)]
pub struct Intent {
    pub mx: f32,
    pub my: f32,
    pub a: bool,
    pub b: bool,
    pub c: bool,
    /// CPU: face this way without moving
    pub face: i32,
    /// CPU: where to aim a shot (-1 far post .. 1 near post)
    pub aim: f32,
    /// CPU: cross to this kid
    pub to: Option<usize>,
}

impl Intent {
    pub fn from_pad(now: Pad, e: Pad) -> Self {
        let mx = (now.right as i32 - now.left as i32) as f32;
        let my = (now.down as i32 - now.up as i32) as f32;
        Intent { mx, my, a: e.a, b: e.b, c: e.c, face: 0, aim: my, to: None }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Act {
    Idle,
    /// a kick, header or cross can meet the ball for `WINDOW` frames
    Swing { kind: Kind, cross: bool, hit: bool },
    /// on your back after a bicycle kick, or after a dive
    Down,
    /// keeper: hands out
    Catch,
    /// keeper: flying sideways (-1 = far, 1 = near)
    Dive(i32),
    Punch,
    /// keeper: ball in the hands
    Hold,
    Roll,
    Cheer,
    Sulk,
    /// billenbrand: bent over
    Bent,
    Ouch,
}

#[derive(Clone, Default, Debug)]
pub struct KidStats {
    /// goals by kind (KINDS order)
    pub goals: [u32; 5],
    /// points taken off keepers
    pub scored: i32,
    pub saves: u32,
    pub wissels: u32,
    pub conceded: i32,
    pub brand_hits: u32,
    pub shots: u32,
    pub crosses: u32,
}

#[derive(Clone, Debug)]
pub struct Kid {
    /// index into KIDS
    pub id: usize,
    pub ctrl: Ctrl,
    pub skill: f32,
    pub pts: i32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    /// -1 = facing the goal (left), 1 = back to the goal
    pub face: i32,
    pub act: Act,
    pub at: u32,
    pub anim: f32,
    /// ignore the ball for a few frames after a touch
    pub cool: u32,
    pub stats: KidStats,
    pub bubble: Option<(String, u32)>,
    /// frames the ball has been at this kid's feet
    pub holding: u32,
}

impl Kid {
    pub fn new(id: usize, ctrl: Ctrl, skill: f32, x: f32, y: f32) -> Self {
        Kid {
            id,
            ctrl,
            skill,
            pts: 10,
            x,
            y,
            z: 0.0,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            face: -1,
            act: Act::Idle,
            at: 0,
            anim: 0.0,
            cool: 0,
            stats: KidStats::default(),
            bubble: None,
            holding: 0,
        }
    }
    pub fn air(&self) -> bool {
        self.z > 0.5
    }
    pub fn human(&self) -> Option<usize> {
        match self.ctrl {
            Ctrl::Human(p) => Some(p),
            _ => None,
        }
    }
    pub fn speed(&self) -> f32 {
        1.0 + 0.13 * snel(self.id) as f32
    }
    pub fn busy(&self) -> bool {
        matches!(self.act, Act::Down | Act::Dive(_) | Act::Cheer | Act::Sulk | Act::Bent | Act::Ouch | Act::Roll)
    }
    fn set(&mut self, a: Act) {
        self.act = a;
        self.at = 0;
    }
    pub fn say(&mut self, s: &str, frames: u32) {
        self.bubble = Some((s.to_string(), frames));
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Shot {
    pub kind: Kind,
    pub by: usize,
    pub valid: bool,
    pub bounced: bool,
}

#[derive(Clone, Debug)]
pub struct Ball {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    /// at a field kid's feet
    pub owner: Option<usize>,
    /// in the keeper's hands
    pub held: bool,
    pub last_touch: Option<usize>,
    pub last_field: Option<usize>,
    /// in the air since the last touch
    pub air: bool,
    /// who played it into the air
    pub from: Option<usize>,
    pub shot: Option<Shot>,
    pub cross_to: Option<usize>,
    /// where a cross comes down
    pub land: Option<(f32, f32)>,
    pub in_net: bool,
    /// on the goal's roof
    pub roof: bool,
    /// carried by the poodle
    pub dog: bool,
    pub spin: f32,
}

impl Ball {
    fn new() -> Self {
        Ball {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            owner: None,
            held: false,
            last_touch: None,
            last_field: None,
            air: false,
            from: None,
            shot: None,
            cross_to: None,
            land: None,
            in_net: false,
            roof: false,
            dog: false,
            spin: 0.0,
        }
    }
    pub fn speed(&self) -> f32 {
        (self.vx * self.vx + self.vy * self.vy + self.vz * self.vz).sqrt()
    }
    pub fn loose(&self) -> bool {
        self.owner.is_none() && !self.held && !self.dog
    }
    /// Where and when the ball comes down at height `z` (frames from now),
    /// ignoring the goal and the kids.
    pub fn when_at(&self, z: f32) -> Option<(f32, f32, f32)> {
        // z + vz t - G/2 t^2 = z  -> falling branch
        let a = -0.5 * G;
        let b = self.vz;
        let c = self.z - z;
        let d = b * b - 4.0 * a * c;
        if d < 0.0 {
            return None;
        }
        let t = (-b - d.sqrt()) / (2.0 * a);
        if t < 0.0 {
            return None;
        }
        Some((self.x + self.vx * t, self.y + self.vy * t, t))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// the crossbar contest decides the first keeper
    Latje,
    /// the counting-out rhyme
    CountOut,
    Play,
    Goal,
    Wissel,
    Car,
    /// somebody is on zero
    Over,
    Brand,
    Done,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Why {
    Caught,
    Out,
    Bounced,
    Fence,
}

#[derive(Clone, Debug)]
pub struct Banner {
    pub text: String,
    pub sub: String,
    pub t: u32,
    pub len: u32,
    pub color: u32,
}

#[derive(Clone, Debug)]
pub struct Popup {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub t: u32,
    pub color: u32,
}

/// The crossbar contest.
#[derive(Clone, Debug, Default)]
pub struct Latje {
    /// kids still in the contest
    pub left: Vec<usize>,
    pub turn: usize,
    pub round: usize,
    /// misses this round
    pub missed: Vec<usize>,
    /// 0 = aiming, 1 = ball flying
    pub stage: u32,
    pub st: u32,
    /// the needle (0..1, 0.5 = the bar)
    pub needle: f32,
    pub hit: Option<bool>,
    /// counting out: word and the kid pointed at
    pub word: usize,
    pub point: usize,
    pub start: usize,
}

/// The billenbrand.
#[derive(Clone, Debug, Default)]
pub struct Brand {
    pub shooters: Vec<usize>,
    pub turn: usize,
    pub stage: u32,
    pub st: u32,
    pub aim: f32,
    pub hits: u32,
    pub hit: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct Car {
    pub x: f32,
    pub color: usize,
}

#[derive(Clone, Debug)]
pub struct Dog {
    pub x: f32,
    pub y: f32,
    pub tx: f32,
    pub ty: f32,
    pub has: bool,
    pub t: u32,
    pub leaving: bool,
}

#[derive(Clone, Debug)]
pub struct Pigeon {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vz: f32,
    pub fly: bool,
    pub peck: u32,
}

pub struct World {
    pub venue: usize,
    pub kids: Vec<Kid>,
    pub brains: Vec<Brain>,
    pub keeper: usize,
    pub ball: Ball,
    pub phase: Phase,
    pub pt: u32,
    pub t: u32,
    pub play_t: u32,
    pub rng: Rng,
    pub sfx: Vec<Sfx>,
    pub banner: Option<Banner>,
    pub popups: Vec<Popup>,
    pub latje: Latje,
    pub brand: Brand,
    pub loser: Option<usize>,
    /// the mums called everybody in
    pub dinner: bool,
    pub why: Option<Why>,
    pub car: Option<Car>,
    pub next_car: u32,
    pub dog: Option<Dog>,
    pub next_dog: u32,
    pub pigeons: Vec<Pigeon>,
    pub shake: u32,
    /// frames nothing happened to a dead ball
    stuck: u32,
    prev: [Pad; 2],
    /// the goal that was just scored (for the banner and popups)
    pub last_goal: Option<(Kind, usize, i32)>,
    pub log: Vec<String>,
    /// every touch of the ball counts up (brains plan per touch)
    pub touches: u32,
    /// caught, out, bounced, fence, disallowed, keeper's ball, parries
    pub tally: [u32; 7],
    /// a bot plays this human's kid (screenshots)
    pub auto: [Option<BotKind>; 2],
    /// 1P: goal points the human must score for respect
    pub quota: Option<i32>,
}

impl World {
    /// `roster`: (kid id, control, CPU skill) for the four kids.
    pub fn new(venue: usize, roster: [(usize, Ctrl, f32); 4], seed: u64) -> Self {
        let mut rng = Rng(seed.wrapping_mul(0x2545_F491_4F6C_DD1D) | 1);
        let spots = [(150.0, 22.0), (170.0, 50.0), (196.0, 30.0), (214.0, 58.0)];
        let kids: Vec<Kid> = roster.iter().enumerate().map(|(k, &(id, ctrl, skill))| Kid::new(id, ctrl, skill, spots[k].0, spots[k].1)).collect();
        let brains = (0..4).map(|k| Brain::new(seed.wrapping_add(k as u64 * 977))).collect();
        let mut pigeons = Vec::new();
        for _ in 0..4 {
            pigeons.push(Pigeon { x: rng.range(80.0, 230.0), y: rng.range(4.0, 68.0), z: 0.0, vx: 0.0, vz: 0.0, fly: false, peck: rng.next() % 60 });
        }
        let next_car = 60 * 25 + rng.int(60 * 20) as u32;
        let next_dog = 60 * 35 + rng.int(60 * 30) as u32;
        let mut w = World {
            venue,
            kids,
            brains,
            keeper: 0,
            ball: Ball::new(),
            phase: Phase::Latje,
            pt: 0,
            t: 0,
            play_t: 0,
            rng,
            sfx: Vec::new(),
            banner: None,
            popups: Vec::new(),
            latje: Latje::default(),
            brand: Brand::default(),
            loser: None,
            dinner: false,
            why: None,
            car: None,
            next_car,
            dog: None,
            next_dog,
            pigeons,
            shake: 0,
            stuck: 0,
            prev: [Pad::default(); 2],
            last_goal: None,
            log: Vec::new(),
            touches: 0,
            tally: [0; 7],
            auto: [None; 2],
            quota: None,
        };
        w.latje.left = (0..4).collect();
        w.start_latje_turn();
        w
    }

    /// The probe or autopilot habits of a kid, if a bot plays it.
    pub fn bot_kind(&self, k: usize) -> Option<BotKind> {
        match self.kids[k].ctrl {
            Ctrl::Probe(b) => Some(b),
            Ctrl::Human(p) => self.auto[p.min(1)],
            Ctrl::Cpu => None,
        }
    }

    pub fn def(&self) -> &'static Venue {
        &VENUES[self.venue % VENUES.len()]
    }

    pub fn finished(&self) -> bool {
        self.phase == Phase::Done
    }

    pub fn field(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.kids.len()).filter(move |&k| k != self.keeper)
    }

    /// The kid a human plays.
    pub fn human_kid(&self, p: usize) -> Option<usize> {
        self.kids.iter().position(|k| k.ctrl == Ctrl::Human(p))
    }

    fn banner(&mut self, text: &str, sub: &str, len: u32, color: u32) {
        self.banner = Some(Banner { text: text.to_string(), sub: sub.to_string(), t: 0, len, color });
    }

    fn popup(&mut self, x: f32, y: f32, text: &str, color: u32) {
        self.popups.push(Popup { x, y, text: text.to_string(), t: 0, color });
    }

    fn gauss(&mut self) -> f32 {
        (self.rng.f() + self.rng.f() + self.rng.f() + self.rng.f() - 2.0) * 1.7
    }

    // ------------------------------------------------------------ update

    pub fn update(&mut self, pads: &[Pad; 2]) {
        self.t += 1;
        self.pt += 1;
        let e = [edge(pads[0], self.prev[0]), edge(pads[1], self.prev[1])];
        self.prev = *pads;
        let mut intents = [Intent::default(); 4];
        for (k, it) in intents.iter_mut().enumerate() {
            let auto = matches!(self.kids[k].ctrl, Ctrl::Human(p) if self.auto[p.min(1)].is_some());
            *it = match self.kids[k].ctrl {
                Ctrl::Human(p) if !auto => Intent::from_pad(pads[p.min(1)], e[p.min(1)]),
                _ => {
                    let mut b = std::mem::take(&mut self.brains[k]);
                    let i = bot::think(self, k, &mut b);
                    self.brains[k] = b;
                    i
                }
            };
        }
        match self.phase {
            Phase::Latje => self.update_latje(&intents),
            Phase::CountOut => self.update_count_out(),
            Phase::Play => {
                self.play_t += 1;
                self.update_play(&intents);
                if self.play_t >= MAX_GAME && self.phase == Phase::Play {
                    self.call_dinner();
                }
            }
            Phase::Goal => {
                self.auto_kids(false);
                self.step_ball_free();
                if self.pt > 130 {
                    self.give_keeper_ball();
                    self.phase = Phase::Play;
                    self.pt = 0;
                }
            }
            Phase::Wissel => {
                self.auto_kids(true);
                if self.pt > 110 {
                    self.give_keeper_ball();
                    self.phase = Phase::Play;
                    self.pt = 0;
                }
            }
            Phase::Car => self.update_car(),
            Phase::Over => {
                self.auto_kids(false);
                self.step_ball_free();
                if self.pt > 150 {
                    self.start_brand();
                }
            }
            Phase::Brand => self.update_brand(&intents),
            Phase::Done => {}
        }
        self.update_pigeons();
        self.tick_text();
    }

    fn tick_text(&mut self) {
        if let Some(b) = self.banner.as_mut() {
            b.t += 1;
            if b.t > b.len {
                self.banner = None;
            }
        }
        for p in self.popups.iter_mut() {
            p.t += 1;
            p.y -= 0.4;
        }
        self.popups.retain(|p| p.t < 70);
        for k in self.kids.iter_mut() {
            if let Some((_, n)) = k.bubble.as_mut() {
                if *n == 0 {
                    k.bubble = None;
                } else {
                    *n -= 1;
                }
            }
        }
        if self.shake > 0 {
            self.shake -= 1;
        }
    }

    // ------------------------------------------------------------ play

    fn update_play(&mut self, intents: &[Intent; 4]) {
        for k in 0..self.kids.len() {
            if k == self.keeper {
                self.control_keeper(k, intents[k]);
            } else {
                self.control_field(k, intents[k]);
            }
            self.kid_physics(k);
        }
        if self.phase != Phase::Play {
            return;
        }
        self.step_ball();
        if self.phase != Phase::Play {
            return;
        }
        self.contacts();
        if self.phase != Phase::Play {
            return;
        }
        self.update_dog();
        self.check_stuck();
        // a car comes while the keeper holds the ball
        if self.def().cars && self.ball.held && self.play_t > self.next_car && self.kids[self.keeper].act == Act::Hold {
            self.start_car();
        }
    }

    fn control_field(&mut self, k: usize, it: Intent) {
        let kid = &mut self.kids[k];
        kid.at += 1;
        if kid.cool > 0 {
            kid.cool -= 1;
        }
        let air = kid.air();
        match kid.act {
            Act::Swing { .. } if kid.at > 16 => kid.set(Act::Idle),
            Act::Down if kid.at > 34 => kid.set(Act::Idle),
            Act::Cheer | Act::Sulk if kid.at > 60 => kid.set(Act::Idle),
            _ => {}
        }
        let can_move = matches!(kid.act, Act::Idle) || matches!(kid.act, Act::Swing { .. });
        let sp = kid.speed() * if matches!(kid.act, Act::Swing { .. }) && !air { 0.3 } else { 1.0 };
        if !air {
            if can_move {
                let (mut mx, mut my) = (it.mx, it.my);
                let l = (mx * mx + my * my).sqrt();
                if l > 1.0 {
                    mx /= l;
                    my /= l;
                }
                kid.vx = mx * sp;
                kid.vy = my * sp * 0.75;
                if it.mx > 0.2 {
                    kid.face = 1;
                } else if it.mx < -0.2 {
                    kid.face = -1;
                }
            } else {
                kid.vx *= 0.8;
                kid.vy *= 0.8;
            }
            if it.face != 0 && can_move {
                kid.face = it.face;
            }
        }
        if !can_move {
            return;
        }
        let owner = self.ball.owner == Some(k);
        if it.b && !air && !matches!(kid.act, Act::Swing { .. }) {
            kid.vz = JUMP_V + 0.1 * snel(kid.id) as f32;
            kid.z = 0.1;
            if owner {
                self.ball.owner = None;
            }
        }
        if it.a || it.c {
            if owner {
                let to = it.to.or_else(|| self.pick_mate(k, it.mx, it.my));
                if it.c {
                    self.cross(k, to);
                } else {
                    self.ground_pass(k, to);
                }
                return;
            }
            let kid = &mut self.kids[k];
            if !matches!(kid.act, Act::Swing { .. }) {
                let rel = self.ball.z - kid.z;
                let kind = shot_kind(kid.face, kid.air(), rel, kid_def_acrobat(kid.id)).unwrap_or(if kid.air() { Kind::Header } else { Kind::Volley });
                kid.set(Act::Swing { kind, cross: it.c, hit: false });
                self.brains[k].aim = it.aim;
                self.brains[k].to = it.to;
                if !it.c && kind == Kind::Bicycle {
                    self.sfx.push(Sfx::Whiff);
                }
            }
        }
    }

    fn control_keeper(&mut self, k: usize, it: Intent) {
        let held = self.ball.held;
        let kid = &mut self.kids[k];
        kid.at += 1;
        if kid.cool > 0 {
            kid.cool -= 1;
        }
        match kid.act {
            Act::Catch | Act::Punch if kid.at > 14 => kid.set(Act::Idle),
            Act::Dive(_) if kid.at > 22 => kid.set(Act::Down),
            Act::Down if kid.at > 22 => kid.set(Act::Idle),
            Act::Roll if kid.at > 14 => kid.set(Act::Idle),
            Act::Swing { .. } | Act::Cheer | Act::Sulk if kid.at > 30 => kid.set(Act::Idle),
            _ => {}
        }
        if held && kid.act != Act::Hold && !matches!(kid.act, Act::Down | Act::Dive(_)) {
            kid.set(Act::Hold);
        }
        let air = kid.air();
        kid.face = 1;
        match kid.act {
            Act::Dive(d) => {
                kid.vy = d as f32 * 2.4 * (1.0 - kid.at as f32 / 26.0);
                kid.vx = 0.0;
                kid.z = (kid.at as f32 * 0.6).min(6.0) * if kid.at > 14 { (22 - kid.at.min(22)) as f32 / 8.0 } else { 1.0 };
                return;
            }
            Act::Down | Act::Roll => {
                kid.vx = 0.0;
                kid.vy = 0.0;
                return;
            }
            _ => {}
        }
        if !air {
            let sp = 1.2 + 0.1 * snel(kid.id) as f32;
            let (mut mx, mut my) = (it.mx, it.my);
            let l = (mx * mx + my * my).sqrt();
            if l > 1.0 {
                mx /= l;
                my /= l;
            }
            kid.vx = mx * sp;
            kid.vy = my * sp * 0.8;
        }
        if kid.act == Act::Hold {
            // a human keeper gets three seconds, then the ball goes by itself
            let auto = kid.human().is_some() && kid.at > 180;
            if it.a || it.c || auto {
                self.roll_out(k, it.my);
            }
            return;
        }
        if it.b && !air {
            kid.vz = JUMP_V + 0.15 * keep(kid.id) as f32;
            kid.z = 0.1;
        }
        if it.a {
            if it.my.abs() > 0.3 && !air {
                kid.set(Act::Dive(if it.my > 0.0 { 1 } else { -1 }));
            } else {
                kid.set(Act::Catch);
            }
        } else if it.c {
            kid.set(Act::Punch);
        }
    }

    fn kid_physics(&mut self, k: usize) {
        let keeper = k == self.keeper;
        let kid = &mut self.kids[k];
        kid.x += kid.vx;
        kid.y += kid.vy;
        if kid.z > 0.0 || kid.vz > 0.0 {
            if !matches!(kid.act, Act::Dive(_)) {
                kid.z += kid.vz;
                kid.vz -= KID_G;
            }
            if kid.z <= 0.0 {
                kid.z = 0.0;
                kid.vz = 0.0;
                if let Act::Swing { kind: Kind::Bicycle, hit: true, .. } = kid.act {
                    kid.set(Act::Down);
                }
            }
        }
        if keeper {
            kid.x = kid.x.clamp(GOAL_X + 3.0, 150.0);
        } else {
            kid.x = kid.x.clamp(GOAL_X + 4.0, 250.0);
        }
        kid.y = kid.y.clamp(-3.0, DEPTH + 4.0);
        let moving = kid.vx.abs() + kid.vy.abs() > 0.2;
        if moving && !kid.air() {
            kid.anim += 0.18 + 0.05 * (kid.vx.abs() + kid.vy.abs());
        }
        if self.ball.owner == Some(k) {
            kid.holding += 1;
        } else {
            kid.holding = 0;
        }
    }

    /// The mate a field kid passes to: the one in the held direction, or
    /// the one closest to the goal.
    pub fn pick_mate(&self, k: usize, mx: f32, my: f32) -> Option<usize> {
        let me = &self.kids[k];
        let mates: Vec<usize> = self.field().filter(|&m| m != k).collect();
        if mates.is_empty() {
            return None;
        }
        if mx.abs() + my.abs() > 0.1 {
            let l = (mx * mx + my * my).sqrt();
            let (dx, dy) = (mx / l, my / l);
            mates.iter().copied().max_by(|&a, &b| {
                let s = |m: usize| {
                    let o = &self.kids[m];
                    let (ox, oy) = (o.x - me.x, (o.y - me.y) * 1.6);
                    let d = (ox * ox + oy * oy).sqrt().max(1.0);
                    (ox * dx + oy * dy) / d
                };
                s(a).partial_cmp(&s(b)).unwrap_or(std::cmp::Ordering::Equal)
            })
        } else {
            mates.iter().copied().min_by(|&a, &b| self.kids[a].x.partial_cmp(&self.kids[b].x).unwrap_or(std::cmp::Ordering::Equal))
        }
    }

    /// Frames a dead ball has been lying with nobody near.
    pub fn stuck_for(&self) -> u32 {
        self.stuck
    }

    fn touch(&mut self, k: usize) {
        self.touches += 1;
        self.ball.last_touch = Some(k);
        if k != self.keeper {
            self.ball.last_field = Some(k);
        }
        self.ball.owner = None;
        self.ball.held = false;
        self.ball.roof = false;
        self.kids[k].cool = 10;
        self.stuck = 0;
    }

    /// A lob that comes down at `to`'s feet.
    pub fn cross(&mut self, k: usize, to: Option<usize>) {
        let (sx, sy) = (self.kids[k].x, self.kids[k].y);
        let (mut tx, mut ty) = match to {
            Some(m) => {
                let o = &self.kids[m];
                (o.x - 4.0 + o.vx * 20.0, o.y + o.vy * 20.0)
            }
            None => (sx - 50.0, goal_mid()),
        };
        let err = 2.5 + (5 - schot(self.kids[k].id)) as f32 * 1.6;
        tx += self.gauss() * err;
        ty += self.gauss() * err * 0.6;
        let ty = ty.clamp(2.0, DEPTH - 2.0);
        let z0 = self.ball.z.max(1.0);
        let dist = ((tx - sx).powi(2) + (ty - sy).powi(2)).sqrt();
        // a lob: high enough to head, higher the longer it is
        let apex = (36.0 + dist * 0.1 + self.rng.range(0.0, 8.0)).clamp(36.0, 58.0).max(z0 + 6.0);
        let vz = (2.0 * G * (apex - z0)).sqrt();
        let t = (vz + (vz * vz + 2.0 * G * z0).sqrt()) / G;
        self.touch(k);
        let b = &mut self.ball;
        b.z = z0;
        b.vx = (tx - b.x) / t;
        b.vy = (ty - b.y) / t;
        b.vz = vz;
        b.air = true;
        b.from = Some(k);
        b.shot = None;
        b.cross_to = to;
        b.land = Some((tx, ty));
        b.spin = 0.2;
        self.kids[k].stats.crosses += 1;
        self.sfx.push(Sfx::Kick);
    }

    pub fn ground_pass(&mut self, k: usize, to: Option<usize>) {
        let (sx, sy) = (self.kids[k].x, self.kids[k].y);
        let (tx, ty) = match to {
            Some(m) => (self.kids[m].x, self.kids[m].y),
            None => (sx + 40.0, sy),
        };
        let (dx, dy) = (tx - sx, ty - sy);
        let d = (dx * dx + dy * dy).sqrt().max(1.0);
        let v = (d * 0.032 + 0.6).clamp(1.2, 3.4);
        self.touch(k);
        let b = &mut self.ball;
        b.vx = dx / d * v;
        b.vy = dy / d * v;
        b.vz = 0.0;
        b.z = 0.0;
        b.air = false;
        b.from = None;
        b.shot = None;
        b.cross_to = to;
        b.land = None;
        self.sfx.push(Sfx::Kick);
    }

    fn roll_out(&mut self, k: usize, my: f32) {
        let kid = &mut self.kids[k];
        kid.set(Act::Roll);
        let (x, y) = (kid.x, kid.y);
        let spread = if my.abs() > 0.2 { my * 0.9 } else { self.rng.range(-0.6, 0.6) };
        let b = &mut self.ball;
        b.held = false;
        b.owner = None;
        b.x = x + 6.0;
        b.y = y;
        b.z = 0.0;
        b.vx = 2.6 + self.rng.f() * 0.5;
        b.vy = spread * 1.2;
        b.vz = 0.0;
        b.air = false;
        b.from = None;
        b.shot = None;
        b.cross_to = None;
        b.land = None;
        b.last_touch = Some(k);
        b.last_field = None;
        self.kids[k].cool = 20;
        self.stuck = 0;
        self.sfx.push(Sfx::Kick);
    }

    /// The shot itself: aimed at the goal, spread by the stats, the distance
    /// and how well the ball was met.
    fn shoot(&mut self, k: usize, kind: Kind, off: f32) {
        let kid = &self.kids[k];
        let stat = if kind.head() { kop(kid.id) } else { schot(kid.id) };
        let aim = self.brains[k].aim.clamp(-1.0, 1.0);
        let (sx, sy) = (self.ball.x, self.ball.y);
        let z0 = self.ball.z.max(1.0);
        let mut ty = goal_mid() + aim * 11.0;
        let mut tz = if kind == Kind::Header { 11.0 } else { 13.0 };
        let dist = ((sx - GOAL_X).powi(2) + (sy - ty).powi(2)).sqrt();
        let (sdy, sdz) = kind.spread();
        let f = (1.45 - 0.15 * stat as f32) * (0.5 + dist / 110.0) * (1.0 + off);
        ty += self.gauss() * sdy * f;
        tz += self.gauss() * sdz * f;
        let speed = kind.speed() * (0.85 + 0.05 * stat as f32);
        let t = (dist / speed).max(6.0);
        let valid = self.ball.air && self.ball.from.is_some() && self.ball.from != Some(k) && self.ball.z > 1.0;
        self.touch(k);
        let b = &mut self.ball;
        b.vx = (GOAL_X - 2.0 - sx) / t;
        b.vy = (ty - sy) / t;
        b.vz = (tz - z0 + 0.5 * G * t * t) / t;
        b.z = z0;
        b.air = true;
        b.from = Some(k);
        b.shot = Some(Shot { kind, by: k, valid, bounced: false });
        b.cross_to = None;
        b.land = None;
        b.spin = 0.4;
        self.kids[k].stats.shots += 1;
        self.sfx.push(if kind.head() { Sfx::Header } else { Sfx::Kick });
        if kind == Kind::Bicycle {
            self.shake = 6;
        }
    }

    /// Swings, keeper hands, bodies and dribbling.
    fn contacts(&mut self) {
        if !self.ball.loose() || self.ball.in_net {
            // at a kid's feet: only the keeper's hands can take it (not
            // implemented: field kids keep the ball) and only swings matter
            if let Some(o) = self.ball.owner {
                self.dribble(o);
            }
            return;
        }
        let b = self.ball.clone();
        // keeper first
        let kp = self.keeper;
        if self.kids[kp].cool == 0 && self.keeper_contact(kp, &b) {
            return;
        }
        // swings
        for k in 0..self.kids.len() {
            if k == kp {
                continue;
            }
            let kid = &self.kids[k];
            if kid.cool > 0 {
                continue;
            }
            if let Act::Swing { cross, hit: false, .. } = kid.act {
                if kid.at > WINDOW + if kid.air() { 6 } else { 0 } {
                    continue;
                }
                let rel = b.z - kid.z;
                let dx = b.x - (kid.x + kid.face as f32 * 3.0);
                let dy = b.y - kid.y;
                let near = (dx / 10.0).powi(2) + (dy / 6.5).powi(2);
                if near > 1.0 || rel < -2.0 || rel > 38.0 {
                    continue;
                }
                // on the ground: a pass along the ground
                if b.z < 1.5 && b.vz.abs() < 0.8 {
                    let to = self.brains[k].to.or_else(|| self.pick_mate(k, 0.0, 0.0));
                    if cross {
                        self.cross(k, to);
                    } else {
                        self.ground_pass(k, to);
                    }
                    self.mark_hit(k, None);
                    return;
                }
                let Some(kind) = shot_kind(kid.face, kid.air(), rel, kid_def_acrobat(kid.id)) else { continue };
                let sweet = match kind {
                    Kind::Volley | Kind::Heel => 6.0,
                    Kind::Chest => 16.0,
                    Kind::Header => 26.0,
                    Kind::Bicycle => 20.0,
                };
                let off = (near.sqrt() * 0.6 + ((rel - sweet).abs() / 14.0).min(1.0) * 0.4).min(1.0);
                if cross {
                    let to = self.brains[k].to.or_else(|| self.pick_mate(k, 0.0, 0.0));
                    self.cross(k, to);
                    self.mark_hit(k, Some(kind));
                } else {
                    self.shoot(k, kind, off);
                    self.mark_hit(k, Some(kind));
                }
                return;
            }
        }
        // bodies and dribbling
        for k in 0..self.kids.len() {
            if k == kp || self.kids[k].cool > 0 {
                continue;
            }
            let kid = &self.kids[k];
            let rel = b.z - kid.z;
            let dx = b.x - kid.x;
            let dy = b.y - kid.y;
            let sp = (b.vx * b.vx + b.vy * b.vy).sqrt();
            if b.z < 2.0 && b.vz.abs() < 1.0 && sp < 2.7 && dx.abs() < 7.0 && dy.abs() < 4.5 && !kid.busy() && !kid.air() {
                // at the feet
                self.touch(k);
                let bb = &mut self.ball;
                bb.owner = Some(k);
                bb.from = None;
                bb.air = false;
                bb.shot = None;
                bb.cross_to = None;
                bb.land = None;
                self.kids[k].cool = 0;
                return;
            }
            let mine = b.cross_to == Some(k);
            let swinging = matches!(kid.act, Act::Swing { hit: false, .. }) && kid.at <= WINDOW + 6;
            if mine && !swinging && !kid.air() && !kid.busy() && dx.abs() < 7.0 && dy.abs() < 4.5 && rel < 4.0 {
                // the receiver lets it drop and traps it
                self.touch(k);
                let bb = &mut self.ball;
                bb.owner = Some(k);
                bb.from = None;
                bb.air = false;
                bb.shot = None;
                bb.cross_to = None;
                bb.land = None;
                self.kids[k].cool = 0;
                return;
            }
            if !mine && b.shot.is_some() && b.speed() > 1.0 && dx.abs() < 4.5 && dy.abs() < 3.5 && (2.0..30.0).contains(&rel) && !kid.busy() {
                // a shot hits somebody in the way
                self.touch(k);
                let bb = &mut self.ball;
                bb.vx *= -0.35;
                bb.vy *= 0.5;
                bb.vz = bb.vz.abs() * 0.4 + 0.5;
                bb.from = Some(k);
                bb.shot = None;
                bb.land = None;
                self.sfx.push(Sfx::Bounce);
                return;
            }
        }
    }

    fn mark_hit(&mut self, k: usize, kind: Option<Kind>) {
        if let Act::Swing { kind: kk, cross, .. } = self.kids[k].act {
            let at = self.kids[k].at;
            self.kids[k].act = Act::Swing { kind: kind.unwrap_or(kk), cross, hit: true };
            self.kids[k].at = at;
        }
    }

    fn dribble(&mut self, o: usize) {
        let kid = &self.kids[o];
        if kid.busy() || kid.air() || o == self.keeper {
            self.ball.owner = None;
            return;
        }
        let b = &mut self.ball;
        b.x = kid.x + kid.face as f32 * 5.0 + kid.vx * 2.0;
        b.y = kid.y + 0.5;
        b.z = 0.0;
        b.vx = kid.vx;
        b.vy = kid.vy;
        b.vz = 0.0;
        b.spin = kid.vx.abs() * 0.1;
        // dribbled into the keeper's area: the keeper may grab it
        let (bx, by) = (b.x, b.y);
        let kp = &self.kids[self.keeper];
        if in_area(bx, by) && (kp.x - bx).abs() < 7.0 && (kp.y - by).abs() < 5.0 && matches!(kp.act, Act::Catch | Act::Dive(_)) {
            self.ball.owner = None;
            self.caught(self.keeper);
        }
    }

    /// The keeper meets the ball: catch, parry, punch or body block.
    fn keeper_contact(&mut self, kp: usize, b: &Ball) -> bool {
        let kid = &self.kids[kp];
        let rel = b.z - kid.z;
        let (hx, hy, rx, ry, z0, z1) = match kid.act {
            Act::Catch | Act::Punch => (kid.x + 3.0, kid.y, 9.0, 7.5, -2.0, 34.0),
            Act::Dive(d) => (kid.x + 2.0, kid.y + d as f32 * 6.0, 10.0, 12.0, -3.0, 20.0),
            _ => (kid.x + 2.0, kid.y, 5.0, 4.5, 0.0, 28.0),
        };
        let dx = b.x - hx;
        let dy = b.y - hy;
        let near = (dx / rx).powi(2) + (dy / ry).powi(2);
        if near > 1.0 || rel < z0 || rel > z1 {
            return false;
        }
        let sp = b.speed();
        let area = in_area(kid.x, kid.y) || kid.pts == 1;
        match kid.act {
            Act::Catch | Act::Dive(_) => {
                // a slow ball on the ground: just pick it up
                if b.z < 3.0 && sp < 1.6 && area {
                    self.caught(kp);
                    return true;
                }
                let k = keep(kid.id) as f32;
                let mut p = 0.32 + 0.11 * k - 0.10 * (sp - 3.0) - 0.35 * near.sqrt();
                if matches!(kid.act, Act::Dive(_)) {
                    p *= 0.85;
                }
                if rel > 26.0 {
                    p -= 0.1;
                }
                let p = p.clamp(0.08, 0.96);
                if area && self.rng.f() < p {
                    self.caught(kp);
                } else {
                    self.parry(kp, 0.0);
                }
                true
            }
            Act::Punch => {
                self.parry(kp, 1.0);
                true
            }
            Act::Hold => false,
            _ => {
                if sp > 1.4 && !matches!(kid.act, Act::Down) {
                    self.parry(kp, -0.4);
                    true
                } else if sp <= 1.4 && b.z < 3.0 && area && kid.ctrl == Ctrl::Cpu {
                    false
                } else {
                    false
                }
            }
        }
    }

    fn parry(&mut self, kp: usize, power: f32) {
        let r1 = self.rng.range(-1.0, 1.0);
        let r2 = self.rng.f();
        self.ball.last_touch = Some(kp);
        self.ball.owner = None;
        self.ball.held = false;
        self.kids[kp].cool = 12;
        self.touches += 1;
        self.tally[6] += 1;
        self.stuck = 0;
        let b = &mut self.ball;
        b.vx = b.vx.abs() * (0.45 + 0.2 * power) + 0.9 + 1.4 * power.max(0.0);
        b.vy = b.vy * 0.4 + r1 * (1.0 + power.max(0.0));
        b.vz = (b.vz * 0.3).max(0.0) + 1.3 + r2 + 1.2 * power.max(0.0);
        b.air = true;
        b.from = Some(kp);
        b.cross_to = None;
        b.land = None;
        b.spin = 0.5;
        self.sfx.push(Sfx::Punch);
        if power > 0.5 {
            self.popup(self.kids[kp].x, self.kids[kp].y - 34.0, "WEGGESTOMPT!", 0xF8E840);
        } else {
            self.popup(self.kids[kp].x, self.kids[kp].y - 34.0, "GEKEERD!", 0xF8E840);
        }
    }

    fn caught(&mut self, kp: usize) {
        self.sfx.push(Sfx::Catch);
        let lf = self.ball.last_field;
        let was_shot = self.ball.shot.is_some();
        self.ball.held = true;
        self.ball.owner = None;
        self.ball.shot = None;
        self.ball.cross_to = None;
        self.ball.land = None;
        self.ball.air = false;
        self.kids[kp].set(Act::Hold);
        if was_shot {
            self.kids[kp].stats.saves += 1;
        }
        match lf {
            Some(f) if f != kp => self.wissel(f, Why::Caught),
            _ => {}
        }
    }

    // ------------------------------------------------------------ ball

    /// The ball outside play (after a goal): it just flies and rests.
    fn step_ball_free(&mut self) {
        if self.ball.held || self.ball.owner.is_some() {
            return;
        }
        self.ball_motion();
    }

    fn ball_motion(&mut self) -> bool {
        let b = &mut self.ball;
        let px = b.x;
        b.vz -= G;
        b.x += b.vx;
        b.y += b.vy;
        b.z += b.vz;
        b.spin *= 0.99;
        let mut ground = false;
        if b.in_net {
            b.vx *= 0.86;
            b.vy *= 0.86;
            if b.x < NET_X + BALL_R {
                b.x = NET_X + BALL_R;
                b.vx = b.vx.abs() * 0.1;
            }
            b.y = b.y.clamp(G_Y0 + BALL_R, G_Y1 - BALL_R);
            if b.z > BAR - BALL_R {
                b.z = BAR - BALL_R;
                b.vz = -b.vz.abs() * 0.2;
            }
        }
        // the wall behind the goal
        if b.x < WALL_X + BALL_R {
            b.x = WALL_X + BALL_R;
            b.vx = b.vx.abs() * 0.5;
        }
        // the roof of the goal
        if !b.in_net && b.x < GOAL_X && b.x > NET_X && (G_Y0..=G_Y1).contains(&b.y) && b.z < BAR + BALL_R && b.z > BAR - 4.0 && b.vz < 0.0 {
            b.z = BAR + BALL_R;
            b.vz = -b.vz * 0.25;
            b.vx *= 0.6;
            b.roof = true;
        }
        let _ = px;
        if b.z <= 0.0 {
            b.z = 0.0;
            if b.vz < -0.7 {
                b.vz = -b.vz * 0.55;
                ground = true;
            } else {
                b.vz = 0.0;
                b.vx *= 0.972;
                b.vy *= 0.972;
                if b.vx.abs() + b.vy.abs() > 0.05 {
                    ground = true;
                }
                if b.vx.abs() + b.vy.abs() < 0.03 {
                    b.vx = 0.0;
                    b.vy = 0.0;
                }
            }
        }
        if b.y < -16.0 {
            b.y = -16.0;
            b.vy = b.vy.abs() * 0.3;
        }
        if b.y > DEPTH + 18.0 {
            b.y = DEPTH + 18.0;
            b.vy = -b.vy.abs() * 0.3;
        }
        if b.x > 262.0 {
            b.x = 262.0;
            b.vx = -b.vx.abs() * 0.3;
        }
        ground
    }

    fn step_ball(&mut self) {
        if self.ball.held {
            let kp = &self.kids[self.keeper];
            let b = &mut self.ball;
            b.x = kp.x + 4.0;
            b.y = kp.y + 0.5;
            b.z = kp.z + 15.0;
            b.vx = 0.0;
            b.vy = 0.0;
            b.vz = 0.0;
            return;
        }
        if self.ball.owner.is_some() {
            return;
        }
        if self.ball.dog {
            if let Some(d) = &self.dog {
                self.ball.x = d.x - 6.0;
                self.ball.y = d.y;
                self.ball.z = 4.0;
            }
            return;
        }
        let px = self.ball.x;
        let ground = self.ball_motion();
        self.posts_and_bar();
        // into the goal?
        let b = &self.ball;
        if !b.in_net && px > GOAL_X && b.x <= GOAL_X && b.y > G_Y0 + 1.0 && b.y < G_Y1 - 1.0 && b.z < BAR - 1.0 {
            self.ball.in_net = true;
            self.sfx.push(Sfx::Net);
            self.into_goal();
            return;
        }
        if ground {
            let was_air = self.ball.air;
            self.ball.air = false;
            if let Some(s) = self.ball.shot.as_mut() {
                s.bounced = true;
            }
            if was_air {
                self.ball.cross_to = None;
                self.ball.land = None;
                self.sfx.push(Sfx::Bounce);
                self.scare_pigeons(self.ball.x, self.ball.y, 30.0);
            }
            let b = &self.ball;
            let out = !b.in_net && (b.x > FIELD_X1 || b.y < 0.0 || b.y > DEPTH || b.x < GOAL_X);
            if out {
                let fence = self.def().fence && b.y < 0.0;
                match b.last_field {
                    Some(f) => self.wissel(f, if fence { Why::Fence } else { Why::Out }),
                    None => self.keepers_ball(),
                }
            }
        }
    }

    fn posts_and_bar(&mut self) {
        let b = &mut self.ball;
        if b.in_net {
            return;
        }
        let mut hit = None;
        for py in [G_Y0, G_Y1] {
            let (dx, dy) = (b.x - GOAL_X, (b.y - py) * 1.3);
            let d = (dx * dx + dy * dy).sqrt();
            if d < BALL_R + 1.2 && b.z < BAR + 1.0 && d > 0.01 {
                let (nx, ny) = (dx / d, dy / d / 1.3);
                let dot = b.vx * nx + b.vy * ny;
                if dot < 0.0 {
                    b.vx = (b.vx - 2.0 * dot * nx) * 0.6;
                    b.vy = (b.vy - 2.0 * dot * ny) * 0.6;
                    b.x = GOAL_X + nx * (BALL_R + 1.3);
                    hit = Some(false);
                }
            }
        }
        if b.y > G_Y0 && b.y < G_Y1 {
            let (dx, dz) = (b.x - GOAL_X, b.z - BAR);
            let d = (dx * dx + dz * dz).sqrt();
            if d < BALL_R + 1.0 && d > 0.01 {
                let (nx, nz) = (dx / d, dz / d);
                let dot = b.vx * nx + b.vz * nz;
                if dot < 0.0 {
                    b.vx = (b.vx - 2.0 * dot * nx) * 0.65;
                    b.vz = (b.vz - 2.0 * dot * nz) * 0.65;
                    b.x = GOAL_X + nx * (BALL_R + 1.1);
                    b.z = BAR + nz * (BALL_R + 1.1);
                    hit = Some(true);
                }
            }
        }
        if let Some(bar) = hit {
            self.sfx.push(Sfx::Post);
            let (x, y) = (self.ball.x, self.ball.y);
            if self.phase == Phase::Play {
                self.popup(x + 6.0, y - 40.0, if bar { "LAT!" } else { "PAAL!" }, 0xF8F8F8);
            }
        }
    }

    /// The ball crossed the line between the posts.
    fn into_goal(&mut self) {
        let shot = self.ball.shot;
        let lf = self.ball.last_field;
        match shot {
            Some(s) if s.valid && !s.bounced => self.goal(s.kind, s.by),
            Some(s) if s.bounced => self.wissel(s.by, Why::Bounced),
            _ if !self.ball.air => match lf {
                Some(f) => self.wissel(f, Why::Bounced),
                None => self.keepers_ball(),
            },
            _ => {
                // straight from a cross, or off your own touch
                let (x, y) = (GOAL_X + 10.0, goal_mid());
                self.popup(x, y - 40.0, "TELT NIET!", 0xE87860);
                self.banner("AFGEKEURD!", "FIRST THROUGH THE AIR FROM A MATE", 100, 0xE87860);
                self.sfx.push(Sfx::Whistle);
                self.phase = Phase::Goal;
                self.pt = 40;
                self.last_goal = None;
                self.tally[4] += 1;
            }
        }
    }

    fn goal(&mut self, kind: Kind, by: usize) {
        let kp = self.keeper;
        let v = kind.value();
        let before = self.kids[kp].pts;
        let after = before - v;
        self.kids[by].stats.goals[kind.index()] += 1;
        let had = self.kids[by].stats.scored;
        self.kids[by].stats.scored += v;
        if let Some(q) = self.quota {
            if self.kids[by].human().is_some() && had < q && had + v >= q {
                let (x, y) = (self.kids[by].x, self.kids[by].y);
                self.popup(x, y - 52.0, "RESPECT!", 0x60E0F8);
            }
        }
        self.kids[kp].stats.conceded += v;
        self.kids[by].set(Act::Cheer);
        self.kids[kp].set(Act::Sulk);
        self.sfx.push(Sfx::Cheer);
        self.last_goal = Some((kind, by, v));
        self.shake = if kind == Kind::Bicycle { 20 } else { 8 };
        let name = kid(self.kids[by].id).name;
        let title = if kind == Kind::Bicycle { "OMHAAL!!!".to_string() } else { "GOAL!".to_string() };
        let (x, y) = (GOAL_X + 6.0, self.ball.y);
        self.popup(x + 10.0, y - 44.0, &format!("-{}", v), 0xF8E840);
        let gh = GOAL_HECKLES[self.rng.int(GOAL_HECKLES.len() as i32) as usize];
        let ks = KEEPER_SULKS[self.rng.int(KEEPER_SULKS.len() as i32) as usize];
        self.kids[by].say(gh, 70);
        if after == 0 {
            self.kids[kp].pts = 0;
            self.banner(&title, &format!("{} {}! {} OP NUL!", kind.name(), name, kid(self.kids[kp].id).name), 150, 0xF8E840);
            self.loser = Some(kp);
            self.phase = Phase::Over;
            self.pt = 0;
            self.sfx.push(Sfx::Whistle);
            self.log.push(format!("{} on zero at {}s", kid(self.kids[kp].id).name, self.play_t / 60));
            return;
        }
        if after < 0 {
            self.kids[kp].pts = 10 + after;
            self.banner("DOORGESCHOTEN!", &format!("{} - {} = {}: BACK TO {}", before, v, after, 10 + after), 130, 0x60E0F8);
            self.kids[kp].say("YES! TERUG NAAR TIEN!", 80);
        } else {
            self.kids[kp].pts = after;
            self.banner(&title, &format!("{} {}! -{}", name, kind.name(), v), 120, 0xF8E840);
            self.kids[kp].say(ks, 70);
        }
        self.phase = Phase::Goal;
        self.pt = 0;
    }

    fn wissel(&mut self, new_k: usize, why: Why) {
        let old = self.keeper;
        if new_k == old {
            self.keepers_ball();
            return;
        }
        self.keeper = new_k;
        self.why = Some(why);
        self.tally[why as usize] += 1;
        self.touches += 1;
        self.kids[new_k].stats.wissels += 1;
        self.kids[new_k].set(Act::Sulk);
        self.kids[old].set(Act::Cheer);
        self.kids[new_k].vz = 0.0;
        self.kids[new_k].z = 0.0;
        let name = kid(self.kids[new_k].id).name;
        let (title, col) = match why {
            Why::Caught => {
                self.kids[old].say("HEBBES!", 60);
                ("GEVANGEN!", 0x60E0F8)
            }
            Why::Out => ("UIT!", 0xF8F8F8),
            Why::Bounced => ("GESTUITERD!", 0xF89060),
            Why::Fence => {
                self.kids[old].say("MEVROUW KNORR!", 80);
                ("OVER HET HEK!", 0xF8F8F8)
            }
        };
        let len = if why == Why::Fence { 60 } else { 110 };
        self.banner(title, &format!("WISSEL! {} IN DE GOAL", name), len, col);
        self.sfx.push(Sfx::Groan);
        self.ball.shot = None;
        self.ball.cross_to = None;
        self.ball.land = None;
        self.ball.owner = None;
        self.phase = Phase::Wissel;
        self.pt = 0;
        self.brains.iter_mut().for_each(|b| b.reset());
    }

    /// Nobody to blame: the keeper takes the ball and starts again.
    fn keepers_ball(&mut self) {
        self.tally[5] += 1;
        self.touches += 1;
        self.ball.shot = None;
        self.ball.cross_to = None;
        self.ball.land = None;
        self.ball.owner = None;
        self.phase = Phase::Wissel;
        self.pt = 60;
        self.why = None;
    }

    fn give_keeper_ball(&mut self) {
        let kp = self.keeper;
        let b = &mut self.ball;
        b.held = true;
        b.owner = None;
        b.in_net = false;
        b.roof = false;
        b.dog = false;
        b.air = false;
        b.shot = None;
        b.from = None;
        b.cross_to = None;
        b.land = None;
        b.last_field = None;
        b.last_touch = Some(kp);
        let kid = &mut self.kids[kp];
        kid.set(Act::Hold);
        self.stuck = 0;
        self.touches += 1;
        if let Some(d) = self.dog.as_mut() {
            d.has = false;
            d.leaving = true;
        }
    }

    /// Kids walk to their places while play waits.
    fn auto_kids(&mut self, wissel: bool) {
        let kp = self.keeper;
        let mid = goal_mid();
        for k in 0..self.kids.len() {
            let kid = &mut self.kids[k];
            kid.at += 1;
            if matches!(kid.act, Act::Swing { .. } | Act::Catch | Act::Punch | Act::Dive(_) | Act::Roll) && kid.at > 20 {
                kid.set(Act::Idle);
            }
            if kid.act == Act::Down && kid.at > 30 {
                kid.set(Act::Idle);
            }
            if kid.z > 0.0 {
                kid.z = (kid.z + kid.vz).max(0.0);
                kid.vz -= KID_G;
                if kid.z == 0.0 {
                    kid.vz = 0.0;
                }
            }
            let (tx, ty) = if k == kp {
                (GOAL_X + 8.0, mid)
            } else if wissel {
                let slot = [(118.0, 22.0), (140.0, 52.0), (170.0, 34.0), (200.0, 60.0)][k];
                (kid.x.max(slot.0 - 10.0).min(slot.0 + 40.0), kid.y * 0.5 + slot.1 * 0.5)
            } else {
                (kid.x.max(AREA_X + 20.0), kid.y)
            };
            let (dx, dy) = (tx - kid.x, ty - kid.y);
            let d = (dx * dx + dy * dy).sqrt();
            let sp = if k == kp && wissel { 2.2 } else { 1.2 };
            if d > 1.5 && !matches!(kid.act, Act::Down | Act::Bent) {
                kid.vx = dx / d * sp;
                kid.vy = dy / d * sp * 0.8;
                kid.x += kid.vx;
                kid.y += kid.vy;
                kid.anim += 0.25;
                if k != kp {
                    kid.face = if dx < 0.0 { -1 } else { 1 };
                }
            } else {
                kid.vx = 0.0;
                kid.vy = 0.0;
                if k != kp {
                    kid.face = -1;
                }
            }
        }
        if wissel && !self.ball.held {
            // the ball goes back to the new keeper
            let kpk = &self.kids[kp];
            let b = &mut self.ball;
            let (tx, ty) = (kpk.x + 4.0, kpk.y);
            b.x += (tx - b.x) * 0.06;
            b.y += (ty - b.y) * 0.06;
            let k = (self.pt as f32 / 110.0).min(1.0);
            b.z = (k * std::f32::consts::PI).sin() * 30.0 + 15.0 * k;
            b.in_net = false;
            b.dog = false;
        }
    }

    fn check_stuck(&mut self) {
        let b = &self.ball;
        let dead = b.loose() && b.speed() < 0.05 && b.z < 0.5;
        let nobody = self.field().all(|k| {
            let kid = &self.kids[k];
            (kid.x - b.x).abs() + (kid.y - b.y).abs() > 14.0
        });
        if (dead && nobody) || b.roof {
            self.stuck += 1;
        } else {
            self.stuck = 0;
        }
        // a ball the keeper can reach in his area: the keeper takes it
        if self.stuck > 60 * 8 {
            self.stuck = 0;
            self.keepers_ball();
        }
    }

    fn call_dinner(&mut self) {
        // the kid with the fewest points loses
        let mut lo = 0;
        for k in 1..self.kids.len() {
            if self.kids[k].pts < self.kids[lo].pts {
                lo = k;
            }
        }
        self.dinner = true;
        self.loser = Some(lo);
        self.phase = Phase::Over;
        self.pt = 0;
        self.banner("ETEN!!!", &format!("ALL MUMS CALL. {} HAS THE FEWEST", kid(self.kids[lo].id).name), 150, 0xF8E840);
        self.sfx.push(Sfx::Whistle);
    }

    // ------------------------------------------------------------ latjetrap

    fn start_latje_turn(&mut self) {
        let l = &mut self.latje;
        l.stage = 0;
        l.st = 0;
        l.hit = None;
        let k = l.left[l.turn];
        let b = &mut self.ball;
        b.x = SPOT_X - 4.0;
        b.y = goal_mid();
        b.z = 0.0;
        b.vx = 0.0;
        b.vy = 0.0;
        b.vz = 0.0;
        b.held = false;
        b.owner = None;
        b.in_net = false;
        let kid = &mut self.kids[k];
        kid.x = SPOT_X;
        kid.y = goal_mid();
        kid.face = -1;
        kid.set(Act::Idle);
    }

    fn update_latje(&mut self, intents: &[Intent; 4]) {
        let shooter = self.latje.left[self.latje.turn];
        // the others watch from the side
        for k in 0..self.kids.len() {
            if k == shooter {
                continue;
            }
            let slot = [(150.0, 8.0), (170.0, 6.0), (190.0, 9.0), (210.0, 7.0)][k];
            let kid = &mut self.kids[k];
            let (dx, dy) = (slot.0 - kid.x, slot.1 - kid.y);
            let d = (dx * dx + dy * dy).sqrt();
            if d > 1.5 {
                kid.x += dx / d * 1.6;
                kid.y += dy / d * 1.6;
                kid.anim += 0.25;
                kid.face = if dx < 0.0 { -1 } else { 1 };
            } else {
                kid.face = -1;
            }
            kid.at += 1;
            if matches!(kid.act, Act::Cheer | Act::Sulk) && kid.at > 60 {
                kid.set(Act::Idle);
            }
        }
        let l = &mut self.latje;
        l.st += 1;
        match l.stage {
            0 => {
                l.needle = 0.5 + 0.5 * (l.st as f32 * 0.075).sin();
                let go = intents[shooter].a || (self.kids[shooter].human().is_some() && l.st > 600);
                if go && l.st > 20 {
                    let v = l.needle;
                    self.latje_shoot(shooter, v);
                }
            }
            _ => {
                if l.st < 90 {
                    self.ball_motion();
                    self.posts_and_bar();
                    let b = &self.ball;
                    if !b.in_net && b.x <= GOAL_X && b.y > G_Y0 && b.y < G_Y1 && b.z < BAR {
                        self.ball.in_net = true;
                        self.sfx.push(Sfx::Net);
                    }
                    if self.latje.st == 30 {
                        let hit = self.latje.hit == Some(true);
                        let kid = &mut self.kids[shooter];
                        kid.set(if hit { Act::Cheer } else { Act::Sulk });
                        if hit {
                            self.sfx.push(Sfx::Cheer);
                            self.popup(GOAL_X + 4.0, goal_mid() - 44.0, "LAT! VEILIG!", 0xF8E840);
                        } else {
                            self.sfx.push(Sfx::Groan);
                            self.popup(GOAL_X + 4.0, goal_mid() - 44.0, "MIS!", 0xE87860);
                        }
                    }
                } else {
                    self.next_latje();
                }
            }
        }
    }

    /// The shot at the crossbar: the needle sets the height at the line.
    pub fn latje_shoot(&mut self, k: usize, v: f32) {
        let tz = BAR + (v - 0.5) * 44.0;
        let hit = (tz - BAR).abs() < 2.8;
        let l = &mut self.latje;
        l.stage = 1;
        l.st = 0;
        l.hit = Some(hit);
        if !hit {
            l.missed.push(k);
        }
        let b = &mut self.ball;
        let t = (b.x - GOAL_X) / 4.0;
        b.vx = (GOAL_X - b.x) / t;
        b.vy = 0.0;
        b.vz = (tz - b.z + 0.5 * G * t * t) / t;
        b.z = 1.0;
        b.in_net = false;
        let kid = &mut self.kids[k];
        kid.set(Act::Swing { kind: Kind::Volley, cross: false, hit: true });
        self.sfx.push(Sfx::Kick);
    }

    fn next_latje(&mut self) {
        let l = &mut self.latje;
        l.turn += 1;
        if l.turn < l.left.len() {
            self.start_latje_turn();
            return;
        }
        // the round is over
        let missed = std::mem::take(&mut l.missed);
        l.round += 1;
        l.turn = 0;
        if missed.len() == 1 {
            self.first_keeper(missed[0]);
            return;
        }
        if !missed.is_empty() {
            l.left = missed;
        }
        if l.round >= 3 {
            l.word = 0;
            l.st = 0;
            l.start = self.rng.int(l.left.len() as i32) as usize;
            l.point = l.start;
            self.phase = Phase::CountOut;
            self.pt = 0;
            self.banner("AFTELLEN!", "IEN MIEN MUTTEN...", 90, 0xF8E840);
            return;
        }
        let names: Vec<&str> = self.latje.left.iter().map(|&k| kid(self.kids[k].id).name).collect();
        self.banner(&format!("RONDE {}", self.latje.round + 1), &names.join(" "), 80, 0xF8F8F8);
        self.start_latje_turn();
    }

    fn update_count_out(&mut self) {
        let l = &mut self.latje;
        l.st += 1;
        if l.st.is_multiple_of(18) && l.word < COUNT_OUT.len() {
            l.word += 1;
            l.point = (l.start + l.word - 1) % l.left.len();
            self.sfx.push(Sfx::Blip);
        }
        if l.word >= COUNT_OUT.len() && l.st > 18 * COUNT_OUT.len() as u32 + 50 {
            let k = l.left[l.point];
            self.first_keeper(k);
        }
    }

    fn first_keeper(&mut self, k: usize) {
        self.keeper = k;
        self.kids[k].pts = 11;
        self.kids[k].set(Act::Sulk);
        self.kids[k].stats.wissels += 1;
        let name = kid(self.kids[k].id).name;
        self.banner(&format!("{} KEEPT!", name), "STARTS WITH 11 POINTS", 120, 0xF8E840);
        self.sfx.push(Sfx::Whistle);
        self.ball.in_net = false;
        self.phase = Phase::Wissel;
        self.pt = 0;
        self.why = None;
    }

    // ------------------------------------------------------------ car

    fn start_car(&mut self) {
        self.car = Some(Car { x: 290.0, color: self.rng.int(4) as usize });
        self.phase = Phase::Car;
        self.pt = 0;
        self.banner("AUTO!!!", "EVERYBODY TO THE SIDE", 100, 0xF8F8F8);
        self.sfx.push(Sfx::Honk);
        self.next_car = self.play_t + 60 * 40 + self.rng.int(60 * 40) as u32;
        let k = self.rng.int(4) as usize;
        self.kids[k].say("AUTO!", 60);
    }

    fn update_car(&mut self) {
        let kp = self.keeper;
        for k in 0..self.kids.len() {
            let kid = &mut self.kids[k];
            kid.at += 1;
            if k == kp {
                continue;
            }
            let ty = if kid.y < 36.0 { -2.0 } else { DEPTH + 3.0 };
            let dy = ty - kid.y;
            if dy.abs() > 1.0 {
                kid.y += dy.signum() * 1.4;
                kid.anim += 0.25;
            }
        }
        if let Some(c) = self.car.as_mut() {
            c.x -= if self.pt < 40 { 0.6 } else { 2.4 };
            if self.pt == 50 || self.pt == 54 {
                self.sfx.push(Sfx::Honk);
            }
            if c.x < -60.0 {
                self.car = None;
                self.phase = Phase::Play;
                self.pt = 0;
                let kpk = &mut self.kids[kp];
                kpk.at = 0;
            }
        }
    }

    // ------------------------------------------------------------ the poodle

    fn update_dog(&mut self) {
        if !self.def().dog {
            return;
        }
        if self.dog.is_none() {
            let b = &self.ball;
            if self.play_t > self.next_dog && !b.held && !b.dog && b.z < 1.0 && !in_area(b.x, b.y) && b.x < FIELD_X1 {
                self.dog = Some(Dog { x: 262.0, y: b.y.clamp(4.0, 68.0), tx: b.x, ty: b.y, has: false, t: 0, leaving: false });
                self.next_dog = self.play_t + 60 * 50 + self.rng.int(60 * 40) as u32;
                self.banner("FIFI!", "THE POODLE WANTS TO PLAY", 90, 0xF8C8E0);
                self.sfx.push(Sfx::Bark);
            }
            return;
        }
        // the poodle wants any ball on the ground, even at somebody's feet
        let (bx, by, bloose) = (self.ball.x, self.ball.y, !self.ball.held && !self.ball.dog && self.ball.z < 3.0 && !in_area(self.ball.x, self.ball.y));
        let r1 = self.rng.range(70.0, 230.0);
        let r2 = self.rng.range(6.0, 66.0);
        let mut drop_at: Option<usize> = None;
        let mut gone = false;
        let mut bark = false;
        {
            let kids = &self.kids;
            let kp = self.keeper;
            let d = self.dog.as_mut().unwrap();
            d.t += 1;
            if d.leaving {
                d.tx = 290.0;
            } else if !d.has {
                if bloose {
                    d.tx = bx;
                    d.ty = by;
                } else {
                    d.leaving = true;
                }
                if (d.x - bx).abs() < 5.0 && (d.y - by).abs() < 4.0 && bloose {
                    d.has = true;
                    bark = true;
                    d.tx = r1;
                    d.ty = r2;
                }
                if d.t > 600 {
                    d.leaving = true;
                }
            } else {
                if (d.x - d.tx).abs() + (d.y - d.ty).abs() < 4.0 {
                    d.tx = r1;
                    d.ty = r2;
                }
                for (k, kid) in kids.iter().enumerate() {
                    if k != kp && (kid.x - d.x).abs() < 9.0 && (kid.y - d.y).abs() < 6.0 {
                        drop_at = Some(k);
                    }
                }
                if d.t > 420 {
                    drop_at = drop_at.or(Some(usize::MAX));
                }
            }
            let (dx, dy) = (d.tx - d.x, d.ty - d.y);
            let l = (dx * dx + dy * dy).sqrt().max(0.01);
            let sp: f32 = if d.has { 1.9 } else { 2.8 };
            d.x += dx / l * sp.min(l);
            d.y += dy / l * sp.min(l) * 0.8;
            if d.leaving && d.x > 280.0 {
                gone = true;
            }
        }
        if bark {
            if let Some(o) = self.ball.owner {
                self.kids[o].say("HE! FIFI!", 50);
            }
            self.ball.dog = true;
            self.ball.owner = None;
            self.ball.shot = None;
            self.ball.cross_to = None;
            self.ball.land = None;
            self.sfx.push(Sfx::Bark);
        }
        if let Some(k) = drop_at {
            let d = self.dog.as_mut().unwrap();
            d.has = false;
            d.leaving = true;
            self.ball.dog = false;
            self.ball.vx = 0.0;
            self.ball.vy = 0.0;
            self.ball.vz = 1.0;
            self.ball.air = false;
            self.ball.from = None;
            if k != usize::MAX {
                self.kids[k].say("GOED ZO FIFI!", 50);
            }
            self.sfx.push(Sfx::Bark);
        }
        if gone {
            self.dog = None;
        }
    }

    // ------------------------------------------------------------ pigeons

    fn scare_pigeons(&mut self, x: f32, y: f32, r: f32) {
        for p in self.pigeons.iter_mut() {
            if !p.fly && (p.x - x).abs() < r && (p.y - y).abs() < r * 0.6 {
                p.fly = true;
                p.vx = if p.x > x { 1.6 } else { -1.6 };
                p.vz = 1.4;
            }
        }
    }

    fn update_pigeons(&mut self) {
        let mut scare = Vec::new();
        for k in &self.kids {
            if k.vx.abs() + k.vy.abs() > 0.5 {
                scare.push((k.x, k.y));
            }
        }
        for (x, y) in scare {
            self.scare_pigeons(x, y, 12.0);
        }
        let t = self.t;
        for p in self.pigeons.iter_mut() {
            if p.fly {
                p.x += p.vx;
                p.z += p.vz;
                p.vz = (p.vz - 0.02).max(0.4);
                if p.z > 90.0 || p.x < -20.0 || p.x > 280.0 {
                    // land somewhere else later
                    p.fly = false;
                    p.z = 0.0;
                    p.x = 60.0 + ((t * 37 + p.peck * 13) % 180) as f32;
                    p.y = 4.0 + ((t * 53 + p.peck * 7) % 64) as f32;
                    p.peck = (p.peck + 17) % 60;
                }
            }
        }
    }

    // ------------------------------------------------------------ billenbrand

    fn start_brand(&mut self) {
        let Some(lo) = self.loser else {
            self.phase = Phase::Done;
            return;
        };
        let shooters: Vec<usize> = (0..self.kids.len()).filter(|&k| k != lo).collect();
        self.brand = Brand { shooters, ..Default::default() };
        self.phase = Phase::Brand;
        self.pt = 0;
        let kid = &mut self.kids[lo];
        kid.x = GOAL_X + 12.0;
        kid.y = goal_mid();
        kid.z = 0.0;
        kid.face = -1;
        kid.set(Act::Bent);
        kid.say("NIET TE HARD!", 90);
        self.banner("BILLENBRAND!", &format!("{} BENDS OVER", kid_name(self.kids[lo].id)), 120, 0xF86890);
        self.ball.held = false;
        self.ball.owner = None;
        self.ball.in_net = false;
        self.start_brand_turn();
    }

    fn start_brand_turn(&mut self) {
        let br = &mut self.brand;
        br.stage = 0;
        br.st = 0;
        br.hit = None;
        let k = br.shooters[br.turn];
        let kid = &mut self.kids[k];
        kid.x = SPOT_X;
        kid.y = goal_mid();
        kid.face = -1;
        kid.set(Act::Idle);
        let b = &mut self.ball;
        b.x = SPOT_X - 5.0;
        b.y = goal_mid();
        b.z = 0.0;
        b.vx = 0.0;
        b.vy = 0.0;
        b.vz = 0.0;
        b.in_net = false;
    }

    fn update_brand(&mut self, intents: &[Intent; 4]) {
        let lo = self.loser.unwrap_or(0);
        let shooter = self.brand.shooters[self.brand.turn];
        for k in 0..self.kids.len() {
            if k == shooter || k == lo {
                continue;
            }
            let slot = [(160.0, 8.0), (180.0, 6.0), (200.0, 9.0), (220.0, 7.0)][k];
            let kid = &mut self.kids[k];
            let (dx, dy) = (slot.0 - kid.x, slot.1 - kid.y);
            let d = (dx * dx + dy * dy).sqrt();
            if d > 1.5 {
                kid.x += dx / d * 1.6;
                kid.y += dy / d * 1.6;
                kid.anim += 0.25;
            }
            kid.face = -1;
            kid.at += 1;
            if matches!(kid.act, Act::Cheer) && kid.at > 60 {
                kid.set(Act::Idle);
            }
        }
        // the loser may wiggle
        {
            let it = intents[lo];
            let kid = &mut self.kids[lo];
            kid.at += 1;
            if kid.act == Act::Ouch && kid.at > 40 {
                kid.set(Act::Bent);
            }
            if kid.human().is_some() {
                kid.y = (kid.y + it.my * 0.9).clamp(goal_mid() - 10.0, goal_mid() + 10.0);
            }
        }
        let br = &mut self.brand;
        br.st += 1;
        match br.stage {
            0 => {
                if self.pt < 60 && br.turn == 0 {
                    return;
                }
                br.aim = (br.st as f32 * 0.06).sin();
                let go = intents[shooter].a || (self.kids[shooter].human().is_some() && br.st > 360);
                if go && br.st > 20 {
                    let ty = goal_mid() + br.aim * 14.0;
                    br.stage = 1;
                    br.st = 0;
                    let b = &mut self.ball;
                    let t = (b.x - GOAL_X - 12.0) / 3.6;
                    b.vx = (GOAL_X + 12.0 - b.x) / t;
                    b.vy = (ty - b.y) / t;
                    b.vz = (9.0 - b.z + 0.5 * G * t * t) / t;
                    self.kids[shooter].set(Act::Swing { kind: Kind::Volley, cross: false, hit: true });
                    self.sfx.push(Sfx::Kick);
                }
            }
            _ => {
                if br.st < 80 {
                    let px = self.ball.x;
                    let undecided = br.hit.is_none();
                    self.ball_motion();
                    let target = GOAL_X + 13.0;
                    if undecided && px > target && self.ball.x <= target {
                        let ly = self.kids[lo].y;
                        let hit = (self.ball.y - ly).abs() < 4.5;
                        self.brand.hit = Some(hit);
                        if hit {
                            self.brand.hits += 1;
                            self.kids[shooter].stats.brand_hits += 1;
                            self.kids[lo].set(Act::Ouch);
                            self.kids[lo].say("AUUU!", 50);
                            self.kids[lo].vz = 2.0;
                            self.kids[lo].z = 0.1;
                            self.kids[shooter].set(Act::Cheer);
                            self.sfx.push(Sfx::Ouch);
                            self.shake = 8;
                            self.ball.vx = self.ball.vx.abs() * 0.4;
                            self.ball.vz = 2.0;
                            self.popup(GOAL_X + 14.0, ly - 40.0, "RAAK!", 0xF86890);
                        } else {
                            self.popup(GOAL_X + 14.0, ly - 40.0, "MIS!", 0xF8F8F8);
                            self.sfx.push(Sfx::Whiff);
                        }
                    }
                    // the loser hops after a hit
                    let kid = &mut self.kids[lo];
                    if kid.z > 0.0 {
                        kid.z = (kid.z + kid.vz).max(0.0);
                        kid.vz -= KID_G;
                    }
                } else {
                    self.brand.turn += 1;
                    if self.brand.turn >= self.brand.shooters.len() {
                        self.phase = Phase::Done;
                        self.pt = 0;
                    } else {
                        self.start_brand_turn();
                    }
                }
            }
        }
    }
}

pub fn kid_name(id: usize) -> &'static str {
    kid(id).name
}

fn kid_def_acrobat(id: usize) -> bool {
    kid(id).acrobat
}

/// What a touch becomes, from the facing (-1 = to the goal), being in the
/// air, and the ball's height over the kid's feet.
pub fn shot_kind(face: i32, air: bool, rel: f32, acrobat: bool) -> Option<Kind> {
    let (lo, hi) = if acrobat { (5.0, 38.0) } else { (9.0, 33.0) };
    if air && face == 1 && rel >= lo && rel <= hi {
        return Some(Kind::Bicycle);
    }
    if rel < 12.0 {
        Some(if face == 1 { Kind::Heel } else { Kind::Volley })
    } else if rel < 20.0 {
        Some(Kind::Chest)
    } else if rel <= 34.0 {
        Some(Kind::Header)
    } else {
        None
    }
}
