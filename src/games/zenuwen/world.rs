//! The street simulation at 60 Hz: one thief, the BOA's with their view
//! cones and suspicion, parked bikes with locks, pedestrians, Sjaak's boat
//! and everything that makes noise. Pure logic; nothing here draws.
//!
//! World coordinates: x along the street (0..len), y depth (0 = the house
//! fronts at the back, `DEPTH` = the quay edge at the front). The canal is
//! below the quay.

use super::bot::{Bot, BotKind};
use super::data::*;
use crate::engine::input::{edge, Pad};
pub use crate::engine::rng::Rng;
use std::f32::consts::TAU;

pub const DEPTH: f32 = 60.0;
pub const FAR_ROW: f32 = 5.0;
pub const QUAY_ROW: f32 = 55.0;
pub const COUNTDOWN: u32 = 120;
/// Frames the stage-over banner shows before the stage ends.
pub const OVER: u32 = 170;
/// Frames a BOA stands whistling before running after you.
pub const CHASE_WHISTLE: u32 = 30;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Sfx {
    Blip,
    Confirm,
    Click,
    Saw,
    Clank,
    Krak,
    Unlock,
    Whistle,
    Hey,
    Toss,
    Cash,
    Splash,
    Bell,
    Busted,
    Squawk,
    Cheer,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ctrl {
    Human(usize),
    Cpu,
}

// ------------------------------------------------------------------ bikes

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BikeState {
    Parked,
    /// unlocked and standing, nobody on it
    Loose,
    Ridden,
    Delivered,
    Sunk,
}

#[derive(Clone, Debug)]
pub struct Bike {
    pub kind: Kind,
    pub x: f32,
    pub y: f32,
    pub home: (f32, f32),
    pub face_left: bool,
    pub color: usize,
    pub lock: Lock,
    /// the lock it came with (a confiscated bike gets it back)
    pub lock0: Lock,
    /// work done on the current lock, 0..1
    pub progress: f32,
    pub state: BikeState,
    /// frames left that a cut lock dangles from it
    pub hot: u32,
    /// a BOA knows this one is stolen
    pub flagged: bool,
    /// fished out of the canal by the crane boat
    pub fished: bool,
}

// ------------------------------------------------------------------ thief

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TState {
    Walk,
    Pick,
    Ride,
    Hide,
    Casual,
    Fall,
    Busted,
    Swim,
    Climb,
    Toss,
    Out,
}

#[derive(Clone, Debug, Default)]
pub struct TStats {
    pub steals: u32,
    pub chases: u32,
    pub cutters: u32,
    pub swims: u32,
    pub crashes: u32,
    pub clanks: u32,
    /// frames some BOA had you in view
    pub seen: u32,
}

#[derive(Clone, Debug)]
pub struct Thief {
    pub who: Who,
    pub ctrl: Ctrl,
    pub x: f32,
    pub y: f32,
    pub face_left: bool,
    pub state: TState,
    pub st_t: u32,
    /// the bike you ride
    pub bike: Option<usize>,
    /// the bike whose lock you work on
    pub pick: Option<usize>,
    pub v: f32,
    pub vy: f32,
    pub moving: bool,
    pub running: bool,
    pub anim: f32,
    /// U-lock: needle phase and the centre of the green zone
    pub needle: f32,
    pub zone: f32,
    pub last_tap: u32,
    pub saw_left: bool,
    pub saw_n: u32,
    pub cutters: u32,
    pub fines: u32,
    /// delivered bikes: kind, color, on the order
    pub delivered: Vec<(Kind, usize, bool)>,
    /// BOA's ignore you for a moment after a fine
    pub grace: u32,
    /// BOA's know your face for a while after a chase starts
    pub wanted: u32,
    pub wet: u32,
    pub edge_t: u32,
    pub spot: Option<usize>,
    pub hide_seen: bool,
    /// guilt right after you left a stolen bike
    pub hot: u32,
    pub stats: TStats,
    /// a bike flying onto Sjaak's boat: kind, color, from (x, y), to x
    pub toss: Option<(Kind, usize, f32, f32, f32)>,
    /// whistle (0) or selfie (1) when acting casual
    pub casual_kind: u8,
    /// 2P: what the BOA player earned off you
    pub complaints: u32,
}

// ------------------------------------------------------------------ BOA's

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoaState {
    Patrol,
    /// glancing back over the shoulder
    Look,
    Coffee,
    Investigate,
    Chase,
    Search,
    Ticket,
    /// at the quay edge, waiting for a swimmer to come out
    Wait,
}

#[derive(Clone, Debug)]
pub struct Dog {
    pub x: f32,
    pub y: f32,
    pub face_left: bool,
    pub bark: u32,
    pub anim: f32,
}

#[derive(Clone, Debug)]
pub struct Boa {
    pub name: &'static str,
    pub kind: BoaKind,
    pub ctrl: Ctrl,
    pub x: f32,
    pub y: f32,
    pub face_left: bool,
    pub state: BoaState,
    pub st_t: u32,
    pub sus: f32,
    pub x0: f32,
    pub x1: f32,
    pub lane: f32,
    pub vision: f32,
    pub look_every: u32,
    pub next_look: u32,
    pub coffee: Option<f32>,
    pub coffee_cool: u32,
    pub target: (f32, f32),
    pub lost: u32,
    pub chase_t: u32,
    pub cool: u32,
    pub say: Option<(&'static str, u32)>,
    pub anim: f32,
    pub moving: bool,
    pub running: bool,
    pub dog: Option<Dog>,
    /// 2P: fines written and complaints earned by the BOA player
    pub fines: u32,
    pub complaints: u32,
    pub whistle_t: u32,
    pub grab_t: u32,
}

impl Boa {
    pub fn speeds(&self) -> (f32, f32, f32) {
        // patrol, investigate, chase
        match self.kind {
            BoaKind::Foot => (0.45, 0.8, 1.42),
            BoaKind::Bike => (1.0, 1.4, 2.35),
            BoaKind::Chief => (0.4, 0.75, 1.32),
        }
    }
    pub fn alert(&self) -> bool {
        matches!(self.state, BoaState::Chase | BoaState::Investigate | BoaState::Wait)
    }
}

// ------------------------------------------------------------------ the rest

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PState {
    Walk,
    Photo,
    Fall,
    Dance,
}

#[derive(Clone, Debug)]
pub struct Ped {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub style: usize,
    pub pal: usize,
    pub state: PState,
    pub st_t: u32,
    pub anim: f32,
    pub crowd: bool,
    pub suitcase: bool,
    pub face_left: bool,
    pub home: f32,
}

/// Sjaak the fence in his boat.
#[derive(Clone, Debug)]
pub struct Sjaak {
    pub x: f32,
    pub target: f32,
    pub wait: u32,
    pub face_left: bool,
    pub moving: bool,
    pub pile: Vec<(Kind, usize)>,
    pub happy: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoatKind {
    Tour,
    Party,
    Crane,
}

#[derive(Clone, Debug)]
pub struct Boat {
    pub kind: BoatKind,
    pub x: f32,
    pub vx: f32,
    pub t: u32,
    /// crane: where it stops to fish
    pub stop: f32,
    pub done: bool,
}

#[derive(Clone, Debug)]
pub struct Heron {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub flying: bool,
    pub gone: bool,
    pub t: u32,
}

#[derive(Clone, Debug)]
pub struct Popup {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: u32,
    pub t: u32,
}

#[derive(Clone, Debug)]
pub struct Banner {
    pub text: String,
    pub color: u32,
    pub t: i32,
    pub dur: i32,
    pub scale: i32,
    pub pop: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Noise {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub loud: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FxKind {
    Splash,
    Krak,
    Dust,
    Drip,
    Note,
    Flash,
    /// how far a loud noise carries
    Ring,
}

#[derive(Clone, Copy, Debug)]
pub struct Fx {
    pub kind: FxKind,
    pub x: f32,
    pub y: f32,
    pub t: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Countdown,
    Play,
    Over,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Cleared,
    TimeUp,
    Busted,
}

/// Who a view (camera) follows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum View {
    Thief,
    Boa(usize),
}

pub struct World {
    pub stage: usize,
    pub def: &'static StageDef,
    pub len: f32,
    pub t: u32,
    pub phase: Phase,
    pub phase_t: u32,
    pub time_left: u32,
    pub thief: Thief,
    pub boas: Vec<Boa>,
    pub bikes: Vec<Bike>,
    pub peds: Vec<Ped>,
    pub spots: Vec<(Spot, f32, f32)>,
    pub sjaak: Sjaak,
    pub boats: Vec<Boat>,
    pub herons: Vec<Heron>,
    pub order: Vec<(Kind, bool)>,
    pub noise: Vec<Noise>,
    pub popups: Vec<Popup>,
    pub banners: [Option<Banner>; 2],
    pub fx: Vec<Fx>,
    pub views: Vec<View>,
    pub cam: [f32; 2],
    pub sfx: Vec<Sfx>,
    pub rng: Rng,
    pub result: Option<Outcome>,
    pub bot: Option<Bot>,
    prev: [Pad; 2],
    pub tour_at: u32,
    pub crane_at: u32,
    pub shake: u32,
    /// any BOA is chasing (for the music)
    pub chasing: bool,
    /// 2P: the BOA player, as an index into `boas`
    pub human_boa: Option<usize>,
    /// every catch: what the BOA was doing, and what the thief was doing
    pub busts: Vec<(BoaState, TState)>,
    /// a rare sight: a bike stuck up this tree (index into `spots`)
    pub tree_bike: Option<usize>,
}

fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (a.0 - b.0, a.1 - b.1);
    (dx * dx + dy * dy).sqrt()
}

impl World {
    /// `thief`: who and how they are played. `boa_pad`: 2P, the pad of the
    /// BOA player (who replaces the CPU BOA's but one).
    pub fn new(stage: usize, who: Who, ctrl: Ctrl, boa_pad: Option<usize>, seed: u64, bot: Option<BotKind>) -> Self {
        let def = &STAGES[stage.min(STAGES.len() - 1)];
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let len = def.len;
        let spots: Vec<(Spot, f32, f32)> = def.spots.to_vec();

        // ---- bikes: one per order item, then the stage's extras, in free slots
        let mut kinds: Vec<Kind> = def.order.to_vec();
        for &(k, n) in def.bikes {
            for _ in 0..n {
                kinds.push(k);
            }
        }
        let mut slots: Vec<(f32, f32)> = Vec::new();
        for row in [FAR_ROW, QUAY_ROW] {
            let mut x = 60.0;
            while x < len - 50.0 {
                let blocked = spots.iter().any(|&(_, sx, sy)| (sx - x).abs() < 18.0 && (sy - row).abs() < 20.0);
                if !blocked {
                    slots.push((x, row));
                }
                x += 17.0;
            }
        }
        // shuffle
        for i in (1..slots.len()).rev() {
            let j = rng.int(i as i32 + 1) as usize;
            slots.swap(i, j);
        }
        let mut bikes = Vec::new();
        for k in kinds {
            // the golden bike stands by the chief's route, on the quay
            let slot = if k == Kind::Gold {
                let mid = def.boas.iter().find(|b| b.kind == BoaKind::Chief).map(|b| (b.x0 + b.x1) / 2.0).unwrap_or(len / 2.0);
                let i = slots.iter().enumerate().filter(|(_, s)| s.1 == QUAY_ROW).min_by_key(|(_, s)| ((s.0 - mid).abs()) as i32).map(|(i, _)| i).unwrap_or(0);
                slots.remove(i)
            } else if !slots.is_empty() {
                // keep the start area clear
                let i = slots.iter().position(|s| s.0 > 120.0).unwrap_or(0);
                slots.remove(i)
            } else {
                (rng.range(80.0, len - 80.0), QUAY_ROW)
            };
            let mut lock = k.def().lock;
            if k == Kind::Oma && rng.f() < def.oma_chain {
                lock = Lock::Chain;
            }
            let color = match k {
                Kind::Gold => 5,
                Kind::Wreck => 0,
                _ => rng.int(BIKE_COLORS.len() as i32) as usize,
            };
            bikes.push(Bike {
                kind: k,
                x: slot.0,
                y: slot.1,
                home: slot,
                face_left: rng.int(2) == 0,
                color,
                lock,
                lock0: lock,
                progress: 0.0,
                state: BikeState::Parked,
                hot: 0,
                flagged: false,
                fished: false,
            });
        }

        // ---- BOA's (2P: the BOA player plus one CPU colleague)
        let mut boas = Vec::new();
        for (i, s) in def.boas.iter().enumerate() {
            if boa_pad.is_some() && i >= 1 {
                break;
            }
            let x = s.x0 + (s.x1 - s.x0) * rng.range(0.3, 0.7);
            boas.push(Boa {
                name: s.name,
                kind: s.kind,
                ctrl: Ctrl::Cpu,
                x,
                y: s.y,
                face_left: rng.int(2) == 0,
                state: BoaState::Patrol,
                st_t: 0,
                sus: 0.0,
                x0: s.x0,
                x1: s.x1,
                lane: s.y,
                vision: s.vision,
                look_every: s.look_every,
                next_look: (s.look_every as f32 * rng.range(0.5, 1.2)) as u32,
                coffee: if s.coffee { spots.iter().find(|p| p.0 == Spot::Kiosk).map(|p| p.1) } else { None },
                coffee_cool: 600,
                target: (x, s.y),
                lost: 0,
                chase_t: 0,
                cool: 0,
                say: None,
                anim: 0.0,
                moving: false,
                running: false,
                dog: if s.kind == BoaKind::Chief { Some(Dog { x: x - 14.0, y: s.y + 4.0, face_left: false, bark: 0, anim: 0.0 }) } else { None },
                fines: 0,
                complaints: 0,
                whistle_t: 0,
                grab_t: 0,
            });
        }
        let mut human_boa = None;
        if let Some(p) = boa_pad {
            boas.push(Boa {
                name: "BOA",
                kind: BoaKind::Foot,
                ctrl: Ctrl::Human(p),
                x: len * 0.6,
                y: 30.0,
                face_left: true,
                state: BoaState::Patrol,
                st_t: 0,
                sus: 0.0,
                x0: 0.0,
                x1: len,
                lane: 30.0,
                vision: 80.0,
                look_every: 99999,
                next_look: 99999,
                coffee: None,
                coffee_cool: 0,
                target: (0.0, 0.0),
                lost: 0,
                chase_t: 0,
                cool: 0,
                say: None,
                anim: 0.0,
                moving: false,
                running: false,
                dog: None,
                fines: 0,
                complaints: 0,
                whistle_t: 0,
                grab_t: 0,
            });
            human_boa = Some(boas.len() - 1);
        }

        // ---- pedestrians and the crowd
        let mut peds = Vec::new();
        for _ in 0..def.peds {
            let x = rng.range(80.0, len - 40.0);
            let vx = rng.range(0.25, 0.45) * if rng.int(2) == 0 { -1.0 } else { 1.0 };
            let suitcase = def.venue == Venue::Station && rng.f() < 0.5;
            peds.push(Ped {
                x,
                y: rng.range(14.0, 46.0),
                vx: if suitcase { vx * 0.75 } else { vx },
                vy: 0.0,
                style: rng.int(6) as usize,
                pal: rng.int(12) as usize,
                state: PState::Walk,
                st_t: rng.int(300) as u32,
                anim: 0.0,
                crowd: false,
                suitcase,
                face_left: vx < 0.0,
                home: x,
            });
        }
        if def.crowd > 0 {
            let centers = [len * 0.16, len * 0.33, len * 0.62, len * 0.86];
            for k in 0..def.crowd {
                let c = centers[k % centers.len()] + rng.range(-34.0, 34.0);
                peds.push(Ped {
                    x: c,
                    y: rng.range(10.0, 44.0),
                    vx: 0.0,
                    vy: 0.0,
                    style: rng.int(6) as usize,
                    pal: 100 + rng.int(6) as usize,
                    state: PState::Dance,
                    st_t: rng.int(200) as u32,
                    anim: rng.range(0.0, 4.0),
                    crowd: true,
                    suitcase: false,
                    face_left: rng.int(2) == 0,
                    home: c,
                });
            }
        }

        // ---- herons on the quay edge, away from the trees
        let mut herons = Vec::new();
        for _ in 0..def.herons {
            let x = rng.range(160.0, len - 60.0);
            herons.push(Heron { x, y: DEPTH + 1.0, z: 0.0, vx: 0.0, flying: false, gone: false, t: rng.int(100) as u32 });
        }

        let mut boats = Vec::new();
        if def.venue == Venue::Koningsdag {
            boats.push(Boat { kind: BoatKind::Party, x: len * 0.2, vx: 0.3, t: 0, stop: 0.0, done: false });
            boats.push(Boat { kind: BoatKind::Party, x: len * 0.75, vx: -0.25, t: 40, stop: 0.0, done: false });
        }
        let sx = len * rng.range(0.35, 0.6);
        let crane_at = if rng.f() < 0.5 { 60 * rng.range(35.0, 90.0) as u32 } else { u32::MAX };
        let cutters = def.cutters;
        let mut views = vec![View::Thief];
        if let Some(h) = human_boa {
            views.push(View::Boa(h));
        }
        let mut w = World {
            stage,
            def,
            len,
            t: 0,
            phase: Phase::Countdown,
            phase_t: 0,
            time_left: def.time * 60,
            thief: Thief {
                who,
                ctrl,
                x: 36.0,
                y: 30.0,
                face_left: false,
                state: TState::Walk,
                st_t: 0,
                bike: None,
                pick: None,
                v: 0.0,
                vy: 0.0,
                moving: false,
                running: false,
                anim: 0.0,
                needle: 0.0,
                zone: 0.5,
                last_tap: 0,
                saw_left: false,
                saw_n: 0,
                cutters,
                fines: 0,
                delivered: Vec::new(),
                grace: 0,
                wanted: 0,
                wet: 0,
                edge_t: 0,
                spot: None,
                hide_seen: false,
                hot: 0,
                stats: TStats::default(),
                toss: None,
                casual_kind: 0,
                complaints: 0,
            },
            boas,
            bikes,
            peds,
            spots,
            sjaak: Sjaak { x: sx, target: sx, wait: 300, face_left: false, moving: false, pile: vec![(Kind::Wreck, 0), (Kind::Oma, 3)], happy: 0 },
            boats,
            herons,
            order: def.order.iter().map(|&k| (k, false)).collect(),
            noise: Vec::new(),
            popups: Vec::new(),
            banners: [None, None],
            fx: Vec::new(),
            views,
            cam: [0.0; 2],
            sfx: Vec::new(),
            rng,
            result: None,
            bot: bot.map(|k| Bot::new(k, seed ^ 0xB07)),
            prev: [Pad::default(); 2],
            tour_at: 600,
            crane_at,
            shake: 0,
            chasing: false,
            human_boa,
            busts: Vec::new(),
            tree_bike: None,
        };
        if w.rng.f() < 0.35 {
            let trees: Vec<usize> = (0..w.spots.len()).filter(|&k| w.spots[k].0 == Spot::Tree).collect();
            if !trees.is_empty() {
                w.tree_bike = Some(trees[w.rng.int(trees.len() as i32) as usize]);
            }
        }
        for v in 0..w.views.len() {
            w.cam[v] = w.cam_target(v);
        }
        w.banner_all("KLAAR?", rgb_w(), 70, 3, true);
        w
    }

    // ---------------------------------------------------------------- queries

    pub fn finished(&self) -> bool {
        self.phase == Phase::Over && self.phase_t >= OVER
    }

    pub fn subject(&self, v: usize) -> (f32, f32) {
        match self.views.get(v).copied().unwrap_or(View::Thief) {
            View::Thief => (self.thief.x, self.thief.y),
            View::Boa(i) => (self.boas[i].x, self.boas[i].y),
        }
    }

    fn cam_target(&self, v: usize) -> f32 {
        let (x, _) = self.subject(v);
        let lead = if v == 0 && self.thief.state == TState::Ride { self.thief.v * 18.0 } else { 0.0 };
        (x + lead - 128.0).clamp(0.0, self.len - 256.0)
    }

    /// Money so far: deliveries minus fines.
    pub fn cash(&self) -> i32 {
        let got: i32 = self.thief.delivered.iter().map(|&(k, _, on)| if on { k.def().value } else { k.def().value / 2 }).sum();
        got - self.thief.fines as i32 * FINE
    }

    pub fn order_done(&self) -> bool {
        self.order.iter().all(|o| o.1)
    }

    /// Order kinds still missing.
    pub fn needed(&self) -> Vec<Kind> {
        self.order.iter().filter(|o| !o.1).map(|o| o.0).collect()
    }

    /// How well BOA `b` sees the point (x, y): 0 = not at all, up to 1.
    pub fn sight(&self, b: &Boa, x: f32, y: f32) -> f32 {
        let mut dx = x - b.x;
        if b.face_left {
            dx = -dx;
        }
        let dy = y - b.y;
        // a little bit of awareness right behind them
        if dx < 0.0 {
            return if dx > -9.0 && dy.abs() < 7.0 { 0.4 } else { 0.0 };
        }
        let mut range = b.vision;
        if b.state == BoaState::Coffee {
            range *= 0.3;
        }
        if dx > range {
            return 0.0;
        }
        let half = 9.0 + dx * 0.36;
        if dy.abs() > half {
            return 0.0;
        }
        // trees, the krul, the glass bin and the kiosk block the view
        for &(sp, sx, sy) in &self.spots {
            let between = if b.x < x { sx > b.x + 3.0 && sx < x - 3.0 } else { sx < b.x - 3.0 && sx > x + 3.0 };
            if !between {
                continue;
            }
            let k = (sx - b.x) / (x - b.x);
            let yy = b.y + (y - b.y) * k;
            if (yy - sy).abs() < sp.block() {
                return 0.0;
            }
        }
        1.0 - 0.55 * dx / range
    }

    /// Can BOA's see the thief at all right now?
    pub fn thief_visible(&self) -> bool {
        match self.thief.state {
            TState::Hide => self.thief.hide_seen,
            TState::Busted | TState::Out => false,
            _ => true,
        }
    }

    /// How suspicious what the thief does right now is, per frame of being seen.
    pub fn activity(&self) -> f32 {
        let th = &self.thief;
        match th.state {
            TState::Pick => 1.0 / 40.0,
            TState::Toss => 1.0 / 25.0,
            TState::Ride => match th.bike {
                Some(i) if self.bikes[i].hot > 0 || self.bikes[i].flagged => 1.0 / 45.0,
                _ => 0.0,
            },
            TState::Walk if th.running && th.moving => 1.0 / 110.0,
            TState::Fall => 1.0 / 90.0,
            TState::Climb => 1.0 / 160.0,
            _ => 0.0,
        }
    }

    /// Caught now by a CPU BOA, would a fine stick? They only know a bike
    /// is stolen by the cut lock dangling from it, or from a chase.
    pub fn guilty(&self) -> bool {
        let th = &self.thief;
        match th.state {
            TState::Pick | TState::Toss => true,
            TState::Ride => th.wanted > 0 || th.bike.map(|i| self.bikes[i].hot > 0 || self.bikes[i].flagged).unwrap_or(false),
            _ => th.hot > 0 || th.wanted > 0,
        }
    }

    /// 2P: the BOA player saw it all, so any stolen bike under you counts.
    pub fn guilty_for_player(&self) -> bool {
        self.thief.state == TState::Ride || self.guilty()
    }

    /// Bike `i` is within reach of the thief.
    pub fn in_reach(&self, i: usize) -> bool {
        let b = &self.bikes[i];
        (b.x - self.thief.x).abs() < 11.0 && (b.y - self.thief.y).abs() < 10.0
    }

    /// The bike the thief would work on (or get on) when pressing A.
    pub fn bike_at_hand(&self) -> Option<usize> {
        let th = &self.thief;
        self.bikes
            .iter()
            .enumerate()
            .filter(|(i, b)| matches!(b.state, BikeState::Parked | BikeState::Loose) && self.in_reach(*i))
            .min_by_key(|(_, b)| ((b.x - th.x).abs() + (b.y - th.y).abs()) as i32)
            .map(|(i, _)| i)
    }

    pub fn spot_at_hand(&self) -> Option<usize> {
        let th = &self.thief;
        self.spots.iter().position(|&(sp, sx, sy)| sp.hides() && (sx - th.x).abs() < 13.0 && (sy - th.y).abs() < 15.0)
    }

    pub fn sjaak_near(&self) -> bool {
        (self.sjaak.x - self.thief.x).abs() < 24.0 && self.thief.y > DEPTH - 12.0
    }

    // ---------------------------------------------------------------- effects

    fn say(&mut self, b: usize, s: &'static str) {
        self.boas[b].say = Some((s, 100));
    }

    pub fn popup(&mut self, x: f32, y: f32, text: &str, color: u32) {
        self.popups.push(Popup { x, y, text: text.to_string(), color, t: 0 });
    }

    pub fn banner_all(&mut self, text: &str, color: u32, dur: i32, scale: i32, pop: bool) {
        for v in 0..self.views.len().min(2) {
            self.banners[v] = Some(Banner { text: text.to_string(), color, t: 0, dur, scale, pop });
        }
    }

    /// A banner on the thief's view only.
    fn banner_thief(&mut self, text: &str, color: u32, dur: i32) {
        self.banners[0] = Some(Banner { text: text.to_string(), color, t: 0, dur, scale: 0, pop: false });
    }

    fn make_noise(&mut self, x: f32, y: f32, r: f32, loud: bool) {
        self.noise.push(Noise { x, y, r, loud });
    }

    // ---------------------------------------------------------------- update

    pub fn update(&mut self, pads: &[Pad; 2]) {
        self.t += 1;
        self.phase_t += 1;
        let mut pads = *pads;
        if self.thief.ctrl == Ctrl::Cpu
            && let Some(mut b) = self.bot.take() {
                pads[0] = b.pad(self);
                self.bot = Some(b);
            }
        let e = [edge(pads[0], self.prev[0]), edge(pads[1], self.prev[1])];
        let held = pads;
        self.prev = pads;

        match self.phase {
            Phase::Countdown => {
                if self.phase_t == 70 {
                    self.banner_all("JATTEN!", rgb_y(), 60, 4, true);
                    self.sfx.push(Sfx::Whistle);
                }
                if self.phase_t >= COUNTDOWN {
                    self.phase = Phase::Play;
                    self.phase_t = 0;
                }
                self.update_ambient();
            }
            Phase::Play => {
                let tp = match self.thief.ctrl {
                    Ctrl::Human(p) => p.min(1),
                    Ctrl::Cpu => 0,
                };
                self.update_thief(held[tp], e[tp]);
                for i in 0..self.boas.len() {
                    match self.boas[i].ctrl {
                        Ctrl::Human(p) => self.update_human_boa(i, held[p.min(1)], e[p.min(1)]),
                        Ctrl::Cpu => self.update_boa(i),
                    }
                }
                self.update_sight();
                self.process_noise();
                self.update_ambient();
                self.time_left = self.time_left.saturating_sub(1);
                if self.time_left == 600 {
                    self.popup(self.thief.x, self.thief.y - 30.0, "NOG 10 SECONDEN!", rgb_r());
                }
                self.check_end();
            }
            Phase::Over => {
                self.update_ambient();
                // the thief stands still; BOA's keep walking
                for i in 0..self.boas.len() {
                    if self.boas[i].ctrl == Ctrl::Cpu {
                        self.update_boa(i);
                    }
                }
            }
        }
        // cameras, banners, popups, fx
        for v in 0..self.views.len().min(2) {
            let tgt = self.cam_target(v);
            self.cam[v] += (tgt - self.cam[v]) * 0.12;
            self.cam[v] = self.cam[v].clamp(0.0, self.len - 256.0);
            if let Some(b) = &mut self.banners[v] {
                b.t += 1;
                if b.t > b.dur {
                    self.banners[v] = None;
                }
            }
        }
        for p in &mut self.popups {
            p.t += 1;
            p.y -= 0.35;
        }
        self.popups.retain(|p| p.t < 70);
        for f in &mut self.fx {
            f.t += 1;
            if f.kind == FxKind::Note {
                f.y -= 0.4;
            }
            if f.kind == FxKind::Drip {
                f.y += 0.5;
            }
        }
        self.fx.retain(|f| f.t < if f.kind == FxKind::Splash { 40 } else { 30 });
        self.shake = self.shake.saturating_sub(1);
        self.chasing = self.boas.iter().any(|b| matches!(b.state, BoaState::Chase | BoaState::Wait));
    }

    fn check_end(&mut self) {
        if self.result.is_some() {
            return;
        }
        let out = if self.order_done() && self.thief.toss.is_none() {
            Some(Outcome::Cleared)
        } else if self.thief.fines >= MAX_FINES && self.thief.state != TState::Busted {
            Some(Outcome::Busted)
        } else if self.time_left == 0 {
            Some(Outcome::TimeUp)
        } else {
            None
        };
        if let Some(o) = out {
            self.result = Some(o);
            self.phase = Phase::Over;
            self.phase_t = 0;
            if self.thief.state != TState::Swim {
                self.thief.state = TState::Out;
            }
            match o {
                Outcome::Cleared => {
                    self.banner_all("WEG WAS IE!", rgb_y(), OVER as i32, 0, false);
                    self.sfx.push(Sfx::Cheer);
                }
                Outcome::TimeUp => {
                    self.banner_all("TE LAAT!", rgb_r(), OVER as i32, 0, false);
                    self.sfx.push(Sfx::Whistle);
                }
                Outcome::Busted => {
                    self.banner_all("OPGEPAKT!", rgb_r(), OVER as i32, 0, false);
                    self.sfx.push(Sfx::Busted);
                }
            }
        }
    }

    // ---------------------------------------------------------------- thief

    fn update_thief(&mut self, p: Pad, e: Pad) {
        let th = &mut self.thief;
        th.st_t += 1;
        th.grace = th.grace.saturating_sub(1);
        th.wanted = th.wanted.saturating_sub(1);
        th.hot = th.hot.saturating_sub(1);
        if th.wet > 0 {
            th.wet -= 1;
            if th.wet.is_multiple_of(9) && th.state != TState::Swim {
                self.fx.push(Fx { kind: FxKind::Drip, x: th.x + (th.wet % 5) as f32 - 2.0, y: th.y - 6.0, t: 0 });
            }
        }
        for b in &mut self.bikes {
            b.hot = b.hot.saturating_sub(1);
        }
        let dx = p.right as i32 as f32 - p.left as i32 as f32;
        let dy = p.down as i32 as f32 - p.up as i32 as f32;
        match self.thief.state {
            TState::Walk | TState::Casual => self.thief_walk(p, e, dx, dy),
            TState::Pick => self.thief_pick(p, e),
            TState::Ride => self.thief_ride(p, e, dx, dy),
            TState::Hide => {
                let th = &mut self.thief;
                if e.c || e.a || e.left || e.right || e.up || e.down {
                    th.state = TState::Walk;
                    th.st_t = 0;
                    th.spot = None;
                    th.hide_seen = false;
                    if let Some(sp) = self.spots.iter().find(|s| (s.1 - th.x).abs() < 2.0) {
                        th.y = (sp.2 - 10.0).clamp(12.0, DEPTH - 6.0);
                        if sp.2 < 20.0 {
                            th.y = sp.2 + 9.0;
                        }
                    }
                }
            }
            TState::Fall => {
                if self.thief.st_t > 45 {
                    self.thief.state = TState::Walk;
                    self.thief.st_t = 0;
                }
            }
            TState::Busted => {
                if self.thief.st_t > 150 {
                    let th = &mut self.thief;
                    th.state = TState::Walk;
                    th.st_t = 0;
                    th.grace = 240;
                }
            }
            TState::Swim => {
                let th = &mut self.thief;
                th.x = (th.x + dx * 0.55).clamp(10.0, self.len - 10.0);
                th.moving = dx != 0.0;
                if dx != 0.0 {
                    th.face_left = dx < 0.0;
                }
                th.anim += 0.08;
                th.wet = 600;
                if (p.up && th.st_t > 40) || (th.st_t > 60 * 20) {
                    th.state = TState::Climb;
                    th.st_t = 0;
                }
            }
            TState::Climb => {
                if self.thief.st_t > 36 {
                    let th = &mut self.thief;
                    th.state = TState::Walk;
                    th.st_t = 0;
                    th.y = DEPTH - 2.0;
                }
            }
            TState::Toss => {
                if self.thief.st_t == 26
                    && let Some((kind, color, _, _, _)) = self.thief.toss.take() {
                        self.deliver(kind, color);
                    }
                if self.thief.st_t > 34 {
                    self.thief.state = TState::Walk;
                    self.thief.st_t = 0;
                }
            }
            TState::Out => {}
        }
    }

    fn thief_walk(&mut self, p: Pad, e: Pad, dx: f32, dy: f32) {
        let who = self.thief.who;
        // C: hide at a spot, or act casual while held
        if e.c {
            if let Some(si) = self.spot_at_hand() {
                let seen = self.thief_seen_by_any();
                let (_, sx, sy) = self.spots[si];
                let th = &mut self.thief;
                th.state = TState::Hide;
                th.st_t = 0;
                th.spot = Some(si);
                th.hide_seen = seen;
                th.x = sx;
                th.y = sy;
                th.running = false;
                return;
            }
            self.thief.casual_kind = (self.t / 7 % 2) as u8;
        }
        if p.c {
            let th = &mut self.thief;
            if th.state != TState::Casual {
                th.st_t = 0;
            }
            th.state = TState::Casual;
            th.moving = false;
            th.running = false;
            if th.st_t % 40 == 1 && th.casual_kind == 0 {
                self.fx.push(Fx { kind: FxKind::Note, x: th.x + if th.face_left { -6.0 } else { 6.0 }, y: th.y - 30.0, t: 0 });
            }
            return;
        }
        if self.thief.state == TState::Casual {
            self.thief.state = TState::Walk;
            self.thief.st_t = 0;
        }
        // A: work a lock, or get on a loose bike
        if e.a
            && let Some(i) = self.bike_at_hand() {
                let b = &self.bikes[i];
                if b.state == BikeState::Loose || b.lock == Lock::None {
                    self.mount(i, b.lock == Lock::None && b.state == BikeState::Parked);
                } else {
                    let th = &mut self.thief;
                    th.state = TState::Pick;
                    th.st_t = 0;
                    th.grace = 0;
                    th.pick = Some(i);
                    th.face_left = b.x < th.x;
                    th.needle = 0.0;
                    th.zone = 0.3 + 0.4 * self.rng.f();
                    th.saw_n = 0;
                }
                return;
            }
        let th = &mut self.thief;
        th.running = p.b;
        let sp = if p.b { who.run_speed() } else { who.walk_speed() };
        let (mut vx, mut vy) = (dx * sp, dy * sp * 0.75);
        if dx != 0.0 && dy != 0.0 {
            vx *= 0.8;
            vy *= 0.8;
        }
        if th.wet > 0 {
            vx *= 0.85;
            vy *= 0.85;
        }
        th.moving = dx != 0.0 || dy != 0.0;
        if dx != 0.0 {
            th.face_left = dx < 0.0;
        }
        th.x = (th.x + vx).clamp(6.0, self.len - 6.0);
        th.y += vy;
        if th.y > DEPTH {
            th.y = DEPTH;
            // keep walking into the canal: you jump
            if p.down {
                th.edge_t += 1;
                if th.edge_t > 22 {
                    th.edge_t = 0;
                    self.plons(false);
                    return;
                }
            }
        } else {
            th.edge_t = 0;
        }
        th.y = th.y.max(10.0);
        if th.moving {
            th.anim += if th.running { 0.2 } else { 0.12 };
        }
    }

    fn thief_seen_by_any(&self) -> bool {
        self.boas.iter().any(|b| b.ctrl == Ctrl::Cpu && b.state != BoaState::Ticket && self.sight(b, self.thief.x, self.thief.y) > 0.0)
    }

    fn mount(&mut self, i: usize, cold: bool) {
        let th = &mut self.thief;
        th.state = TState::Ride;
        th.st_t = 0;
        th.bike = Some(i);
        th.pick = None;
        th.v = 0.0;
        th.vy = 0.0;
        let b = &mut self.bikes[i];
        b.state = BikeState::Ridden;
        th.x = b.x;
        th.y = b.y.clamp(10.0, DEPTH - 2.0);
        th.face_left = b.face_left;
        if !cold && b.hot == 0 && !b.fished && b.kind != Kind::Wreck {
            b.hot = 60;
        }
    }

    fn thief_pick(&mut self, _p: Pad, e: Pad) {
        let Some(i) = self.thief.pick else {
            self.thief.state = TState::Walk;
            return;
        };
        let lock = self.bikes[i].lock;
        let who = self.thief.who;
        // leaving: up/down/C (and left/right unless sawing a chain)
        let saw = matches!(lock, Lock::Chain) || (lock == Lock::Double && self.bikes[i].progress < 1.0 && self.bikes[i].lock == Lock::Double);
        if e.up || e.down || e.c || (!saw && (e.left || e.right)) || !self.in_reach(i) {
            let th = &mut self.thief;
            th.state = TState::Walk;
            th.st_t = 0;
            th.pick = None;
            return;
        }
        let (x, y) = (self.thief.x, self.thief.y);
        // B: the bolt cutter
        if e.b && self.thief.cutters > 0 {
            self.thief.cutters -= 1;
            self.thief.stats.cutters += 1;
            self.bikes[i].progress = 1.0;
            self.bikes[i].lock = Lock::None;
            self.sfx.push(Sfx::Krak);
            self.make_noise(x, y, 170.0, true);
            self.fx.push(Fx { kind: FxKind::Krak, x: self.bikes[i].x, y: self.bikes[i].y, t: 0 });
            self.fx.push(Fx { kind: FxKind::Ring, x, y, t: 0 });
            self.popup(x, y - 34.0, "KRAK!", rgb_y());
            self.shake = 8;
            self.unlocked(i);
            return;
        }
        let th = &mut self.thief;
        match lock {
            Lock::Ring => {
                if e.a && th.st_t >= th.last_tap + 4 {
                    th.last_tap = th.st_t;
                    self.bikes[i].progress += 1.0 / who.ring_taps() as f32;
                    self.sfx.push(Sfx::Click);
                    self.noise.push(Noise { x, y, r: 26.0, loud: false });
                }
            }
            Lock::Chain | Lock::Double => {
                let stroke = (e.left && !th.saw_left) || (e.right && th.saw_left) || (th.saw_n == 0 && (e.left || e.right));
                if stroke {
                    th.saw_left = e.left;
                    th.saw_n += 1;
                    self.bikes[i].progress += 1.0 / who.chain_strokes() as f32;
                    if th.saw_n.is_multiple_of(2) {
                        self.sfx.push(Sfx::Saw);
                    }
                    self.noise.push(Noise { x, y, r: 46.0, loud: false });
                }
                if lock == Lock::Double && self.bikes[i].progress >= 1.0 {
                    // the chain is off; now the U-lock
                    self.bikes[i].lock = Lock::U;
                    self.bikes[i].progress = 0.0;
                    self.sfx.push(Sfx::Unlock);
                    self.popup(x, y - 34.0, "NU DE BEUGEL!", rgb_w());
                    let th = &mut self.thief;
                    th.needle = 0.0;
                    th.zone = 0.3 + 0.4 * self.rng.f();
                    return;
                }
            }
            Lock::U => {
                th.needle += TAU / who.u_period();
                if e.a {
                    let n = 0.5 + 0.5 * th.needle.sin();
                    if (n - th.zone).abs() <= who.u_zone() / 2.0 {
                        self.bikes[i].progress += 1.0 / 3.0 + 0.001;
                        th.zone = 0.2 + 0.6 * self.rng.f();
                        self.sfx.push(Sfx::Click);
                    } else {
                        self.bikes[i].progress = (self.bikes[i].progress - 1.0 / 3.0).max(0.0);
                        th.stats.clanks += 1;
                        self.sfx.push(Sfx::Clank);
                        self.noise.push(Noise { x, y, r: 110.0, loud: false });
                        self.popup(x, y - 34.0, "KLENG!", rgb_r());
                    }
                }
            }
            Lock::None => {}
        }
        if self.bikes[i].progress >= 1.0 {
            self.bikes[i].lock = Lock::None;
            self.unlocked(i);
        }
    }

    fn unlocked(&mut self, i: usize) {
        self.thief.stats.steals += 1;
        self.sfx.push(Sfx::Unlock);
        self.mount(i, false);
        self.bikes[i].hot = 300;
        let (x, y) = (self.thief.x, self.thief.y);
        let needed = self.needed().contains(&self.bikes[i].kind);
        self.popup(x, y - 34.0, if needed { "GEJAT!" } else { "GEJAT! (NIET BESTELD)" }, if needed { rgb_y() } else { rgb_w() });
    }

    fn thief_ride(&mut self, p: Pad, e: Pad, dx: f32, dy: f32) {
        let Some(bi) = self.thief.bike else {
            self.thief.state = TState::Walk;
            return;
        };
        let kind = self.bikes[bi].kind;
        // A: throw it on Sjaak's boat, or ring the bell
        if e.a {
            if self.sjaak_near() {
                let b = &mut self.bikes[bi];
                b.state = BikeState::Delivered;
                let th = &mut self.thief;
                th.toss = Some((kind, b.color, th.x, th.y, self.sjaak.x));
                th.bike = None;
                th.state = TState::Toss;
                th.st_t = 0;
                th.v = 0.0;
                th.face_left = self.sjaak.x < th.x;
                self.sfx.push(Sfx::Toss);
                return;
            }
            self.sfx.push(Sfx::Bell);
            let (x, y) = (self.thief.x, self.thief.y);
            self.make_noise(x, y, 50.0, false);
            let dir = if self.thief.face_left { -1.0 } else { 1.0 };
            for pd in &mut self.peds {
                let ahead = (pd.x - x) * dir;
                if ahead > 0.0 && ahead < 50.0 && (pd.y - y).abs() < 10.0 && pd.state != PState::Fall {
                    pd.vy = if pd.y > y { 0.8 } else { -0.8 };
                    pd.st_t = 0;
                }
            }
        }
        // C: get off and walk away from it
        if e.c {
            self.dismount(false);
            return;
        }
        let th = &mut self.thief;
        let top = kind.def().speed * th.who.ride_mul() * if th.wet > 0 { 0.9 } else { 1.0 };
        if dx != 0.0 {
            let target = dx * top;
            let acc = if target * th.v < 0.0 { 0.12 } else { 0.055 };
            th.v += (target - th.v).clamp(-acc, acc);
            th.face_left = dx < 0.0;
        } else {
            th.v -= th.v.signum() * th.v.abs().min(0.03);
        }
        th.vy = dy * if th.v.abs() > 0.3 { 0.95 } else { 0.6 };
        th.x += th.v;
        if th.x < 6.0 || th.x > self.len - 6.0 {
            th.x = th.x.clamp(6.0, self.len - 6.0);
            th.v = 0.0;
        }
        th.y += th.vy;
        th.moving = th.v.abs() > 0.05;
        th.anim += th.v.abs() * 0.12;
        th.running = false;
        if th.y > DEPTH {
            th.y = DEPTH;
            if p.down {
                th.edge_t += 1;
                if th.edge_t > 14 {
                    th.edge_t = 0;
                    self.plons(true);
                    return;
                }
            }
        } else {
            th.edge_t = 0;
        }
        th.y = th.y.max(8.0);
        let (x, y, v) = (th.x, th.y, th.v);
        let b = &mut self.bikes[bi];
        b.x = x;
        b.y = y;
        b.face_left = th.face_left;
        // knock over pedestrians (and yourself)
        if v.abs() > 1.0 {
            let mut hit = None;
            for (k, pd) in self.peds.iter().enumerate() {
                if pd.state != PState::Fall && (pd.x - x).abs() < 7.0 && (pd.y - y).abs() < 5.0 {
                    hit = Some(k);
                }
            }
            if let Some(k) = hit {
                let pd = &mut self.peds[k];
                pd.state = PState::Fall;
                pd.st_t = 0;
                pd.face_left = v > 0.0;
                self.popup(x, y - 34.0, "AU!", rgb_r());
                self.crash();
                return;
            }
            // and BOA's: a chasing one grabs you, a patrolling one gets very interested
            for k in 0..self.boas.len() {
                let bo = &self.boas[k];
                if bo.ctrl == Ctrl::Cpu && (bo.x - x).abs() < 7.0 && (bo.y - y).abs() < 5.0 && bo.state != BoaState::Ticket {
                    if self.thief.grace > 0 {
                        continue;
                    }
                    if bo.state == BoaState::Chase {
                        self.bust(k);
                    } else if bo.alert() && self.guilty() {
                        // squeezed past a surprised BOA: now it's a chase
                        self.boas[k].sus = 1.0;
                        self.say(k, "HE!!");
                        continue;
                    } else {
                        self.boas[k].sus = self.boas[k].sus.max(0.9);
                        self.say(k, "HE!");
                        self.crash();
                    }
                    return;
                }
            }
        }
        // herons hate bikes
        for h in &mut self.herons {
            if !h.flying && !h.gone && (h.x - x).abs() < 34.0 && v.abs() > 0.8 {
                h.flying = true;
                h.t = 0;
                h.vx = if h.x > x { 1.3 } else { -1.3 };
                self.sfx.push(Sfx::Squawk);
                self.noise.push(Noise { x: h.x, y: h.y, r: 90.0, loud: false });
            }
        }
    }

    /// Off the bike: it stays standing, unlocked.
    fn dismount(&mut self, fell: bool) {
        let th = &mut self.thief;
        if let Some(bi) = th.bike.take() {
            let b = &mut self.bikes[bi];
            b.state = BikeState::Loose;
            b.x = th.x;
            b.y = th.y.clamp(8.0, DEPTH);
            if b.hot > 0 || b.flagged {
                th.hot = 120;
            }
        }
        th.state = if fell { TState::Fall } else { TState::Walk };
        th.st_t = 0;
        th.v = 0.0;
        if !fell {
            th.y = (th.y - 6.0).max(10.0);
        }
    }

    fn crash(&mut self) {
        self.thief.stats.crashes += 1;
        self.sfx.push(Sfx::Clank);
        let (x, y) = (self.thief.x, self.thief.y);
        self.noise.push(Noise { x, y, r: 90.0, loud: false });
        self.fx.push(Fx { kind: FxKind::Dust, x, y, t: 0 });
        self.shake = 6;
        self.dismount(true);
    }

    /// Into the canal. On a bike the bike is gone.
    fn plons(&mut self, riding: bool) {
        let th = &mut self.thief;
        if riding
            && let Some(bi) = th.bike.take() {
                let kind = self.bikes[bi].kind;
                let left = self.bikes.iter().enumerate().filter(|(k, b)| *k != bi && b.kind == kind && matches!(b.state, BikeState::Parked | BikeState::Loose)).count();
                let need = self.order.iter().filter(|o| o.0 == kind && !o.1).count();
                let b = &mut self.bikes[bi];
                if left < need {
                    // the last one of its kind on the order: the city fishes it out and parks it again
                    b.state = BikeState::Parked;
                    b.x = b.home.0;
                    b.y = b.home.1;
                    b.lock = b.lock0;
                    b.progress = 0.0;
                    b.hot = 0;
                    b.flagged = false;
                } else {
                    b.state = BikeState::Sunk;
                    b.y = DEPTH + 8.0;
                }
            }
        th.state = TState::Swim;
        th.st_t = 0;
        th.v = 0.0;
        th.y = DEPTH + 12.0;
        th.stats.swims += 1;
        let x = th.x;
        self.sfx.push(Sfx::Splash);
        self.fx.push(Fx { kind: FxKind::Splash, x, y: DEPTH + 12.0, t: 0 });
        self.banner_thief("PLONS!", rgb_c(), 70);
        self.make_noise(x, DEPTH, 80.0, false);
    }

    fn deliver(&mut self, kind: Kind, color: usize) {
        let on = if let Some(o) = self.order.iter_mut().find(|o| o.0 == kind && !o.1) {
            o.1 = true;
            true
        } else {
            false
        };
        self.thief.delivered.push((kind, color, on));
        self.sjaak.pile.push((kind, color));
        self.sjaak.happy = 90;
        self.sfx.push(Sfx::Cash);
        let v = if on { kind.def().value } else { kind.def().value / 2 };
        let (x, y) = (self.sjaak.x, DEPTH + 4.0);
        self.popup(x, y - 30.0, &format!("+ EUR {}", v), if on { rgb_y() } else { rgb_w() });
        if on && !self.order_done() {
            self.banner_thief("BESTELD!", rgb_g(), 60);
        } else if !on {
            self.popup(x, y - 40.0, "NIET BESTELD...", rgb_w());
        }
    }

    // ---------------------------------------------------------------- BOA's

    fn update_boa(&mut self, i: usize) {
        let (pat, inv, chase) = self.boas[i].speeds();
        let th_pos = (self.thief.x, self.thief.y);
        let len = self.len;
        let swim = matches!(self.thief.state, TState::Swim | TState::Climb);
        let b = &mut self.boas[i];
        b.st_t += 1;
        b.cool = b.cool.saturating_sub(1);
        b.coffee_cool = b.coffee_cool.saturating_sub(1);
        b.whistle_t = b.whistle_t.saturating_sub(1);
        if let Some((_, t)) = &mut b.say {
            *t = t.saturating_sub(1);
            if *t == 0 {
                b.say = None;
            }
        }
        let mut goal: Option<((f32, f32), f32)> = None;
        b.running = false;
        match b.state {
            BoaState::Patrol => {
                let end = if b.face_left { b.x0 } else { b.x1 };
                // wander a little in depth
                let lane = b.lane + (b.st_t as f32 * 0.01 + b.x0 * 0.1).sin() * 5.0;
                goal = Some(((end, lane), pat));
                if (b.x - end).abs() < 2.0 {
                    b.face_left = !b.face_left;
                    b.state = BoaState::Look;
                    b.st_t = 0;
                }
                if b.next_look > 0 {
                    b.next_look -= 1;
                } else {
                    // a glance back over the shoulder
                    b.face_left = !b.face_left;
                    b.state = BoaState::Look;
                    b.st_t = 0;
                    b.next_look = b.look_every;
                }
                if let Some(cx) = b.coffee
                    && b.coffee_cool == 0 && (b.x - cx).abs() < 3.0 {
                        b.state = BoaState::Coffee;
                        b.st_t = 0;
                        b.say = Some(("EVEN KOFFIE...", 90));
                    }
            }
            BoaState::Look => {
                if b.st_t > 90 {
                    // turn back the way they were going
                    if b.x > b.x0 + 4.0 && b.x < b.x1 - 4.0 {
                        b.face_left = !b.face_left;
                    }
                    b.state = BoaState::Patrol;
                    b.st_t = 0;
                }
            }
            BoaState::Coffee => {
                // facing the kiosk, back to the street
                b.face_left = false;
                if b.st_t > 420 {
                    b.state = BoaState::Patrol;
                    b.st_t = 0;
                    b.coffee_cool = 1500;
                }
            }
            BoaState::Investigate => {
                goal = Some((b.target, inv));
                if dist((b.x, b.y), b.target) < 4.0 {
                    b.state = BoaState::Search;
                    b.st_t = 0;
                }
                if b.st_t > 600 {
                    b.state = BoaState::Search;
                    b.st_t = 0;
                }
            }
            BoaState::Chase => {
                b.chase_t += 1;
                b.running = true;
                let mut t = b.target;
                if swim {
                    t.1 = t.1.min(DEPTH - 2.0);
                }
                // first the whistle, then the run: a head start for the thief
                if b.chase_t > CHASE_WHISTLE {
                    goal = Some((t, chase));
                } else {
                    b.whistle_t = b.whistle_t.max(2);
                    b.face_left = th_pos.0 < b.x;
                }
                if swim && (b.x - th_pos.0).abs() < 14.0 && b.lost < 20 {
                    b.state = BoaState::Wait;
                    b.st_t = 0;
                    b.say = Some(("KOM ERUIT!", 120));
                } else if b.lost > 150 {
                    b.state = BoaState::Search;
                    b.st_t = 0;
                    b.say = Some(("WAAR IS IE?", 100));
                    b.sus = 0.5;
                } else if b.kind != BoaKind::Bike && b.chase_t > 60 * 15 {
                    b.state = BoaState::Search;
                    b.st_t = 0;
                    b.say = Some(("PFFF...", 100));
                    b.sus = 0.4;
                }
            }
            BoaState::Wait => {
                // pace along the quay with the swimmer
                if swim {
                    goal = Some(((th_pos.0, DEPTH - 3.0), inv));
                } else {
                    b.state = BoaState::Chase;
                    b.st_t = 0;
                }
                if b.st_t > 360 {
                    b.state = BoaState::Search;
                    b.st_t = 0;
                    b.say = Some(("TE KOUD.", 100));
                    b.sus = 0.3;
                }
            }
            BoaState::Search => {
                if b.st_t.is_multiple_of(45) {
                    b.face_left = !b.face_left;
                }
                if b.st_t > 170 {
                    b.state = BoaState::Patrol;
                    b.st_t = 0;
                    b.chase_t = 0;
                    b.sus = b.sus.min(0.3);
                    b.face_left = b.x > (b.x0 + b.x1) / 2.0;
                }
            }
            BoaState::Ticket => {
                if b.st_t > 150 {
                    b.state = BoaState::Patrol;
                    b.st_t = 0;
                    b.cool = 200;
                    b.chase_t = 0;
                    b.face_left = b.x > (b.x0 + b.x1) / 2.0;
                }
            }
        }
        b.moving = false;
        if let Some(((gx, gy), sp)) = goal {
            let (dx, dy) = (gx - b.x, gy - b.y);
            let d = (dx * dx + dy * dy).sqrt();
            if d > 0.5 {
                let s = sp.min(d);
                b.x += dx / d * s;
                b.y += dy / d * s * 0.8;
                if dx.abs() > 0.3 {
                    b.face_left = dx < 0.0;
                }
                b.moving = true;
                b.anim += s * 0.11;
            }
        }
        b.x = b.x.clamp(4.0, len - 4.0);
        b.y = b.y.clamp(8.0, DEPTH - 1.0);
        // the dog trots along, a bit behind
        if let Some(d) = &mut b.dog {
            let back = if b.face_left { 12.0 } else { -12.0 };
            let (tx, ty) = (b.x + back, (b.y + 5.0).min(DEPTH));
            let (dx, dy) = (tx - d.x, ty - d.y);
            let dd = (dx * dx + dy * dy).sqrt();
            if dd > 2.0 {
                let s = (dd * 0.1).clamp(0.3, 2.2);
                d.x += dx / dd * s;
                d.y += dy / dd * s;
                d.anim += s * 0.2;
                d.face_left = dx < 0.0;
            }
            d.bark = d.bark.saturating_sub(1);
        }
        // catching
        let b = &self.boas[i];
        let close = (b.x - th_pos.0).abs() < 9.0 && (b.y - th_pos.1).abs() < 8.0;
        if close && self.thief.grace == 0 && !swim && self.result.is_none() {
            let catchable = match self.thief.state {
                TState::Busted | TState::Out | TState::Swim | TState::Toss => false,
                TState::Hide => self.thief.hide_seen,
                _ => true,
            };
            // a surprised BOA can't grab someone flying past on a bike
            let fast = self.thief.state == TState::Ride && self.thief.v.abs() > 1.2;
            if catchable {
                match b.state {
                    BoaState::Chase if b.chase_t > CHASE_WHISTLE => self.bust(i),
                    BoaState::Investigate | BoaState::Search if fast && self.guilty() => {
                        self.boas[i].sus = 1.0;
                    }
                    BoaState::Investigate | BoaState::Search if fast => {}
                    BoaState::Investigate | BoaState::Search if self.guilty() => self.bust(i),
                    BoaState::Investigate | BoaState::Search if self.thief.state == TState::Hide => {
                        // found, but nothing to fine: out you come
                        let th = &mut self.thief;
                        th.state = TState::Walk;
                        th.st_t = 0;
                        th.spot = None;
                        th.hide_seen = false;
                        th.y = (th.y - 10.0).clamp(12.0, DEPTH - 4.0);
                        let b = &mut self.boas[i];
                        b.state = BoaState::Search;
                        b.st_t = 0;
                        b.sus = 0.6;
                        b.say = Some(("KOM DAAR MAAR UIT.", 100));
                    }
                    BoaState::Investigate => {
                        let b = &mut self.boas[i];
                        b.state = BoaState::Search;
                        b.st_t = 0;
                        b.say = Some(("HM. NIKS AAN DE HAND.", 100));
                        b.sus = 0.2;
                    }
                    _ => {}
                }
            }
        }
    }

    /// 2P: the BOA player walks, runs, grabs (A) and whistles (C).
    fn update_human_boa(&mut self, i: usize, p: Pad, e: Pad) {
        let len = self.len;
        let b = &mut self.boas[i];
        b.st_t += 1;
        b.cool = b.cool.saturating_sub(1);
        b.whistle_t = b.whistle_t.saturating_sub(1);
        b.grab_t = b.grab_t.saturating_sub(1);
        if let Some((_, t)) = &mut b.say {
            *t = t.saturating_sub(1);
            if *t == 0 {
                b.say = None;
            }
        }
        if b.state == BoaState::Ticket {
            if b.st_t > 150 {
                b.state = BoaState::Patrol;
                b.st_t = 0;
            }
            return;
        }
        let dx = p.right as i32 as f32 - p.left as i32 as f32;
        let dy = p.down as i32 as f32 - p.up as i32 as f32;
        let sp = if p.b { 1.55 } else { 0.95 };
        b.running = p.b && (dx != 0.0 || dy != 0.0);
        b.moving = dx != 0.0 || dy != 0.0;
        b.x = (b.x + dx * sp * if dy != 0.0 { 0.8 } else { 1.0 }).clamp(4.0, len - 4.0);
        b.y = (b.y + dy * sp * 0.75).clamp(8.0, DEPTH - 1.0);
        if dx != 0.0 {
            b.face_left = dx < 0.0;
        }
        if b.moving {
            b.anim += sp * 0.11;
        }
        if e.c && b.cool == 0 {
            b.whistle_t = 40;
            b.cool = 240;
            let (x, y) = (b.x, b.y);
            self.sfx.push(Sfx::Whistle);
            for k in 0..self.boas.len() {
                let o = &mut self.boas[k];
                if k != i && o.ctrl == Ctrl::Cpu && (o.x - x).abs() < 260.0 && !matches!(o.state, BoaState::Chase | BoaState::Ticket) {
                    o.state = BoaState::Investigate;
                    o.st_t = 0;
                    o.target = (x, y);
                    o.say = Some(("IK KOM!", 80));
                }
            }
        }
        if e.a && self.boas[i].grab_t == 0 {
            self.boas[i].grab_t = 30;
            let b = &self.boas[i];
            let th = &self.thief;
            let reach = (b.x - th.x).abs() < 15.0 && (b.y - th.y).abs() < 11.0;
            let catchable = !matches!(th.state, TState::Busted | TState::Out | TState::Swim | TState::Climb | TState::Toss) && th.grace == 0;
            if reach && catchable && self.result.is_none() {
                if self.guilty_for_player() {
                    self.boas[i].fines += 1;
                    self.bust(i);
                } else {
                    self.boas[i].complaints += 1;
                    self.thief.complaints += 1;
                    self.thief.grace = 120;
                    let (x, y) = (self.thief.x, self.thief.y);
                    self.popup(x, y - 34.0, &format!("KLACHT! -{}", COMPLAINT), rgb_r());
                    self.boas[i].say = Some(("OEPS...", 90));
                    self.sfx.push(Sfx::Hey);
                }
            } else {
                self.sfx.push(Sfx::Hey);
                self.boas[i].say = Some(("STAAN BLIJVEN!", 60));
            }
        }
    }

    /// Caught by BOA `i`: a fine, the bike goes back, a short break.
    fn bust(&mut self, i: usize) {
        self.busts.push((self.boas[i].state, self.thief.state));
        let th = &mut self.thief;
        th.fines += 1;
        th.state = TState::Busted;
        th.st_t = 0;
        th.wanted = 0;
        th.hot = 0;
        th.hide_seen = false;
        th.spot = None;
        th.v = 0.0;
        let back = th.bike.take().or(th.pick.take());
        if let Some(bi) = back {
            let b = &mut self.bikes[bi];
            if matches!(b.state, BikeState::Ridden | BikeState::Loose | BikeState::Parked) {
                b.state = BikeState::Parked;
                b.x = b.home.0;
                b.y = b.home.1;
                b.lock = b.lock0;
                b.progress = 0.0;
                b.hot = 0;
                b.flagged = false;
            }
        }
        let (x, y) = (self.thief.x, self.thief.y);
        let b = &mut self.boas[i];
        b.state = BoaState::Ticket;
        b.st_t = 0;
        b.face_left = x < b.x;
        b.x = x + if b.face_left { 12.0 } else { -12.0 };
        b.say = None;
        for o in &mut self.boas {
            o.sus = 0.0;
            if o.ctrl == Ctrl::Cpu && matches!(o.state, BoaState::Chase | BoaState::Investigate | BoaState::Wait | BoaState::Search) {
                o.state = BoaState::Patrol;
                o.st_t = 0;
                o.chase_t = 0;
            }
        }
        self.sfx.push(Sfx::Busted);
        let left = MAX_FINES.saturating_sub(self.thief.fines);
        self.banner_thief("BEKEURING!", rgb_r(), 90);
        self.popup(x, y - 40.0, &format!("BOETE EUR {}", FINE), rgb_r());
        if left > 0 && self.thief.ctrl != Ctrl::Cpu {
            self.popup(x, y - 52.0, if left == 1 { "NOG 1 KANS!" } else { "NOG 2 KANSEN" }, rgb_w());
        }
    }

    /// What every CPU BOA sees this frame, and what they make of it.
    fn update_sight(&mut self) {
        let visible = self.thief_visible() && self.result.is_none();
        let act = self.activity();
        let (tx, ty) = (self.thief.x, self.thief.y);
        let who = self.thief.who;
        // blending into a crowd
        let near = self.peds.iter().filter(|p| (p.x - tx).abs() < 16.0 && (p.y - ty).abs() < 12.0 && p.state != PState::Fall).count();
        let crowd_mul = if near >= 2 { 0.7 } else { 1.0 };
        let grace = self.thief.grace > 0;
        let wanted = self.thief.wanted > 0;
        let mut seen_any = false;
        let mut start_chase = None;
        for i in 0..self.boas.len() {
            if self.boas[i].ctrl != Ctrl::Cpu || self.boas[i].state == BoaState::Ticket {
                continue;
            }
            let s = if visible { self.sight(&self.boas[i], tx, ty) } else { 0.0 };
            // the chief's dog sniffs out hiders
            if self.thief.state == TState::Hide && !self.thief.hide_seen
                && let Some(d) = &mut self.boas[i].dog
                    && (d.x - tx).abs() < 26.0 && (d.y - ty).abs() < 18.0 && d.bark == 0 {
                        d.bark = 90;
                        self.thief.hide_seen = true;
                        self.sfx.push(Sfx::Hey);
                        let b = &mut self.boas[i];
                        b.state = BoaState::Investigate;
                        b.st_t = 0;
                        b.target = (tx, (ty - 6.0).max(8.0));
                        b.say = Some(("BRUNO? WAT RUIK JE?", 100));
                    }
            let b = &mut self.boas[i];
            if s > 0.0 && !grace {
                seen_any = true;
                let mut rate = act;
                if wanted {
                    rate = rate.max(1.0 / 12.0);
                }
                rate *= who.sus_mul() * crowd_mul;
                b.sus = (b.sus + rate * s).min(1.2);
                if b.state == BoaState::Chase {
                    b.target = (tx, ty);
                    b.lost = 0;
                }
                if matches!(b.state, BoaState::Wait) {
                    b.lost = 0;
                }
            } else {
                if b.state == BoaState::Chase {
                    b.lost += 1;
                }
                if !matches!(b.state, BoaState::Chase | BoaState::Wait) {
                    b.sus = (b.sus - 1.0 / 400.0).max(0.0);
                }
            }
            if self.thief.state == TState::Casual && s > 0.0 && !wanted {
                b.sus = (b.sus - 1.0 / 200.0).max(0.0);
            }
            if b.cool > 0 {
                b.sus = b.sus.min(0.3);
                continue;
            }
            if b.sus >= 1.0 && s > 0.0 && !matches!(b.state, BoaState::Chase | BoaState::Wait) {
                start_chase = Some(i);
            } else if b.sus >= 0.35 && matches!(b.state, BoaState::Patrol | BoaState::Look | BoaState::Coffee | BoaState::Search) && s > 0.0 {
                b.state = BoaState::Investigate;
                b.st_t = 0;
                b.target = (tx, ty);
                b.say = Some(("HM?", 60));
            } else if b.state == BoaState::Investigate && s > 0.0 {
                b.target = (tx, ty.min(DEPTH - 1.0));
            }
        }
        if seen_any {
            self.thief.stats.seen += 1;
        }
        if let Some(i) = start_chase {
            self.start_chase(i);
        }
        // whoever chases and sees you keeps your face in mind
        if self.boas.iter().any(|b| b.state == BoaState::Chase && b.lost == 0) {
            self.thief.wanted = 420;
            if let Some(bi) = self.thief.bike {
                self.bikes[bi].flagged = true;
            }
        }
    }

    fn start_chase(&mut self, i: usize) {
        let (tx, ty) = (self.thief.x, self.thief.y);
        let heckles = ["HE! JIJ DAAR!", "DAT IS NIET JOUW FIETS!", "STAAN BLIJVEN!", "IK HEB JE GEZIEN HOOR!"];
        let boss = self.def.boss.ends_with(self.boas[i].name);
        let h = if boss { self.def.heckles[(self.t as usize / 7) % self.def.heckles.len()] } else { heckles[(self.t as usize / 7 + i) % heckles.len()] };
        let b = &mut self.boas[i];
        b.state = BoaState::Chase;
        b.st_t = 0;
        b.chase_t = 0;
        b.lost = 0;
        b.target = (tx, ty);
        b.whistle_t = 40;
        b.say = Some((h, 110));
        let bx = b.x;
        // a new chase only when nobody was after you already
        if !self.boas.iter().enumerate().any(|(k, o)| k != i && matches!(o.state, BoaState::Chase | BoaState::Wait)) && self.thief.wanted == 0 {
            self.thief.stats.chases += 1;
        }
        self.thief.wanted = 420;
        self.sfx.push(Sfx::Whistle);
        if let Some(bi) = self.thief.bike {
            self.bikes[bi].flagged = true;
        }
        // colleagues nearby come running
        for k in 0..self.boas.len() {
            let o = &mut self.boas[k];
            if k != i && o.ctrl == Ctrl::Cpu && (o.x - bx).abs() < 240.0 && !matches!(o.state, BoaState::Chase | BoaState::Ticket | BoaState::Wait) {
                o.state = BoaState::Investigate;
                o.st_t = 0;
                o.target = (tx, ty.min(DEPTH - 1.0));
                o.sus = o.sus.max(0.7);
                o.say = Some(("IK KOM!", 70));
            }
        }
    }

    /// Everyone near a noise comes to look.
    fn process_noise(&mut self) {
        let noise = std::mem::take(&mut self.noise);
        for n in noise {
            for b in &mut self.boas {
                if b.ctrl != Ctrl::Cpu || matches!(b.state, BoaState::Chase | BoaState::Ticket | BoaState::Wait) || b.cool > 0 {
                    continue;
                }
                if dist((b.x, b.y), (n.x, n.y)) < n.r {
                    let first = b.state != BoaState::Investigate;
                    b.sus = (b.sus + if n.loud { 0.55 } else { 0.06 }).min(0.95);
                    if n.loud || b.sus > 0.3 {
                        b.state = BoaState::Investigate;
                        b.target = (n.x, n.y.min(DEPTH - 1.0));
                        if first {
                            b.st_t = 0;
                            b.face_left = n.x < b.x;
                            b.say = Some((if n.loud { "WAT WAS DAT?!" } else { "HM?" }, 80));
                        }
                    } else if first {
                        // a glance toward the sound
                        b.face_left = n.x < b.x;
                    }
                }
            }
        }
    }

    // ---------------------------------------------------------------- ambient

    /// A new place for Sjaak to moor: a good way off, away from BOA's, and
    /// a clear bit of quay (no parked bikes, trees or the krul).
    fn mooring(&mut self) -> f32 {
        let mut best = (self.sjaak.x, -1.0f32);
        for _ in 0..16 {
            let x = self.rng.range(90.0, self.len - 90.0);
            let far = (x - self.sjaak.x).abs().min(260.0);
            let boa = self.boas.iter().map(|b| (b.x - x).abs()).fold(999.0f32, f32::min).min(120.0);
            let busy = self.bikes.iter().any(|b| b.state == BikeState::Parked && b.y > 40.0 && (b.x - x).abs() < 16.0)
                || self.spots.iter().any(|s| s.2 > 40.0 && (s.1 - x).abs() < 22.0);
            let score = far + boa * 1.5 - if busy { 200.0 } else { 0.0 };
            if score > best.1 {
                best = (x, score);
            }
        }
        best.0
    }

    fn update_ambient(&mut self) {
        let len = self.len;
        // pedestrians
        let (tx, ty) = (self.thief.x, self.thief.y);
        for p in &mut self.peds {
            p.st_t += 1;
            match p.state {
                PState::Walk => {
                    p.x += p.vx;
                    if p.vy != 0.0 {
                        p.y += p.vy;
                        if p.st_t > 20 {
                            p.vy = 0.0;
                        }
                    }
                    p.y = p.y.clamp(12.0, DEPTH - 6.0);
                    p.anim += p.vx.abs() * 0.11;
                    if p.x < 20.0 || p.x > len - 20.0 {
                        p.vx = -p.vx;
                        p.x = p.x.clamp(20.0, len - 20.0);
                    }
                    p.face_left = p.vx < 0.0;
                    if p.st_t > 400 && self.rng.int(300) == 0 && !p.suitcase {
                        p.state = PState::Photo;
                        p.st_t = 0;
                    }
                }
                PState::Photo => {
                    if p.st_t > 120 {
                        p.state = PState::Walk;
                        p.st_t = 0;
                    }
                }
                PState::Fall => {
                    if p.st_t > 100 {
                        p.state = if p.crowd { PState::Dance } else { PState::Walk };
                        p.st_t = 0;
                    }
                }
                PState::Dance => {
                    p.anim += 0.05;
                    // drift around the party spot, and make room for a rider
                    let want = p.home + (p.anim * 0.3).sin() * 10.0;
                    p.x += (want - p.x) * 0.02;
                    if (p.x - tx).abs() < 10.0 && (p.y - ty).abs() < 6.0 {
                        p.y += if p.y > ty { 0.4 } else { -0.4 };
                    }
                    p.y = p.y.clamp(10.0, DEPTH - 6.0);
                }
            }
        }
        // Sjaak putters between moorings, and doesn't wait for a BOA
        let boa_near = self.boas.iter().any(|b| (b.x - self.sjaak.x).abs() < 30.0 && b.y > DEPTH - 24.0 && b.state != BoaState::Ticket);
        if boa_near && self.sjaak.wait > 1 && !self.sjaak.moving && self.thief.state != TState::Toss {
            self.sjaak.wait = 1;
            let x = self.sjaak.x;
            self.popup(x, DEPTH - 24.0, "POLITIE! WEG HIER!", rgb_c());
        }
        if self.sjaak.wait == 1 {
            let t = self.mooring();
            self.sjaak.target = t;
        }
        let sj = &mut self.sjaak;
        sj.happy = sj.happy.saturating_sub(1);
        if sj.wait > 0 {
            sj.wait -= 1;
            sj.moving = false;
        } else {
            let d = sj.target - sj.x;
            sj.moving = true;
            sj.face_left = d < 0.0;
            sj.x += d.signum() * d.abs().min(0.45);
            if d.abs() < 0.5 {
                sj.wait = 420 + self.rng.int(300) as u32;
            }
        }
        // the tour boat, party boats and the crane boat
        if self.t >= self.tour_at && !self.boats.iter().any(|b| b.kind == BoatKind::Tour) {
            let left = self.rng.int(2) == 0;
            self.boats.push(Boat { kind: BoatKind::Tour, x: if left { len + 80.0 } else { -80.0 }, vx: if left { -0.55 } else { 0.55 }, t: 0, stop: 0.0, done: false });
            self.tour_at = self.t + 1500 + self.rng.int(900) as u32;
        }
        if self.t == self.crane_at {
            let stop = (self.thief.x + self.rng.range(-120.0, 120.0)).clamp(100.0, len - 100.0);
            self.boats.push(Boat { kind: BoatKind::Crane, x: -60.0, vx: 0.6, t: 0, stop, done: false });
        }
        let mut fished = None;
        for b in &mut self.boats {
            b.t += 1;
            match b.kind {
                BoatKind::Tour => b.x += b.vx,
                BoatKind::Party => {
                    b.x += b.vx;
                    if b.x < 40.0 || b.x > len - 40.0 {
                        b.vx = -b.vx;
                    }
                }
                BoatKind::Crane => {
                    if !b.done && (b.x - b.stop).abs() < 1.0 {
                        if b.t > 20000 {
                            b.t = 0;
                        }
                        b.vx = 0.0;
                        if b.t == 250 {
                            fished = Some(b.x);
                        }
                        if b.t > 400 {
                            b.done = true;
                            b.vx = 0.6;
                        }
                    } else {
                        if !b.done && b.x < b.stop && b.x + b.vx >= b.stop {
                            b.x = b.stop;
                            b.t = 0;
                        }
                        b.x += b.vx;
                    }
                }
            }
        }
        self.boats.retain(|b| b.x > -120.0 && b.x < len + 120.0);
        if let Some(x) = fished {
            self.bikes.push(Bike {
                kind: Kind::Wreck,
                x,
                y: DEPTH - 2.0,
                home: (x, DEPTH - 2.0),
                face_left: false,
                color: 3,
                lock: Lock::None,
                lock0: Lock::None,
                progress: 0.0,
                state: BikeState::Loose,
                hot: 0,
                flagged: false,
                fished: true,
            });
            self.popup(x, DEPTH - 30.0, "VERS GEVIST!", rgb_c());
            self.sfx.push(Sfx::Splash);
        }
        // herons
        for h in &mut self.herons {
            h.t += 1;
            if h.flying {
                h.x += h.vx;
                h.z += 0.7;
                if h.z > 140.0 {
                    h.gone = true;
                    h.flying = false;
                    h.t = 0;
                }
            } else if h.gone && h.t > 1800 {
                h.gone = false;
                h.z = 0.0;
                h.x = self.rng.range(120.0, len - 60.0);
            }
        }
        // a heron startled by a runner
        let running = self.thief.running && self.thief.moving;
        for k in 0..self.herons.len() {
            let h = &self.herons[k];
            if running && !h.flying && !h.gone && (h.x - tx).abs() < 22.0 && (h.y - ty).abs() < 14.0 {
                let h = &mut self.herons[k];
                h.flying = true;
                h.t = 0;
                h.vx = if h.x > tx { 1.3 } else { -1.3 };
                let (x, y) = (h.x, h.y);
                self.sfx.push(Sfx::Squawk);
                self.noise.push(Noise { x, y, r: 90.0, loud: false });
            }
        }
    }
}

// a few colors the simulation hands to popups and banners
pub fn rgb_w() -> u32 {
    crate::engine::gfx::rgb(248, 248, 248)
}
pub fn rgb_y() -> u32 {
    crate::engine::gfx::rgb(248, 216, 48)
}
pub fn rgb_r() -> u32 {
    crate::engine::gfx::rgb(232, 64, 48)
}
pub fn rgb_g() -> u32 {
    crate::engine::gfx::rgb(120, 216, 88)
}
pub fn rgb_c() -> u32 {
    crate::engine::gfx::rgb(96, 200, 248)
}
