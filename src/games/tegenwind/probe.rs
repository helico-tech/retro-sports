//! Balance tools without a window: `sim` races CPU riders against each
//! other, `probe` puts scripted "human" bots on the start line against the
//! 1P ladder to check the difficulty curve and look for exploits.

use super::data::*;
use super::race::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BotKind {
    /// alternates as fast as a human can, nothing else
    Masher,
    /// perfect rhythm, nothing else (no lanes, no hops)
    Metronome,
    /// a decent human: rhythm with some noise, dodges, hops, rings, tucks
    Decent,
    /// decent, but hunts for a wheel to sit on
    Drafter,
    /// decent, but never tucks
    NoTuck,
    /// decent, but never drafts on purpose and keeps to one lane
    NoDraft,
    /// a casual player: slow rhythm, late reactions
    Lazy,
}

pub struct Bot {
    pub kind: BotKind,
    rng: Rng,
    side: bool,
    next: i32,
    react: i32,
    seen: Option<f32>,
    pad: Pad,
    held_up: bool,
}

impl Bot {
    pub fn new(kind: BotKind, seed: u64) -> Self {
        Bot { kind, rng: Rng(seed.wrapping_mul(0x2545_F491_4F6C_DD1D) | 1), side: false, next: 0, react: 0, seen: None, pad: Pad::default(), held_up: false }
    }

    fn noise(&mut self, n: i32) -> i32 {
        // triangle noise in -n..=n
        self.rng.int(n + 1) - self.rng.int(n + 1)
    }

    /// The pad a human would hold this frame while riding rider `me`.
    pub fn pad(&mut self, race: &Race, me: usize) -> Pad {
        let mut p = Pad::default();
        let r = &race.riders[me];
        if race.phase != Phase::Race || !r.riding() || r.finish.is_some() {
            self.pad = p;
            return p;
        }
        let g = race.gap_for(me);
        let decentish = !matches!(self.kind, BotKind::Masher | BotKind::Metronome);
        // pedalling
        let press = match self.kind {
            BotKind::Masher => race.frame % 4 == 0,
            BotKind::Metronome => r.since > g,
            BotKind::Lazy => {
                if self.next == 0 {
                    self.next = g + 6 + self.noise(3);
                }
                r.since >= self.next
            }
            _ => {
                if self.next == 0 {
                    self.next = g + 1 + self.noise(2);
                }
                r.since >= self.next
            }
        };
        if press && r.z <= 0.0 {
            self.side = !self.side;
            if self.side {
                p.left = true;
            } else {
                p.right = true;
            }
            self.next = 0;
            // a held key from last frame would not make a new press: release first
            if (p.left && self.pad.left) || (p.right && self.pad.right) {
                p.left = false;
                p.right = false;
            }
        }
        if !decentish {
            self.pad = p;
            return p;
        }
        let st = race.def();
        let (wind, _) = st.wind_at(r.x);
        let b = r.bike.def();
        let look = 120.0;
        // the first thing in my lane
        let mut hz: Option<(f32, OKind)> = None;
        for o in &race.objs {
            if o.gone || matches!(o.kind, OKind::Food(_)) {
                continue;
            }
            let dx = o.x - r.x;
            if !(-4.0..look).contains(&dx) {
                continue;
            }
            let lane_hit = o.kind.wide() || (o.y - r.y).abs() < 0.5 || (o.kind == OKind::Sheep && o.state == 1 && o.y < r.y + 0.5 && o.y > r.y - 1.4);
            if !lane_hit {
                continue;
            }
            if !o.blocking() && !o.kind.slow_rider() {
                continue;
            }
            if o.kind.slow_rider() && dx > 50.0 {
                continue;
            }
            hz = Some((dx, o.kind));
            break;
        }
        // reaction time: you see it, then you act
        let react_frames = if self.kind == BotKind::Lazy { 26 } else { 12 };
        match hz {
            Some((dx, _)) => {
                if self.seen.is_none() {
                    self.seen = Some(dx);
                    self.react = react_frames + self.rng.int(8);
                }
            }
            None => self.seen = None,
        }
        self.react -= 1;
        let other = 1 - r.lane;
        let clear = |lane: i32| -> bool {
            let ly = lane as f32;
            !race.objs.iter().any(|o| !o.gone && !matches!(o.kind, OKind::Food(_)) && !o.kind.wide() && (o.blocking() || o.kind.slow_rider()) && (o.y - ly).abs() < 0.6 && o.x - r.x > -20.0 && o.x - r.x < 90.0)
                && !race.riders.iter().enumerate().any(|(j, q)| j != me && q.riding() && (q.y - ly).abs() < 0.5 && (q.x - r.x).abs() < 22.0)
        };
        let mut busy = false;
        if let Some((dx, kind)) = hz {
            busy = dx < 100.0;
            if self.react <= 0 {
                if kind == OKind::Tourist && dx < 100.0 && self.rng.f() < 0.15 {
                    p.a = !self.pad.a;
                }
                let can_hop = hop_clears(b.hop, kind.height(), kind.width(), r.v);
                if !kind.wide() && clear(other) && (self.kind != BotKind::NoDraft || !can_hop) {
                    if other == 0 {
                        p.up = true;
                    } else {
                        p.down = true;
                    }
                } else if can_hop {
                    let lead = (b.hop / GRAV + self.noise(3) as f32) * r.v;
                    let ox = dx + r.x;
                    if ox - r.x <= lead {
                        p.b = !self.pad.b;
                    }
                } else if r.v > CRASH_V * 0.8 && dx < 40.0 + r.v * 22.0 && !kind.slow_rider() {
                    p.left = true;
                    p.right = true;
                }
            }
        }
        // side wind: far lane, lean in
        if wind.side() > 0.3 && !busy {
            if r.lane == 1 && clear(0) {
                p.up = true;
            }
            if r.y > r.lane as f32 + 0.05 {
                p.up = true;
            }
        } else if wind.side() > 0.0 && r.y > r.lane as f32 + 0.1 {
            p.up = true;
        }
        // drafting
        if !busy && !p.up && !p.down {
            let want = match self.kind {
                BotKind::Drafter => 0.08,
                BotKind::NoDraft => 0.0,
                _ => 0.02,
            };
            let wheel = race.riders.iter().enumerate().any(|(j, q)| j != me && q.riding() && q.finish.is_none() && (q.y - other as f32).abs() < 0.3 && q.x - r.x > 18.0 && q.x - r.x < 80.0);
            if wheel && self.rng.f() < want && clear(other) && !(wind.side() > 0.3 && other == 1) {
                if other == 0 {
                    p.up = true;
                } else {
                    p.down = true;
                }
            }
            // food in the other lane when hungry
            if r.energy < 60.0 && race.objs.iter().any(|o| !o.gone && matches!(o.kind, OKind::Food(_)) && (o.y - other as f32).abs() < 0.3 && o.x - r.x > 30.0 && o.x - r.x < 100.0) && clear(other) {
                if other == 0 {
                    p.up = true;
                } else {
                    p.down = true;
                }
            }
        }
        // tuck in a headwind when nothing is near
        if !matches!(self.kind, BotKind::NoTuck | BotKind::Lazy) && wind.head() > 0.35 && !busy && !(wind.side() > 0.3 && r.y > 0.5) {
            p.c = true;
        }
        // the edge detector needs a release between presses
        if p.up && self.held_up {
            p.up = false;
        }
        self.held_up = p.up || p.down;
        if p.down && self.pad.down {
            p.down = false;
        }
        self.pad = p;
        p
    }
}

/// The 1P field on a stage: you, then the three rivals.
pub fn ladder_field(bike: Bike, stage_i: usize, human: Ctrl) -> Vec<(Who, Bike, Ctrl, f32)> {
    let sk = STAGES[stage_i].cpu_skill;
    let mut f = vec![(Who::Kees, bike, human, 0.5)];
    for w in RIVALS {
        f.push((w, w.def().bike, Ctrl::Cpu, sk));
    }
    f
}

pub fn run_race(race: &mut Race, bots: &mut [(usize, Bot)]) {
    let mut prev = [Pad::default(); 2];
    let mut n = 0;
    while !race.finished() && n < 60 * 60 * 8 {
        let mut pads = [Pad::default(); 2];
        for (k, (me, bot)) in bots.iter_mut().enumerate() {
            pads[k] = bot.pad(race, *me);
        }
        race.update(&pads, &prev);
        prev = pads;
        race.sfx.clear();
        n += 1;
    }
}

#[derive(Default)]
struct Acc {
    n: u32,
    top2: u32,
    wins: u32,
    time: f64,
    place: u32,
    perfect: u64,
    good: u64,
    wasted: u64,
    crashes: u32,
    bonks: u32,
    draft: u64,
    frames: u64,
    cargo: i64,
    dnf: u32,
}

pub fn sim(n: u32) {
    println!("CPU races: 4 CPU riders per race, {} races per stage", n);
    for stage_i in 0..3 {
        let st = &STAGES[stage_i];
        let mut wins = [0u32; 5];
        let mut first_t = 0.0f64;
        let mut spread = 0.0f64;
        let mut max_spread = 0.0f32;
        let mut crashes = 0u32;
        let mut bumps = 0u32;
        let mut waits = 0u32;
        let mut wait_frames = 0u64;
        for k in 0..n {
            let bike = BIKES[(k % 4) as usize];
            let mut field = ladder_field(bike, stage_i, Ctrl::Cpu);
            field[0].3 = st.cpu_skill;
            let mut race = Race::new(stage_i, &field, k as u64 * 31 + stage_i as u64 * 7 + 1);
            run_race(&mut race, &mut []);
            let res = race.results();
            wins[race.riders[res[0].0].who.index()] += 1;
            let t0 = res[0].1 as f32 / 60.0;
            let t3 = res[3].1 as f32 / 60.0;
            first_t += t0 as f64;
            spread += (t3 - t0) as f64;
            max_spread = max_spread.max(t3 - t0);
            for r in &race.riders {
                crashes += r.stats.crashes;
                bumps += r.stats.bumps;
                if r.stats.wait_frames > 0 {
                    waits += 1;
                    wait_frames += r.stats.wait_frames as u64;
                }
            }
        }
        let nf = n.max(1) as f64;
        println!("{}  (length {:.0} px)", st.name, st.length());
        println!(
            "  winner time {:.1}s   field spread avg {:.1}s max {:.1}s   crashes/race {:.2}   bumps/race {:.1}   bridge waits/race {:.2} ({:.1}s each)",
            first_t / nf,
            spread / nf,
            max_spread,
            crashes as f64 / nf,
            bumps as f64 / nf,
            waits as f64 / nf,
            wait_frames as f64 / 60.0 / waits.max(1) as f64
        );
        let names: Vec<String> = ALL_WHO.iter().filter(|w| **w != Who::Anouk).map(|w| format!("{} {:.0}%", w.def().short, 100.0 * wins[w.index()] as f64 / nf)).collect();
        println!("  wins: {}", names.join("   "));
    }
}

pub fn probe(per: u32) {
    let kinds = [BotKind::Masher, BotKind::Metronome, BotKind::Lazy, BotKind::Decent, BotKind::NoTuck, BotKind::NoDraft, BotKind::Drafter];
    println!("Human bots vs the 1P ladder, {} races per stage per bike", per);
    println!("{:<10} {:<10} {:>5} {:>5} {:>6} {:>5} {:>6} {:>6} {:>6} {:>5} {:>5} {:>6} {:>5}", "bot", "stage", "top2%", "win%", "time", "place", "perf%", "waste%", "draft%", "crash", "bonk", "cargo", "dnf");
    for &kind in &kinds {
        for stage_i in 0..3 {
            let mut a = Acc::default();
            let mut per_bike = [(0u32, 0u32); 4];
            for (bi, &bike) in BIKES.iter().enumerate() {
                for s in 0..per {
                    let seed = s as u64 * 977 + stage_i as u64 * 13 + bi as u64 * 101 + 5;
                    let field = ladder_field(bike, stage_i, Ctrl::Human(0));
                    let mut race = Race::new(stage_i, &field, seed);
                    let mut bots = [(0usize, Bot::new(kind, seed))];
                    run_race(&mut race, &mut bots);
                    let res = race.results();
                    let place = res.iter().position(|e| e.0 == 0).unwrap();
                    let me = &race.riders[0];
                    a.n += 1;
                    per_bike[bi].1 += 1;
                    if place < 2 {
                        a.top2 += 1;
                    }
                    if place == 0 {
                        a.wins += 1;
                        per_bike[bi].0 += 1;
                    }
                    a.place += place as u32 + 1;
                    a.time += res[place].1.min(60 * 600) as f64 / 60.0;
                    a.perfect += me.stats.perfect as u64;
                    a.good += me.stats.good as u64;
                    a.wasted += me.stats.wasted as u64;
                    a.crashes += me.stats.crashes;
                    a.bonks += me.stats.bonks;
                    a.draft += me.stats.draft_frames as u64;
                    a.frames += race.clock as u64;
                    a.cargo += me.cargo as i64;
                    if me.dnf {
                        a.dnf += 1;
                    }
                }
            }
            let n = a.n.max(1) as f64;
            let strokes = (a.perfect + a.good + a.wasted).max(1) as f64;
            println!(
                "{:<10} {:<10} {:>5.0} {:>5.0} {:>6.1} {:>5.2} {:>6.0} {:>6.0} {:>6.0} {:>5.2} {:>5.2} {:>6.2} {:>5}",
                format!("{:?}", kind),
                STAGES[stage_i].name,
                100.0 * a.top2 as f64 / n,
                100.0 * a.wins as f64 / n,
                a.time / n,
                a.place as f64 / n,
                100.0 * a.perfect as f64 / strokes,
                100.0 * a.wasted as f64 / strokes,
                100.0 * a.draft as f64 / a.frames.max(1) as f64,
                a.crashes as f64 / n,
                a.bonks as f64 / n,
                a.cargo as f64 / n,
                a.dnf
            );
            if kind == BotKind::Decent {
                let s: Vec<String> = BIKES.iter().enumerate().map(|(bi, b)| format!("{} {:.0}%", b.def().name, 100.0 * per_bike[bi].0 as f64 / per_bike[bi].1.max(1) as f64)).collect();
                println!("{:<21} wins by bike: {}", "", s.join("  "));
            }
        }
    }
}
