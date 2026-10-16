//! TIENEN! headless balance tools: `sim` (four CPU kids per venue) and
//! `probe` (a human-like bot with three CPU kids through the ladder).

use super::bot::BotKind;
use super::data::*;
use super::world::*;
use crate::engine::input::Pad;

pub struct Game {
    pub secs: f32,
    /// goal points seat 0 scored
    pub scored: i32,
    pub loser: usize,
    pub dinner: bool,
    pub goals: [u32; 5],
    pub tally: [u32; 7],
    pub latje_secs: f32,
}

/// Play one game to the end (without the billenbrand).
pub fn play(venue: usize, roster: [(usize, Ctrl, f32); 4], seed: u64) -> Game {
    let mut w = World::new(venue, roster, seed);
    let pads = [Pad::default(); 2];
    let mut latje = 0;
    for _ in 0..60 * 60 * 8 {
        w.update(&pads);
        if matches!(w.phase, Phase::Latje | Phase::CountOut) {
            latje += 1;
        }
        if matches!(w.phase, Phase::Over | Phase::Brand | Phase::Done) {
            break;
        }
    }
    let mut goals = [0; 5];
    for k in &w.kids {
        for (g, n) in goals.iter_mut().zip(k.stats.goals.iter()) {
            *g += n;
        }
    }
    Game { secs: w.play_t as f32 / 60.0, scored: w.kids[0].stats.scored, loser: w.loser.unwrap_or(9), dinner: w.dinner, goals, tally: w.tally, latje_secs: latje as f32 / 60.0 }
}

fn cpu_roster(venue: usize, extra: usize) -> [(usize, Ctrl, f32); 4] {
    let v = &VENUES[venue];
    [(PICKS[extra % 4], Ctrl::Cpu, v.skill), (v.kids[0], Ctrl::Cpu, v.skill), (v.kids[1], Ctrl::Cpu, v.skill), (v.kids[2], Ctrl::Cpu, v.skill)]
}

pub fn sim(n: u32) {
    let n = n.max(1);
    println!("TIENEN! sim: {} games per venue, four CPU kids", n);
    for (vi, v) in VENUES.iter().enumerate() {
        let mut secs = Vec::new();
        let mut goals = [0u32; 5];
        let mut tally = [0u32; 7];
        let mut dinner = 0;
        let mut latje = 0.0;
        let mut losers = [0u32; 4];
        for g in 0..n {
            let r = play(vi, cpu_roster(vi, g as usize), 1000 + g as u64 * 31 + vi as u64);
            secs.push(r.secs);
            for (a, b) in goals.iter_mut().zip(r.goals.iter()) {
                *a += b;
            }
            for (a, b) in tally.iter_mut().zip(r.tally.iter()) {
                *a += b;
            }
            if r.dinner {
                dinner += 1;
            }
            if r.loser < 4 {
                losers[r.loser] += 1;
            }
            latje += r.latje_secs;
        }
        secs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let avg = secs.iter().sum::<f32>() / n as f32;
        let nf = n as f32;
        println!("{} (skill {:.1})", v.name, v.skill);
        println!(
            "  game {:.0}s (median {:.0}, min {:.0}, max {:.0}), latjetrap {:.0}s, ETEN! {:.0}%",
            avg,
            secs[secs.len() / 2],
            secs[0],
            secs[secs.len() - 1],
            latje / nf,
            100.0 * dinner as f32 / nf
        );
        println!(
            "  goals per game: volley {:.1} chest {:.1} header {:.1} heel {:.1} bicycle {:.1}",
            goals[0] as f32 / nf,
            goals[1] as f32 / nf,
            goals[2] as f32 / nf,
            goals[3] as f32 / nf,
            goals[4] as f32 / nf
        );
        println!(
            "  wissels per game: caught {:.1} out {:.1} bounced {:.1} fence {:.1}; disallowed {:.1}, keeper's ball {:.1}, parries {:.1}",
            tally[0] as f32 / nf,
            tally[1] as f32 / nf,
            tally[2] as f32 / nf,
            tally[3] as f32 / nf,
            tally[4] as f32 / nf,
            tally[5] as f32 / nf,
            tally[6] as f32 / nf
        );
        println!("  losers by seat: {:?} (seat 0 = a pick)", losers);
    }
}

pub fn probe(n: u32) {
    let n = n.max(1);
    println!("TIENEN! probe: {} games per bot and venue", n);
    for kind in [BotKind::Decent, BotKind::Lazy, BotKind::Greedy] {
        let mut line = format!("{:?}:", kind);
        for (vi, v) in VENUES.iter().enumerate() {
            let mut wins = 0;
            let mut safe = 0;
            let mut scored = 0;
            let mut secs = 0.0;
            let mut dinner = 0;
            for g in 0..n {
                let pick = PICKS[g as usize % 4];
                let roster = [(pick, Ctrl::Probe(kind), 0.75), (v.kids[0], Ctrl::Cpu, v.skill), (v.kids[1], Ctrl::Cpu, v.skill), (v.kids[2], Ctrl::Cpu, v.skill)];
                let r = play(vi, roster, 5000 + g as u64 * 17 + vi as u64 * 3);
                if r.loser != 0 {
                    safe += 1;
                    if r.scored >= v.quota {
                        wins += 1;
                    }
                }
                scored += r.scored;
                if r.dinner {
                    dinner += 1;
                }
                secs += r.secs;
            }
            line += &format!(
                "  {} {:.0}% (safe {:.0}%, scored {:.1}/{}, {:.0}s{})",
                v.short,
                100.0 * wins as f32 / n as f32,
                100.0 * safe as f32 / n as f32,
                scored as f32 / n as f32,
                v.quota,
                secs / n as f32,
                if dinner > 0 { format!(", eten {}", dinner) } else { String::new() }
            );
        }
        println!("{}", line);
    }
}
