//! One DISC DOGS! match: two owner+dog pairs take turns, 5 tosses each.
//! Pure simulation (no drawing, no I/O), stepped at 60 Hz.
//!
//! The world is seen from the side: x runs from the throw line (0) to the
//! end of the field (`FIELD_END`, 100 yards, 4 px per yard), z is height
//! above the ground. Everything else (the squirrel crossing the field) has
//! a small depth `d` that is only used for drawing and for "on the line".

use super::ai::Brain;
use super::data::{Breed, Stage, PAIRS};
use crate::engine::gfx::rgb;
pub use crate::engine::input::{edge, Pad};
pub use crate::engine::rng::Rng;

pub const YD: f32 = 4.0;
pub const FIELD_END: f32 = 400.0;
pub const TOSSES: usize = 5;
pub const OWNER_X: f32 = -10.0;
pub const DOG_START: f32 = 14.0;
pub const HAND_Z: f32 = 20.0;
/// frames of a full flip
pub const FLIP_LEN: i32 = 44;
/// frames for the power meter to go from empty to full
pub const METER_HALF: i32 = 32;
pub const PERFECT_FROM: f32 = 0.93;
pub const ANGLE_MIN: f32 = 0.0;
pub const ANGLE_MAX: f32 = 45.0;

const DISC_G: f32 = 0.036;
const LIFT: f32 = 0.0021;
const DRAG: f32 = 0.0036;
const WIND_AIR: f32 = 0.5;
pub const DOG_G: f32 = 0.115;
const RESULT_FRAMES: i32 = 130;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Sfx {
    Throw,
    Bark,
    Chomp,
    Flip,
    Oof,
    Splash,
    Squawk,
    Chitter,
    Gust,
    Tick,
    Ding,
    Whistle,
    Cheer,
    Blip,
    Confirm,
    WahWah,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ctrl {
    Human(usize),
    Cpu,
}

// ------------------------------------------------------------------ disc

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DiscState {
    Held,
    Flying,
    /// sliding along the ground after landing
    Skid,
    /// lying still: too late
    Stopped,
    /// bobbing in the pond
    Floating,
    /// in a dog's mouth
    Carried,
    /// in a seagull's beak
    Gulled,
    /// gone (sea, hedge, crowd)
    Gone,
}

#[derive(Clone, Copy, Debug)]
pub struct Disc {
    pub x: f32,
    pub z: f32,
    pub vx: f32,
    pub vz: f32,
    pub floater: bool,
    pub state: DiscState,
    pub spin: f32,
    pub t: i32,
}

impl Disc {
    pub fn new() -> Self {
        Disc { x: OWNER_X + 6.0, z: HAND_Z, vx: 0.0, vz: 0.0, floater: false, state: DiscState::Held, spin: 0.0, t: 0 }
    }

    /// Launch speed for a meter value (0..1).
    pub fn launch_speed(power: f32, perfect: bool, floater: bool) -> f32 {
        let v = 1.8 + 2.1 * power.clamp(0.0, 1.0);
        let v = if perfect { v * 1.05 } else { v };
        if floater {
            v * 0.8
        } else {
            v
        }
    }

    pub fn launch(&mut self, angle_deg: f32, power: f32, perfect: bool, floater: bool) {
        let a = (angle_deg + if floater { 9.0 } else { 0.0 }).to_radians();
        let v = Disc::launch_speed(power, perfect, floater);
        self.vx = v * a.cos();
        self.vz = v * a.sin();
        self.floater = floater;
        self.state = DiscState::Flying;
        self.t = 0;
    }

    /// Airspeed below which the disc stalls and wobbles.
    pub fn wobble(&self) -> f32 {
        let s = (self.vx * self.vx + self.vz * self.vz).sqrt();
        (1.3 - s).clamp(0.0, 1.0)
    }

    /// One frame of flight: gravity, lift that grows with speed, drag and
    /// the wind. Used by the match and by the CPU to predict the flight.
    pub fn step_air(&mut self, wind: f32) {
        let wk = if self.floater { 2.6 } else { 1.0 };
        let ux = self.vx - wind * WIND_AIR * wk;
        let u = (ux * ux + self.vz * self.vz).sqrt();
        let lift_k = if self.floater { LIFT * 1.6 } else { LIFT };
        // the disc stalls when it gets slow: lift fades out
        let stall = ((u - 0.5) / 0.75).clamp(0.0, 1.0);
        let lift = (lift_k * ux * ux).min(DISC_G * 1.1) * stall;
        let drag = if self.floater { DRAG * 1.3 } else { DRAG };
        self.vx -= drag * u * ux;
        self.vz -= drag * u * self.vz;
        self.vz += lift - DISC_G;
        // a floater sinks gently: its lift never quite lets go
        if self.floater && self.vz < -0.7 {
            self.vz = -0.7;
        }
        self.x += self.vx;
        self.z += self.vz;
        self.spin += 0.3 + self.vx.abs() * 0.15;
        self.t += 1;
    }
}

impl Default for Disc {
    fn default() -> Self {
        Self::new()
    }
}

/// A predicted flight: the disc position every frame until it lands
/// (or 600 frames).
pub fn predict(d: &Disc, wind: f32, out: &mut Vec<(f32, f32)>) {
    out.clear();
    let mut d = *d;
    for _ in 0..600 {
        d.step_air(wind);
        out.push((d.x, d.z));
        if d.z <= 0.0 {
            break;
        }
    }
}

// ------------------------------------------------------------------ dog

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DogState {
    Sit,
    Stand,
    Run,
    Air,
    Tumble,
    Stare,
    Swim,
    /// result: hopping with the disc
    Happy,
    /// result: ears down
    Guilty,
    /// result or sulking: lying flat
    Lie,
    /// result: shaking off pond water
    Shake,
    /// result: barking at the seagull
    Bark,
}

#[derive(Clone, Copy, Debug)]
pub struct DogBody {
    /// mouth offset from the body origin when standing (x forward, z up)
    pub mouth: (f32, f32),
    pub top: f32,
    pub jump_v: f32,
    pub zoom_cap: f32,
    /// frames of staring at a squirrel
    pub stare: i32,
}

pub fn dog_body(pair: usize) -> DogBody {
    let d = &PAIRS[pair].dog;
    let mouth = match d.breed {
        Breed::Corgi => (11.0, 9.0),
        Breed::Collie => (11.0, 13.0),
        Breed::Afghan => (12.0, 17.0),
        Breed::Keeshond => (10.0, 12.0),
    };
    DogBody {
        mouth,
        top: 1.75 + 0.2 * d.speed as f32,
        jump_v: 2.15 + 0.3 * d.jump as f32,
        zoom_cap: 40.0 + 18.0 * d.zoomies as f32,
        stare: 18 + (5 - d.focus) * 16,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Dog {
    pub pair: usize,
    pub body: DogBody,
    pub x: f32,
    pub z: f32,
    pub vx: f32,
    pub vz: f32,
    pub face: f32,
    pub state: DogState,
    pub anim: f32,
    /// frames into a flip, or -1
    pub flip_t: i32,
    pub has_disc: bool,
    /// caught in the air: the kind of catch is decided on landing
    pub caught_air: bool,
    pub flip_with_disc: bool,
    pub zoom: f32,
    pub zooming: bool,
    pub stare_t: i32,
    pub tumble_t: i32,
    pub air_t: i32,
    /// blanket index the dog is on (sausage grabbed once per visit)
    pub on_blanket: Option<usize>,
    /// frames of free zoomies after being called back from a squirrel
    pub boost: i32,
    /// the owner whistled during this stare
    pub called: bool,
}

impl Dog {
    pub fn new(pair: usize) -> Self {
        let body = dog_body(pair);
        Dog {
            pair,
            body,
            x: DOG_START,
            z: 0.0,
            vx: 0.0,
            vz: 0.0,
            face: 1.0,
            state: DogState::Sit,
            anim: 0.0,
            flip_t: -1,
            has_disc: false,
            caught_air: false,
            flip_with_disc: false,
            zoom: body.zoom_cap,
            zooming: false,
            stare_t: 0,
            tumble_t: 0,
            air_t: 0,
            on_blanket: None,
            boost: 0,
            called: false,
        }
    }

    /// Where the mouth is, in world coordinates.
    pub fn mouth(&self) -> (f32, f32) {
        if self.flip_t >= 0 {
            // the mouth goes round with the flip (a back flip)
            let a = self.flip_t as f32 / FLIP_LEN as f32 * std::f32::consts::TAU;
            let (c, s) = (a.cos(), a.sin());
            let r = 8.0;
            return (self.x + self.face * r * c, self.z + 9.0 + r * s);
        }
        let lift = if self.state == DogState::Air { 2.0 } else { 0.0 };
        (self.x + self.face * self.body.mouth.0, self.z + self.body.mouth.1 + lift)
    }

    pub fn grounded(&self) -> bool {
        self.z <= 0.0 && self.state != DogState::Air
    }

    /// Frames until the dog lands, from its height and vertical speed.
    pub fn time_to_land(&self) -> f32 {
        if self.z <= 0.0 && self.vz <= 0.0 {
            return 0.0;
        }
        let v = self.vz;
        (v + (v * v + 2.0 * DOG_G * self.z.max(0.0)).sqrt()) / DOG_G
    }
}

/// Height a jumping dog reaches after `t` frames, and how long the jump lasts.
pub fn jump_height(v0: f32, t: f32) -> f32 {
    v0 * t - 0.5 * DOG_G * t * t
}

pub fn jump_airtime(v0: f32) -> f32 {
    2.0 * v0 / DOG_G
}

// ------------------------------------------------------------------ critters and props

#[derive(Clone, Copy, Debug)]
pub struct Squirrel {
    pub x: f32,
    /// depth: +28 at the front edge, -30 at the back; 0 is the dog's line
    pub d: f32,
    pub t: i32,
    pub wait: i32,
    pub taunt: i32,
    pub gone: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Gull {
    pub x: f32,
    pub z: f32,
    pub vx: f32,
    pub vz: f32,
    pub has_disc: bool,
    pub t: i32,
    pub leaving: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Blanket {
    pub x: f32,
    pub w: f32,
    pub sausages: i32,
    pub color: u32,
    pub sleeper: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Kite {
    pub kid_x: f32,
    pub here: bool,
}

impl Kite {
    /// The string: from the kid's hand up to the kite.
    pub fn string(&self, frame: u64) -> ((f32, f32), (f32, f32)) {
        let bob = ((frame as f32) * 0.03).sin() * 4.0;
        ((self.kid_x + 4.0, 15.0), (self.kid_x + 44.0 + bob, 118.0 + bob * 0.5))
    }
}

// ------------------------------------------------------------------ results

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CatchKind {
    Ground,
    Rolling,
    Soggy,
    Jump,
    Flip,
}

impl CatchKind {
    pub fn mult(self) -> i32 {
        match self {
            CatchKind::Jump => 2,
            CatchKind::Flip => 3,
            _ => 1,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            CatchKind::Ground => "CATCH!",
            CatchKind::Rolling => "ROLLER!",
            CatchKind::Soggy => "SOGGY CATCH!",
            CatchKind::Jump => "JUMP CATCH!",
            CatchKind::Flip => "FLIP CATCH!",
        }
    }
    pub fn short(self) -> &'static str {
        match self {
            CatchKind::Ground => "CATCH",
            CatchKind::Rolling => "ROLLER",
            CatchKind::Soggy => "SOGGY",
            CatchKind::Jump => "JUMP",
            CatchKind::Flip => "FLIP",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MissKind {
    Dropped,
    Oof,
    Seagull,
    Sea,
    Hedge,
    Crowd,
    Sunk,
}

impl MissKind {
    pub fn label(self) -> &'static str {
        match self {
            MissKind::Dropped => "MISSED!",
            MissKind::Oof => "OOF!",
            MissKind::Seagull => "SEAGULL'D!",
            MissKind::Sea => "LOST AT SEA!",
            MissKind::Hedge => "IN THE HEDGE!",
            MissKind::Crowd => "INTO THE CROWD!",
            MissKind::Sunk => "GLUB GLUB!",
        }
    }
    pub fn short(self) -> &'static str {
        match self {
            MissKind::Dropped => "MISS",
            MissKind::Oof => "OOF",
            MissKind::Seagull => "GULL",
            MissKind::Sea => "SEA",
            MissKind::Hedge => "HEDGE",
            MissKind::Crowd => "CROWD",
            MissKind::Sunk => "SUNK",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Catch(CatchKind),
    Miss(MissKind),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TossResult {
    pub outcome: Outcome,
    pub yards: i32,
    pub perfect: bool,
    pub shoulder: bool,
    pub sausages: i32,
}

pub const PERFECT_BONUS: i32 = 5;
pub const SHOULDER_BONUS: i32 = 10;
pub const SAUSAGE_BONUS: i32 = 5;

impl TossResult {
    pub fn points(&self) -> i32 {
        match self.outcome {
            Outcome::Catch(k) => {
                self.yards * k.mult()
                    + if self.perfect { PERFECT_BONUS } else { 0 }
                    + if self.shoulder { SHOULDER_BONUS } else { 0 }
                    + self.sausages * SAUSAGE_BONUS
            }
            Outcome::Miss(_) => 0,
        }
    }
    pub fn caught(&self) -> bool {
        matches!(self.outcome, Outcome::Catch(_))
    }
}

// ------------------------------------------------------------------ effects

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FxKind {
    Dust,
    Sparkle,
    Drop,
    Star,
    Feather,
    Crumb,
    Leaf,
}

#[derive(Clone, Copy, Debug)]
pub struct Fx {
    pub x: f32,
    pub z: f32,
    pub vx: f32,
    pub vz: f32,
    pub life: i32,
    pub max: i32,
    pub kind: FxKind,
}

#[derive(Clone, Debug)]
pub struct Popup {
    pub x: f32,
    pub z: f32,
    pub text: String,
    pub color: u32,
    pub life: i32,
}

#[derive(Clone, Debug)]
pub struct Banner {
    pub text: String,
    pub t: i32,
    pub dur: i32,
    pub color: u32,
    /// pop in place instead of sliding in
    pub pop: bool,
    pub scale: i32,
}

/// A line in the comment box at the bottom (who says it, what).
#[derive(Clone, Debug)]
pub struct Quip {
    /// pair index of the speaker, or None for the announcer
    pub who: Option<usize>,
    pub text: String,
    pub t: i32,
}

// ------------------------------------------------------------------ match

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Ready,
    Aim,
    Power,
    Release,
    Flight,
    Result,
    Over,
}

#[derive(Clone, Debug)]
pub struct Side {
    pub pair: usize,
    pub ctrl: Ctrl,
    pub results: Vec<TossResult>,
    pub misses_in_row: u32,
}

impl Side {
    pub fn total(&self) -> i32 {
        self.results.iter().map(|r| r.points()).sum()
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Stats {
    pub tosses: u32,
    pub catches: u32,
    pub ground: u32,
    pub rolling: u32,
    pub soggy: u32,
    pub jumps: u32,
    pub flips: u32,
    pub oofs: u32,
    pub gulls: u32,
    pub out: u32,
    pub dropped: u32,
    pub sunk: u32,
    pub flips_tried: u32,
    pub yards_caught: u32,
    pub throw_yards: u32,
    pub points: u32,
    pub perfects: u32,
    pub floaters: u32,
    pub shoulders: u32,
    pub sausages: u32,
    pub stares: u32,
    pub snags: u32,
    pub sudden: u32,
    pub frames: u64,
}

pub struct Match {
    pub stage: Stage,
    pub sides: [Side; 2],
    pub brains: [Option<Brain>; 2],
    pub turn: usize,
    /// number of turns in regular play (2 x TOSSES), grows in sudden dog
    pub turns: usize,
    pub phase: Phase,
    pub phase_t: i32,
    pub disc: Disc,
    pub dog: Dog,
    pub angle: f32,
    pub meter_t: i32,
    pub power: f32,
    pub perfect: bool,
    pub floater: bool,
    pub wind: f32,
    gust_in: i32,
    pub squirrel: Option<Squirrel>,
    squirrel_due: Option<i32>,
    pub stared: bool,
    pub gull: Option<Gull>,
    gull_due: bool,
    pub blankets: Vec<Blanket>,
    pub pond: Option<(f32, f32)>,
    pub kite: Kite,
    pub snagged: bool,
    pub sleeper_awake: i32,
    pub sausages_now: i32,
    shoulder_now: bool,
    pub cam: f32,
    pub frame: u64,
    pub rng: Rng,
    pub sfx: Vec<Sfx>,
    pub fx: Vec<Fx>,
    pub popups: Vec<Popup>,
    pub banner: Option<Banner>,
    pub quip: Option<Quip>,
    pub shake: i32,
    pub excite: f32,
    pub result: Option<TossResult>,
    pub stats: Stats,
    pub winner: Option<usize>,
    pub coin_toss: bool,
    /// the throw line: frames since the last throw (owner animation)
    pub throw_anim: i32,
    /// the owner reacts after a toss: >0 cheer, <0 face-palm
    pub owner_mood: i32,
    pub hint: bool,
    pub idle_t: i32,
    /// C held during the power phase (shows FLOATER)
    pub c_held: bool,
    prev: [Pad; 2],
    last_tap: [i64; 2],
    prev_disc: (f32, f32),
    prev_mouth: (f32, f32),
}

impl Match {
    pub fn new(pairs: [usize; 2], ctrls: [Ctrl; 2], skills: [f32; 2], stage: Stage, seed: u64) -> Self {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let brains = [0, 1].map(|s| match ctrls[s] {
            Ctrl::Cpu => Some(Brain::cpu(skills[s], seed.wrapping_add(s as u64 * 977 + 1))),
            Ctrl::Human(_) => None,
        });
        let mut blankets = Vec::new();
        let mut pond = None;
        let mut kite = Kite { kid_x: 0.0, here: false };
        if stage == Stage::Park {
            let px = 150.0 + rng.range(0.0, 70.0);
            pond = Some((px, px + 30.0));
            // two picnic blankets, not in the pond
            let cols = [rgb(232, 56, 64), rgb(64, 120, 232), rgb(248, 200, 48)];
            let mut tries = 0;
            while blankets.len() < 2 && tries < 50 {
                tries += 1;
                let x = 70.0 + rng.range(0.0, 260.0);
                let w = 30.0;
                let clear_pond = x + w < px - 6.0 || x > px + 36.0;
                let clear = blankets.iter().all(|b: &Blanket| (b.x - x).abs() > 60.0);
                if clear_pond && clear {
                    let n = blankets.len();
                    blankets.push(Blanket { x, w, sausages: 3, color: cols[(n + rng.int(2) as usize) % 3], sleeper: n == 0 });
                }
            }
            kite.kid_x = 190.0 + rng.range(0.0, 110.0);
        }
        let dog = Dog::new(pairs[0]);
        let mut m = Match {
            stage,
            sides: [0, 1].map(|s| Side { pair: pairs[s], ctrl: ctrls[s], results: Vec::new(), misses_in_row: 0 }),
            brains,
            turn: 0,
            turns: TOSSES * 2,
            phase: Phase::Ready,
            phase_t: 0,
            disc: Disc::new(),
            dog,
            angle: 20.0,
            meter_t: 0,
            power: 0.0,
            perfect: false,
            floater: false,
            wind: 0.0,
            gust_in: 0,
            squirrel: None,
            squirrel_due: None,
            stared: false,
            gull: None,
            gull_due: false,
            blankets,
            pond,
            kite,
            snagged: false,
            sleeper_awake: 0,
            sausages_now: 0,
            shoulder_now: false,
            cam: AIM_CAM,
            frame: 0,
            rng,
            sfx: Vec::new(),
            fx: Vec::new(),
            popups: Vec::new(),
            banner: None,
            quip: None,
            shake: 0,
            excite: 0.0,
            result: None,
            stats: Stats::default(),
            winner: None,
            coin_toss: false,
            throw_anim: -1,
            owner_mood: 0,
            hint: true,
            idle_t: 0,
            c_held: false,
            prev: [Pad::default(); 2],
            last_tap: [-100; 2],
            prev_disc: (0.0, 0.0),
            prev_mouth: (0.0, 0.0),
        };
        m.setup_turn();
        m
    }

    /// The side throwing now.
    pub fn thrower(&self) -> usize {
        self.turn % 2
    }

    /// Toss number of the current turn (1-based).
    pub fn toss_no(&self) -> usize {
        self.turn / 2 + 1
    }

    pub fn sudden(&self) -> bool {
        self.turn >= TOSSES * 2
    }

    pub fn finished(&self) -> bool {
        self.phase == Phase::Over
    }

    /// The meter value (0..1) for the current power phase.
    pub fn meter(&self) -> f32 {
        let k = self.meter_t % (METER_HALF * 2);
        let v = if k <= METER_HALF { k } else { METER_HALF * 2 - k };
        v as f32 / METER_HALF as f32
    }

    pub fn human(&self, side: usize) -> Option<usize> {
        match self.sides[side].ctrl {
            Ctrl::Human(p) => Some(p),
            Ctrl::Cpu => None,
        }
    }

    fn setup_turn(&mut self) {
        let side = self.thrower();
        let pair = self.sides[side].pair;
        self.dog = Dog::new(pair);
        if self.sides[side].misses_in_row >= 3 {
            self.dog.state = DogState::Lie;
        }
        self.disc = Disc::new();
        self.angle = 20.0;
        self.meter_t = 0;
        self.power = 0.0;
        self.perfect = false;
        self.floater = false;
        let wm = self.stage.wind_max();
        self.wind = self.rng.range(-wm, wm);
        self.gust_in = 60 + self.rng.int(80);
        self.squirrel = None;
        self.squirrel_due = None;
        self.stared = false;
        self.gull = None;
        let p_squirrel = match self.stage {
            Stage::Park => 0.35,
            Stage::Bowl => 0.1,
            Stage::Pier => 0.0,
        };
        if self.rng.f() < p_squirrel {
            self.squirrel_due = Some(14 + self.rng.int(50));
        }
        self.gull_due = self.stage == Stage::Pier && self.rng.f() < 0.4;
        self.kite.here = self.stage == Stage::Park && self.rng.f() < 0.35;
        self.snagged = false;
        self.sleeper_awake = 0;
        self.sausages_now = 0;
        self.shoulder_now = false;
        self.result = None;
        self.throw_anim = -1;
        self.owner_mood = 0;
        self.cam = AIM_CAM;
        self.popups.clear();
        self.fx.clear();
        self.quip = None;
        self.idle_t = 0;
        self.c_held = false;
        self.phase = Phase::Ready;
        self.phase_t = 0;
        let who = PAIRS[pair].dog.name;
        let txt = if self.sudden() {
            "SUDDEN DOG!".to_string()
        } else if self.toss_no() == TOSSES && side == 1 {
            "FINAL TOSS!".to_string()
        } else {
            format!("TOSS {}", self.toss_no())
        };
        self.banner = Some(Banner { text: txt, t: 0, dur: 70, color: rgb(248, 248, 248), pop: false, scale: 0 });
        if self.dog.state == DogState::Lie {
            self.quip = Some(Quip { who: Some(pair), text: format!("{} IS SULKING. TRY A NICE THROW.", who), t: 0 });
        }
        for b in self.brains.iter_mut().flatten() {
            b.new_turn();
        }
    }

    fn banner(&mut self, text: &str, color: u32, pop: bool) {
        self.banner = Some(Banner { text: text.to_string(), t: 0, dur: 80, color, pop, scale: 0 });
    }

    fn popup(&mut self, x: f32, z: f32, text: String, color: u32) {
        self.popups.push(Popup { x, z, text, color, life: 60 });
    }

    fn fx(&mut self, x: f32, z: f32, vx: f32, vz: f32, life: i32, kind: FxKind) {
        if self.fx.len() < 160 {
            self.fx.push(Fx { x, z, vx, vz, life, max: life, kind });
        }
    }

    fn burst(&mut self, x: f32, z: f32, n: usize, kind: FxKind, spread: f32) {
        for _ in 0..n {
            let (vx, vz) = (self.rng.range(-spread, spread), self.rng.range(0.3, spread * 1.6));
            let life = 20 + self.rng.int(20);
            self.fx(x, z, vx, vz, life, kind);
        }
    }

    // ------------------------------------------------------------------ update

    pub fn update(&mut self, pads: &[Pad; 2]) {
        self.frame += 1;
        self.stats.frames += 1;
        self.phase_t += 1;
        let side = self.thrower();
        // whoever plays this side: a human pad or the brain
        let pad = match self.sides[side].ctrl {
            Ctrl::Human(p) => pads[p],
            Ctrl::Cpu => {
                let mut b = self.brains[side].take().expect("cpu side has a brain");
                let p = b.pad(self, side);
                self.brains[side] = Some(b);
                p
            }
        };
        let e = edge(pad, self.prev[side]);
        self.prev = [Pad::default(); 2];
        self.prev[side] = pad;

        match self.phase {
            Phase::Ready => {
                if self.phase_t > 60 {
                    self.phase = Phase::Aim;
                    self.phase_t = 0;
                    if self.dog.state != DogState::Lie {
                        self.dog.state = DogState::Sit;
                    }
                }
            }
            Phase::Aim => self.update_aim(pad, e),
            Phase::Power => self.update_power(pad),
            Phase::Release => {
                self.throw_anim += 1;
                if self.throw_anim == 4 {
                    self.disc.launch(self.angle, self.power, self.perfect, self.floater);
                    self.disc.x = OWNER_X + 8.0;
                    self.disc.z = HAND_Z;
                    self.sfx.push(Sfx::Throw);
                    self.stats.tosses += 1;
                    if self.floater {
                        self.stats.floaters += 1;
                    }
                    if self.perfect {
                        self.stats.perfects += 1;
                        self.sfx.push(Sfx::Ding);
                        self.popup(OWNER_X + 8.0, HAND_Z + 14.0, "PERFECT!".into(), rgb(248, 232, 64));
                        let (x, z) = (self.disc.x, self.disc.z);
                        self.burst(x, z, 10, FxKind::Sparkle, 1.2);
                    }
                    if self.floater {
                        self.popup(OWNER_X + 8.0, HAND_Z + 24.0, "FLOATER".into(), rgb(160, 224, 248));
                    }
                    self.phase = Phase::Flight;
                    self.phase_t = 0;
                    self.dog.state = DogState::Stand;
                    self.sfx.push(Sfx::Bark);
                    self.prev_disc = (self.disc.x, self.disc.z);
                    self.prev_mouth = self.dog.mouth();
                }
            }
            Phase::Flight => {
                self.throw_anim += 1;
                self.update_flight(pad, e);
            }
            Phase::Result => {
                self.throw_anim += 1;
                self.update_result();
            }
            Phase::Over => {}
        }
        self.update_fx();
        self.update_camera();
    }

    fn update_aim(&mut self, pad: Pad, e: Pad) {
        if pad.up && self.phase_t % 2 == 0 {
            self.angle = (self.angle + 1.0).min(ANGLE_MAX);
        }
        if pad.down && self.phase_t % 2 == 0 {
            self.angle = (self.angle - 1.0).max(ANGLE_MIN);
        }
        if e.up || e.down {
            self.sfx.push(Sfx::Tick);
        }
        if self.dog.state == DogState::Lie && self.phase_t > 90 {
            self.dog.state = DogState::Sit;
        }
        let auto = self.phase_t > 600;
        if e.a || auto {
            self.phase = Phase::Power;
            self.phase_t = 0;
            self.meter_t = 0;
            if auto {
                let who = PAIRS[self.sides[self.thrower()].pair].owner.name;
                self.quip = Some(Quip { who: None, text: format!("{} FELL ASLEEP. AUTO THROW!", who), t: 0 });
            }
        }
    }

    fn update_power(&mut self, pad: Pad) {
        self.meter_t += 1;
        self.c_held = pad.c;
        if self.meter_t % 8 == 0 {
            self.sfx.push(Sfx::Tick);
        }
        let auto = self.phase_t > 300;
        if !pad.a || auto {
            self.power = self.meter();
            if auto {
                self.power = 0.3;
            }
            self.perfect = self.power >= PERFECT_FROM;
            self.floater = pad.c;
            self.phase = Phase::Release;
            self.phase_t = 0;
            self.throw_anim = 0;
            self.hint = false;
        }
    }

    fn update_flight(&mut self, pad: Pad, e: Pad) {
        let side = self.thrower();
        // ---- wind gusts
        if self.stage != Stage::Park {
            self.gust_in -= 1;
            if self.gust_in <= 0 {
                let big = self.stage == Stage::Bowl;
                self.gust_in = if big { 70 + self.rng.int(70) } else { 150 + self.rng.int(100) };
                let push = self.rng.range(-1.0, 1.0) * if big { 0.6 } else { 0.35 };
                self.wind = (self.wind + push).clamp(-1.2, 1.2);
                if big && self.disc.state == DiscState::Flying {
                    self.sfx.push(Sfx::Gust);
                    let (x, z) = (self.disc.x, self.disc.z + 12.0);
                    let arrow = if self.wind > 0.0 { "GUST! >>" } else { "<< GUST!" };
                    self.popup(x, z, arrow.into(), rgb(200, 232, 248));
                }
            }
        }

        // ---- disc
        self.prev_disc = (self.disc.x, self.disc.z);
        self.update_disc();

        // ---- dog
        self.prev_mouth = self.dog.mouth();
        self.update_dog(pad, e, side);

        // ---- critters
        self.update_squirrel(e);
        self.update_gull();

        // ---- catch
        if !self.dog.has_disc && self.result.is_none() {
            self.try_catch();
        }
        if self.dog.has_disc {
            let (mx, mz) = self.dog.mouth();
            self.disc.x = mx;
            self.disc.z = mz - 1.0;
            self.disc.state = DiscState::Carried;
            if self.dog.grounded() && self.dog.caught_air {
                // landed with it
                self.dog.caught_air = false;
                let kind = if self.dog.flip_with_disc { CatchKind::Flip } else { CatchKind::Jump };
                self.finish_catch(kind);
            }
        }
        if self.result.is_none() && self.phase_t > 60 * 15 {
            self.finish_miss(MissKind::Dropped);
        }
        if let Some(r) = self.result {
            let _ = r;
            self.phase = Phase::Result;
            self.phase_t = 0;
        }
    }

    fn update_disc(&mut self) {
        let wind = self.wind;
        match self.disc.state {
            DiscState::Flying => {
                self.disc.step_air(wind);
                // the kite string
                if self.kite.here && !self.snagged && self.disc.z > 30.0 {
                    let (a, b) = self.kite.string(self.frame);
                    let (p, q) = (self.prev_disc, (self.disc.x, self.disc.z));
                    if segments_cross(a, b, p, q) {
                        self.snagged = true;
                        self.stats.snags += 1;
                        self.disc.vx *= 0.2;
                        self.disc.vz = self.disc.vz.min(0.0) - 0.4;
                        self.sfx.push(Sfx::Tick);
                        let (x, z) = (self.disc.x, self.disc.z + 10.0);
                        self.popup(x, z, "SNAGGED!".into(), rgb(248, 136, 200));
                        self.burst(x, z - 10.0, 6, FxKind::Leaf, 0.8);
                    }
                }
                if self.disc.x >= FIELD_END && self.stage != Stage::Pier && self.disc.z < 60.0 {
                    let k = if self.stage == Stage::Park { MissKind::Hedge } else { MissKind::Crowd };
                    self.disc.state = DiscState::Gone;
                    if k == MissKind::Hedge {
                        self.burst(FIELD_END, self.disc.z.max(6.0), 10, FxKind::Leaf, 1.0);
                    } else {
                        self.sfx.push(Sfx::Cheer);
                    }
                    self.finish_miss(k);
                    return;
                }
                if self.disc.z <= 0.0 {
                    self.disc.z = 0.0;
                    let x = self.disc.x;
                    if x >= FIELD_END {
                        self.disc.state = DiscState::Gone;
                        self.sfx.push(Sfx::Splash);
                        self.burst(x, 0.0, 12, FxKind::Drop, 1.2);
                        self.finish_miss(MissKind::Sea);
                    } else if self.in_pond(x) {
                        self.disc.state = DiscState::Floating;
                        self.disc.vx = 0.0;
                        self.disc.t = 0;
                        self.sfx.push(Sfx::Splash);
                        self.burst(x, 0.0, 10, FxKind::Drop, 1.0);
                    } else {
                        self.disc.state = DiscState::Skid;
                        self.disc.vx *= 0.4;
                        self.disc.vz = 0.0;
                        self.disc.t = 0;
                        self.fx(x, 0.0, -0.3, 0.4, 14, FxKind::Dust);
                        self.fx(x, 0.0, 0.3, 0.5, 14, FxKind::Dust);
                    }
                }
            }
            DiscState::Skid => {
                self.disc.x += self.disc.vx;
                self.disc.vx *= 0.86;
                self.disc.spin += self.disc.vx;
                self.disc.t += 1;
                if self.in_pond(self.disc.x) {
                    self.disc.state = DiscState::Floating;
                    self.disc.t = 0;
                    self.sfx.push(Sfx::Splash);
                } else if self.disc.vx.abs() < 0.12 || self.disc.t > 24 {
                    self.disc.state = DiscState::Stopped;
                    // landing on the sleeper's blanket wakes him up
                    if self.blankets.iter().any(|b| b.sleeper && self.disc.x >= b.x && self.disc.x <= b.x + b.w) {
                        self.sleeper_awake = 120;
                        self.popup(self.disc.x, 26.0, "HEY!".into(), rgb(248, 248, 248));
                    }
                    self.finish_miss(MissKind::Dropped);
                }
            }
            DiscState::Floating => {
                self.disc.t += 1;
                self.disc.x += self.wind * 0.08;
                if let Some((a, b)) = self.pond {
                    self.disc.x = self.disc.x.clamp(a + 4.0, b - 4.0);
                }
                if self.disc.t > 360 {
                    self.disc.state = DiscState::Gone;
                    self.finish_miss(MissKind::Sunk);
                }
            }
            _ => {}
        }
    }

    pub fn in_pond(&self, x: f32) -> bool {
        self.pond.map(|(a, b)| x > a && x < b).unwrap_or(false)
    }

    fn update_dog(&mut self, pad: Pad, e: Pad, side: usize) {
        let d = &mut self.dog;
        if d.state == DogState::Tumble {
            d.tumble_t -= 1;
            d.vx *= 0.9;
            d.x += d.vx;
            if d.tumble_t <= 0 {
                d.state = DogState::Stand;
            }
            return;
        }
        if d.state == DogState::Stare {
            d.vx = 0.0;
            if e.c {
                d.stare_t -= 50;
                d.called = true;
                self.sfx.push(Sfx::Whistle);
            }
            d.stare_t -= 1;
            if d.stare_t <= 0 {
                d.state = DogState::Stand;
                let pair = self.sides[side].pair;
                let (x, z) = (d.x, d.z + 30.0);
                if d.called {
                    // called back: a burst of zoomies to make up for it
                    d.boost = 30;
                    let good = PAIRS[pair].dog.good;
                    self.popup(x, z, good.to_string(), rgb(248, 232, 64));
                } else {
                    self.popup(x, z, "...FINE.".to_string(), rgb(200, 200, 216));
                }
                self.sfx.push(Sfx::Bark);
            }
            return;
        }
        // double tap for zoomies
        if e.right || e.left {
            let k = if e.right { 0 } else { 1 };
            if self.frame as i64 - self.last_tap[k] < 16 && d.zoom > 4.0 {
                d.zooming = true;
                let (x, z) = (d.x, d.z + 28.0);
                self.popup(x, z, "ZOOMIES!".into(), rgb(248, 152, 32));
                self.sfx.push(Sfx::Bark);
            }
            self.last_tap[k] = self.frame as i64;
        }
        let d = &mut self.dog;
        let dir = (pad.right as i32 - pad.left as i32) as f32;
        if dir == 0.0 || d.zoom <= 0.0 {
            d.zooming = false;
        }
        let swimming = d.z <= 0.0 && self.pond.map(|(a, b)| d.x > a + 2.0 && d.x < b - 2.0).unwrap_or(false);
        let blanket = if d.z <= 0.0 { self.blankets.iter().position(|b| d.x >= b.x && d.x <= b.x + b.w) } else { None };
        let mut top = d.body.top;
        if d.zooming {
            top *= 1.4;
            d.zoom -= 1.0;
        } else if d.boost > 0 {
            top *= 1.3;
            d.boost -= 1;
        }
        if swimming {
            top = 1.8;
        } else if blanket.is_some() {
            top *= 0.55;
        }
        let in_air = d.state == DogState::Air;
        if in_air {
            // a little air control
            d.vx = (d.vx + dir * 0.07).clamp(-top * 1.4, top * 1.4);
        } else if dir != 0.0 {
            let acc = 0.14;
            if d.vx * dir < 0.0 {
                d.vx += dir * acc * 2.0;
            } else {
                d.vx += dir * acc;
            }
            d.vx = d.vx.clamp(-top, top);
            d.face = dir;
        } else {
            let dec = 0.14;
            if d.vx.abs() <= dec {
                d.vx = 0.0;
            } else {
                d.vx -= dec * d.vx.signum();
            }
            // standing still: look at the disc
            if d.vx == 0.0 && !d.has_disc {
                let dx = self.disc.x - d.x;
                if dx.abs() > 6.0 {
                    d.face = dx.signum();
                }
            }
        }
        // a slowdown can leave the dog faster than allowed: ease down
        if !in_air && d.vx.abs() > top {
            d.vx = d.vx.signum() * (d.vx.abs() - 0.15).max(top);
        }
        d.x += d.vx;
        let max_x = FIELD_END - 4.0;
        if d.x > max_x {
            d.x = max_x;
            d.vx = 0.0;
        }
        if d.x < -60.0 {
            d.x = -60.0;
            d.vx = 0.0;
        }
        if d.vx.abs() > 0.05 {
            d.anim += d.vx.abs() * 0.12;
        }
        // leap
        if e.b && !in_air && !swimming {
            d.state = DogState::Air;
            d.vz = d.body.jump_v;
            d.air_t = 0;
            self.sfx.push(Sfx::Bark);
        }
        let d = &mut self.dog;
        // flip
        if e.a && d.state == DogState::Air && d.flip_t < 0 {
            d.flip_t = 0;
            self.stats.flips_tried += 1;
            self.sfx.push(Sfx::Flip);
        }
        let d = &mut self.dog;
        if d.state == DogState::Air {
            d.air_t += 1;
            d.z += d.vz;
            d.vz -= DOG_G;
            if d.flip_t >= 0 {
                d.flip_t += 1;
                if d.flip_t >= FLIP_LEN {
                    if d.has_disc {
                        d.flip_with_disc = true;
                    }
                    d.flip_t = -1;
                }
            }
            if d.z <= 0.0 {
                d.z = 0.0;
                d.vz = 0.0;
                if d.flip_t >= 0 {
                    // landed mid-flip
                    d.flip_t = -1;
                    d.state = DogState::Tumble;
                    d.tumble_t = 50;
                    self.sfx.push(Sfx::Oof);
                    self.shake = 6;
                    let (x, z) = (self.dog.x, 14.0);
                    self.burst(x, z, 5, FxKind::Star, 0.8);
                    self.fx(x, 0.0, -0.5, 0.5, 14, FxKind::Dust);
                    self.fx(x, 0.0, 0.5, 0.5, 14, FxKind::Dust);
                    if self.dog.has_disc {
                        self.dog.has_disc = false;
                        self.dog.caught_air = false;
                        self.disc.state = DiscState::Stopped;
                        self.disc.z = 0.0;
                        self.finish_miss(MissKind::Oof);
                    } else {
                        self.popup(x, 30.0, "OOF!".into(), rgb(248, 96, 72));
                    }
                    return;
                }
                let swimming = self.pond.map(|(a, b)| d.x > a + 2.0 && d.x < b - 2.0).unwrap_or(false);
                d.state = if swimming { DogState::Swim } else { DogState::Stand };
                if swimming {
                    self.sfx.push(Sfx::Splash);
                    let x = self.dog.x;
                    self.burst(x, 0.0, 10, FxKind::Drop, 1.0);
                } else {
                    let x = self.dog.x;
                    self.fx(x, 0.0, -0.4, 0.3, 12, FxKind::Dust);
                }
            }
        } else {
            let was_swim = d.state == DogState::Swim;
            d.state = if swimming {
                DogState::Swim
            } else if d.vx.abs() > 0.05 {
                DogState::Run
            } else {
                DogState::Stand
            };
            if swimming && !was_swim {
                self.sfx.push(Sfx::Splash);
                let x = self.dog.x;
                self.burst(x, 0.0, 8, FxKind::Drop, 0.9);
            }
            // dust puffs when running fast
            let d = self.dog;
            if d.state == DogState::Run && d.vx.abs() > 1.6 && self.frame % 6 == 0 {
                self.fx(d.x - d.face * 8.0, 0.0, -d.face * 0.3, 0.3, 12, FxKind::Dust);
            }
        }
        // sausages on picnic blankets
        let d = self.dog;
        if d.z <= 0.0 {
            match blanket {
                Some(i) if self.dog.on_blanket != Some(i) => {
                    self.dog.on_blanket = Some(i);
                    if self.blankets[i].sausages > 0 && !self.dog.has_disc {
                        self.blankets[i].sausages -= 1;
                        self.sausages_now += 1;
                        self.stats.sausages += 1;
                        self.sfx.push(Sfx::Chomp);
                        self.popup(d.x, 30.0, format!("SAUSAGE +{}", SAUSAGE_BONUS), rgb(248, 160, 96));
                        self.burst(d.x + d.face * 10.0, 8.0, 5, FxKind::Crumb, 0.6);
                    }
                }
                None => self.dog.on_blanket = None,
                _ => {}
            }
        }
    }

    fn update_squirrel(&mut self, e: Pad) {
        let _ = e;
        if let Some(due) = self.squirrel_due {
            if self.phase_t >= due && self.disc.state == DiscState::Flying {
                self.squirrel_due = None;
                // across the dog's path, ahead of it but before the landing spot
                let mut pts = Vec::new();
                predict(&self.disc, self.wind, &mut pts);
                let land = pts.last().map(|p| p.0).unwrap_or(self.disc.x);
                let ahead = self.dog.x + 50.0 + self.rng.range(0.0, 50.0);
                if land - self.dog.x > 70.0 {
                    let x = ahead.min(land - 20.0);
                    self.squirrel = Some(Squirrel { x, d: 30.0, t: 0, wait: 0, taunt: 0, gone: false });
                    self.sfx.push(Sfx::Chitter);
                }
            }
        }
        let Some(mut s) = self.squirrel else { return };
        s.t += 1;
        if s.gone {
            s.d -= 1.6;
            s.x += 0.6;
        } else if self.dog.state == DogState::Stare {
            // a standoff on the line
            s.taunt += 1;
        } else if s.taunt > 0 {
            s.gone = true;
        } else {
            // hop across in little bursts
            let hop = (s.t / 10) % 3 != 2;
            if hop {
                s.d -= 0.75;
            }
        }
        if s.d < -40.0 {
            self.squirrel = None;
            return;
        }
        // the dog spots it
        let d = &self.dog;
        if !self.stared && !s.gone && s.d.abs() < 12.0 && (d.x - s.x).abs() < 24.0 && d.z <= 0.0 && !d.has_disc {
            self.stared = true;
            self.stats.stares += 1;
            let st = d.body.stare;
            self.dog.state = DogState::Stare;
            self.dog.stare_t = st;
            self.dog.vx = 0.0;
            self.dog.face = if s.x >= self.dog.x { 1.0 } else { -1.0 };
            self.dog.zooming = false;
            self.sfx.push(Sfx::Chitter);
            self.popup(s.x, 34.0, "SQUIRREL!".into(), rgb(248, 152, 64));
            let speak = self.human(self.thrower()).is_some();
            if speak {
                self.quip = Some(Quip { who: None, text: "TAP C TO SNAP OUT OF IT!".into(), t: 0 });
            }
        }
        if self.dog.state != DogState::Stare && s.taunt > 0 {
            s.gone = true;
        }
        self.squirrel = Some(s);
    }

    fn update_gull(&mut self) {
        if self.gull_due && self.gull.is_none() && self.disc.state == DiscState::Flying && self.disc.t > 20 && self.disc.vz < 0.3 && self.disc.z > 30.0 {
            self.gull_due = false;
            let x = self.disc.x + 110.0;
            self.gull = Some(Gull { x, z: 120.0, vx: -1.6, vz: 0.0, has_disc: false, t: 0, leaving: false });
            self.sfx.push(Sfx::Squawk);
            self.popup(x - 20.0, 100.0, "SQUAWK!".into(), rgb(248, 248, 248));
        }
        let Some(mut g) = self.gull else { return };
        g.t += 1;
        if g.has_disc || g.leaving {
            g.vx = (g.vx + 0.05).min(2.0);
            g.vz = (g.vz + 0.04).min(1.2);
            if g.has_disc {
                self.disc.x = g.x + 2.0;
                self.disc.z = g.z - 5.0;
            }
        } else if self.disc.state == DiscState::Flying {
            // home in on the disc
            let (dx, dz) = (self.disc.x - g.x, self.disc.z - g.z);
            let dist = (dx * dx + dz * dz).sqrt().max(0.01);
            let sp = 2.4;
            g.vx += (dx / dist * sp - g.vx) * 0.08;
            g.vz += (dz / dist * sp - g.vz) * 0.08;
            if dist < 6.0 {
                g.has_disc = true;
                self.disc.state = DiscState::Gulled;
                self.sfx.push(Sfx::Squawk);
                let (x, z) = (g.x, g.z);
                self.burst(x, z, 6, FxKind::Feather, 0.8);
                self.finish_miss(MissKind::Seagull);
            }
        } else {
            // missed it: off it goes, grumbling
            g.leaving = true;
        }
        g.x += g.vx;
        g.z += g.vz;
        if g.z > 200.0 || g.x > self.cam + 400.0 || g.t > 900 {
            self.gull = None;
            return;
        }
        self.gull = Some(g);
    }

    fn try_catch(&mut self) {
        let d = self.dog;
        if matches!(d.state, DogState::Tumble | DogState::Stare) {
            return;
        }
        let (mx, mz) = d.mouth();
        let ds = self.disc;
        let caught = match ds.state {
            // not straight out of the owner's hand
            DiscState::Flying if ds.t < 24 => false,
            DiscState::Flying => {
                let (rx, rz) = if d.flip_t >= 0 { (6.0, 6.0) } else { (6.0, 5.0) };
                (ds.x - mx).abs() <= rx && (ds.z - mz).abs() <= rz
            }
            DiscState::Skid => d.z <= 0.0 && (ds.x - (d.x + d.face * d.body.mouth.0 * 0.7)).abs() <= 8.0,
            DiscState::Floating => d.state == DogState::Swim && (ds.x - (d.x + d.face * 6.0)).abs() <= 8.0,
            _ => false,
        };
        if !caught {
            return;
        }
        self.sfx.push(Sfx::Chomp);
        self.dog.has_disc = true;
        self.dog.zooming = false;
        // over the shoulder: running away from the thrower, the disc came from behind
        let shoulder = d.face > 0.0 && d.vx > 0.9 && self.prev_disc.0 < self.prev_mouth.0 && ds.state == DiscState::Flying;
        self.shoulder_now = shoulder;
        if d.state == DogState::Air {
            self.dog.caught_air = true;
            let (x, z) = (mx, mz);
            self.burst(x, z, 6, FxKind::Sparkle, 0.9);
        } else {
            let kind = match ds.state {
                DiscState::Skid => CatchKind::Rolling,
                DiscState::Floating => CatchKind::Soggy,
                _ => CatchKind::Ground,
            };
            self.finish_catch(kind);
        }
    }

    fn finish_catch(&mut self, kind: CatchKind) {
        let side = self.thrower();
        let yards = ((self.dog.x / YD).round() as i32).max(1);
        let r = TossResult { outcome: Outcome::Catch(kind), yards, perfect: self.perfect, shoulder: self.shoulder_now, sausages: self.sausages_now };
        self.result = Some(r);
        self.sides[side].results.push(r);
        self.sides[side].misses_in_row = 0;
        let st = &mut self.stats;
        st.catches += 1;
        st.yards_caught += yards as u32;
        st.throw_yards += yards as u32;
        st.points += r.points() as u32;
        if r.shoulder {
            st.shoulders += 1;
        }
        match kind {
            CatchKind::Ground => st.ground += 1,
            CatchKind::Rolling => st.rolling += 1,
            CatchKind::Soggy => st.soggy += 1,
            CatchKind::Jump => st.jumps += 1,
            CatchKind::Flip => st.flips += 1,
        }
        let col = match kind {
            CatchKind::Flip => rgb(248, 96, 200),
            CatchKind::Jump => rgb(96, 224, 248),
            _ => rgb(248, 248, 248),
        };
        self.banner(kind.label(), col, false);
        self.sfx.push(Sfx::Cheer);
        self.excite = if kind.mult() >= 2 { 1.0 } else { 0.6 };
        self.owner_mood = 1;
        let (x, z) = (self.dog.x, 44.0);
        self.popup(x, z, format!("+{}", r.points()), rgb(248, 232, 64));
        if r.shoulder {
            self.popup(x, z + 12.0, "OVER THE SHOULDER!".into(), rgb(160, 248, 120));
        }
        if kind == CatchKind::Flip {
            self.shake = 4;
        }
        self.dog.state = if kind == CatchKind::Soggy { DogState::Shake } else { DogState::Happy };
        self.dog.vx = 0.0;
        if self.dog.state == DogState::Shake {
            self.sfx.push(Sfx::Splash);
        }
    }

    fn finish_miss(&mut self, kind: MissKind) {
        if self.result.is_some() {
            return;
        }
        let side = self.thrower();
        let land = self.disc.x;
        let r = TossResult { outcome: Outcome::Miss(kind), yards: 0, perfect: self.perfect, shoulder: false, sausages: self.sausages_now };
        self.result = Some(r);
        self.sides[side].results.push(r);
        self.sides[side].misses_in_row += 1;
        let st = &mut self.stats;
        st.throw_yards += (land / YD).max(0.0) as u32;
        match kind {
            MissKind::Dropped => st.dropped += 1,
            MissKind::Oof => st.oofs += 1,
            MissKind::Seagull => st.gulls += 1,
            MissKind::Sea | MissKind::Hedge | MissKind::Crowd => st.out += 1,
            MissKind::Sunk => st.sunk += 1,
        }
        self.banner(kind.label(), rgb(232, 64, 72), kind != MissKind::Dropped);
        self.sfx.push(Sfx::WahWah);
        self.owner_mood = -1;
        let d = &mut self.dog;
        d.zooming = false;
        if d.state != DogState::Tumble && d.state != DogState::Air {
            d.vx = 0.0;
            d.state = match kind {
                MissKind::Seagull => DogState::Bark,
                _ if self.sides[side].misses_in_row >= 2 => DogState::Lie,
                _ => DogState::Guilty,
            };
        }
        let n = self.rng.int(super::data::MISS_QUIPS.len() as i32) as usize;
        let text = match kind {
            MissKind::Seagull => "THAT GULL HAS A CRIMINAL RECORD.".to_string(),
            MissKind::Sea => "THE FISH SAY THANK YOU.".to_string(),
            MissKind::Crowd => "A FAN IN ROW 12 KEEPS IT. RUDE.".to_string(),
            MissKind::Hedge => "THE HEDGE ALWAYS WINS.".to_string(),
            MissKind::Sunk => "IT IS A POND DISC NOW.".to_string(),
            MissKind::Oof => "NICE FLIP. SHAME ABOUT THE LANDING.".to_string(),
            MissKind::Dropped => super::data::MISS_QUIPS[n].to_string(),
        };
        self.quip = Some(Quip { who: None, text, t: 0 });
    }

    fn update_result(&mut self) {
        let t = self.phase_t;
        // the dog's little show
        let d = &mut self.dog;
        match d.state {
            DogState::Tumble => {
                d.tumble_t -= 1;
                d.vx *= 0.9;
                d.x += d.vx;
                if d.tumble_t <= 0 {
                    d.state = DogState::Guilty;
                }
            }
            DogState::Air => {
                d.z += d.vz;
                d.vz -= DOG_G;
                d.x += d.vx;
                if d.z <= 0.0 {
                    d.z = 0.0;
                    d.vz = 0.0;
                    d.vx = 0.0;
                    d.flip_t = -1;
                    d.state = DogState::Guilty;
                }
            }
            DogState::Happy => {
                // little hops of joy
                if d.z <= 0.0 && t % 24 == 0 && t < 100 {
                    d.vz = 1.6;
                }
                d.z += d.vz;
                d.vz -= DOG_G;
                if d.z <= 0.0 {
                    d.z = 0.0;
                    d.vz = 0.0;
                }
            }
            DogState::Shake => {
                if t % 5 == 0 && t < 70 {
                    let (x, face) = (d.x, d.face);
                    let _ = face;
                    for k in 0..3 {
                        let vx = self.rng.range(-1.4, 1.4);
                        let vz = self.rng.range(0.5, 1.5);
                        self.fx(x + k as f32 - 1.0, 8.0, vx, vz, 20, FxKind::Drop);
                    }
                }
                if t > 70 {
                    self.dog.state = DogState::Happy;
                }
            }
            DogState::Bark => {
                if t % 30 == 10 && t < 100 {
                    self.sfx.push(Sfx::Bark);
                }
            }
            _ => {}
        }
        if self.dog.has_disc {
            let (mx, mz) = self.dog.mouth();
            self.disc.x = mx;
            self.disc.z = mz - 1.0;
        }
        self.update_gull();
        if let Some(mut s) = self.squirrel {
            s.gone = true;
            s.d -= 1.6;
            s.x += 0.6;
            self.squirrel = if s.d < -40.0 { None } else { Some(s) };
        }
        if self.sleeper_awake > 0 {
            self.sleeper_awake -= 1;
        }
        if t >= RESULT_FRAMES {
            self.next_turn();
        }
    }

    fn next_turn(&mut self) {
        self.turn += 1;
        if self.turn >= self.turns {
            let (a, b) = (self.sides[0].total(), self.sides[1].total());
            if a != b {
                self.end(if a > b { 0 } else { 1 });
                return;
            }
            if self.turns < TOSSES * 2 + 6 {
                // tied: one more toss each
                self.turns += 2;
                self.stats.sudden += 1;
            } else {
                // still tied after three sudden-dog rounds: flip a coin
                self.coin_toss = true;
                let w = self.rng.int(2) as usize;
                self.end(w);
                return;
            }
        }
        self.setup_turn();
    }

    fn end(&mut self, w: usize) {
        self.winner = Some(w);
        self.phase = Phase::Over;
        self.phase_t = 0;
        let name = PAIRS[self.sides[w].pair].dog.name;
        let txt = format!("{} WINS!", name);
        self.banner = Some(Banner { text: txt, t: 0, dur: 9999, color: rgb(248, 248, 248), pop: true, scale: 0 });
        self.sfx.push(Sfx::Whistle);
        self.sfx.push(Sfx::Cheer);
        self.excite = 1.0;
        self.quip = None;
        self.popups.clear();
    }

    fn update_fx(&mut self) {
        for f in self.fx.iter_mut() {
            f.x += f.vx;
            f.z += f.vz;
            f.life -= 1;
            match f.kind {
                FxKind::Dust => {
                    f.vx *= 0.9;
                    f.vz *= 0.9;
                }
                FxKind::Drop | FxKind::Crumb => f.vz -= 0.12,
                FxKind::Feather | FxKind::Leaf => {
                    f.vz = (f.vz - 0.03).max(-0.4);
                    f.vx *= 0.97;
                }
                FxKind::Sparkle | FxKind::Star => {
                    f.vx *= 0.92;
                    f.vz *= 0.92;
                }
            }
            if f.z < 0.0 && matches!(f.kind, FxKind::Drop | FxKind::Crumb) {
                f.life = 0;
            }
        }
        self.fx.retain(|f| f.life > 0);
        for p in self.popups.iter_mut() {
            p.life -= 1;
            p.z += 0.35;
        }
        self.popups.retain(|p| p.life > 0);
        if let Some(b) = self.banner.as_mut() {
            b.t += 1;
            if b.t > b.dur {
                self.banner = None;
            }
        }
        if let Some(q) = self.quip.as_mut() {
            q.t += 1;
            if q.t > 200 {
                self.quip = None;
            }
        }
        if self.shake > 0 {
            self.shake -= 1;
        }
        self.excite = (self.excite - 0.006).max(0.0);
    }

    fn update_camera(&mut self) {
        let target = match self.phase {
            Phase::Ready | Phase::Aim | Phase::Power | Phase::Release => AIM_CAM,
            Phase::Flight => {
                let (dx, gx) = (self.disc.x, self.dog.x);
                let mid = (dx + gx) * 0.5 - 128.0;
                // keep the disc on screen, the dog if it fits
                mid.clamp(dx - 216.0, dx - 40.0)
            }
            Phase::Result | Phase::Over => self.dog.x - 128.0,
        };
        let target = target.clamp(CAM_MIN, CAM_MAX);
        if self.phase == Phase::Ready {
            self.cam = target;
        } else {
            self.cam += (target - self.cam) * 0.12;
        }
    }
}

pub const CAM_MIN: f32 = -96.0;
/// The camera while aiming: the waiting pair, the owner and the meter in view.
pub const AIM_CAM: f32 = -86.0;
pub const CAM_MAX: f32 = 288.0;

/// Do segments ab and pq cross?
fn segments_cross(a: (f32, f32), b: (f32, f32), p: (f32, f32), q: (f32, f32)) -> bool {
    let cross = |o: (f32, f32), u: (f32, f32), v: (f32, f32)| (u.0 - o.0) * (v.1 - o.1) - (u.1 - o.1) * (v.0 - o.0);
    let d1 = cross(a, b, p);
    let d2 = cross(a, b, q);
    let d3 = cross(p, q, a);
    let d4 = cross(p, q, b);
    (d1 > 0.0) != (d2 > 0.0) && (d3 > 0.0) != (d4 > 0.0)
}
