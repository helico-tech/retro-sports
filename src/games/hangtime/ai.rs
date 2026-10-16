//! CPU pilots. A brain looks at the world like a pilot would (storks,
//! clouds, the slope, its own variometer) and answers with a `Pad`, so CPU
//! pilots fly by exactly the same rules as you. The same brain with other
//! settings is the probe's scripted "human".

use super::course::LEN;
use super::flight::{Race, State, TRIM};
use super::pilots::PILOTS;
use crate::engine::input::Pad;
use crate::engine::rng::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Style {
    /// plans height for the next gate and climbs only what it needs
    Normal,
    /// never climbs: nose down, tucked, straight for the field
    Dive,
    /// climbs every bit of lift to the top before moving on
    Cautious,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Goal {
    Glide,
    /// weave around this x until high enough
    Climb { x: f32, until: f32 },
}

#[derive(Clone, Debug)]
pub struct Brain {
    pub skill: f32,
    pub style: Style,
    rng: Rng,
    t: u32,
    step: i8,
    next_step: u32,
    launch_at: f32,
    flare_h: f32,
    goal: Goal,
    scan_t: u32,
    weak: u32,
    /// how far off the brain is when it judges where lift is
    err: f32,
    weave: f32,
    pressed_turn: bool,
    flare_slop: f32,
    burning: bool,
}

impl Brain {
    pub fn new(skill: f32, style: Style, seed: u64) -> Self {
        let mut rng = Rng(seed | 1);
        let sk = skill.clamp(0.0, 1.0);
        let noise = (1.0 - sk) * rng.range(-18.0, 12.0);
        let launch_at = -12.0 + noise;
        let flare_h = 5.5 + (1.0 - sk) * rng.range(-5.0, 9.0);
        Brain {
            skill: sk,
            style,
            rng,
            t: 0,
            step: 1,
            next_step: 0,
            launch_at,
            flare_h,
            goal: Goal::Glide,
            scan_t: 0,
            weak: 0,
            err: 0.0,
            weave: 22.0,
            pressed_turn: false,
            flare_slop: (1.0 - sk) * 10.0,
            burning: false,
        }
    }

    pub fn think(&mut self, race: &Race, i: usize) -> Pad {
        self.t += 1;
        let p = &race.pilots[i];
        let mut pad = Pad::default();
        match p.state {
            State::Run | State::Hop(_) => {
                // alternate left and right, faster with more skill
                if self.t >= self.next_step {
                    self.step = -self.step;
                    if self.step < 0 {
                        pad.left = true;
                    } else {
                        pad.right = true;
                    }
                    let period = 3.0 + (1.0 - self.skill) * 4.0 + self.rng.range(0.0, 2.0 * (1.0 - self.skill));
                    self.next_step = self.t + period as u32;
                }
                let l = race.course.launch;
                if matches!(p.state, State::Run) && p.x >= l.x1 + self.launch_at {
                    pad.b = true;
                }
            }
            State::Fly => self.fly(race, i, &mut pad),
            _ => {}
        }
        // turns are taps: release between presses
        if pad.left || pad.right {
            if self.pressed_turn && !matches!(p.state, State::Run | State::Hop(_)) {
                pad.left = false;
                pad.right = false;
                self.pressed_turn = false;
            } else {
                self.pressed_turn = true;
            }
        } else {
            self.pressed_turn = false;
        }
        pad
    }

    fn fly(&mut self, race: &Race, i: usize, pad: &mut Pad) {
        let p = &race.pilots[i];
        let c = &race.course;
        let d = &PILOTS[p.def];
        let (x, y) = (p.x, p.y);
        let agl = p.agl(c);
        let here = race.air(p.def, x, y);
        let glide_est = 7.0 + self.skill * 1.5;

        // the next gate still to fly through
        let next = c.gates.iter().enumerate().find(|(k, g)| p.gates & (1 << k) == 0 && g.x > x - 6.0).map(|(_, g)| *g);
        let (tx, need) = match next {
            Some(g) => {
                let dist = (g.x - x).max(0.0);
                let aim = g.lo + (g.hi - g.lo) * (0.45 + 0.1 * self.skill);
                (g.x, aim + dist / glide_est)
            }
            None => {
                let dist = (c.target - x).abs();
                (c.target, c.ground_at(c.target) + 35.0 + dist / glide_est)
            }
        };
        // never plan to fly into a hill
        let mut terrain = 0.0f32;
        let mut sx = x;
        while sx < tx.min(x + 500.0) {
            let dist = sx - x;
            terrain = terrain.max(c.ground_at(sx) + 26.0 + dist / glide_est);
            sx += 20.0;
        }
        let need = need.max(terrain);
        let final_glide = next.is_none();

        // scan for lift now and then
        if self.t >= self.scan_t {
            self.scan_t = self.t + 18 + ((1.0 - self.skill) * 20.0) as u32;
            self.err = (1.0 - self.skill) * self.rng.range(-22.0, 22.0);
            self.weave = 16.0 + (1.0 - self.skill) * 18.0 + self.rng.range(0.0, 8.0);
            let climbing = matches!(self.goal, Goal::Climb { .. });
            let wants = match self.style {
                Style::Dive => false,
                Style::Cautious => y < c.thermals.iter().map(|t| t.top).fold(0.0, f32::max) - 40.0,
                Style::Normal => y < need + if climbing { 12.0 } else { -4.0 },
            };
            if wants && !(final_glide && agl < 60.0) {
                let fwd = 140.0 + 320.0 * self.skill;
                let back = 50.0 + 130.0 * self.skill;
                let center = match self.goal {
                    Goal::Climb { x: cx, .. } => cx,
                    Goal::Glide => x,
                };
                let mut best: Option<(f32, f32)> = None;
                let mut dx = -back;
                while dx <= fwd {
                    let sx = (x + dx).clamp(20.0, LEN as f32 - 20.0);
                    // lift behind a hill we cannot fly over does not count
                    let blocked = (1..=4).any(|k| c.ground_at(x + dx * k as f32 / 4.0) > y - 14.0);
                    let l = if blocked { -1.0 } else { race.air(p.def, sx, y) };
                    // prefer lift ahead, close by, and near what we are already working
                    let cost = dx.abs() * 0.00035 + if dx < 0.0 { 0.04 } else { 0.0 } + if climbing { (sx - center).abs() * 0.0006 } else { 0.0 };
                    let score = l - cost;
                    if l > 0.14 && best.map(|b| score > b.1).unwrap_or(true) {
                        best = Some((sx, score));
                    }
                    dx += 12.0;
                }
                match best {
                    Some((bx, _)) => {
                        let until = match self.style {
                            Style::Cautious => 1.0e6,
                            _ => need + 14.0 + self.skill * 10.0,
                        };
                        self.goal = Goal::Climb { x: bx + self.err, until };
                    }
                    None => self.goal = Goal::Glide,
                }
            } else if !wants {
                self.goal = Goal::Glide;
            }
        }

        // topping out: lift too weak to bother
        if let Goal::Climb { x: cx, until } = self.goal {
            if (x - cx).abs() < 40.0 && race.air(p.def, cx, y) < 0.10 {
                self.weak += 1;
            } else {
                self.weak = 0;
            }
            if y > until || self.weak > 90 {
                self.goal = Goal::Glide;
                self.weak = 0;
                self.scan_t = self.t + 90;
            }
        }

        let ahead = |dir: f32| -> f32 { c.ground_at(x + dir * 50.0).max(c.ground_at(x + dir * 25.0)) };
        let mut dir_want = p.face;
        match self.goal {
            Goal::Climb { x: cx, .. } => {
                // weave through the lift
                if (x - cx) * p.face > self.weave {
                    dir_want = -p.face;
                }
            }
            Goal::Glide => {
                let goal_x = if final_glide { self.final_aim(race, i) } else { tx };
                if (goal_x - x) * p.face < -8.0 {
                    dir_want = -p.face;
                }
            }
        }
        // do not fly into the hill
        if ahead(p.face) > y - 12.0 && ahead(-p.face) < y - 12.0 {
            dir_want = -p.face;
        }
        if dir_want != p.face && p.turn == 0 {
            if dir_want < 0.0 {
                pad.left = true;
            } else {
                pad.right = true;
            }
        }

        // speed to fly
        let near_stall = p.v < d.stall_speed() + 0.07;
        match self.style {
            Style::Dive => {
                pad.down = true;
                pad.c = agl > 50.0;
            }
            _ => {
                let gliding = matches!(self.goal, Goal::Glide);
                if gliding && !final_glide {
                    if here < -0.06 && self.skill > 0.4 {
                        pad.down = true;
                    }
                    if y > need + 70.0 && self.skill > 0.45 && agl > 80.0 {
                        pad.c = true;
                    }
                    // too high for the gate: dive into it
                    if let Some(g) = next {
                        let dist = g.x - x;
                        if dist > 0.0 && dist < 220.0 && y > g.hi - 14.0 - dist / glide_est {
                            pad.down = true;
                            pad.c = y > g.hi + 40.0;
                        }
                    }
                }
            }
        }
        if final_glide && self.style != Style::Dive {
            if self.burning && matches!(self.goal, Goal::Glide) {
                pad.down = true;
                pad.c = agl > 110.0 && self.skill > 0.3;
            }
            self.approach(race, i, pad);
        }
        if near_stall {
            pad.up = false;
            pad.down = true;
        }

        // flare
        let flare_at = (self.flare_h + if self.style == Style::Dive { 2.0 } else { 0.0 }).max(1.5);
        if agl < flare_at && agl < 30.0 && (final_glide || agl < flare_at) {
            pad.b = true;
        }
    }

    /// Where to steer on final glide: straight for the field, then S-turns
    /// short of the target to burn off spare height.
    fn final_aim(&mut self, race: &Race, i: usize) -> f32 {
        let p = &race.pilots[i];
        let c = &race.course;
        let agl = p.y - c.ground_at(c.target);
        // where the glide would touch down from here, heading right
        let ratio = (TRIM + c.wind) / 0.085;
        let land = p.x + agl * ratio;
        let box_r = c.target - 50.0 - (1.0 - self.skill) * 30.0;
        let box_l = box_r - 170.0;
        self.burning = false;
        if p.x < box_l || land < c.target + 10.0 {
            return c.target + 40.0;
        }
        self.burning = agl > 60.0;
        // too high: weave in the box
        if p.face > 0.0 { if p.x > box_r { box_l } else { box_r + 20.0 } } else if p.x < box_l { box_r } else { box_l - 20.0 }
    }

    fn approach(&mut self, race: &Race, i: usize, pad: &mut Pad) {
        let p = &race.pilots[i];
        let c = &race.course;
        let agl = p.agl(c);
        if agl > 45.0 || p.face < 0.0 {
            return;
        }
        // predicted touchdown point at the current glide
        let sink = (-p.vario).max(0.05);
        let gs = p.v + c.wind;
        let land = p.x + agl / sink * gs + self.flare_slop * 0.5;
        if land > c.target + 6.0 {
            pad.down = agl > 12.0;
        } else if land < c.target - 10.0 && agl > 10.0 && self.skill > 0.3 {
            pad.up = (self.t / 6) % 3 == 0;
        }
    }
}
