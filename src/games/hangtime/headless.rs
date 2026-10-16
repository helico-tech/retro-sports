//! Headless tools: scripted screenshots of every screen, a CPU simulation
//! and probe bots ("humans" with a style) for balance.

use super::ai::{Brain, Style};
use super::app::{App, Mode, Scene, LADDER};
use super::course::*;
use super::flight::*;
use crate::engine::gfx::Frame;
use crate::engine::input::Pad;
use crate::engine::png::{save_png, save_sheet};
use std::path::Path;

fn press(app: &mut App, p: Pad) {
    app.update([p, Pad::default()], false);
    app.update([Pad::default(); 2], false);
}

fn a() -> Pad {
    Pad { a: true, ..Pad::default() }
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

/// Step the app with player 1 (and 2) flown by brains.
fn fly_step(app: &mut App, brains: &mut [Brain; 2]) {
    let mut pads = [Pad::default(); 2];
    if let Some(r) = &app.race {
        for (k, b) in brains.iter_mut().enumerate() {
            if k < r.pilots.len() && matches!(r.pilots[k].ctrl, Ctrl::Human(_)) {
                pads[k] = b.think(r, k);
            }
        }
    }
    app.update(pads, false);
}

pub fn shots(out: &str) {
    let dir = Path::new(out);
    std::fs::create_dir_all(dir).unwrap();
    let mut app = App::new(42);
    run(&mut app, 70);
    snap(&app, dir, "01_title.png");
    press(&mut app, a());
    run(&mut app, 8);
    snap(&app, dir, "02_title_menu.png");
    press(&mut app, a());
    run(&mut app, 30);
    snap(&app, dir, "03_select.png");
    press(&mut app, Pad { down: true, ..Pad::default() });
    run(&mut app, 5);
    press(&mut app, a());
    run(&mut app, 70);
    snap(&app, dir, "04_intro.png");
    press(&mut app, a());
    run(&mut app, 40);
    snap(&app, dir, "05_ready.png");

    // fly stage 1 with a decent bot in the player's seat
    let mut brains = [Brain::new(0.7, Style::Normal, 3), Brain::new(0.6, Style::Normal, 4)];
    let mut seen: Vec<String> = Vec::new();
    let mut periodic = 0;
    let mut n = 0;
    let mut paused_shot = false;
    let mut sheet: Vec<Frame> = Vec::new();
    while app.scene == Scene::Play && n < 60 * 60 * 6 {
        fly_step(&mut app, &mut brains);
        n += 1;
        let r = app.race.as_ref().unwrap();
        let p = &r.pilots[0];
        if matches!(p.state, State::Run) && seen.iter().all(|s| s != "run") && p.x > r.course.launch.x0 + 20.0 {
            seen.push("run".into());
            snap(&app, dir, "06_run.png");
        }
        if let Some(b) = &r.banners[0]
            && b.t == 12 && !seen.contains(&b.text) {
                seen.push(b.text.clone());
                let name = format!("07_{}.png", b.text.to_lowercase().replace([' ', '!', '?'], "_").trim_end_matches('_'));
                snap(&app, dir, &name);
            }
        // a turn in progress, frame by frame
        if p.turn > 0 && sheet.len() < 8 && p.airborne() {
            let mut f = Frame::new();
            app.draw(&mut f);
            sheet.push(f);
        }
        if p.vario > 0.3 && seen.iter().all(|s| s != "climb") && p.airborne() {
            seen.push("climb".into());
            snap(&app, dir, "08_thermal_climb.png");
        }
        if n % 600 == 300 && periodic < 10 {
            periodic += 1;
            snap(&app, dir, &format!("09_play_{:02}.png", periodic));
        }
        if p.state == State::Fly && p.agl(&r.course) < 7.0 && p.clock > 600 && seen.iter().all(|s| s != "final") {
            seen.push("final".into());
            snap(&app, dir, "11_final_approach.png");
        }
        let landed = p.state == State::Done;
        if !paused_shot && n == 1500 {
            paused_shot = true;
            press(&mut app, Pad { start: true, ..Pad::default() });
            snap(&app, dir, "10_pause.png");
            press(&mut app, Pad { start: true, ..Pad::default() });
            continue;
        }
        if landed && seen.iter().all(|s| s != "landed") {
            seen.push("landed".into());
            snap(&app, dir, "11_landed_done.png");
        }
    }
    if !sheet.is_empty() {
        save_sheet(&sheet, 4, &dir.join("12_turn_sequence.png"), 2);
        println!("wrote 12_turn_sequence.png");
    }
    if app.scene == Scene::Results {
        run(&mut app, 40);
        snap(&app, dir, "13_results_tally.png");
        run(&mut app, 200);
        snap(&app, dir, "14_results.png");
    }

    // the other venues: a CPU flight, shots along the way
    for (k, venue) in VENUES.iter().enumerate() {
        let mut d = App::new(100 + k as u64);
        d.mode = Mode::One;
        d.cursor[0] = k + 1;
        d.stage = k;
        let mut brains = [Brain::new(0.8, Style::Normal, 9 + k as u64), Brain::new(0.6, Style::Normal, 4)];
        // go through select -> intro -> play like a player
        run(&mut d, 25);
        press(&mut d, a());
        press(&mut d, a());
        for _ in 0..((k + 1) % 4) {
            press(&mut d, Pad { right: true, ..Pad::default() });
        }
        d.stage = k;
        press(&mut d, a());
        run(&mut d, 12);
        if d.scene == Scene::Intro {
            // the select screen always starts at stage 1: jump to this venue
            d.stage = k;
            d.race = None;
            d.prepare_stage();
            run(&mut d, 80);
            snap(&d, dir, &format!("15_intro_{}.png", k + 1));
            press(&mut d, a());
        }
        let mut shot = 0;
        let mut storm = false;
        let mut m = 0;
        while d.scene == Scene::Play && m < 60 * 60 * 5 {
            fly_step(&mut d, &mut brains);
            m += 1;
            let r = d.race.as_ref().unwrap();
            if m % 900 == 450 && shot < 5 {
                shot += 1;
                snap(&d, dir, &format!("16_{}_{:02}.png", venue_tag(*venue), shot));
            }
            if !storm && r.pilots.iter().any(|p| matches!(p.state, State::Suck(60))) {
                storm = true;
                snap(&d, dir, "17_cloudsuck.png");
            }
        }
    }

    // the storm cloud up close (whoever flies near it)
    let mut s = App::new(77);
    s.mode = Mode::One;
    s.stage = 2;
    s.prepare_stage();
    s.scene = Scene::Play;
    if let Some(r) = s.race.as_mut() {
        let st = r.course.storm.unwrap();
        let p = &mut r.pilots[0];
        p.state = State::Fly;
        p.x = st.x - 120.0;
        p.y = st.base - 60.0;
        p.v = 0.8;
    }
    s.snap_cams();
    for _ in 0..30 {
        s.update([Pad::default(); 2], false);
    }
    snap(&s, dir, "18_storm.png");
    let mut k = 0;
    while k < 400 {
        s.update([Pad::default(); 2], false);
        k += 1;
        if s.race.as_ref().map(|r| matches!(r.pilots[0].state, State::Suck(70))).unwrap_or(false) {
            snap(&s, dir, "19_cloudsuck_banner.png");
        }
    }

    // geese, UFO and the lake splash, staged
    let mut g = App::new(78);
    g.mode = Mode::One;
    g.stage = 1;
    g.prepare_stage();
    g.scene = Scene::Play;
    if let Some(r) = g.race.as_mut() {
        r.t = r.flocks[1].t0 + 200;
        let (gx, gy) = r.flocks[1].pos(r.t, r.course.wind).unwrap();
        let p = &mut r.pilots[0];
        p.state = State::Fly;
        p.x = gx - 70.0;
        p.y = gy;
        p.v = 0.7;
        r.ufo = Some(Ufo { x: gx + 110.0, y: gy + 50.0, t: 100 });
    }
    g.snap_cams();
    for k in 0..90 {
        g.update([Pad { up: k % 3 == 0, ..Pad::default() }, Pad::default()], false);
    }
    snap(&g, dir, "20_geese_ufo.png");
    if let Some(r) = g.race.as_mut() {
        let p = &mut r.pilots[0];
        p.x = 2000.0;
        p.y = 76.0;
        p.v = 0.8;
        p.state = State::Fly;
        p.turn = 0;
    }
    g.snap_cams();
    for _ in 0..80 {
        g.update([Pad::default(); 2], false);
    }
    snap(&g, dir, "21_splash.png");

    // a sheep landing
    let mut h = App::new(79);
    h.mode = Mode::One;
    h.stage = 0;
    h.prepare_stage();
    h.scene = Scene::Play;
    if let Some(r) = h.race.as_mut() {
        // a dizzy sheep does not run away
        r.sheep[0].mode = SheepMode::Dizzy;
        r.sheep[0].t = -400;
        let sx = r.sheep[0].x;
        let p = &mut r.pilots[0];
        p.state = State::Flare(0);
        p.x = sx - 3.0;
        p.y = r.course.ground_at(sx) + 2.0;
        p.v = 0.5;
        p.vy = 0.1;
    }
    h.snap_cams();
    for _ in 0..24 {
        h.update([Pad::default(); 2], false);
    }
    snap(&h, dir, "22_baaad_landing.png");

    // 2P split screen
    let mut t = App::new(7);
    run(&mut t, 30);
    press(&mut t, a());
    press(&mut t, Pad { down: true, ..Pad::default() });
    press(&mut t, a());
    run(&mut t, 20);
    snap(&t, dir, "23_select_2p.png");
    press(&mut t, a());
    t.update([Pad::default(), a()], false);
    t.update([Pad::default(); 2], false);
    run(&mut t, 70);
    snap(&t, dir, "24_intro_2p.png");
    press(&mut t, a());
    let mut brains = [Brain::new(0.7, Style::Normal, 3), Brain::new(0.5, Style::Normal, 4)];
    for k in 0..60 * 60 * 6 {
        if t.scene != Scene::Play {
            break;
        }
        fly_step(&mut t, &mut brains);
        if k == 300 || k == 2400 || k == 4800 {
            snap(&t, dir, &format!("25_split_{}.png", k / 60));
        }
    }
    if t.scene == Scene::Results {
        run(&mut t, 260);
        snap(&t, dir, "26_results_2p.png");
    }

    // attract mode
    let mut dm = App::new(5);
    run(&mut dm, 960);
    run(&mut dm, 600);
    snap(&dm, dir, "27_demo.png");

    // end screens
    let mut e = App::new(9);
    e.mode = Mode::One;
    e.cursor[0] = 2;
    e.scene = Scene::Continue;
    e.count = 9;
    run(&mut e, 70);
    snap(&e, dir, "28_continue.png");
    e.scene = Scene::GameOver;
    e.t = 0;
    run(&mut e, 40);
    snap(&e, dir, "29_game_over.png");
    e.make_confetti();
    e.scene = Scene::Champion;
    e.t = 200;
    run(&mut e, 100);
    snap(&e, dir, "30_champion.png");

    // sprite sheets
    let art = super::art::Art::new();
    for (k, f) in super::render::sprite_sheet(&art).iter().enumerate() {
        save_png(f, &dir.join(format!("31_sprites_{k}.png")), 3);
        println!("wrote 31_sprites_{k}.png");
    }
}

fn venue_tag(v: Venue) -> &'static str {
    match v {
        Venue::Dunes => "dunes",
        Venue::Alps => "alps",
        Venue::Canyon => "canyon",
    }
}

/// Fly one race to the end; returns the race.
pub fn fly(venue: Venue, entries: &[Entry], seed: u64) -> Race {
    let mut r = Race::new(venue, entries, seed);
    let mut n = 0;
    while !r.finished() && n < 60 * 60 * 12 {
        r.update(&[Pad::default(); 2]);
        r.sfx.clear();
        n += 1;
    }
    r
}

#[derive(Default)]
struct Agg {
    n: f64,
    done: f64,
    score: f64,
    time: f64,
    gates: f64,
    field: f64,
    dist: f64,
    stalls: f64,
    soft: f64,
    crash: f64,
    splash: f64,
}

impl Agg {
    fn add(&mut self, r: &Race, p: &Pilot) {
        let c = &r.course;
        self.n += 1.0;
        self.done += (p.state == State::Done) as i32 as f64;
        self.score += p.total() as f64;
        self.time += (p.clock + p.walk_frames) as f64 / 60.0;
        self.gates += p.gates_made() as f64;
        let ok = c.in_field(p.land_x) && !matches!(p.landing, Some(Landing::Crash | Landing::Splash));
        self.field += ok as i32 as f64;
        if ok {
            self.dist += (p.land_x - c.target).abs() as f64;
        }
        self.stalls += p.stalls as f64;
        self.soft += matches!(p.landing, Some(Landing::Soft | Landing::Perfect)) as i32 as f64;
        self.crash += (p.landing == Some(Landing::Crash)) as i32 as f64;
        self.splash += (p.landing == Some(Landing::Splash)) as i32 as f64;
    }
    fn line(&self) -> String {
        let n = self.n.max(1.0);
        format!(
            "done {:>3.0}% score {:>5.0} time {:>4.0}s gates {:.2} field {:>3.0}% off {:>3.0}px soft {:>3.0}% crash {:>2.0}% splash {:>2.0}% stalls {:.2}",
            100.0 * self.done / n,
            self.score / n,
            self.time / n,
            self.gates / n,
            100.0 * self.field / n,
            self.dist / self.field.max(1.0),
            100.0 * self.soft / n,
            100.0 * self.crash / n,
            100.0 * self.splash / n,
            self.stalls / n
        )
    }
}

/// CPU pilots only: completion, scores, landing accuracy per venue and skill.
pub fn sim(n: u32) {
    println!("CPU pilots, {} races per row (4 pilots each)", n);
    for venue in VENUES {
        for &skill in &[0.2f32, 0.4, 0.6, 0.8] {
            let mut agg = Agg::default();
            let mut by_def = [0f64; 4];
            for s in 0..n {
                let entries: Vec<Entry> = (0..4).map(|k| (k, Ctrl::Cpu, skill, Style::Normal)).collect();
                let r = fly(venue, &entries, s as u64 * 31 + 7);
                for p in &r.pilots {
                    agg.add(&r, p);
                    by_def[p.def] += p.total() as f64 / n as f64;
                }
            }
            println!("{:<15} skill {:.1}: {} | by pilot {:.0} {:.0} {:.0} {:.0}", venue.name(), skill, agg.line(), by_def[0], by_def[1], by_def[2], by_def[3]);
        }
    }
    // the ladder as it is played
    println!("ladder rivals (stage skills {:?})", LADDER);
    for (k, venue) in VENUES.iter().enumerate() {
        let mut best = 0.0;
        let mut agg = Agg::default();
        for s in 0..n {
            let entries: Vec<Entry> = (0..3).map(|j| (j + 1, Ctrl::Cpu, LADDER[k][j], Style::Normal)).collect();
            let r = fly(*venue, &entries, s as u64 * 13 + 1);
            best += r.pilots.iter().map(|p| p.total()).max().unwrap_or(0) as f64 / n as f64;
            for p in &r.pilots {
                agg.add(&r, p);
            }
        }
        println!("  stage {} {:<15} best rival {:>5.0} | {}", k + 1, venue.name(), best, agg.line());
    }
}

/// Probe bots in the player's seat against the 1P ladder.
pub fn probe(n: u32) {
    let bots: [(&str, f32, Style); 5] = [
        ("dive", 0.5, Style::Dive),
        ("cautious", 0.5, Style::Cautious),
        ("lazy", 0.2, Style::Normal),
        ("decent", 0.6, Style::Normal),
        ("good", 0.8, Style::Normal),
    ];
    println!("probe bots vs the ladder, {} races per stage per pilot", n);
    for (name, skill, style) in bots {
        let mut line = format!("{:<9}", name);
        for (k, venue) in VENUES.iter().enumerate() {
            let mut wins = 0;
            let mut cnt = 0;
            let mut agg = Agg::default();
            for me in 0..4 {
                for s in 0..n {
                    let seed = s as u64 * 7 + 3 + me as u64 * 101 + k as u64 * 1009;
                    let mut entries: Vec<Entry> = vec![(me, Ctrl::Cpu, skill, style)];
                    for (j, &sk) in LADDER[k].iter().enumerate() {
                        entries.push(((me + 1 + j) % 4, Ctrl::Cpu, sk, Style::Normal));
                    }
                    let r = fly(*venue, &entries, seed);
                    cnt += 1;
                    if r.ranking()[0] == 0 {
                        wins += 1;
                    }
                    agg.add(&r, &r.pilots[0]);
                }
            }
            line += &format!(" | S{} win {:>3.0}% sc {:>4.0} t {:>3.0}s g {:.1} fld {:>3.0}%", k + 1, 100.0 * wins as f32 / cnt as f32, agg.score / agg.n, agg.time / agg.n, agg.gates / agg.n, 100.0 * agg.field / agg.n);
        }
        println!("{}", line);
    }
}

/// Print one CPU pilot's flight, a line every second.
pub fn trace(venue: Venue, skill: f32, style: Style, seed: u64) {
    let entries: Vec<Entry> = vec![(0, Ctrl::Cpu, skill, style)];
    let mut r = Race::new(venue, &entries, seed);
    let mut n = 0;
    while !r.finished() && n < 60 * 60 * 12 {
        r.update(&[Pad::default(); 2]);
        r.sfx.clear();
        n += 1;
        let p = &r.pilots[0];
        let fine = std::env::var("FINE").is_ok() && p.gates.count_ones() == 4 && p.agl(&r.course) < 120.0 && p.airborne();
        if n % 60 == 0 || (fine && n % 10 == 0) {
            println!(
                "t{:>4} x{:>5.0} y{:>4.0} agl{:>4.0} v{:.2} vy{:+.2} face{:+} st {:?} gates {:b} air {:+.2}",
                n / 60,
                p.x,
                p.y,
                p.agl(&r.course),
                p.v,
                p.vario,
                p.face,
                p.state,
                p.gates,
                r.air(p.def, p.x, p.y)
            );
        }
    }
    let p = &r.pilots[0];
    println!("landing {:?} at {:.0} (target {:.0}) total {} gates {} time {}", p.landing, p.land_x, r.course.target, p.total(), p.gate_pts, p.clock / 60);
}
