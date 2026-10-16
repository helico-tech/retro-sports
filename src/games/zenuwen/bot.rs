//! The CPU thief (attract mode, `sim`) and the probe bots that stand in for
//! human players. A bot only reads the world and returns a pad, like a
//! player with a controller.

use super::data::*;
use super::world::*;
use crate::engine::input::Pad;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BotKind {
    /// a decent human: waits for gaps, acts casual, hides when chased
    Decent,
    /// runs everywhere, cuts every lock that is not a ring lock
    Reckless,
    /// waits for big gaps, never uses the cutter
    Sneaky,
    /// ignores the BOA's
    Lazy,
    /// the CPU thief with a skill 0..1
    Cpu(f32),
}

impl BotKind {
    pub fn name(self) -> &'static str {
        match self {
            BotKind::Decent => "DECENT",
            BotKind::Reckless => "RECKLESS",
            BotKind::Sneaky => "SNEAKY",
            BotKind::Lazy => "LAZY",
            BotKind::Cpu(_) => "CPU",
        }
    }
}

pub struct Bot {
    pub kind: BotKind,
    rng: Rng,
    /// frames before danger makes it stop what it is doing
    react: u32,
    /// extra safety distance, px
    margin: f32,
    /// chance to hit the green zone of a U-lock
    u_acc: f32,
    target: Option<usize>,
    /// bikes it gave up on, with frames left
    avoid: Vec<(usize, u32)>,
    alarm: u32,
    safe_t: u32,
    wait_t: u32,
    tap_gap: u32,
    tap_t: u32,
    /// U-lock: this pass through the zone is a miss
    miss: bool,
    in_zone: bool,
    hide_t: u32,
    last: Pad,
    retarget: u32,
}

impl Bot {
    pub fn new(kind: BotKind, seed: u64) -> Self {
        let (react, margin, u_acc) = match kind {
            BotKind::Decent => (14, 24.0, 0.72),
            BotKind::Reckless => (24, 0.0, 0.55),
            BotKind::Sneaky => (10, 50.0, 0.72),
            BotKind::Lazy => (999, 0.0, 0.5),
            BotKind::Cpu(s) => ((30.0 - 20.0 * s) as u32, 10.0 + 30.0 * s, 0.5 + 0.4 * s),
        };
        Bot {
            kind,
            rng: Rng(seed | 1),
            react,
            margin,
            u_acc,
            target: None,
            avoid: Vec::new(),
            alarm: 0,
            safe_t: 0,
            wait_t: 0,
            tap_gap: 6,
            tap_t: 0,
            miss: false,
            in_zone: false,
            hide_t: 0,
            last: Pad::default(),
            retarget: 0,
        }
    }

    fn careful(&self) -> bool {
        !matches!(self.kind, BotKind::Lazy | BotKind::Reckless)
    }

    /// Is (x, y) watched now, or about to be?
    fn danger(&self, w: &World, x: f32, y: f32) -> (bool, bool) {
        let mut now = false;
        let mut soon = false;
        for b in &w.boas {
            if b.state == BoaState::Ticket {
                continue;
            }
            if w.sight(b, x, y) > 0.0 {
                now = true;
            }
            let dx = x - b.x;
            let toward = (dx > 0.0) != b.face_left;
            let reach = b.vision + self.margin + if b.kind == BoaKind::Bike { 50.0 } else { 20.0 };
            if (toward && dx.abs() < reach) || dx.abs() < 34.0 + self.margin || (b.alert() && dx.abs() < 160.0) {
                soon = true;
            }
        }
        (now, soon)
    }

    fn chased(&self, w: &World) -> Option<f32> {
        w.boas
            .iter()
            .filter(|b| matches!(b.state, BoaState::Chase | BoaState::Wait) || (b.state == BoaState::Investigate && w.thief.wanted > 0))
            .min_by_key(|b| (b.x - w.thief.x).abs() as i32)
            .map(|b| b.x)
    }

    /// Press a button only on a fresh frame, so it counts as a press.
    fn press(&self, pad: &mut Pad, which: char) {
        match which {
            'a' => pad.a = !self.last.a,
            'b' => pad.b = !self.last.b,
            'c' => pad.c = !self.last.c,
            'u' => pad.up = !self.last.up,
            'l' => pad.left = !self.last.left,
            'r' => pad.right = !self.last.right,
            _ => {}
        }
    }

    fn walk_to(&self, pad: &mut Pad, w: &World, gx: f32, gy: f32) {
        let th = &w.thief;
        if gx > th.x + 1.5 {
            pad.right = true;
        } else if gx < th.x - 1.5 {
            pad.left = true;
        }
        if gy > th.y + 1.5 {
            pad.down = true;
        } else if gy < th.y - 1.5 {
            pad.up = true;
        }
    }

    pub fn pad(&mut self, w: &World) -> Pad {
        let mut pad = Pad::default();
        if w.phase == Phase::Play {
            self.think(w, &mut pad);
        }
        self.last = pad;
        pad
    }

    fn think(&mut self, w: &World, pad: &mut Pad) {
        for a in &mut self.avoid {
            a.1 = a.1.saturating_sub(1);
        }
        self.avoid.retain(|a| a.1 > 0);
        let th = &w.thief;
        let chaser = if self.kind == BotKind::Lazy { None } else { self.chased(w) };
        match th.state {
            TState::Busted | TState::Fall | TState::Toss | TState::Out | TState::Climb => {
                self.target = None;
            }
            TState::Swim => {
                // swim away from the BOA's, climb out when they gave up
                let near = w.boas.iter().filter(|b| b.state != BoaState::Ticket).map(|b| (b.x - th.x).abs()).fold(999.0f32, f32::min);
                let waiting = w.boas.iter().any(|b| b.state == BoaState::Wait);
                if near > 90.0 && !waiting {
                    self.press(pad, 'u');
                    pad.up = true;
                } else if let Some(cx) = w.boas.iter().filter(|b| b.state != BoaState::Ticket).min_by_key(|b| (b.x - th.x).abs() as i32).map(|b| b.x) {
                    if cx < th.x {
                        pad.right = true;
                    } else {
                        pad.left = true;
                    }
                }
            }
            TState::Hide => {
                self.hide_t += 1;
                let alert = w.boas.iter().any(|b| b.alert() && (b.x - th.x).abs() < 120.0);
                let close = w.boas.iter().any(|b| (b.x - th.x).abs() < 50.0);
                if !alert && !close && self.hide_t > 100 || self.hide_t > 60 * 12 {
                    self.press(pad, 'c');
                    self.hide_t = 0;
                }
            }
            TState::Ride => self.ride(w, pad, chaser),
            TState::Pick => self.pick(w, pad, chaser),
            TState::Walk | TState::Casual => self.walk(w, pad, chaser),
        }
    }

    fn ride(&mut self, w: &World, pad: &mut Pad, chaser: Option<f32>) {
        let th = &w.thief;
        let Some(bi) = th.bike else { return };
        let bike = &w.bikes[bi];
        // flee first, toward Sjaak if he is on the far side
        if let Some(cx) = chaser {
            let away = if cx < th.x { 1.0 } else { -1.0 };
            let sj = w.sjaak.x - th.x;
            if sj * away > 0.0 || (cx - th.x).abs() > 120.0 {
                self.ride_to_sjaak(w, pad);
            } else {
                // run for it, away from the chaser, along the middle of the street
                if away > 0.0 {
                    pad.right = true;
                } else {
                    pad.left = true;
                }
                if (away > 0.0 && th.x > w.len - 30.0) || (away < 0.0 && th.x < 30.0) {
                    // cornered: ditch the bike and walk
                    self.press(pad, 'c');
                }
                if th.y < 26.0 {
                    pad.down = true;
                } else if th.y > 40.0 {
                    pad.up = true;
                }
            }
            return;
        }
        // a hot bike: don't parade it in front of a BOA
        if self.careful() && bike.hot > 0 {
            let (now, _) = self.danger(w, th.x, th.y);
            let ahead = w.boas.iter().any(|b| {
                let dx = b.x - th.x;
                (dx > 0.0) != th.face_left && dx.abs() < b.vision + 40.0 && b.state != BoaState::Ticket
            });
            if ahead && !now {
                // wait it out
                return;
            }
        }
        self.ride_to_sjaak(w, pad);
        // ring at pedestrians in the way
        let dir = if th.face_left { -1.0 } else { 1.0 };
        if w.peds.iter().any(|p| {
            let a = (p.x - th.x) * dir;
            a > 0.0 && a < 28.0 && (p.y - th.y).abs() < 6.0
        }) && !w.sjaak_near()
        {
            self.press(pad, 'a');
            // and steer round them
            pad.up = th.y > 30.0;
            pad.down = th.y <= 30.0;
        }
    }

    fn ride_to_sjaak(&mut self, w: &World, pad: &mut Pad) {
        let th = &w.thief;
        let watched = self.careful() && self.chased(w).is_none() && self.danger(w, th.x, th.y).0;
        if w.sjaak_near() && th.v.abs() < 1.6 && !watched {
            self.press(pad, 'a');
            return;
        }
        let gx = w.sjaak.x;
        if (gx - th.x).abs() > 10.0 {
            if gx > th.x {
                pad.right = true;
            } else {
                pad.left = true;
            }
        } else if th.v.abs() > 1.5 {
            // brake
            if th.v > 0.0 {
                pad.left = true;
            } else {
                pad.right = true;
            }
        }
        let gy = DEPTH - 5.0;
        if th.y < gy - 1.0 && (gx - th.x).abs() < 160.0 {
            pad.down = true;
        }
    }

    fn pick(&mut self, w: &World, pad: &mut Pad, chaser: Option<f32>) {
        let th = &w.thief;
        let Some(bi) = th.pick else { return };
        let bike = &w.bikes[bi];
        let (now, soon) = self.danger(w, th.x, th.y);
        let careful = self.careful();
        // someone coming over to look counts too
        let coming = w.boas.iter().any(|b| matches!(b.state, BoaState::Investigate | BoaState::Search) && (b.x - th.x).abs() < 80.0);
        if (now || coming || (soon && matches!(self.kind, BotKind::Sneaky))) && careful || chaser.is_some() {
            self.alarm += 1;
        } else {
            self.alarm = 0;
        }
        // cutter?
        let nearest = w.boas.iter().map(|b| (b.x - th.x).abs()).fold(999.0f32, f32::min);
        let use_cutter = th.cutters > 0
            && match self.kind {
                BotKind::Reckless => bike.lock != Lock::Ring,
                BotKind::Decent => matches!(bike.lock, Lock::U | Lock::Double) && nearest > 220.0 && bike.progress < 0.34,
                BotKind::Cpu(s) => matches!(bike.lock, Lock::U | Lock::Double) && nearest > 260.0 - 60.0 * s && bike.progress < 0.34,
                _ => false,
            };
        if use_cutter {
            self.press(pad, 'b');
            return;
        }
        if self.alarm >= self.react {
            // stop and look innocent
            self.press(pad, 'u');
            self.alarm = 0;
            self.safe_t = 0;
            return;
        }
        self.tap_t += 1;
        match bike.lock {
            Lock::Ring => {
                if self.tap_t >= self.tap_gap {
                    self.press(pad, 'a');
                    if pad.a {
                        self.tap_t = 0;
                        self.tap_gap = 5 + self.rng.int(4) as u32;
                    }
                }
            }
            Lock::Chain | Lock::Double => {
                if self.tap_t >= self.tap_gap {
                    if th.saw_left {
                        self.press(pad, 'r');
                    } else {
                        self.press(pad, 'l');
                    }
                    if pad.left || pad.right {
                        self.tap_t = 0;
                        self.tap_gap = 4 + self.rng.int(4) as u32;
                    }
                }
            }
            Lock::U => {
                let n = 0.5 + 0.5 * th.needle.sin();
                let half = th.who.u_zone() / 2.0;
                let d = (n - th.zone).abs();
                let inside = d <= half * 0.8;
                if inside && !self.in_zone {
                    self.miss = self.rng.f() > self.u_acc;
                }
                self.in_zone = inside;
                if inside && !self.miss {
                    self.press(pad, 'a');
                } else if self.miss && d > half + 0.02 && d < half + 0.1 {
                    self.press(pad, 'a');
                    self.miss = false;
                }
            }
            Lock::None => {}
        }
    }

    fn walk(&mut self, w: &World, pad: &mut Pad, chaser: Option<f32>) {
        let th = &w.thief;
        let careful = self.careful();
        if self.kind == BotKind::Reckless {
            pad.b = true;
        }
        if let Some(cx) = chaser {
            self.flee(w, pad, cx);
            return;
        }
        // pick a target bike
        self.retarget = self.retarget.saturating_sub(1);
        let valid = |i: usize| matches!(w.bikes[i].state, BikeState::Parked | BikeState::Loose) && w.needed().contains(&w.bikes[i].kind);
        if self.target.map(|i| !valid(i)).unwrap_or(true) || self.retarget == 0 {
            self.retarget = 90;
            let needed = w.needed();
            let best = w
                .bikes
                .iter()
                .enumerate()
                .filter(|(i, b)| matches!(b.state, BikeState::Parked | BikeState::Loose) && needed.contains(&b.kind) && !self.avoid.iter().any(|a| a.0 == *i))
                .min_by_key(|(_, b)| {
                    let mut c = (b.x - th.x).abs() + (b.y - th.y).abs() * 0.5;
                    if b.state == BikeState::Loose {
                        c -= 150.0;
                    }
                    if careful {
                        for bo in &w.boas {
                            if (bo.x - b.x).abs() < bo.vision {
                                c += 90.0;
                            }
                        }
                    }
                    c as i32
                })
                .map(|(i, _)| i);
            if best != self.target {
                self.wait_t = 0;
            }
            self.target = best;
        }
        let Some(ti) = self.target else {
            // nothing left on the order to steal: stroll to Sjaak
            self.walk_to(pad, w, w.sjaak.x, 40.0);
            return;
        };
        let b = &w.bikes[ti];
        let stand_y = if b.y < 20.0 { b.y + 6.0 } else { b.y - 5.0 };
        if !w.in_reach(ti) || (th.x - b.x).abs() > 8.0 {
            self.walk_to(pad, w, b.x, stand_y);
            // careful bots don't walk into a cone with a hot bike in hand... nothing to hide on foot
            return;
        }
        if b.state == BikeState::Loose || b.lock == Lock::None {
            self.press(pad, 'a');
            return;
        }
        // at the bike: wait for a gap
        let (now, soon) = self.danger(w, th.x, th.y);
        let safe = !careful || (!now && !soon);
        if safe {
            self.safe_t += 1;
        } else {
            self.safe_t = 0;
            self.wait_t += 1;
        }
        if now && careful && th.wanted == 0 {
            pad.c = true;
        }
        if self.safe_t > self.react / 2 + 2 {
            self.press(pad, 'a');
            self.safe_t = 0;
        }
        if self.wait_t > 60 * 8 {
            // this one is too watched; try another
            self.avoid.push((ti, 60 * 10));
            self.target = None;
            self.wait_t = 0;
        }
    }

    fn flee(&mut self, w: &World, pad: &mut Pad, cx: f32) {
        let th = &w.thief;
        let away = if cx < th.x { 1.0 } else { -1.0 };
        let (now, _) = self.danger(w, th.x, th.y);
        // a hiding spot on the way, when nobody is looking
        if self.kind != BotKind::Reckless
            && let Some(&(_, sx, sy)) = w
                .spots
                .iter()
                .filter(|s| s.0.hides() && (s.1 - th.x) * away > -10.0 && (s.1 - th.x).abs() < 80.0)
                .min_by_key(|s| (s.1 - th.x).abs() as i32)
            {
                if w.spot_at_hand().is_some() && !now {
                    self.press(pad, 'c');
                    return;
                }
                if (sx - cx).abs() > 30.0 {
                    pad.b = true;
                    self.walk_to(pad, w, sx, sy.clamp(12.0, DEPTH - 4.0));
                    return;
                }
            }
        pad.b = true;
        let cornered = (away > 0.0 && th.x > w.len - 24.0) || (away < 0.0 && th.x < 24.0);
        if cornered || (now && (cx - th.x).abs() < 20.0) {
            // into the canal
            pad.down = true;
            return;
        }
        if away > 0.0 {
            pad.right = true;
        } else {
            pad.left = true;
        }
    }
}
