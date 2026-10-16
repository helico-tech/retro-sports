//! CPU pilots. A brain looks at the world like a pilot would (storks,
//! clouds, the slope, its own variometer) and answers with a `Pad`, so CPU
//! pilots fly by exactly the same rules as you. The same brain with other
//! settings is the probe's scripted "human".
//!
//! Skill (0..1) changes how far it looks for lift and how well it judges
//! it, how tightly it weaves in a thermal, whether it speeds up in sink,
//! how well it aims at the hoops and the target, and its flare timing.

use super::course::{Gate, LEN};
use super::flight::{polar_sink, Race, State, ENERGY, TRIM};
use super::pilots::PILOTS;
use crate::engine::input::Pad;
use crate::engine::rng::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Style {
    /// plans height for the next gate and climbs only what it needs
    Normal,
    /// never climbs: nose down, tucked, straight for the field
    Dive,
    /// climbs high in every thermal, never speeds up, lands carefully
    Cautious,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Goal {
    Glide,
    /// weave around this x until high enough
    Climb { x: f32, until: f32 },
    /// too high for the next hoop: fly away from it and lose height
    Burn,
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
    gate_k: Option<usize>,
    gate_aim: f32,
    land_err: f32,
    final_rolled: bool,
    in_final: bool,
    /// (frame, height) when the current climb check started
    climb_ref: (u32, f32),
}

impl Brain {
    pub fn new(skill: f32, style: Style, seed: u64) -> Self {
        let mut rng = Rng(seed | 1);
        // below 0: rookies who make even more mistakes
        let sk = skill.clamp(-0.5, 1.0);
        let launch_at = -11.0 + (1.0 - sk) * rng.range(-16.0, 13.0);
        let flare_h = 5.0 + (1.0 - sk) * rng.range(-4.0, 8.0);
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
            gate_k: None,
            gate_aim: 0.5,
            land_err: 0.0,
            final_rolled: false,
            in_final: false,
            climb_ref: (0, 0.0),
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
                if matches!(p.state, State::Run) && p.x >= l.x1 + self.launch_at && self.t % 2 == 0 {
                    pad.b = true;
                }
                return pad;
            }
            State::Fly => self.fly(race, i, &mut pad),
            _ => {}
        }
        // turns are taps: release between presses
        if pad.left || pad.right {
            if self.pressed_turn {
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

    /// Ground glide ratio facing right at a speed.
    fn glide(&self, race: &Race, def: usize, v: f32) -> f32 {
        (v + race.course.wind) / (polar_sink(v) * PILOTS[def].sink_mul())
    }

    fn fly(&mut self, race: &Race, i: usize, pad: &mut Pad) {
        let p = &race.pilots[i];
        let c = &race.course;
        let d = &PILOTS[p.def];
        let (x, y) = (p.x, p.y);
        let agl = p.agl(c);
        let here = race.air(p.def, x, y);
        let gr = self.glide(race, p.def, TRIM) * (0.78 + 0.1 * self.skill);
        let gr_fast = self.glide(race, p.def, d.tuck_speed());

        // the next hoop still to fly through
        let next_k = c.gates.iter().enumerate().position(|(k, g)| p.gates & (1 << k) == 0 && g.x > x - 6.0);
        if next_k != self.gate_k {
            self.gate_k = next_k;
            // weaker pilots misjudge how high they need to be
            self.gate_aim = 0.5 + (1.0 - self.skill) * self.rng.range(-0.75, 0.55) + self.rng.range(-0.06, 0.06);
        }
        let next: Option<Gate> = next_k.map(|k| c.gates[k]);
        let (tx, need) = match next {
            Some(g) => {
                let dist = (g.x - x).max(0.0);
                (g.x, g.lo + (g.hi - g.lo) * self.gate_aim + dist / gr)
            }
            None => {
                let dist = (c.target - x).abs();
                (c.target, c.ground_at(c.target) + 30.0 + dist / gr)
            }
        };
        // never plan to fly into a hill
        let mut terrain = 0.0f32;
        let mut sx = x;
        while sx < tx.min(x + 500.0) {
            terrain = terrain.max(c.ground_at(sx) + 26.0 + (sx - x) / gr);
            sx += 20.0;
        }
        let need = need.max(terrain);
        // final glide: latched once high enough after the last hoop
        if next.is_none() && (y >= need - 15.0 || x > c.field.0 - 120.0) {
            self.in_final = true;
        }
        if next.is_some() || (y < need - 45.0 && x < c.field.0 - 120.0) {
            self.in_final = false;
        }
        let final_mode = self.in_final;

        // look for lift now and then
        if self.t >= self.scan_t && !final_mode {
            self.scan(race, i, need);
        }
        if final_mode && matches!(self.goal, Goal::Climb { .. }) && x > c.field.0 - 120.0 {
            self.goal = Goal::Glide;
        }

        // topping out: lift too weak to bother, or no height gained lately
        if let Goal::Climb { x: cx, until } = self.goal {
            if (x - cx).abs() < 40.0 && race.air(p.def, cx, y) < 0.10 {
                self.weak += 1;
            } else {
                self.weak = 0;
            }
            if self.t > self.climb_ref.0 + 240 {
                if y - self.climb_ref.1 < 10.0 {
                    self.weak = 10_000;
                }
                self.climb_ref = (self.t, y);
            }
            if y > until || self.weak as f32 > 60.0 + (1.0 - self.skill) * 90.0 {
                self.goal = Goal::Glide;
                self.weak = 0;
                self.scan_t = self.t + 90;
            }
        }

        // way too high for the next hoop: burn height before it
        if let Some(g) = next {
            let dist = g.x - x;
            if self.style != Style::Dive && matches!(self.goal, Goal::Glide) && dist < 320.0 && y > g.hi - 6.0 + dist / gr_fast.max(4.0) {
                self.goal = Goal::Burn;
            }
            if self.goal == Goal::Burn {
                let back = g.x - x;
                if y <= g.lo + (g.hi - g.lo) * self.gate_aim + back / gr * 0.9 || back > 420.0 || agl < 60.0 {
                    self.goal = Goal::Glide;
                }
            }
        } else if self.goal == Goal::Burn {
            self.goal = Goal::Glide;
        }

        let ahead = |dir: f32| -> f32 { c.ground_at(x + dir * 50.0).max(c.ground_at(x + dir * 25.0)) };
        let mut dir_want = p.face;
        match self.goal {
            Goal::Climb { x: cx, .. } => {
                if (x - cx) * p.face > self.weave {
                    dir_want = -p.face;
                }
            }
            // burn height away from the hoop, but first get out of any lift
            Goal::Burn => dir_want = if here > 0.08 { 1.0 } else { -1.0 },
            Goal::Glide => {
                if final_mode {
                    dir_want = self.final_dir(race, i);
                } else if (tx - x) * p.face < -8.0 {
                    dir_want = -p.face;
                }
            }
        }
        // do not fly into the hill, nor back into the storm
        if ahead(p.face) > y - 12.0 && ahead(-p.face) < y - 12.0 {
            dir_want = -p.face;
        }
        if let Some(st) = c.storm {
            let toward = (st.x - x) * dir_want > 0.0;
            if toward && (st.x - x).abs() < st.w + 90.0 && y > st.base - 140.0 && (p.storm_cool > 0 || self.skill > 0.5) {
                dir_want = -dir_want;
            }
        }
        if dir_want != p.face && p.turn == 0 {
            if dir_want < 0.0 {
                pad.left = true;
            } else {
                pad.right = true;
            }
        }

        // speed to fly
        match self.style {
            Style::Dive => {
                pad.down = true;
                pad.c = agl > 50.0;
            }
            _ => match self.goal {
                Goal::Burn => pad.down = true,
                Goal::Climb { .. } => {
                    // over-eager pulling in lift: slow, sometimes a stall
                    if self.skill < 0.45 && (self.t / 7) % 10 < ((0.45 - self.skill) * 14.0) as u32 {
                        pad.up = true;
                    }
                }
                Goal::Glide if final_mode => self.final_pitch(race, i, pad),
                Goal::Glide => {
                    // weaker pilots dawdle on the glides, bar pushed out
                    if self.skill < 0.5 && (self.t / 5) % 10 < ((0.5 - self.skill) * 10.0) as u32 {
                        pad.up = true;
                    }
                    if here < -0.06 && self.skill > 0.35 && self.style == Style::Normal {
                        pad.down = true;
                    }
                    let spare = y - need;
                    if self.style == Style::Normal && spare > 70.0 - self.skill * 30.0 && self.skill > 0.4 && agl > 70.0 {
                        pad.c = true;
                    }
                    if let Some(g) = next {
                        let dist = g.x - x;
                        if dist > 0.0 && dist < 320.0 && y > g.hi - 12.0 - dist / gr {
                            pad.down = true;
                            pad.c = y > g.hi + 30.0;
                        }
                    }
                }
            },
        }
        // the stall warning: good pilots react early, weak ones late
        if p.v < d.stall_speed() + 0.01 + 0.07 * self.skill {
            pad.up = false;
            pad.down = true;
        }

        // flare
        if agl < self.flare_h.max(1.5) && (final_mode || agl < 8.0) {
            pad.b = true;
            pad.c = false;
        }
    }

    fn scan(&mut self, race: &Race, i: usize, need: f32) {
        let p = &race.pilots[i];
        let c = &race.course;
        let (x, y) = (p.x, p.y);
        let sk = self.skill;
        self.scan_t = self.t + 14 + ((1.0 - sk) * 30.0) as u32;
        self.err = (1.0 - sk) * self.rng.range(-24.0, 24.0);
        self.weave = 14.0 + (1.0 - sk) * 32.0 + self.rng.range(0.0, 8.0);
        let climbing = matches!(self.goal, Goal::Climb { .. });
        let wants = match self.style {
            Style::Dive => false,
            Style::Cautious => {
                let near_gate = c.gates.iter().enumerate().any(|(k, g)| p.gates & (1 << k) == 0 && g.x > x && g.x - x < 520.0);
                y < need + if near_gate { -4.0 } else if climbing { 70.0 } else { 30.0 }
            }
            Style::Normal => y < need + if climbing { 12.0 } else { -4.0 },
        };
        if !wants {
            if climbing && self.style == Style::Normal {
                self.goal = Goal::Glide;
            }
            return;
        }
        if self.goal == Goal::Burn {
            return;
        }
        let fwd = 140.0 + 320.0 * sk;
        let back = 50.0 + 130.0 * sk;
        let center = match self.goal {
            Goal::Climb { x: cx, .. } => cx,
            _ => x,
        };
        let mut best: Option<(f32, f32)> = None;
        let mut dx = -back;
        while dx <= fwd {
            let sx = (x + dx).clamp(20.0, LEN as f32 - 20.0);
            // lift behind a hill we cannot fly over does not count
            let blocked = (1..=4).any(|k| c.ground_at(x + dx * k as f32 / 4.0) > y - 14.0);
            let l = if blocked { -1.0 } else { race.air(p.def, sx, y) + (1.0 - sk) * self.rng.range(-0.1, 0.1) };
            // prefer lift ahead, close by, and near what we already work
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
                    Style::Cautious => need + 90.0,
                    // weaker pilots climb more than they need (and lose time)
                    _ => need + 14.0 + sk * 10.0 + (1.0 - sk) * self.rng.range(-10.0, 80.0),
                };
                if !matches!(self.goal, Goal::Climb { .. }) {
                    self.climb_ref = (self.t, y);
                }
                self.goal = Goal::Climb { x: bx + self.err, until };
            }
            None => self.goal = Goal::Glide,
        }
    }

    /// Final glide: straight in when the height fits, S-turns when too high.
    fn final_dir(&mut self, race: &Race, i: usize) -> f32 {
        let p = &race.pilots[i];
        let c = &race.course;
        if !self.final_rolled {
            self.final_rolled = true;
            self.land_err = (1.0 - self.skill) * self.rng.range(-48.0, 40.0);
        }
        // glide to the point where we flare; the flare floats on a bit
        let aim = c.target - 4.0 + self.land_err;
        // speed is height too: it floats you on when you slow down
        let energy = (p.v * p.v - TRIM * TRIM) * ENERGY * self.skill;
        let agl = p.y - c.ground_at(c.target) - self.flare_h.max(1.5) + energy;
        let dist = aim - p.x;
        let gr_trim = self.glide(race, p.def, TRIM);
        let gr_dive = self.glide(race, p.def, 1.22);
        if p.face > 0.0 {
            if dist < -4.0 && agl > 16.0 {
                return -1.0;
            }
            let _ = gr_dive;
            if dist > 0.0 && dist < 320.0 && agl > 14.0 && dist / agl < gr_trim * 0.8 {
                // too high: turn away and burn it off
                return -1.0;
            }
            1.0
        } else {
            // flying away: turn back when the glide home fits
            let back = aim - p.x;
            let fits = (agl - 3.0) * gr_trim * 0.97 <= back + 10.0;
            if fits || p.x < c.field.0 - 80.0 { 1.0 } else { -1.0 }
        }
    }

    fn final_pitch(&mut self, race: &Race, i: usize, pad: &mut Pad) {
        let p = &race.pilots[i];
        let c = &race.course;
        if p.face < 0.0 {
            pad.down = p.y - c.ground_at(c.target) > 40.0;
            return;
        }
        let aim = c.target - 4.0 + self.land_err;
        let energy = (p.v * p.v - TRIM * TRIM) * ENERGY * self.skill;
        let agl = (p.y - c.ground_at(c.target) - self.flare_h.max(1.5) + energy).max(1.0);
        let dist = aim - p.x;
        if dist <= 0.0 {
            pad.down = agl > 10.0;
            return;
        }
        let gr_trim = self.glide(race, p.def, TRIM);
        let gr_dive = self.glide(race, p.def, 1.22);
        let need = dist / agl;
        if dist < 330.0 {
            // close in: S-turns do the big work; a slower glide is steeper
            if need < gr_trim * 0.95 && p.v > 0.68 && p.y - c.ground_at(c.target) > 10.0 {
                pad.up = self.t % 3 == 0;
            }
            return;
        }
        if need < gr_dive && dist > 150.0 && agl > 50.0 {
            // far out and high: race in tucked
            pad.down = true;
            pad.c = true;
        } else if need < gr_trim {
            // push the bar out part of the time: a steeper glide
            let k = ((gr_trim - need) / (gr_trim - gr_dive)).clamp(0.0, 1.0);
            let slots = 10;
            // no diving in the last few metres: that speed floats you long
            pad.down = ((self.t % slots) as f32) < k * slots as f32 && p.y - c.ground_at(c.target) > 22.0;
        }
    }
}
