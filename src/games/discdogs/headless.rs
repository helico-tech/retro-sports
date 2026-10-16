//! Headless tools: scripted screenshots of every screen and key moment,
//! CPU-vs-CPU balance numbers (`sim`) and scripted "human" bots against
//! the CPU ladder (`probe`). No window needed.

use super::ai::Brain;
use super::app::{App, Mode, Scene, CPU_SKILL};
use super::data::*;
use super::render::*;
use super::scenery::Scenery;
use super::sim::*;
use super::sprites::*;
use crate::engine::gfx::*;
use crate::engine::png::{save_png, save_sheet};
use std::path::Path;

fn press(app: &mut App, p: Pad) {
    app.update([p, Pad::default()], false);
    app.update([Pad::default(); 2], false);
}

fn run(app: &mut App, n: usize) {
    for _ in 0..n {
        app.update([Pad::default(); 2], false);
    }
}

fn snap(app: &App, dir: &Path, name: &str) {
    let mut f = Frame::new();
    app.draw(&mut f);
    save_png(&f, &dir.join(name), 3);
    println!("wrote {}", name);
}

fn snap_match(m: &Match, sc: &Scenery, art: &Art, t: u32, dir: &Path, name: &str) {
    let mut f = Frame::new();
    draw_match(&mut f, m, sc, art, t);
    save_png(&f, &dir.join(name), 3);
    println!("wrote {}", name);
}

fn scenery_for(m: &Match, seed: u64) -> Scenery {
    let blankets: Vec<(f32, f32, u32)> = m.blankets.iter().map(|b| (b.x, b.w, b.color)).collect();
    Scenery::new(m.stage, m.pond, &blankets, seed)
}

const A: Pad = Pad { up: false, down: false, left: false, right: false, a: true, b: false, c: false, start: false };

pub fn shots(out: &str) {
    let dir = Path::new(out);
    std::fs::create_dir_all(dir).unwrap();
    let mut app = App::new(42);

    // ---- title, menu, select, intro
    run(&mut app, 70);
    snap(&app, dir, "01_title.png");
    run(&mut app, 100);
    snap(&app, dir, "01b_title_show.png");
    press(&mut app, A);
    run(&mut app, 8);
    snap(&app, dir, "02_title_menu.png");
    press(&mut app, A);
    run(&mut app, 30);
    snap(&app, dir, "03_select.png");
    press(&mut app, Pad { right: true, ..Pad::default() });
    run(&mut app, 5);
    press(&mut app, A);
    run(&mut app, 14);
    run(&mut app, 70);
    snap(&app, dir, "04_intro.png");

    // ---- a human toss: ready, aim, power, release, flight
    press(&mut app, A);
    run(&mut app, 20);
    snap(&app, dir, "05_ready.png");
    run(&mut app, 60);
    snap(&app, dir, "06_aim.png");
    for _ in 0..10 {
        app.update([Pad { up: true, ..Pad::default() }, Pad::default()], false);
    }
    app.update([Pad::default(); 2], false);
    for _ in 0..20 {
        app.update([A, Pad::default()], false);
    }
    snap(&app, dir, "07_power.png");
    for _ in 0..11 {
        app.update([A, Pad::default()], false);
    }
    snap(&app, dir, "08_power_perfect.png");
    app.update([Pad::default(); 2], false);
    run(&mut app, 3);
    snap(&app, dir, "09_release.png");
    // run the dog after it, leap near the end
    for k in 0..200 {
        let g = app.game.as_ref().unwrap();
        if !matches!(g.phase, Phase::Release | Phase::Flight) {
            break;
        }
        let near = (g.disc.x - g.dog.x).abs() < 16.0 && g.disc.z < 40.0 && g.disc.z > 20.0;
        let p = Pad { right: g.dog.x < g.disc.x + 8.0, b: near, ..Pad::default() };
        app.update([p, Pad::default()], false);
        if k == 40 {
            snap(&app, dir, "10_fetch.png");
        }
    }
    run(&mut app, 10);
    snap(&app, dir, "11_result.png");
    // pause
    press(&mut app, Pad { start: true, ..Pad::default() });
    snap(&app, dir, "12_pause.png");
    press(&mut app, Pad { start: true, ..Pad::default() });

    // ---- 2P select and intro
    let mut app2 = App::new(7);
    run(&mut app2, 30);
    press(&mut app2, A);
    press(&mut app2, Pad { down: true, ..Pad::default() });
    press(&mut app2, A);
    run(&mut app2, 10);
    app2.update([Pad::default(), Pad { down: true, ..Pad::default() }], false);
    run(&mut app2, 20);
    snap(&app2, dir, "13_select_2p.png");
    press(&mut app2, A);
    app2.update([Pad::default(), A], false);
    run(&mut app2, 90);
    snap(&app2, dir, "14_intro_2p.png");

    // ---- every stage, CPU vs CPU
    let art = Art::new();
    for (k, &stage) in STAGES.iter().enumerate() {
        let mut m = Match::new([k, (k + 1) % 4], [Ctrl::Cpu, Ctrl::Cpu], [0.7, 0.7], stage, 30 + k as u64);
        let sc = scenery_for(&m, 5);
        for _ in 0..200 {
            m.update(&[Pad::default(); 2]);
        }
        snap_match(&m, &sc, &art, 200, dir, &format!("15_stage_{}.png", k + 1));
        // the far end of the field
        let mut m2 = Match::new([k, (k + 1) % 4], [Ctrl::Cpu, Ctrl::Cpu], [0.7, 0.7], stage, 30 + k as u64);
        m2.cam = CAM_MAX;
        m2.phase = Phase::Flight;
        m2.dog.x = 370.0;
        m2.dog.state = DogState::Stand;
        m2.disc.state = DiscState::Gone;
        snap_match(&m2, &sc, &art, 300, dir, &format!("16_end_{}.png", k + 1));
    }

    // ---- key moments from CPU matches
    let events: &[&str] = &["jump", "flip", "flipcatch", "oof", "stare", "gull", "gulled", "swim", "soggy", "sausage", "snag", "miss", "sulk", "out", "over"];
    let mut got: Vec<&str> = Vec::new();
    let mut sheet: Vec<Frame> = Vec::new();
    for seed in 1..400u64 {
        if got.len() == events.len() {
            break;
        }
        let stage = STAGES[(seed % 3) as usize];
        let a = (seed % 4) as usize;
        let b = (a + 1 + (seed % 3) as usize) % 4;
        let skill = [0.45, 0.7, 0.9][(seed % 3) as usize];
        let mut m = Match::new([a, b], [Ctrl::Cpu, Ctrl::Cpu], [skill, skill], stage, seed);
        let sc = scenery_for(&m, seed);
        let mut prev = m.stats;
        let mut prev_gull = false;
        let mut pending: Vec<(&str, i32)> = Vec::new();
        let mut t = 0u32;
        let mut flip_seq = false;
        while !m.finished() || m.phase_t < 100 {
            t += 1;
            m.update(&[Pad::default(); 2]);
            m.sfx.clear();
            if t > 60 * 60 * 6 {
                break;
            }
            let st = m.stats;
            let fire = |name: &'static str, delay: i32, pending: &mut Vec<(&str, i32)>| {
                if !got.contains(&name) && !pending.iter().any(|p| p.0 == name) {
                    pending.push((name, delay));
                }
            };
            if m.dog.state == DogState::Air && m.dog.has_disc && m.dog.flip_t < 0 && m.dog.caught_air && m.dog.air_t > 3 {
                fire("jump", 0, &mut pending);
            }
            if m.dog.flip_t == FLIP_LEN / 2 {
                fire("flip", 0, &mut pending);
                if !flip_seq && sheet.is_empty() {
                    flip_seq = true;
                }
            }
            if st.flips > prev.flips {
                fire("flipcatch", 6, &mut pending);
            }
            if st.oofs > prev.oofs || (m.dog.state == DogState::Tumble && m.dog.tumble_t == 40) {
                fire("oof", 4, &mut pending);
            }
            if m.dog.state == DogState::Stare && m.dog.stare_t == m.dog.body.stare - 12 {
                fire("stare", 0, &mut pending);
            }
            let gull_now = m.gull.is_some();
            if gull_now && !prev_gull {
                fire("gull", 30, &mut pending);
            }
            prev_gull = gull_now;
            if st.gulls > prev.gulls {
                fire("gulled", 25, &mut pending);
            }
            if m.dog.state == DogState::Swim && m.phase == Phase::Flight {
                fire("swim", 10, &mut pending);
            }
            if st.soggy > prev.soggy {
                fire("soggy", 20, &mut pending);
            }
            if st.sausages > prev.sausages {
                fire("sausage", 2, &mut pending);
            }
            if st.snags > prev.snags {
                fire("snag", 2, &mut pending);
            }
            if st.dropped > prev.dropped {
                fire("miss", 50, &mut pending);
            }
            if m.phase == Phase::Aim && m.dog.state == DogState::Lie && m.phase_t == 30 {
                fire("sulk", 0, &mut pending);
            }
            if st.out > prev.out {
                fire("out", 30, &mut pending);
            }
            if m.finished() && m.phase_t == 40 {
                fire("over", 0, &mut pending);
            }
            prev = st;
            if flip_seq && sheet.len() < 16 && t % 3 == 0 {
                let mut f = Frame::new();
                draw_match(&mut f, &m, &sc, &art, t);
                sheet.push(f);
            }
            if flip_seq && (m.phase != Phase::Flight && m.phase != Phase::Result) {
                flip_seq = false;
            }
            for p in pending.iter_mut() {
                p.1 -= 1;
            }
            while let Some(i) = pending.iter().position(|p| p.1 < 0) {
                let (name, _) = pending.remove(i);
                let n = events.iter().position(|e| *e == name).unwrap() + 20;
                snap_match(&m, &sc, &art, t, dir, &format!("{}_{}.png", n, name));
                got.push(name);
            }
        }
    }
    if !sheet.is_empty() {
        save_sheet(&sheet, 4, &dir.join("40_flip_sequence.png"), 2);
        println!("wrote 40_flip_sequence.png");
    }
    // the rare ones, forced: a greedy flipper for an OOF, a huge floater for the sea
    if !got.contains(&"oof") {
        for seed in 0..60u64 {
            let mut m = Match::new([1, 2], [Ctrl::Human(0), Ctrl::Cpu], [0.5, 0.5], Stage::Park, 500 + seed);
            let sc = scenery_for(&m, seed);
            let mut bot = Brain::human("greedy", seed);
            let mut t = 0u32;
            let mut done = false;
            while !m.finished() && !done {
                t += 1;
                let p = if m.thrower() == 0 { bot.pad(&m, 0) } else { Pad::default() };
                m.update(&[p, Pad::default()]);
                m.sfx.clear();
                if m.dog.state == DogState::Tumble && m.dog.tumble_t == 38 {
                    snap_match(&m, &sc, &art, t, dir, "23_oof.png");
                    done = true;
                    got.push("oof");
                }
            }
            if done {
                break;
            }
        }
    }
    for (k, &stage) in STAGES.iter().enumerate() {
        let mut m = Match::new([3, 0], [Ctrl::Human(0), Ctrl::Cpu], [0.5, 0.5], stage, 77);
        let sc = scenery_for(&m, 77);
        let mut t = 0u32;
        let mut boosted = false;
        while m.phase != Phase::Result && t < 3000 {
            t += 1;
            m.wind = 0.8;
            if m.phase == Phase::Flight && !boosted {
                // a throw for the ages
                m.disc.vx *= 1.7;
                boosted = true;
            }
            let p = match m.phase {
                Phase::Aim => Pad { a: m.phase_t > 10, up: m.angle < 30.0, ..Pad::default() },
                Phase::Power => Pad { a: m.meter_t < METER_HALF - 1, ..Pad::default() },
                _ => Pad::default(),
            };
            m.update(&[p, Pad::default()]);
            m.sfx.clear();
        }
        for _ in 0..50 {
            t += 1;
            m.update(&[Pad::default(); 2]);
        }
        snap_match(&m, &sc, &art, t, dir, &format!("33_out_{}.png", k + 1));
    }
    got.push("out");
    let missing: Vec<&&str> = events.iter().filter(|e| !got.contains(e)).collect();
    if !missing.is_empty() {
        println!("not captured: {:?}", missing);
    }

    // ---- demo, results, continue, game over, champion
    let mut app3 = App::new(9);
    run(&mut app3, 1000);
    run(&mut app3, 500);
    snap(&app3, dir, "41_demo.png");
    let mut app4 = App::new(13);
    app4.mode = Mode::One;
    app4.cursor = [1, 0];
    app4.ladder = vec![2, 3, 0];
    app4.stage = 0;
    app4.start_match([1, 2], [Ctrl::Cpu, Ctrl::Cpu], [0.8, 0.5], Stage::Park);
    for _ in 0..60 * 60 * 5 {
        app4.update([Pad::default(); 2], false);
        if app4.scene == Scene::Results {
            break;
        }
    }
    run(&mut app4, 80);
    snap(&app4, dir, "42_results_tally.png");
    run(&mut app4, 160);
    snap(&app4, dir, "43_results_final.png");
    app4.scene = Scene::Continue;
    app4.t = 0;
    app4.count = 9;
    run(&mut app4, 70);
    snap(&app4, dir, "44_continue.png");
    app4.scene = Scene::GameOver;
    app4.t = 0;
    run(&mut app4, 40);
    snap(&app4, dir, "45_game_over.png");
    app4.make_confetti();
    app4.scene = Scene::Champion;
    app4.t = 0;
    run(&mut app4, 130);
    snap(&app4, dir, "46_champion.png");
    app4.stage = 2;
    app4.mode = Mode::One;
    app4.scene = Scene::Intro;
    app4.t = 0;
    run(&mut app4, 110);
    snap(&app4, dir, "47_intro_stage3.png");

    // ---- sprite sheets
    sheet_png(&dir.join("48_dogs.png"), &dir.join("49_owners_props.png"));
}

/// Every dog frame for every breed, and the owners and props, at 4x.
pub fn sheet_png(dogs: &Path, rest: &Path) {
    let art = Art::new();
    // two breeds per frame, 7 frames a row, cells 36 wide
    let mut frames = Vec::new();
    for half in 0..2 {
        let mut f = Frame::new();
        f.clear(rgb(96, 136, 104));
        for b in 0..2 {
            let pair = half * 2 + b;
            let pal = dog_pal(pair, rgb(232, 56, 56));
            for (k, fr) in DOG_FRAMES.iter().enumerate() {
                let (cx, cy) = ((k % 7) as i32, (k / 7) as i32);
                let x = 18 + cx * 36;
                let y = 32 + (b * 3) as i32 * 36 + cy * 36;
                if (cx + cy) % 2 == 0 {
                    f.rect(x - 18, y - 32, 36, 36, rgb(88, 128, 96));
                }
                draw(&mut f, art.dogs.get(pair, *fr), x, y, false, &pal);
            }
        }
        frames.push(f);
    }
    save_sheet(&frames, 2, dogs, 3);
    let mut g = Frame::new();
    g.clear(rgb(96, 136, 104));
    for pair in 0..4 {
        for (k, p) in OWNER_POSES.iter().enumerate() {
            crate::engine::sprites::blit(&mut g, art.owners.get(pair, *p), 14 + k as i32 * 26, 40 + pair as i32 * 36, false, &owner_pal(pair));
        }
    }
    for (k, s) in art.props.gull.iter().enumerate() {
        draw(&mut g, s, 20 + k as i32 * 22, 186, false, &prop_pal("gull"));
    }
    for (k, s) in art.props.squirrel.iter().enumerate() {
        draw(&mut g, s, 96 + k as i32 * 16, 190, false, &prop_pal("squirrel"));
    }
    draw(&mut g, &art.props.sausage, 150, 190, false, &prop_pal("sausage"));
    art.props.sleeper.draw(&mut g, 180, 190, false);
    art.props.sleeper_up.draw(&mut g, 210, 190, false);
    art.props.kid.draw(&mut g, 240, 190, false);
    for k in 0..3 {
        draw_disc(&mut g, 30 + k * 16, 212, rgb(248, 72, 168), k - 1, k as f32 * 3.0);
    }
    windsock(&mut g, 100, 214, 0.8, 0);
    save_png(&g, rest, 4);
}

// ------------------------------------------------------------------ sim

fn report(label: &str, st: &Stats, games: u32) {
    let t = st.tosses.max(1) as f32;
    let c = st.catches.max(1) as f32;
    println!(
        "{:<10} catch {:>3.0}%  throw {:>4.1}yd  caught at {:>4.1}yd  pts/toss {:>5.1}  G{:.0} R{:.0} S{:.0} J{:.0} F{:.0}%  oof {:.1}% gull {:.1}% out {:.1}%  flips {}/{}  perf {:.0}% float {:.0}%  match {:.0}s",
        label,
        100.0 * st.catches as f32 / t,
        st.throw_yards as f32 / t,
        st.yards_caught as f32 / c,
        st.points as f32 / t,
        100.0 * st.ground as f32 / c,
        100.0 * st.rolling as f32 / c,
        100.0 * st.soggy as f32 / c,
        100.0 * st.jumps as f32 / c,
        100.0 * st.flips as f32 / c,
        100.0 * st.oofs as f32 / t,
        100.0 * st.gulls as f32 / t,
        100.0 * st.out as f32 / t,
        st.flips,
        st.flips_tried,
        100.0 * st.perfects as f32 / t,
        100.0 * st.floaters as f32 / t,
        st.frames as f32 / 60.0 / games.max(1) as f32,
    );
}

fn add(a: &mut Stats, b: &Stats) {
    a.tosses += b.tosses;
    a.catches += b.catches;
    a.ground += b.ground;
    a.rolling += b.rolling;
    a.soggy += b.soggy;
    a.jumps += b.jumps;
    a.flips += b.flips;
    a.oofs += b.oofs;
    a.gulls += b.gulls;
    a.out += b.out;
    a.dropped += b.dropped;
    a.sunk += b.sunk;
    a.flips_tried += b.flips_tried;
    a.yards_caught += b.yards_caught;
    a.throw_yards += b.throw_yards;
    a.points += b.points;
    a.perfects += b.perfects;
    a.floaters += b.floaters;
    a.shoulders += b.shoulders;
    a.sausages += b.sausages;
    a.stares += b.stares;
    a.snags += b.snags;
    a.sudden += b.sudden;
    a.frames += b.frames;
}

fn play_out(m: &mut Match, human: Option<&mut Brain>) {
    let mut f = 0;
    let mut h = human;
    while !m.finished() && f < 60 * 60 * 10 {
        let p = match h.as_deref_mut() {
            Some(b) => b.pad(m, 0),
            None => Pad::default(),
        };
        m.update(&[p, Pad::default()]);
        m.sfx.clear();
        f += 1;
    }
}

/// CPU vs CPU at four skill levels: catch rate, distances, kinds of catch,
/// mishaps, and win rates per dog.
pub fn sim(n: usize) {
    let n = n.max(4) as u32;
    println!("CPU vs CPU, {} matches per skill (both sides the same skill)", n);
    for &skill in &[CPU_SKILL[0], CPU_SKILL[1], 0.7, CPU_SKILL[2], 1.0] {
        let mut st = Stats::default();
        let mut per = [(0u32, 0u32, 0i32, 0u32, 0u32, 0i32, 0i32); 4];
        let mut stage_pts = [(0i32, 0u32); 3];
        for k in 0..n {
            let a = (k % 4) as usize;
            let b = ((k / 4 + 1 + k % 3) % 4) as usize;
            let b = if a == b { (b + 1) % 4 } else { b };
            let stage = STAGES[(k % 3) as usize];
            let mut m = Match::new([a, b], [Ctrl::Cpu, Ctrl::Cpu], [skill, skill], stage, k as u64 + 1);
            play_out(&mut m, None);
            add(&mut st, &m.stats);
            for (s, side) in m.sides.iter().enumerate() {
                let p = &mut per[side.pair];
                p.4 += 1;
                if m.winner == Some(s) {
                    p.3 += 1;
                }
                for r in &side.results {
                    p.0 += 1;
                    if r.caught() {
                        p.1 += 1;
                        p.5 += r.yards;
                        if let Outcome::Catch(k) = r.outcome {
                            p.6 += k.mult();
                        }
                    }
                    p.2 += r.points();
                    stage_pts[stage.index()].0 += r.points();
                    stage_pts[stage.index()].1 += 1;
                }
            }
        }
        report(&format!("skill {:.1}", skill), &st, n);
        let line: Vec<String> = per
            .iter()
            .enumerate()
            .map(|(k, p)| {
                format!(
                    "{} {:.0}% win {:.0}% {:.0}pt {:.0}yd x{:.1}",
                    PAIRS[k].dog.name.split(' ').next_back().unwrap_or(""),
                    100.0 * p.1 as f32 / p.0.max(1) as f32,
                    100.0 * p.3 as f32 / p.4.max(1) as f32,
                    p.2 as f32 / p.0.max(1) as f32,
                    p.5 as f32 / p.1.max(1) as f32,
                    p.6 as f32 / p.1.max(1) as f32
                )
            })
            .collect();
        println!("           catch/win per dog: {}", line.join(" | "));
        println!(
            "           pts/toss by stage: park {:.1} pier {:.1} bowl {:.1}   stares/match {:.2} sausages/match {:.2} snags/match {:.2} sudden dog {}",
            stage_pts[0].0 as f32 / stage_pts[0].1.max(1) as f32,
            stage_pts[1].0 as f32 / stage_pts[1].1.max(1) as f32,
            stage_pts[2].0 as f32 / stage_pts[2].1.max(1) as f32,
            st.stares as f32 / n as f32,
            st.sausages as f32 / n as f32,
            st.snags as f32 / n as f32,
            st.sudden
        );
    }
}

// ------------------------------------------------------------------ probe

fn short(pair: usize) -> &'static str {
    PAIRS[pair].dog.name.split(' ').next_back().unwrap_or("")
}

/// Scripted "human" bots against each step of the 1P ladder.
pub fn probe(per: u32) {
    let per = per.max(1);
    println!("bot vs CPU ladder, {} matches per dog per stage", per);
    println!("{:<8} {:>5} {:>6} {:>6} {:>8} {:>8} {:>8} {:>6}  {}", "bot", "stage", "win%", "catch%", "pts/toss", "cpu pts", "flips", "oof%", "misses: drop/gull/out/sunk, catches: G/R/S/J/F");
    for style in ["lazy", "safe", "greedy", "decent"] {
        for stage in 0..3 {
            let (mut wins, mut games) = (0u32, 0u32);
            let mut dog_wins = [0u32; 4];
            let mut me = Stats::default();
            let (mut my_pts, mut my_n, mut cpu_pts, mut cpu_n) = (0i32, 0u32, 0i32, 0u32);
            for my in 0..4 {
                let opp = (my + 1 + stage) % 4;
                for s in 0..per {
                    let seed = 1000 + s as u64 * 31 + my as u64 * 7 + stage as u64 * 101;
                    let mut m = Match::new([my, opp], [Ctrl::Human(0), Ctrl::Cpu], [0.5, CPU_SKILL[stage]], stage_for(stage), seed);
                    let mut bot = Brain::human(style, seed ^ 0xBEEF);
                    let before = m.stats;
                    play_out(&mut m, Some(&mut bot));
                    let _ = before;
                    games += 1;
                    if m.winner == Some(0) {
                        wins += 1;
                        dog_wins[my] += 1;
                    }
                    for r in &m.sides[0].results {
                        my_pts += r.points();
                        my_n += 1;
                        me.tosses += 1;
                        if r.caught() {
                            me.catches += 1;
                        }
                        match r.outcome {
                            Outcome::Miss(MissKind::Oof) => me.oofs += 1,
                            Outcome::Miss(MissKind::Dropped) => me.dropped += 1,
                            Outcome::Miss(MissKind::Seagull) => me.gulls += 1,
                            Outcome::Miss(MissKind::Sunk) => me.sunk += 1,
                            Outcome::Miss(_) => me.out += 1,
                            Outcome::Catch(CatchKind::Flip) => me.flips += 1,
                            Outcome::Catch(CatchKind::Jump) => me.jumps += 1,
                            Outcome::Catch(CatchKind::Ground) => me.ground += 1,
                            Outcome::Catch(CatchKind::Rolling) => me.rolling += 1,
                            Outcome::Catch(CatchKind::Soggy) => me.soggy += 1,
                        }
                    }
                    for r in &m.sides[1].results {
                        cpu_pts += r.points();
                        cpu_n += 1;
                    }
                }
            }
            println!(
                "{:<8} {:>5} {:>6.0} {:>6.0} {:>8.1} {:>8.1} {:>8} {:>6.1}  {}/{}/{}/{}  {}/{}/{}/{}/{}",
                style,
                stage + 1,
                100.0 * wins as f32 / games as f32,
                100.0 * me.catches as f32 / me.tosses.max(1) as f32,
                my_pts as f32 / my_n.max(1) as f32,
                cpu_pts as f32 / cpu_n.max(1) as f32,
                format!("{}", me.flips),
                100.0 * me.oofs as f32 / me.tosses.max(1) as f32,
                me.dropped,
                me.gulls,
                me.out,
                me.sunk,
                me.ground,
                me.rolling,
                me.soggy,
                me.jumps,
                me.flips
            );
            if style == "decent" {
                let per_dog: Vec<String> = (0..4).map(|k| format!("{} {:.0}%", short(k), 100.0 * dog_wins[k] as f32 / per as f32)).collect();
                println!("{:<14} wins by dog: {}", "", per_dog.join("  "));
            }
        }
    }
}
