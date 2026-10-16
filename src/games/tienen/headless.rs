//! Scripted screenshots of every screen and the key moments of a game,
//! plus a sprite sheet. Player 1's kid is played by the autopilot; a few
//! moments are set up by moving things into place.

use super::app::{App, Mode, Scene};
use super::art::*;
use super::bot::BotKind;
use super::data::*;
use super::world::*;
use crate::engine::gfx::*;
use crate::engine::input::Pad;
use crate::engine::png::save_png;
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

fn world(app: &mut App) -> &mut World {
    app.world.as_mut().unwrap()
}

/// Run until `stop` says so (true) or `n` frames pass (false).
fn until(app: &mut App, n: usize, mut stop: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..n {
        app.update([Pad::default(); 2], false);
        if app.scene != Scene::Play {
            return false;
        }
        if stop(app.world.as_ref().unwrap()) {
            return true;
        }
    }
    false
}

fn start_1p(app: &mut App, stage: usize, pick: usize) {
    app.mode = Mode::One;
    app.cursor = [pick, 1];
    app.stage = stage;
    app.start_play();
    world(app).auto = [Some(BotKind::Decent), Some(BotKind::Decent)];
}

/// A 1P game on the flats, past the latjetrap.
fn fresh(seed: u64) -> App {
    let mut app = App::new(seed);
    start_1p(&mut app, 0, (seed % 4) as usize);
    skip_latje(&mut app, 0);
    app
}

/// Skip the latjetrap: kid `k` keeps.
fn skip_latje(app: &mut App, k: usize) {
    let w = world(app);
    for _ in 0..3000 {
        if w.phase != Phase::Latje && w.phase != Phase::CountOut {
            break;
        }
        w.update(&[Pad::default(); 2]);
    }
    let _ = k;
}

pub fn shots(out: &str) {
    let dir = Path::new(out);
    std::fs::create_dir_all(dir).unwrap();
    let a = Pad { a: true, ..Pad::default() };

    // ------------------------------------------------ menus
    let mut app = App::new(42);
    run(&mut app, 70);
    snap(&app, dir, "01_title.png");
    press(&mut app, a);
    run(&mut app, 8);
    snap(&app, dir, "02_title_menu.png");
    press(&mut app, a);
    run(&mut app, 30);
    snap(&app, dir, "03_select.png");
    press(&mut app, Pad { down: true, ..Pad::default() });
    press(&mut app, Pad { right: true, ..Pad::default() });
    run(&mut app, 9);
    snap(&app, dir, "04_select_lieke.png");
    press(&mut app, Pad { up: true, ..Pad::default() });
    press(&mut app, Pad { left: true, ..Pad::default() });
    press(&mut app, a);
    run(&mut app, 70);
    snap(&app, dir, "05_intro_flat.png");
    press(&mut app, a);
    world(&mut app).auto = [Some(BotKind::Decent), Some(BotKind::Decent)];
    run(&mut app, 50);
    snap(&app, dir, "06_latjetrap.png");
    if until(&mut app, 600, |w| w.latje.stage == 1 && w.latje.st == 14) {
        snap(&app, dir, "07_latjetrap_shot.png");
    }
    until(&mut app, 4000, |w| w.phase == Phase::Wissel && w.pt == 50);
    snap(&app, dir, "08_first_keeper.png");
    until(&mut app, 400, |w| w.phase == Phase::Play && w.pt == 60);
    snap(&app, dir, "09_kickoff.png");
    if until(&mut app, 60 * 20, |w| w.ball.cross_to.is_some() && w.ball.air && w.ball.vz < 0.0 && w.ball.z < 30.0) {
        snap(&app, dir, "10_cross.png");
    }
    if until(&mut app, 60 * 40, |w| w.kids.iter().any(|k| matches!(k.act, Act::Swing { kind: Kind::Header, hit: true, .. }) && k.at == 4)) {
        snap(&app, dir, "11_header.png");
    }
    if until(&mut app, 60 * 60, |w| w.phase == Phase::Goal && w.pt == 30) {
        snap(&app, dir, "12_goal.png");
    }
    // fresh games from here on, one per moment
    let mut app = fresh(11);
    if until(&mut app, 60 * 90, |w| w.phase == Phase::Wissel && w.pt == 30 && w.why.is_some()) {
        snap(&app, dir, "13_wissel.png");
    }
    let mut app = fresh(12);
    if until(&mut app, 60 * 90, |w| w.kids.iter().any(|k| matches!(k.act, Act::Dive(_)) && k.at == 8)) {
        snap(&app, dir, "14_dive.png");
    }
    // the car
    let mut app = fresh(13);
    world(&mut app).next_car = 0;
    if until(&mut app, 60 * 90, |w| w.phase == Phase::Car && w.pt == 80) {
        snap(&app, dir, "15_auto.png");
    }
    // the poodle
    let mut app = fresh(14);
    world(&mut app).next_dog = 0;
    if until(&mut app, 60 * 90, |w| w.dog.as_ref().map(|d| d.has && d.t > 40).unwrap_or(false)) {
        snap(&app, dir, "16_fifi.png");
    }
    // a bicycle kick, set up: a cross falls onto Lieke's back
    let mut app = fresh(15);
    until(&mut app, 60 * 3, |_| false);
    {
        let w = world(&mut app);
        w.phase = Phase::Play;
        let k = (0..4).find(|&k| k != w.keeper).unwrap_or(0);
        let kd = &mut w.kids[k];
        kd.id = 3;
        kd.x = 90.0;
        kd.y = 36.0;
        kd.z = 18.0;
        kd.face = 1;
        kd.act = Act::Swing { kind: Kind::Bicycle, cross: false, hit: true };
        kd.at = 6;
        let b = &mut w.ball;
        b.held = false;
        b.owner = None;
        b.x = 84.0;
        b.y = 36.0;
        b.z = 40.0;
        b.vx = -4.0;
        b.vy = 0.0;
        b.vz = 0.2;
        b.land = None;
        w.popups.clear();
        w.banner = None;
    }
    snap(&app, dir, "17_omhaal.png");
    app.paused = true;
    snap(&app, dir, "18_pause.png");
    app.paused = false;
    // the end: a CPU keeper on 1 until somebody else scores the last point
    let mut app = fresh(16);
    for _ in 0..60 * 60 * 6 {
        if let Some(w) = app.world.as_mut() {
            let kp = w.keeper;
            if w.phase == Phase::Play && w.kids[kp].human().is_none() && w.kids[kp].pts > 1 {
                w.kids[kp].pts = 1;
            }
            if w.phase == Phase::Over && w.pt == 40 {
                break;
            }
        }
        app.update([Pad::default(); 2], false);
    }
    snap(&app, dir, "19_op_nul.png");
    if until(&mut app, 600, |w| w.phase == Phase::Brand && w.brand.stage == 0 && w.brand.st == 30 && w.pt > 60) {
        snap(&app, dir, "20_billenbrand.png");
    }
    if until(&mut app, 2000, |w| w.phase == Phase::Brand && w.brand.hit == Some(true) && w.brand.st == 30) {
        snap(&app, dir, "21_billenbrand_raak.png");
    }
    until(&mut app, 3000, |_| false);
    run(&mut app, 200);
    snap(&app, dir, "22_results.png");
    // continue and game over, from a lost game
    app.cleared = false;
    app.mode = Mode::One;
    app.count = 7;
    app.scene = Scene::Continue;
    app.t = 1;
    run(&mut app, 30);
    snap(&app, dir, "23_continue.png");
    app.scene = Scene::GameOver;
    app.t = 1;
    run(&mut app, 120);
    snap(&app, dir, "24_game_over.png");

    // ------------------------------------------------ the other venues
    for (st, name) in [(1usize, "school"), (2, "kooi")] {
        let mut app = App::new(7 + st as u64);
        app.mode = Mode::One;
        app.cursor = [st, 1];
        app.stage = st;
        app.go_intro_for_shots();
        run(&mut app, 80);
        snap(&app, dir, &format!("{}_intro_{}.png", 24 + st * 2 - 1, name));
        start_1p(&mut app, st, st);
        skip_latje(&mut app, 0);
        until(&mut app, 60 * 12, |_| false);
        snap(&app, dir, &format!("{}_{}.png", 24 + st * 2, name));
    }
    // the fence at the school: a shot flies over it into Mevrouw Knorr's garden
    {
        let mut app = App::new(31);
        start_1p(&mut app, 1, 0);
        skip_latje(&mut app, 0);
        until(&mut app, 60 * 4, |w| w.phase == Phase::Play && !w.ball.held);
        {
            let w = world(&mut app);
            let s = (0..4).find(|&k| k != w.keeper).unwrap_or(0);
            let b = &mut w.ball;
            b.owner = None;
            b.held = false;
            b.x = 70.0;
            b.y = 8.0;
            b.z = 30.0;
            b.vx = 0.2;
            b.vy = -1.6;
            b.vz = 0.5;
            b.last_field = Some(s);
            b.air = true;
        }
        if until(&mut app, 60 * 10, |w| w.why == Some(Why::Fence) && w.phase == Phase::Wissel && w.pt == 80) {
            snap(&app, dir, "29_knorr.png");
        }
    }

    // ------------------------------------------------ demo and 2P
    let mut app = App::new(5);
    run(&mut app, super::app::IDLE_DEMO as usize + 60 * 25);
    snap(&app, dir, "30_demo.png");
    let mut app = App::new(6);
    app.mode = Mode::Two;
    app.cursor = [0, 3];
    app.venue2p = 2;
    app.go_intro_for_shots();
    run(&mut app, 80);
    snap(&app, dir, "31_intro_2p.png");
    app.start_play();
    world(&mut app).auto = [Some(BotKind::Decent), Some(BotKind::Decent)];
    skip_latje(&mut app, 0);
    until(&mut app, 60 * 15, |_| false);
    snap(&app, dir, "32_2p.png");
    until(&mut app, 60 * 60 * 6, |w| w.finished() && w.pt > 41);
    run(&mut app, 400);
    snap(&app, dir, "33_results_2p.png");
    let mut app = App::new(8);
    app.go_champion_for_shots();
    run(&mut app, 200);
    snap(&app, dir, "34_champion.png");

    sheet(dir);
}

/// Every pose of the four kids, and the props.
fn sheet(dir: &Path) {
    let art = Art::new();
    for id in PICKS {
        let p = art.style(kid(id).style);
        let pal = kid_palette(id);
        let poses = [
            &p.stand, &p.run[0], &p.run[1], &p.jump, &p.kick[0], &p.kick[1], &p.heel, &p.chest, &p.header, &p.header_jump, &p.bike[0], &p.bike[1], &p.ready, &p.high, &p.dive,
            &p.hold, &p.punch, &p.roll, &p.down, &p.victory, &p.cheer, &p.sulk, &p.bent, &p.ouch, &p.sit,
        ];
        let mut f = Frame::new();
        f.clear(rgb(120, 168, 120));
        for (k, s) in poses.iter().enumerate() {
            let (cx, cy) = ((k % 8) as i32 * 32 + 16, (k / 8) as i32 * 44 + 38);
            f.hline(cx - 14, cx + 14, cy, rgb(96, 136, 96));
            blit_person(&mut f, s, cx, cy, false, &pal);
        }
        save_png(&f, &dir.join(format!("sheet_{}.png", kid(id).name.to_lowercase())), 3);
    }
    let mut props = Frame::new();
    props.clear(rgb(120, 168, 120));
    art.ball[0].draw(&mut props, 10, 10, &PROPS, false);
    art.ball[1].draw(&mut props, 22, 10, &PROPS, false);
    art.car.draw(&mut props, 40, 4, &car_pal(0), false);
    art.dog[0].draw(&mut props, 90, 8, &PROPS, false);
    art.dog[1].draw(&mut props, 110, 8, &PROPS, false);
    for k in 0..4 {
        art.pigeon[k].draw(&mut props, 130 + k as i32 * 12, 12, &PROPS, false);
    }
    art.glove.draw(&mut props, 180, 10, &PROPS, false);
    art.van.draw(&mut props, 192, 2, &PROPS, false);
    art.gnome.draw(&mut props, 10, 30, &PROPS, false);
    art.sunflower.draw(&mut props, 24, 28, &PROPS, false);
    save_png(&props, &dir.join("sheet_props.png"), 3);
    println!("wrote the sheets");
}
