//! The brain that plays a side: picks angle and power, runs the dog to the
//! landing spot, leaps and flips. It only presses pad buttons, so the CPU
//! and the headless probe bots go through the same input path as a human.

use super::data::PAIRS;
use super::sim::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FlipPolicy {
    Never,
    Always,
    /// flip after a catch when there seems to be time (with a judgement error)
    Judge,
}

#[derive(Clone, Debug)]
pub struct Brain {
    pub skill: f32,
    pub flip: FlipPolicy,
    /// chance per toss to go for a jump catch
    pub p_jump: f32,
    /// chance per toss to try a flip (Judge only)
    pub p_flip: f32,
    /// frames of leap timing error (+-)
    pub jump_noise: f32,
    /// frames of power release error (+-)
    pub release_noise: f32,
    pub angle_noise: f32,
    /// frames before tapping C when the dog stares at a squirrel
    pub react: i32,
    /// frames of error when judging the time left in the air
    pub judge_err: f32,
    /// how close to the dog's limit it throws (0 safe .. 1 greedy)
    pub greed: f32,
    pub use_zoomies: bool,
    /// px of error in judging where the disc comes down (per toss)
    pub spot_err: f32,
    rng: Rng,
    // per toss
    target_angle: f32,
    target_power: f32,
    floater: bool,
    release_at: i32,
    plan_jump: bool,
    plan_flip: bool,
    err: f32,
    bias: f32,
    jump_err: i32,
    traj: Vec<(f32, f32)>,
    target_x: f32,
    leap_in: Option<i32>,
    leap_face: f32,
    replan: i32,
    t: i32,
    tap: i32,
    zoom_step: i32,
    planned: bool,
}

impl Brain {
    fn base(skill: f32, seed: u64) -> Self {
        Brain {
            skill,
            flip: FlipPolicy::Judge,
            p_jump: 0.5,
            p_flip: 0.5,
            jump_noise: 3.0,
            release_noise: 2.0,
            angle_noise: 3.0,
            react: 20,
            judge_err: 4.0,
            greed: 0.5,
            use_zoomies: true,
            spot_err: 0.0,
            bias: 0.0,
            rng: Rng(seed.wrapping_mul(2_654_435_761) | 1),
            target_angle: 20.0,
            target_power: 0.5,
            floater: false,
            release_at: 16,
            plan_jump: false,
            plan_flip: false,
            err: 0.0,
            jump_err: 0,
            traj: Vec::with_capacity(600),
            target_x: 0.0,
            leap_in: None,
            leap_face: 1.0,
            replan: 0,
            t: 0,
            tap: 0,
            zoom_step: 0,
            planned: false,
        }
    }

    /// The CPU at a skill level (0.3 clumsy .. 0.9 sharp).
    pub fn cpu(skill: f32, seed: u64) -> Self {
        let s = skill.clamp(0.0, 1.0);
        let mut b = Brain::base(s, seed);
        b.p_jump = 0.3 + 0.65 * s;
        b.p_flip = (s - 0.25).max(0.0) * 1.1;
        b.jump_noise = 1.5 + (1.0 - s) * 9.0;
        b.release_noise = 1.0 + (1.0 - s) * 6.0;
        b.angle_noise = 1.0 + (1.0 - s) * 8.0;
        b.react = 8 + ((1.0 - s) * 50.0) as i32;
        b.judge_err = 1.0 + (1.0 - s) * 12.0;
        b.greed = s;
        b.spot_err = (1.0 - s) * 26.0;
        b
    }

    /// A scripted "human" for the probe: decent timing, some greed.
    pub fn human(style: &str, seed: u64) -> Self {
        let mut b = Brain::base(0.6, seed);
        b.jump_noise = 3.5;
        b.release_noise = 2.5;
        b.angle_noise = 3.0;
        b.react = 24;
        b.judge_err = 5.0;
        b.spot_err = 8.0;
        b.greed = 0.6;
        match style {
            "safe" => {
                b.flip = FlipPolicy::Never;
                b.p_jump = 0.8;
            }
            "greedy" => {
                b.flip = FlipPolicy::Always;
                b.p_jump = 1.0;
                b.p_flip = 1.0;
            }
            "lazy" => {
                b.flip = FlipPolicy::Never;
                b.p_jump = 0.0;
                b.jump_noise = 6.0;
                b.release_noise = 5.0;
                b.angle_noise = 6.0;
                b.react = 60;
                b.greed = 0.3;
                b.use_zoomies = false;
            }
            _ => {
                // decent
                b.p_jump = 0.75;
                b.p_flip = 0.6;
            }
        }
        b
    }

    pub fn debug(&self) -> (f32, Option<i32>, bool) {
        (self.target_x, self.leap_in, self.plan_jump)
    }

    pub fn new_turn(&mut self) {
        self.planned = false;
        self.leap_in = None;
        self.t = 0;
        self.tap = 0;
        self.zoom_step = 0;
    }

    fn noise(&mut self, amt: f32) -> f32 {
        (self.rng.f() + self.rng.f() - 1.0) * amt
    }

    fn plan_throw(&mut self, m: &Match, side: usize) {
        self.planned = true;
        let pair = m.sides[side].pair;
        let body = dog_body(pair);
        // slow dogs and tailwinds like floaters
        let slow = PAIRS[pair].dog.speed <= 3;
        self.floater = self.rng.f() < if m.wind > 0.35 { 0.5 } else if slow { 0.3 } else { 0.1 };
        let (mdx, mz) = body.mouth;
        let margin = 6.0 + (1.0 - self.greed) * 30.0;
        // try angles and powers; keep the longest throw the dog can still reach
        let mut best = (0.3f32, 30.0f32, -1.0f32);
        let mut a = 15.0f32;
        while a <= 42.0 {
            let mut p = 0.2f32;
            while p <= 1.0 {
                let mut d = Disc::new();
                d.x = OWNER_X + 8.0;
                d.z = HAND_Z;
                d.launch(a, p, p >= PERFECT_FROM, self.floater);
                predict(&d, m.wind, &mut self.traj);
                // when the disc comes down to mouth height
                let k = self.traj.iter().position(|q| q.1 < mz + 4.0 && q.0 > 10.0).unwrap_or(self.traj.len());
                let land = self.traj.get(k.min(self.traj.len().saturating_sub(1))).map(|q| q.0).unwrap_or(0.0);
                let run = (land + mdx - DOG_START).max(0.0) / body.top + 12.0;
                let in_field = land < FIELD_END - 30.0;
                let dry = !m.in_pond(land) || self.skill < 0.45;
                if run + margin < k as f32 && in_field && dry && land > best.2 {
                    best = (p, a, land);
                }
                p += 0.05;
            }
            a += 3.0;
        }
        let (best, base_angle, _) = best;
        self.target_angle = (base_angle + self.noise(self.angle_noise)).clamp(ANGLE_MIN, ANGLE_MAX);
        self.target_power = best;
        let k = (best * METER_HALF as f32).round();
        let n = self.noise(self.release_noise).round();
        self.release_at = ((k + n) as i32).clamp(1, METER_HALF * 2 - 1);
        self.plan_jump = self.rng.f() < self.p_jump;
        self.plan_flip = match self.flip {
            FlipPolicy::Never => false,
            FlipPolicy::Always => true,
            FlipPolicy::Judge => self.rng.f() < self.p_flip,
        };
        self.err = self.noise(self.judge_err);
        self.bias = self.noise(self.spot_err);
        self.jump_err = self.noise(self.jump_noise).round() as i32;
    }

    pub fn pad(&mut self, m: &Match, side: usize) -> Pad {
        let mut pad = Pad::default();
        self.t += 1;
        match m.phase {
            Phase::Ready => {
                if !self.planned {
                    self.plan_throw(m, side);
                }
            }
            Phase::Aim => {
                if !self.planned {
                    self.plan_throw(m, side);
                }
                if m.phase_t < 20 {
                    return pad;
                }
                let da = self.target_angle - m.angle;
                if da >= 1.0 {
                    pad.up = true;
                } else if da <= -1.0 {
                    pad.down = true;
                } else if m.phase_t > 30 {
                    pad.a = true;
                }
            }
            Phase::Power => {
                pad.a = m.meter_t + 1 < self.release_at;
                pad.c = self.floater;
            }
            Phase::Flight => self.fetch(m, &mut pad),
            _ => {}
        }
        pad
    }

    fn fetch(&mut self, m: &Match, pad: &mut Pad) {
        let d = &m.dog;
        let disc = &m.disc;
        // snap out of a squirrel stare
        if d.state == DogState::Stare {
            self.tap += 1;
            if self.tap > self.react && self.tap % 6 < 3 {
                pad.c = true;
            }
            return;
        }
        self.tap = 0;
        // flip with the disc in the mouth
        if d.has_disc {
            if d.state == DogState::Air && d.flip_t < 0 && !d.flip_with_disc && self.plan_flip {
                let left = d.time_to_land() + self.err;
                let go = match self.flip {
                    FlipPolicy::Always => true,
                    _ => left > FLIP_LEN as f32 + 1.0,
                };
                if go {
                    pad.a = true;
                }
            }
            return;
        }
        if matches!(d.state, DogState::Tumble) {
            return;
        }
        // in the air: steer the mouth toward the disc
        if d.state == DogState::Air {
            if self.skill > 0.4 && disc.state == DiscState::Flying {
                let (mx, _) = d.mouth();
                if disc.x > mx + 2.0 {
                    pad.right = true;
                } else if disc.x < mx - 2.0 {
                    pad.left = true;
                }
            }
            return;
        }
        let body = d.body;
        // where to be
        self.replan -= 1;
        match disc.state {
            DiscState::Flying => {
                // a leap that is about to happen is not re-planned
                let committed = self.leap_in.map(|l| l < 10).unwrap_or(false);
                if self.replan <= 0 && !committed {
                    self.replan = 5;
                    predict(disc, m.wind, &mut self.traj);
                    self.choose_target(m);
                }
            }
            DiscState::Skid | DiscState::Floating => {
                self.target_x = disc.x - d.face * body.mouth.0 * 0.7;
                self.leap_in = None;
            }
            _ => return,
        }
        if let Some(l) = self.leap_in.as_mut() {
            *l -= 1;
        }
        let dist = self.target_x - d.x;
        let stop = d.vx * d.vx / (2.0 * 0.14) + 2.0;
        let toward = if dist > 0.0 { 1.0 } else { -1.0 };
        let moving_toward = d.vx * toward > 0.0;
        if dist.abs() > 3.0 && !(moving_toward && dist.abs() < stop) {
            if toward > 0.0 {
                pad.right = true;
            } else {
                pad.left = true;
            }
        }
        // zoomies when the disc gets there first
        if self.use_zoomies && !d.zooming && d.zoom > 20.0 && d.grounded() {
            let frames_left = self.frames_until_low(m);
            if dist.abs() > d.body.top * frames_left * 0.9 && dist.abs() > 40.0 {
                // tap, release, tap
                self.zoom_step += 1;
                let dirp = |pad: &mut Pad, on: bool| {
                    if toward > 0.0 {
                        pad.right = on;
                    } else {
                        pad.left = on;
                    }
                };
                match self.zoom_step {
                    1 => dirp(pad, false),
                    2 => dirp(pad, true),
                    3 => dirp(pad, false),
                    _ => {
                        dirp(pad, true);
                        self.zoom_step = 0;
                    }
                }
            }
        }
        // hop over the pond on the way
        if let Some((a, b)) = m.pond {
            let ahead = if d.vx > 0.0 { a - d.x } else { d.x - b };
            let beyond = if d.vx > 0.0 { self.target_x > b } else { self.target_x < a };
            if d.grounded() && beyond && d.vx.abs() > 1.4 && (4.0..14.0).contains(&ahead) && self.skill > 0.35 {
                pad.b = true;
                self.leap_in = None;
                self.replan = 0;
            }
        }
        // leap
        if let Some(l) = self.leap_in {
            if l <= self.jump_err && d.grounded() && d.state != DogState::Swim {
                pad.b = true;
                self.leap_in = None;
                // turn to face where the disc will be
                pad.left = self.leap_face < 0.0;
                pad.right = self.leap_face > 0.0;
            }
        }
    }

    /// Frames until the predicted disc comes down to mouth height.
    fn frames_until_low(&self, m: &Match) -> f32 {
        let mh = m.dog.body.mouth.1 + 4.0;
        self.traj.iter().position(|q| q.1 < mh).unwrap_or(self.traj.len()) as f32
    }

    fn choose_target(&mut self, m: &Match) {
        let d = &m.dog;
        let body = d.body;
        let (mdx, mz) = body.mouth;
        let air_mz = mz + 2.0;
        let v0 = body.jump_v;
        let apex = v0 * v0 / (2.0 * DOG_G);
        let mut chosen = false;
        // a dog that gets there early waits facing the disc (mouth on the left);
        // one that only just makes it catches on the run (mouth on the right)
        if self.plan_jump && d.grounded() {
            // catch it high: on the way down, between 45% and 85% of the jump
            let (lo, hi) = (air_mz + apex * 0.45, air_mz + apex * 0.85);
            for (k, &(x, z)) in self.traj.iter().enumerate() {
                if z > hi || z < lo || (k > 0 && self.traj[k - 1].1 < z) {
                    continue;
                }
                let k = k as f32 + 1.0;
                let h = z - air_mz;
                let disc = v0 * v0 - 2.0 * DOG_G * h;
                if disc < 0.0 {
                    continue;
                }
                let rise = (v0 - disc.sqrt()) / DOG_G;
                let early = x + mdx;
                let late = x - mdx;
                let need = |tx: f32| (tx - d.x).abs() / body.top + 6.0;
                let tx = if need(early) + rise + 10.0 <= k {
                    early
                } else if need(late) + rise <= k {
                    late
                } else {
                    continue;
                };
                if k - rise > 0.0 {
                    self.target_x = tx;
                    self.leap_face = if tx < x { 1.0 } else { -1.0 };
                    self.leap_in = Some((k - rise).round() as i32);
                    chosen = true;
                    break;
                }
            }
        }
        if !chosen {
            self.leap_in = None;
            // a ground catch at mouth height, or wherever it lands
            let k = self.traj.iter().position(|q| q.1 < mz + 2.0).unwrap_or(self.traj.len().saturating_sub(1));
            if let Some(&(x, _)) = self.traj.get(k) {
                let early = x + mdx;
                let need = (early - d.x).abs() / body.top + 10.0;
                self.target_x = if need <= k as f32 { early } else { x - mdx };
            }
        }
        // the end of the field
        self.target_x = (self.target_x + self.bias).min(FIELD_END - 6.0);
    }
}
