//! Scripted screenshots of every screen and the key moments on the street,
//! plus sprite sheets. Player 1 is driven by a bot; a few moments are set
//! up by moving things in front of the camera.

use super::app::{App, Mode, Scene};
use super::art::*;
use super::bot::{Bot, BotKind};
use super::data::*;
use super::render::{draw_bike, draw_rider};
use super::world::*;
use crate::engine::gfx::*;
use crate::engine::input::Pad;
use crate::engine::png::save_png;
use crate::engine::ui::OUTLINE;
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

/// Play with bots for up to `n` frames, until `stop` says so.
fn play_until(app: &mut App, bots: &mut [Bot; 2], n: usize, mut stop: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..n {
        let mut pads = [Pad::default(); 2];
        if let Some(w) = app.world.as_ref()
            && let Ctrl::Human(p) = w.thief.ctrl {
                pads[p.min(1)] = bots[p.min(1)].pad(w);
            }
        app.update(pads, false);
        if app.scene != Scene::Play {
            return false;
        }
        if stop(app.world.as_ref().unwrap()) {
            return true;
        }
    }
    false
}

fn world(app: &mut App) -> &mut World {
    app.world.as_mut().unwrap()
}

/// Start 1P on a stage, through the select screen and the intro card.
fn start_1p(app: &mut App, stage: usize, thief: usize) {
    app.mode = Mode::One;
    app.cursor = [thief, 1];
    app.stage = stage;
    app.start_play();
}

pub fn shots(out: &str) {
    let dir = Path::new(out);
    std::fs::create_dir_all(dir).unwrap();
    let a = Pad { a: true, ..Pad::default() };
    let mut bots = [Bot::new(BotKind::Decent, 3), Bot::new(BotKind::Decent, 4)];

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
    snap(&app, dir, "04_select_prof.png");
    press(&mut app, Pad { up: true, ..Pad::default() });
    press(&mut app, Pad { left: true, ..Pad::default() });
    press(&mut app, a);
    run(&mut app, 70);
    snap(&app, dir, "05_intro_prinsengracht.png");
    press(&mut app, a);
    run(&mut app, 40);
    snap(&app, dir, "06_countdown.png");
    run(&mut app, 50);
    snap(&app, dir, "07_jatten.png");

    // ------------------------------------------------ the Prinsengracht
    play_until(&mut app, &mut bots, 60 * 4, |_| false);
    snap(&app, dir, "08_street.png");
    if play_until(&mut app, &mut bots, 60 * 30, |w| w.thief.state == TState::Pick && w.thief.st_t > 8) {
        snap(&app, dir, "09_picking.png");
    }
    if play_until(&mut app, &mut bots, 60 * 30, |w| w.thief.state == TState::Ride && w.thief.st_t > 30) {
        snap(&app, dir, "10_riding_hot.png");
    }
    if play_until(&mut app, &mut bots, 60 * 30, |w| w.thief.state == TState::Toss && w.thief.st_t == 12) {
        snap(&app, dir, "11_toss.png");
    }
    // a U-lock, set up right here
    {
        let w = world(&mut app);
        w.thief.state = TState::Walk;
        w.thief.bike = None;
        let (x, y) = (w.thief.x + 6.0, (w.thief.y + 4.0).min(DEPTH));
        if let Some(i) = w.bikes.iter().position(|b| b.kind == Kind::Race && b.state == BikeState::Parked) {
            w.bikes[i].x = x;
            w.bikes[i].y = y;
            w.bikes[i].lock = Lock::U;
            w.thief.x = x - 4.0;
            w.thief.y = y - 4.0;
        }
        for b in &mut w.boas {
            b.x = (b.x + 400.0) % w.len;
        }
    }
    press(&mut app, a);
    run(&mut app, 30);
    snap(&app, dir, "12_u_lock.png");
    // a BOA turns round and wonders
    {
        let w = world(&mut app);
        let (x, y) = (w.thief.x, w.thief.y);
        w.boas[0].x = x + 60.0;
        w.boas[0].y = y - 6.0;
        w.boas[0].face_left = true;
        w.boas[0].state = BoaState::Patrol;
        w.boas[0].sus = 0.0;
    }
    let mut n = 0;
    while n < 60 {
        let st = app.world.as_ref().map(|w| w.thief.state);
        let p = if st == Some(TState::Pick) && n % 6 == 0 { a } else { Pad::default() };
        app.update([p, Pad::default()], false);
        n += 1;
        if app.world.as_ref().map(|w| w.boas[0].sus > 0.5).unwrap_or(true) {
            break;
        }
    }
    snap(&app, dir, "13_suspicious.png");
    // act casual
    for _ in 0..40 {
        app.update([Pad { up: true, c: true, ..Pad::default() }, Pad::default()], false);
    }
    world(&mut app).thief.casual_kind = 1;
    for _ in 0..30 {
        app.update([Pad { c: true, ..Pad::default() }, Pad::default()], false);
    }
    snap(&app, dir, "14_casual_selfie.png");
    // a chase
    {
        let w = world(&mut app);
        for k in 1..w.boas.len() {
            w.boas[k].x = (w.thief.x + 500.0) % w.len;
        }
        w.boas[0].sus = 1.1;
        w.boas[0].x = w.thief.x + 50.0;
        w.boas[0].y = w.thief.y;
        w.boas[0].face_left = true;
        w.boas[0].state = BoaState::Look;
        w.boas[0].cool = 0;
        w.thief.grace = 0;
        w.thief.state = TState::Pick;
        if let Some(i) = w.bike_at_hand() {
            w.thief.pick = Some(i);
        }
    }
    run(&mut app, 3);
    {
        let w = world(&mut app);
        w.thief.state = TState::Walk;
        w.thief.pick = None;
    }
    for _ in 0..20 {
        app.update([Pad { left: true, b: true, ..Pad::default() }, Pad::default()], false);
    }
    // a real chase, from a reckless run on its own street
    {
        let mut other = App::new(77);
        start_1p(&mut other, 0, 0);
        let mut rb = [Bot::new(BotKind::Reckless, 5), Bot::new(BotKind::Reckless, 6)];
        if play_until(&mut other, &mut rb, 60 * 90, |w| w.boas.iter().any(|b| b.state == BoaState::Chase && b.chase_t == 45)) {
            snap(&other, dir, "15_chase.png");
        }
    }
    // hide in the krul
    {
        let w = world(&mut app);
        w.thief.grace = 900;
        w.thief.fines = 0;
        w.thief.wanted = 0;
        if let Some((i, &(_, x, y))) = w.spots.iter().enumerate().find(|s| s.1 .0 == Spot::Krul) {
            w.thief.x = x;
            w.thief.y = y;
            w.thief.state = TState::Hide;
            w.thief.spot = Some(i);
            w.thief.hide_seen = false;
            w.cam[0] = (x - 128.0).clamp(0.0, w.len - 256.0);
            w.boas[0].x = x - 30.0;
            w.boas[0].y = 40.0;
            w.boas[0].state = BoaState::Search;
            w.boas[0].st_t = 0;
            w.boas[0].say = Some(("WAAR IS IE?", 100));
        }
    }
    run(&mut app, 10);
    snap(&app, dir, "16_hide_krul.png");
    // behind a tree
    {
        let w = world(&mut app);
        if let Some((i, &(_, x, y))) = w.spots.iter().enumerate().find(|s| s.1 .0 == Spot::Tree && s.1 .1 > 200.0) {
            w.thief.x = x;
            w.thief.y = y;
            w.thief.spot = Some(i);
            w.cam[0] = (x - 128.0).clamp(0.0, w.len - 256.0);
            w.boas[0].x = x + 40.0;
            w.boas[0].state = BoaState::Search;
            w.boas[0].face_left = true;
        }
    }
    run(&mut app, 10);
    snap(&app, dir, "17_hide_tree.png");
    // caught
    {
        let w = world(&mut app);
        w.thief.fines = 0;
        w.thief.state = TState::Walk;
        w.thief.spot = None;
        w.thief.y = 40.0;
        w.thief.grace = 0;
        let (x, y) = (w.thief.x, w.thief.y);
        w.boas[0].x = x + 30.0;
        w.boas[0].y = y;
        w.boas[0].state = BoaState::Chase;
        w.boas[0].target = (x, y);
        w.boas[0].lost = 0;
    }
    if play_until(&mut app, &mut [Bot::new(BotKind::Lazy, 1), Bot::new(BotKind::Lazy, 2)], 200, |w| w.thief.state == TState::Busted && w.thief.st_t > 40) {
        snap(&app, dir, "18_bekeuring.png");
    }
    // into the canal on a bike
    {
        let w = world(&mut app);
        w.thief.st_t = 999;
    }
    run(&mut app, 2);
    {
        let w = world(&mut app);
        w.thief.grace = 1200;
        w.thief.fines = 0;
        for b in &mut w.boas {
            b.state = BoaState::Patrol;
            b.sus = 0.0;
            b.x = (b.x + 500.0) % w.len;
        }
        if let Some(i) = w.bikes.iter().position(|b| b.kind == Kind::Oma && b.state == BikeState::Parked) {
            w.bikes[i].x = w.thief.x;
            w.bikes[i].y = DEPTH - 3.0;
            w.bikes[i].state = BikeState::Loose;
            w.thief.y = DEPTH - 6.0;
        }
    }
    press(&mut app, a);
    for _ in 0..24 {
        app.update([Pad { down: true, right: true, ..Pad::default() }, Pad::default()], false);
    }
    run(&mut app, 6);
    snap(&app, dir, "19_plons.png");
    run(&mut app, 90);
    snap(&app, dir, "20_swim.png");
    for _ in 0..30 {
        app.update([Pad { up: true, ..Pad::default() }, Pad::default()], false);
    }
    snap(&app, dir, "21_climb_out.png");
    // a heron takes off
    {
        let w = world(&mut app);
        w.thief.state = TState::Walk;
        w.thief.y = DEPTH - 8.0;
        let x = w.thief.x + 40.0;
        if let Some(h) = w.herons.first_mut() {
            h.x = x;
            h.flying = false;
            h.gone = false;
            h.z = 0.0;
        }
    }
    for _ in 0..30 {
        app.update([Pad { right: true, b: true, ..Pad::default() }, Pad::default()], false);
    }
    snap(&app, dir, "22_heron.png");
    // pause
    press(&mut app, Pad { start: true, ..Pad::default() });
    snap(&app, dir, "23_pause.png");
    press(&mut app, Pad { start: true, ..Pad::default() });
    // play it out with the bot to the results
    play_until(&mut app, &mut bots, 60 * 200, |_| false);
    run(&mut app, 100);
    snap(&app, dir, "24_results.png");

    // ------------------------------------------------ Centraal Station
    let mut app = App::new(7);
    start_1p(&mut app, 1, 2);
    app.go_intro_for_shots();
    run(&mut app, 70);
    snap(&app, dir, "25_intro_station.png");
    app.start_play();
    play_until(&mut app, &mut bots, 60 * 10, |_| false);
    snap(&app, dir, "26_station.png");
    {
        let w = world(&mut app);
        let x = w.thief.x;
        if let Some(b) = w.boas.iter_mut().find(|b| b.kind == BoaKind::Bike) {
            b.x = x + 70.0;
            b.face_left = true;
        }
    }
    run(&mut app, 20);
    snap(&app, dir, "27_fatima.png");
    play_until(&mut app, &mut bots, 60 * 60, |w| w.boats.iter().any(|b| b.kind == BoatKind::Tour && (b.x - w.cam[0] - 128.0).abs() < 80.0));
    snap(&app, dir, "28_tour_boat.png");

    // ------------------------------------------------ Koningsdag
    let mut app = App::new(9);
    start_1p(&mut app, 2, 1);
    app.go_intro_for_shots();
    run(&mut app, 70);
    snap(&app, dir, "29_intro_koningsdag.png");
    app.start_play();
    play_until(&mut app, &mut bots, 60 * 6, |_| false);
    {
        let w = world(&mut app);
        let x = w.peds.iter().find(|p| p.crowd).map(|p| p.x).unwrap_or(300.0);
        w.thief.x = x;
        w.thief.y = 30.0;
    }
    run(&mut app, 40);
    snap(&app, dir, "30_crowd.png");
    {
        let w = world(&mut app);
        if let Some(gi) = w.bikes.iter().position(|b| b.kind == Kind::Gold) {
            let (gx, gy) = (w.bikes[gi].x, w.bikes[gi].y);
            w.thief.x = gx - 3.0;
            w.thief.y = gy - 5.0;
            w.thief.state = TState::Walk;
            if let Some(c) = w.boas.iter_mut().find(|b| b.kind == BoaKind::Chief) {
                c.x = gx + 70.0;
                c.face_left = false;
            }
        }
    }
    run(&mut app, 30);
    snap(&app, dir, "31_golden_bike.png");
    {
        let w = world(&mut app);
        if let Some((i, &(_, x, y))) = w.spots.iter().enumerate().find(|s| s.1 .0 == Spot::Tree) {
            w.thief.x = x;
            w.thief.y = y;
            w.thief.state = TState::Hide;
            w.thief.spot = Some(i);
            w.thief.hide_seen = false;
            if let Some(c) = w.boas.iter_mut().find(|b| b.kind == BoaKind::Chief) {
                c.x = x + 40.0;
                c.y = 46.0;
                c.state = BoaState::Patrol;
                if let Some(d) = &mut c.dog {
                    d.x = x + 22.0;
                    d.y = 52.0;
                }
            }
            w.cam[0] = (x - 128.0).clamp(0.0, w.len - 256.0);
        }
    }
    run(&mut app, 30);
    snap(&app, dir, "32_bruno_sniffs.png");

    // ------------------------------------------------ continue, game over, champion
    let mut app = App::new(5);
    start_1p(&mut app, 0, 0);
    {
        let w = world(&mut app);
        w.time_left = 2;
    }
    run(&mut app, 300);
    snap(&app, dir, "33_results_fail.png");
    run(&mut app, 200);
    press(&mut app, a);
    run(&mut app, 30);
    snap(&app, dir, "34_continue.png");
    run(&mut app, 60 * 11);
    snap(&app, dir, "35_game_over.png");
    let mut app = App::new(6);
    app.mode = Mode::One;
    app.score = [2140, 0];
    app.go_champion_for_shots();
    run(&mut app, 120);
    snap(&app, dir, "36_champion.png");

    // ------------------------------------------------ demo
    let mut app = App::new(8);
    run(&mut app, super::app::IDLE_DEMO as usize + 600);
    snap(&app, dir, "37_demo.png");

    // ------------------------------------------------ 2P
    let mut app = App::new(12);
    run(&mut app, 30);
    press(&mut app, a);
    press(&mut app, Pad { down: true, ..Pad::default() });
    press(&mut app, a);
    run(&mut app, 10);
    app.update([Pad::default(), Pad { right: true, ..Pad::default() }], false);
    app.update([Pad::default(); 2], false);
    snap(&app, dir, "38_select_2p.png");
    press(&mut app, a);
    app.update([Pad::default(), Pad { a: true, ..Pad::default() }], false);
    run(&mut app, 70);
    snap(&app, dir, "39_intro_2p.png");
    press(&mut app, a);
    run(&mut app, 60);
    snap(&app, dir, "40_split_countdown.png");
    play_until(&mut app, &mut bots, 60 * 12, |_| false);
    snap(&app, dir, "41_split.png");
    {
        let w = world(&mut app);
        let h = w.human_boa.unwrap();
        w.boas[h].x = w.thief.x + 20.0;
        w.boas[h].y = w.thief.y;
        w.cam[1] = w.cam[0];
    }
    run(&mut app, 5);
    snap(&app, dir, "42_split_close.png");
    play_until(&mut app, &mut bots, 60 * 200, |_| false);
    run(&mut app, 200);
    snap(&app, dir, "43_results_2p.png");
    let mut app = App::new(13);
    app.mode = Mode::Two;
    app.score = [610, 845];
    app.go_champion_for_shots();
    run(&mut app, 120);
    snap(&app, dir, "44_champion_2p.png");

    sheets(dir);
}

/// Every pose of every thief, the BOA's, the bikes and the props.
fn sheets(dir: &Path) {
    let art = Art::new();
    let mut f = Frame::new();
    f.clear(rgb(96, 136, 120));
    let t = 0;
    for (r, &who) in THIEVES.iter().enumerate() {
        let ppl = art.style(who.def().style);
        let pal = thief_palette(who);
        let y = 32 + r as i32 * 34;
        f.text(2, y - 30, who.def().name, rgb(248, 248, 248));
        let poses = [&ppl.stand, &ppl.walk[0], &ppl.walk[2], &ppl.run[0], &ppl.pick[0], &ppl.pick[1], &ppl.casual, &ppl.selfie, &ppl.toss, &ppl.sit, &ppl.fall];
        for (k, s) in poses.iter().enumerate() {
            blit_person(&mut f, s, 12 + k as i32 * 18, y, false, &pal);
        }
        draw_rider(&mut f, &art, who.def().style, &pal, Kind::Oma, None, 220, y, false, 2, 0);
    }
    let y = 32 + 4 * 34;
    let boa = art.style(2);
    for (k, s) in [&boa.stand, &boa.walk[1], &boa.run[0], &boa.write, &boa.whistle, &boa.sip].iter().enumerate() {
        blit_person(&mut f, s, 12 + k as i32 * 18, y, false, &boa_palette(k));
    }
    draw_rider(&mut f, &art, 2, &boa_palette(1), Kind::Oma, Some(boa_bike_pal()), 140, y, false, 3, 1);
    art.dog[0].draw_foot(&mut f, 180, y, &PROPS, false);
    art.heron.draw_foot(&mut f, 200, y, &PROPS, false);
    art.heron_fly[0].draw(&mut f, 214, y - 20, &PROPS, false);
    let y = 32 + 5 * 34 + 2;
    for (k, &kind) in KINDS.iter().enumerate() {
        let lock = kind.def().lock;
        draw_bike(&mut f, &art, kind, k * 2, 20 + k as i32 * 42, y, false, 0, lock, k == 1, t);
    }
    art.krul.draw_foot(&mut f, 228, y + 2, &PROPS, false);
    let y = 222;
    art.glasbak.draw_foot(&mut f, 20, y, &PROPS, false);
    art.cutter.draw(&mut f, 40, y - 8, &PROPS, false);
    art.ticket.draw(&mut f, 60, y - 10, &PROPS, false);
    art.cat.draw(&mut f, 76, y - 7, &PROPS, false);
    art.coot.draw(&mut f, 96, y - 7, &PROPS, false);
    art.bollard.draw_foot(&mut f, 112, y, &PROPS, false);
    f.text(130, y - 8, "SHEET", rgb(248, 248, 248));
    let _ = OUTLINE;
    save_png(&f, &dir.join("50_sheet.png"), 3);
    println!("wrote 50_sheet.png");
}
