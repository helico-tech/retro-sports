//! Headless balance tools: `sim` runs the CPU thief at three skill levels
//! on every stage; `probe` runs human-like bots (decent, reckless, sneaky,
//! lazy) through the ladder with every thief.

use super::bot::{Bot, BotKind};
use super::data::*;
use super::world::*;
use crate::engine::input::Pad;

#[derive(Clone, Debug)]
pub struct RunResult {
    pub outcome: Outcome,
    pub secs: f32,
    pub fines: u32,
    pub cash: i32,
    pub delivered: usize,
    pub chases: u32,
    pub cutters: u32,
    pub swims: u32,
    pub steals: u32,
    pub clanks: u32,
}

/// Play one stage with a bot as player 1, to the end.
pub fn run_stage(stage: usize, who: Who, kind: BotKind, seed: u64) -> RunResult {
    let mut w = World::new(stage, who, Ctrl::Human(0), None, seed, None);
    let mut bot = Bot::new(kind, seed ^ 0x5EED);
    let mut n = 0;
    while !w.finished() && n < 60 * 60 * 6 {
        let p = bot.pad(&w);
        w.update(&[p, Pad::default()]);
        n += 1;
    }
    let th = &w.thief;
    RunResult {
        outcome: w.result.unwrap_or(Outcome::TimeUp),
        secs: (w.def.time * 60 - w.time_left) as f32 / 60.0,
        fines: th.fines,
        cash: w.cash(),
        delivered: th.delivered.len(),
        chases: th.stats.chases,
        cutters: th.stats.cutters,
        swims: th.stats.swims,
        steals: th.stats.steals,
        clanks: th.stats.clanks,
    }
}

fn summary(rs: &[RunResult]) -> String {
    let n = rs.len().max(1) as f32;
    let clear: Vec<&RunResult> = rs.iter().filter(|r| r.outcome == Outcome::Cleared).collect();
    let busted = rs.iter().filter(|r| r.outcome == Outcome::Busted).count() as f32;
    let t = if clear.is_empty() { 0.0 } else { clear.iter().map(|r| r.secs).sum::<f32>() / clear.len() as f32 };
    let avg = |f: &dyn Fn(&RunResult) -> f32| rs.iter().map(f).sum::<f32>() / n;
    format!(
        "clear {:>3.0}%  busted {:>3.0}%  time {:>5.1}s  fines {:.2}  chases {:.1}  delivered {:.1}  cutters {:.1}  swims {:.2}  clanks {:.1}  cash {:>4.0}",
        clear.len() as f32 / n * 100.0,
        busted / n * 100.0,
        t,
        avg(&|r| r.fines as f32),
        avg(&|r| r.chases as f32),
        avg(&|r| r.delivered as f32),
        avg(&|r| r.cutters as f32),
        avg(&|r| r.swims as f32),
        avg(&|r| r.clanks as f32),
        avg(&|r| r.cash as f32),
    )
}

pub fn sim(n: u32) {
    let n = n.max(1);
    for (si, st) in STAGES.iter().enumerate() {
        println!("{}", st.name);
        for skill in [0.3f32, 0.6, 0.9] {
            let rs: Vec<RunResult> = (0..n).map(|k| run_stage(si, THIEVES[k as usize % 4], BotKind::Cpu(skill), 1000 + k as u64 * 7)).collect();
            println!("  cpu {:.1}  {}", skill, summary(&rs));
        }
    }
}

pub fn probe(n: u32) {
    let n = n.max(1);
    for kind in [BotKind::Decent, BotKind::Reckless, BotKind::Sneaky, BotKind::Lazy] {
        println!("{}", kind.name());
        for (si, st) in STAGES.iter().enumerate() {
            let rs: Vec<RunResult> = (0..n).map(|k| run_stage(si, THIEVES[k as usize % 4], kind, 5000 + k as u64 * 13)).collect();
            println!("  {:<16} {}", st.name, summary(&rs));
        }
        if kind == BotKind::Decent {
            for who in THIEVES {
                let line: Vec<String> = (0..STAGES.len())
                    .map(|si| {
                        let c = (0..n).filter(|&k| run_stage(si, who, kind, 9000 + k as u64 * 17).outcome == Outcome::Cleared).count();
                        format!("{:>3.0}%", c as f32 / n as f32 * 100.0)
                    })
                    .collect();
                println!("    {:<10} {}", who.def().name, line.join("  "));
            }
        }
    }
}
