//! Headless tools: scripted screenshots, CPU race simulation (`sim`) and
//! scripted "human" bots against the ladder (`probe`).

use super::app::{App, Mode, Scene};
use super::course::Ground;
use crate::engine::png::save_png;
use std::path::Path;
use crate::engine::gfx::Frame;
use super::race::*;
use super::runners::GERALD;

/// Gerald's speed and the two CPU runners' skill, per stage.
pub const GOAT_FAC: [f32; 3] = [0.86, 0.9, 0.93];
pub const CPU_SKILL: [[f32; 2]; 3] = [[0.5, 0.6], [0.6, 0.7], [0.75, 0.85]];

fn snap(app: &App, dir: &Path, name: &str) {
    let mut f = Frame::new();
    app.draw(&mut f);
    save_png(&f, &dir.join(name), 3);
    println!("wrote {}", name);
}

fn tick(app: &mut App, pads: [Pad; 2]) {
    app.update(pads, false);
    app.sfx.clear();
}

fn idle(app: &mut App, n: usize) {
    for _ in 0..n {
        tick(app, [Pad::default(); 2]);
    }
}

fn press(app: &mut App, p: Pad) {
    tick(app, [p, Pad::default()]);
    tick(app, [Pad::default(); 2]);
}

const A: Pad = Pad { up: false, down: false, left: false, right: false, a: true, b: false, c: false, start: false };

/// Play the current race with Decent bots on the human runners, taking
/// shots at key moments. Returns when the race scene ends.
fn bot_race(app: &mut App, dir: &Path, prefix: &str, max_shots: usize) {
    let mut bots = [Bot::new(BotKind::Decent, 11), Bot::new(BotKind::Decent, 23)];
    let mut seen: Vec<&'static str> = Vec::new();
    let mut periodic = 0;
    let mut n = 0;
    let mut prev: Option<(RunStats, RState, bool, i32, i32)> = None;
    let mut frame = 0u32;
    // animation sheets: a jump and a tumble, every 3rd frame
    let mut sheets: [(Vec<Frame>, bool, &str); 2] = [(Vec::new(), false, "jump_sequence"), (Vec::new(), false, "tumble_sequence")];
    while app.scene == Scene::Play && frame < RACE_CAP + 2000 {
        frame += 1;
        let pads = {
            let race = app.race.as_ref().unwrap();
            let mut pads = [Pad::default(); 2];
            for (k, &v) in race.views.iter().enumerate().take(2) {
                if let Some(p) = race.runners[v].human() {
                    pads[p] = bots[k].pad(race, v);
                }
            }
            pads
        };
        tick(app, pads);
        let Some(race) = app.race.as_ref() else { break };
        let me = &race.runners[race.views[0]];
        let st = (me.stats, me.state, me.bonk, me.grab_t, me.pickle);
        let mut event: Option<&'static str> = None;
        if let Some(p) = prev {
            let s = c_slope(race, me.x);
            if me.state == RState::Tumble && p.1 != RState::Tumble {
                event = Some("tumble");
            } else if me.state == RState::Down && p.1 != RState::Down {
                event = Some("faceplant");
            } else if me.stats.clean > p.0.clean && me.stats.combo >= 2 {
                event = Some("jump_combo");
            } else if me.state == RState::Air && me.vz < 0.5 && me.vz > 0.2 && race.t > 300 {
                event = Some("jump");
            } else if me.stats.aid > p.0.aid {
                event = if me.pickle > 0 { Some("pickle") } else { Some("aid_grab") };
            } else if me.stats.fives > p.0.fives {
                event = Some("high_five");
            } else if me.state == RState::Glissade && me.st_t == 40 {
                event = Some("glissade");
            } else if me.hiking(s) && me.hike > 0.6 {
                event = Some("power_hike");
            } else if me.state == RState::Stun && p.1 != RState::Stun {
                event = Some("bump");
            } else if me.bonk && !p.2 {
                event = Some("bonk");
            } else if me.duck && race.course.obs.iter().any(|o| o.kind.overhead() && (o.x - me.x).abs() < 6.0) {
                event = Some("limbo");
            } else if race.animals.iter().any(|a| a.kind == super::course::AnimalKind::Cow && a.x - me.x < 40.0 && a.x - me.x > 0.0) {
                event = Some("cow");
            } else if race.animals.iter().any(|a| a.marmot_up() && a.x - me.x < 50.0 && a.x - me.x > 10.0) {
                event = Some("marmot");
            } else if race.deer.iter().any(|d| d.active && d.t == DEER_CROSS.0 + 6 && (d.x - me.x).abs() < 80.0) {
                event = Some("deer");
            } else if race.course.fog(me.x) > 0.8 {
                event = Some("fog");
            } else if race.runners.iter().any(|r| r.goat() && r.say.is_some() && (r.x - me.x).abs() < 120.0) && race.t > 400 {
                event = Some("gerald");
            } else if race.runners.iter().any(|r| r.goat() && r.shortcut && (r.x - me.x).abs() < 120.0) {
                event = Some("shortcut");
            } else if me.finished.is_some() && me.st_t == 20 {
                event = Some("finish");
            } else if race.course.props.iter().any(|p| p.kind == super::course::PropKind::SummitCross && (p.x - me.x).abs() < 30.0) {
                event = Some("summit");
            }
        }
        prev = Some(st);
        for (k, sh) in sheets.iter_mut().enumerate() {
            let start = match k {
                0 => me.state == RState::Air && me.st_t == 1 && race.t > 300,
                _ => me.state == RState::Tumble && me.st_t == 1,
            };
            if start && !sh.1 && sh.0.is_empty() {
                sh.1 = true;
            }
            if sh.1 && frame % 3 == 0 {
                let mut f = Frame::new();
                app.draw(&mut f);
                sh.0.push(f);
                if sh.0.len() >= 12 {
                    sh.1 = false;
                    crop_sheet(&sh.0, 40, 110, 112, 84, &dir.join(format!("{}_{}.png", prefix, sh.2)));
                    println!("wrote {}_{}.png", prefix, sh.2);
                }
            }
        }
        if let Some(e) = event {
            if !seen.contains(&e) && n < max_shots {
                seen.push(e);
                n += 1;
                snap(app, dir, &format!("{}_{}.png", prefix, e));
            }
        }
        if frame % 1500 == 700 && periodic < 3 {
            periodic += 1;
            snap(app, dir, &format!("{}_play_{}.png", prefix, periodic));
        }
    }
}

/// The same rectangle of several frames, 4 per row, at 3x.
fn crop_sheet(frames: &[Frame], x0: i32, y0: i32, w: i32, h: i32, path: &Path) {
    let scale = 3usize;
    let cols = 4usize;
    let rows = frames.len().div_ceil(cols);
    let (cw, ch) = (w as usize * scale + 6, h as usize * scale + 6);
    let (tw, th) = (cols * cw, rows * ch);
    let mut data = vec![0u8; tw * th * 3];
    for (k, f) in frames.iter().enumerate() {
        let (ox, oy) = ((k % cols) * cw + 3, (k / cols) * ch + 3);
        for y in 0..h as usize * scale {
            for x in 0..w as usize * scale {
                let c = f.get(x0 + (x / scale) as i32, y0 + (y / scale) as i32);
                let i = ((oy + y) * tw + ox + x) * 3;
                data[i] = (c >> 16) as u8;
                data[i + 1] = (c >> 8) as u8;
                data[i + 2] = c as u8;
            }
        }
    }
    crate::engine::png::write_rgb(path, tw, th, &data);
}

fn c_slope(race: &Race, x: f32) -> f32 {
    race.course.slope(x)
}

pub fn shots(out: &str) {
    let dir = Path::new(out);
    std::fs::create_dir_all(dir).unwrap();
    let mut app = App::new(42);
    idle(&mut app, 80);
    snap(&app, dir, "01_title.png");
    press(&mut app, A);
    idle(&mut app, 8);
    snap(&app, dir, "02_title_menu.png");
    press(&mut app, A);
    idle(&mut app, 20);
    snap(&app, dir, "03_select.png");
    press(&mut app, Pad { right: true, ..Pad::default() });
    idle(&mut app, 10);
    press(&mut app, A);
    idle(&mut app, 20);
    idle(&mut app, 60);
    snap(&app, dir, "04_intro.png");
    press(&mut app, A);
    idle(&mut app, 100);
    snap(&app, dir, "05_countdown.png");
    // the whole ladder with a decent bot
    for stage in 0..3 {
        app.stage = stage;
        if stage > 0 {
            // results -> next intro
            idle(&mut app, 60);
            snap(&app, dir, &format!("{}_intro_stage{}.png", 10 + stage * 10, stage + 1));
            press(&mut app, A);
        }
        let prefix = format!("{}", 11 + stage * 10);
        bot_race(&mut app, dir, &prefix, 30);
        idle(&mut app, 150);
        snap(&app, dir, &format!("{}_results_tally.png", 18 + stage * 10));
        idle(&mut app, 220);
        snap(&app, dir, &format!("{}_results.png", 19 + stage * 10));
        // make sure we go on even if the bot did not qualify
        if !app.qualified {
            app.qualified = true;
        }
        press(&mut app, A);
    }
    idle(&mut app, 200);
    snap(&app, dir, "41_champion.png");

    // continue and game over
    let mut app2 = App::new(5);
    app2.cursor[0] = 2;
    app2.scene = Scene::Continue;
    app2.count = 7;
    idle(&mut app2, 30);
    snap(&app2, dir, "42_continue.png");
    app2.scene = Scene::GameOver;
    idle(&mut app2, 30);
    snap(&app2, dir, "43_game_over.png");

    // pause
    let mut app3 = App::new(9);
    app3.cursor[0] = 3;
    app3.mode = Mode::One;
    app3.start_race(&lineup(0, 3, Ctrl::Human(0)));
    idle(&mut app3, 400);
    press(&mut app3, Pad { start: true, ..Pad::default() });
    snap(&app3, dir, "44_pause.png");

    // 2P: select and split screen
    let mut app4 = App::new(7);
    idle(&mut app4, 30);
    press(&mut app4, A);
    press(&mut app4, Pad { down: true, ..Pad::default() });
    press(&mut app4, A);
    tick(&mut app4, [Pad::default(), Pad { right: true, ..Pad::default() }]);
    idle(&mut app4, 20);
    snap(&app4, dir, "45_select_2p.png");
    app4.cursor = [0, 2];
    app4.locked = [true, true];
    idle(&mut app4, 20);
    idle(&mut app4, 40);
    snap(&app4, dir, "46_intro_2p.png");
    press(&mut app4, A);
    bot_race(&mut app4, dir, "47_2p", 6);
    idle(&mut app4, 400);
    snap(&app4, dir, "48_results_2p.png");

    // attract mode
    let mut app5 = App::new(3);
    idle(&mut app5, 900 + 600);
    snap(&app5, dir, "49_demo.png");

    sprite_sheet(dir);
}

/// Every pose of every runner, the crowd and the animals, as sheets (3x).
pub fn sprite_sheet(dir: &std::path::Path) {
    use super::sprites::*;
    use crate::engine::gfx::rgb;
    use crate::engine::sprites::blit;
    let art = Art::new();
    let bg = rgb(112, 144, 104);
    // 15 poses per sheet, one row per runner
    for (part, chunk) in ALL_TPOSES.chunks(15).enumerate() {
        let mut f = Frame::new();
        f.clear(bg);
        for d in 0..4 {
            for (k, &p) in chunk.iter().enumerate() {
                blit(&mut f, art.runner(d, p), 9 + k as i32 * 16 + (k as i32 % 2) * 0, 34 + d as i32 * 36, false, &super::runners::palette(d));
            }
        }
        // the poses overlap a little at 16 px spacing: draw again at 2x spacing below? keep a clean 2nd sheet
        let mut g = Frame::new();
        g.clear(bg);
        for d in 0..4 {
            for (k, &p) in chunk.iter().enumerate() {
                let (col, row) = (k % 8, k / 8);
                blit(&mut g, art.runner(d, p), 14 + col as i32 * 31, 32 + (d * 2 + row) as i32 * 27, false, &super::runners::palette(d));
            }
        }
        crate::engine::png::save_png(&g, &dir.join(format!("sprites_{}.png", part)), 3);
        println!("wrote sprites_{}.png", part);
    }
    let mut f = Frame::new();
    f.clear(bg);
    let bms: Vec<(&Bm, &BPal)> = vec![
        (&art.goat[0], &art.goat_pal),
        (&art.goat[1], &art.goat_pal),
        (&art.goat[2], &art.goat_pal),
        (&art.goat[3], &art.goat_pal),
        (&art.cow[0], &art.cow_pals[0]),
        (&art.cow[1], &art.cow_pals[0]),
        (&art.cow[0], &art.cow_pals[1]),
        (&art.marmot_up, &art.marmot_pal),
        (&art.marmot_sleep, &art.marmot_pal),
        (&art.deer, &art.deer_pal),
        (&art.cowbell, &art.bell_pal),
        (&art.items[0], &art.item_pal),
        (&art.items[1], &art.item_pal),
        (&art.items[2], &art.item_pal),
        (&art.items[3], &art.item_pal),
    ];
    for (k, (bm, pal)) in bms.iter().enumerate() {
        bm.draw(&mut f, 20 + (k % 7) as i32 * 34, 30 + (k / 7) as i32 * 34, false, pal);
    }
    for st in 0..6 {
        for k in 0..4 {
            blit(&mut f, art.crowd(st, k), 12 + (st * 4 + k) as i32 * 10 % 240, 140 + ((st * 4 + k) / 24) as i32 * 34 + if (st * 4 + k) % 2 == 1 { 34 } else { 0 }, false, &super::runners::crowd_palette(st as u32 * 5 + k as u32));
        }
    }
    crate::engine::png::save_png(&f, &dir.join("sprites_animals.png"), 3);
    println!("wrote sprites_animals.png");
}

fn secs(f: u32) -> f32 {
    f as f32 / 60.0
}

fn run_race(race: &mut Race) {
    while !race.finished() && race.t < RACE_CAP {
        race.update(&[Pad::default(); 2]);
        race.sfx.clear();
    }
}

pub fn sim(n: usize) {
    let mut rng = Rng(4242);
    println!("CPU races, {} per stage (Gerald + 3 smart CPUs of random skill 0.4-0.9):", n);
    for stage in 0..3 {
        let mut win_t = 0u64;
        let mut all_t = 0u64;
        let mut cnt = 0u64;
        let (mut falls, mut bonks, mut wipe, mut stuns) = (0u32, 0u32, 0u32, 0u32);
        let mut goat_place = [0u32; 4];
        let mut capped = 0;
        for k in 0..n {
            let mut racers = vec![(GERALD, Ctrl::Goat(GOAT_FAC[stage]))];
            for j in 0..3 {
                racers.push(((k + j) % 4, Ctrl::Cpu(Strategy::Smart, rng.range(0.4, 0.9))));
            }
            let mut race = Race::new(stage, &racers, k as u64 * 31 + stage as u64);
            run_race(&mut race);
            if race.t >= RACE_CAP {
                capped += 1;
            }
            let rank = race.ranking();
            win_t += race.result_time(rank[0]) as u64;
            for (i, r) in race.runners.iter().enumerate() {
                if !r.goat() {
                    all_t += race.result_time(i) as u64;
                    cnt += 1;
                    falls += r.stats.falls;
                    bonks += r.stats.bonks;
                    wipe += r.stats.wipeouts;
                    stuns += r.stats.stuns;
                }
            }
            let gp = rank.iter().position(|&i| race.runners[i].goat()).unwrap();
            goat_place[gp] += 1;
        }
        println!(
            "stage {} {:<14} winner {:>5.1}s  runners avg {:>5.1}s  falls {:.1} (too fast {:.1})  bumps {:.1}  bonks {:.2}  gerald places {:?}  capped {}",
            stage + 1,
            super::course::STAGE_NAMES[stage],
            secs((win_t / n as u64) as u32),
            secs((all_t / cnt.max(1)) as u32),
            falls as f32 / cnt as f32,
            wipe as f32 / cnt as f32,
            stuns as f32 / cnt as f32,
            bonks as f32 / cnt as f32,
            goat_place,
            capped
        );
    }
    println!();
    println!("pace strategies (same skill 0.7, same runner, all three in one race): avg time, wins, bonks");
    for stage in 0..3 {
        let strats = [Strategy::Smart, Strategy::Steady, Strategy::Sprinter];
        let mut t = [0u64; 3];
        let mut wins = [0u32; 3];
        let mut bonks = [0u32; 3];
        for k in 0..n {
            let def = k % 4;
            let racers: Vec<(usize, Ctrl)> = strats.iter().map(|&s| (def, Ctrl::Cpu(s, 0.7))).collect();
            let mut race = Race::new(stage, &racers, k as u64 * 17 + 3);
            run_race(&mut race);
            for (j, r) in race.runners.iter().enumerate() {
                t[j] += race.result_time(j) as u64;
                bonks[j] += r.stats.bonks;
            }
            wins[race.ranking()[0]] += 1;
        }
        print!("stage {}:", stage + 1);
        for j in 0..3 {
            print!("  {:?} {:.1}s win {:.0}% bonks {:.1}", strats[j], secs((t[j] / n as u64) as u32), 100.0 * wins[j] as f32 / n as f32, bonks[j] as f32 / n as f32);
        }
        println!();
    }
}

// ------------------------------------------------------------------ probe
// Scripted "human" bots playing the 1P ladder through the pad, to check
// the difficulty curve and look for exploits.

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BotKind {
    /// presses right until SPRINT and stays there; jumps and ducks fine
    Sprinter,
    /// stays at RUN, jumps and ducks fine, never poles or eats
    Steady,
    /// paces by terrain and legs, poles, eats, high-fives, glissades,
    /// with human timing noise
    Decent,
    /// leaves the pace at JOG and only jumps
    Lazy,
    /// RUN and jump all the time (bunny hopping over everything)
    Hopper,
}

/// The racers of a 1P stage: you, Gerald and two CPU runners.
pub fn lineup(stage: usize, me: usize, human: Ctrl) -> Vec<(usize, Ctrl)> {
    let others = match stage {
        0 => [(me + 1) % 4, (me + 2) % 4],
        1 => [(me + 2) % 4, (me + 3) % 4],
        _ => [(me + 3) % 4, (me + 1) % 4],
    };
    vec![
        (others[0], Ctrl::Cpu(Strategy::Smart, CPU_SKILL[stage][0])),
        (GERALD, Ctrl::Goat(GOAT_FAC[stage])),
        (others[1], Ctrl::Cpu(Strategy::Smart, CPU_SKILL[stage][1])),
        (me, human),
    ]
}

pub struct Bot {
    kind: BotKind,
    rng: Rng,
    prev: Pad,
    t: u32,
    pole: bool,
    pole_t: i32,
    decided: Option<usize>,
    ok: bool,
    lead: f32,
    grab_off: f32,
}

impl Bot {
    pub fn new(kind: BotKind, seed: u64) -> Self {
        Bot { kind, rng: Rng(seed | 1), prev: Pad::default(), t: 0, pole: false, pole_t: 0, decided: None, ok: true, lead: 0.0, grab_off: 0.0 }
    }

    pub fn pad(&mut self, race: &Race, me: usize) -> Pad {
        self.t += 1;
        let mut p = Pad::default();
        let r = &race.runners[me];
        let c = &race.course;
        let x = r.x;
        let s = c.slope(x);
        let g = c.ground(x);
        let decent = self.kind == BotKind::Decent;
        // pace
        let want = match self.kind {
            BotKind::Sprinter => Pace::Sprint,
            BotKind::Steady => Pace::Run,
            BotKind::Lazy => Pace::Jog,
            BotKind::Hopper => Pace::Run,
            BotKind::Decent => {
                let st = r.stamina;
                let remain = c.finish - x;
                if remain < 900.0 && st > remain * 0.09 + 4.0 {
                    Pace::Sprint
                } else if s > STEEP {
                    Pace::Jog
                } else if s > 0.08 {
                    if st > 50.0 { Pace::Run } else { Pace::Jog }
                } else if s < -0.1 {
                    if matches!(g, Ground::Scree | Ground::Rock | Ground::Snow) || st < 30.0 { Pace::Run } else { Pace::Sprint }
                } else if g == Ground::Mud {
                    Pace::Jog
                } else if st > 25.0 {
                    Pace::Run
                } else {
                    Pace::Jog
                }
            }
        };
        if !self.prev.right && !self.prev.left {
            if want > r.pace && !r.bonk {
                p.right = true;
            } else if want < r.pace {
                p.left = true;
            }
        }
        // poles: alternate A and C about 4-5 times a second
        if decent && s > STEEP && r.state == RState::Run && r.stamina > 22.0 && !r.bonk {
            self.pole_t -= 1;
            if self.pole_t <= 0 {
                self.pole_t = 11 + self.rng.int(5);
                self.pole = !self.pole;
                if self.pole { p.a = true } else { p.c = true }
            }
        }
        // obstacles, with human reaction noise
        let obs = &c.obs;
        let mut k = 0usize.max(c.first_ob(x - 20.0));
        while k < obs.len() && obs[k].x + obs[k].kind.width() / 2.0 + 3.0 < x {
            k += 1;
        }
        if k < obs.len() && r.state != RState::Air {
            let o = obs[k];
            if self.decided != Some(k) {
                self.decided = Some(k);
                let miss = match self.kind {
                    BotKind::Decent => 0.05,
                    BotKind::Lazy => 0.15,
                    _ => 0.03,
                };
                self.ok = self.rng.f() >= miss;
                self.lead = self.rng.range(-5.0, 5.0);
            }
            if self.ok {
                let va = r.v.max(AIR_MIN);
                let vz = 2.5 + 0.08 * super::runners::RUNNERS[r.def].agility as f32;
                if o.kind.overhead() {
                    if o.x - x < 30.0 + self.lead {
                        p.down = true;
                    }
                } else if o.x - x <= va * (vz - s.max(0.0) * va) / 0.16 + self.lead && !self.prev.b {
                    p.b = true;
                }
            }
        }
        // marmots, cows, deer
        for an in &race.animals {
            let d = an.x - x;
            if an.marmot_up() && d > 0.0 && d < r.v.max(AIR_MIN) * 12.0 && !self.prev.b {
                p.b = true;
            }
            if an.kind == super::course::AnimalKind::Cow && d > -14.0 && d < 90.0 && self.kind != BotKind::Lazy {
                p.down = true;
                p.right = false;
            }
        }
        for dx in &race.deer {
            let d = dx.x - x;
            if dx.active && d > -12.0 && d < 40.0 && self.kind != BotKind::Lazy {
                p.down = true;
            }
        }
        if decent {
            if r.wobble > 0.3 {
                p.down = true;
            }
            if g == Ground::Snow && s < -0.12 {
                p.down = true;
            }
            // aid stations and crowds: press A as you pass, a bit early or late
            for a in &c.aids {
                if (x - (a.x + self.grab_off)).abs() < 1.2 && !self.prev.a {
                    p.a = true;
                }
            }
            for cr in &c.crowds {
                if (x - (cr.x + self.grab_off)).abs() < 1.2 && !self.prev.a {
                    p.a = true;
                }
            }
            if self.t % 120 == 0 {
                self.grab_off = self.rng.range(-12.0, 12.0);
            }
        }
        if self.kind == BotKind::Hopper && r.state == RState::Run && !self.prev.b {
            p.b = true;
        }
        if p.b {
            p.down = false;
        }
        self.prev = p;
        p
    }
}

#[derive(Default)]
struct ProbeRes {
    n: u32,
    time: u64,
    places: [u32; 4],
    beat_goat: u32,
    falls: u32,
    bonks: u32,
    goat_time: u64,
}

pub fn probe(per: u32) {
    println!("{:<10} {:>5} {:>8} {:>16} {:>6} {:>6} {:>9} {:>6} {:>6} {:>8}", "bot", "stage", "time", "places 1/2/3/4", "top2%", "win%", "goat beat", "falls", "bonks", "gerald");
    for kind in [BotKind::Lazy, BotKind::Sprinter, BotKind::Steady, BotKind::Hopper, BotKind::Decent] {
        for stage in 0..3 {
            let mut res = ProbeRes::default();
            for me in 0..4 {
                for k in 0..per {
                    let seed = k as u64 * 977 + me as u64 * 13 + stage as u64 * 7 + 1;
                    let mut race = Race::new(stage, &lineup(stage, me, Ctrl::Human(0)), seed);
                    let mut bot = Bot::new(kind, seed);
                    let mi = 3;
                    while !race.finished() && race.t < RACE_CAP {
                        let pad = bot.pad(&race, mi);
                        race.update(&[pad, Pad::default()]);
                        race.sfx.clear();
                    }
                    let rank = race.ranking();
                    let place = rank.iter().position(|&i| i == mi).unwrap();
                    let gp = rank.iter().position(|&i| race.runners[i].goat()).unwrap();
                    res.n += 1;
                    res.time += race.result_time(mi) as u64;
                    res.places[place] += 1;
                    if place < gp {
                        res.beat_goat += 1;
                    }
                    res.falls += race.runners[mi].stats.falls;
                    let gi = race.runners.iter().position(|r| r.goat()).unwrap();
                    res.goat_time += race.result_time(gi) as u64;
                    res.bonks += race.runners[mi].stats.bonks;
                }
            }
            let n = res.n as f32;
            println!(
                "{:<10} {:>5} {:>7.1}s {:>16} {:>6.0} {:>6.0} {:>8.0}% {:>6.1} {:>6.1} {:>7.1}s",
                format!("{:?}", kind),
                stage + 1,
                secs((res.time / res.n as u64) as u32),
                format!("{}/{}/{}/{}", res.places[0], res.places[1], res.places[2], res.places[3]),
                100.0 * (res.places[0] + res.places[1]) as f32 / n,
                100.0 * res.places[0] as f32 / n,
                100.0 * res.beat_goat as f32 / n,
                res.falls as f32 / n,
                res.bonks as f32 / n,
                secs((res.goat_time / res.n as u64) as u32)
            );
        }
    }
}

/// Development aid: trace a decent bot through a stage.
pub fn trace(stage: usize) {
    let me = 1;
    let mut race = Race::new(stage, &lineup(stage, me, Ctrl::Human(0)), 5);
    let mut bot = Bot::new(BotKind::Decent, 5);
    let mut last_bonk = false;
    while !race.finished() && race.t < RACE_CAP {
        let pad = bot.pad(&race, 3);
        race.update(&[pad, Pad::default()]);
        race.sfx.clear();
        let r = &race.runners[3];
        if (race.t % 300 == 0 && race.phase == Phase::Race) || r.bonk != last_bonk {
            let s = race.course.slope(r.x);
            println!("t {:>5.1}s x {:>6.0} slope {:>5.2} {:?} pace {:?} v {:.2} stam {:>5.1} hike {:.2} bonk {} state {:?} zone {}", race.t as f32 / 60.0, r.x, s, race.course.ground(r.x), r.pace, r.v, r.stamina, r.hike, r.bonk, r.state, race.course.zone(r.x, super::course::ZoneKind::Switchback).is_some());
        }
        last_bonk = r.bonk;
    }
}
