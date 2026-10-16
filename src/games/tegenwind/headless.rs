//! Scripted screenshots of every screen and the key moments of a race,
//! plus sprite sheets. Player 1 is driven by a "decent" bot; a few moments
//! are set up by placing things right in front of the camera.

use super::app::{App, Mode, Scene};
use super::art::*;
use super::data::*;
use super::probe::{Bot, BotKind};
use super::race::*;
use super::render::draw_bike_rider;
use crate::engine::gfx::*;
use crate::engine::png::{save_png, save_sheet};
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

fn frame_of(app: &App) -> Frame {
    let mut f = Frame::new();
    app.draw(&mut f);
    f
}

/// Ride with bots for up to `n` frames, until `stop` says so.
fn ride_until(app: &mut App, bots: &mut [Bot; 2], n: usize, mut stop: impl FnMut(&App) -> bool) -> bool {
    for _ in 0..n {
        let mut pads = [Pad::default(); 2];
        if let Some(r) = app.race.as_ref() {
            for (k, &v) in r.views.iter().enumerate().take(2) {
                if matches!(r.riders[v].ctrl, Ctrl::Human(_)) {
                    pads[k] = bots[k].pad(r, v);
                }
            }
        }
        app.update(pads, false);
        if app.scene != Scene::Play {
            return false;
        }
        if stop(app) {
            return true;
        }
    }
    false
}

fn me(app: &App) -> &Rider {
    let r = app.race.as_ref().unwrap();
    &r.riders[r.views[0]]
}

fn race_mut(app: &mut App) -> &mut Race {
    app.race.as_mut().unwrap()
}

/// Put an object a little in front of player 1, in their lane.
fn place(app: &mut App, kind: OKind, ahead: f32, lane: Option<f32>) {
    let r = race_mut(app);
    let v = r.views[0];
    let (x, y) = (r.riders[v].x + ahead, lane.unwrap_or(r.riders[v].lane as f32));
    let mut o = r.objs.iter().find(|o| o.kind == kind).cloned().unwrap_or_else(|| r.objs[0].clone());
    o.kind = kind;
    o.x = x;
    o.y = y;
    o.state = 0;
    o.hit = 0;
    o.gone = false;
    o.t = 0;
    o.z = 0.0;
    o.vx = 0.0;
    r.objs.push(o);
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
    run(&mut app, 9);
    snap(&app, dir, "04_select_bakfiets.png");
    press(&mut app, Pad { up: true, ..Pad::default() });
    press(&mut app, a);
    run(&mut app, 70);
    snap(&app, dir, "05_intro_polder.png");
    press(&mut app, a);
    run(&mut app, 60);
    snap(&app, dir, "06_countdown.png");
    run(&mut app, 150);
    snap(&app, dir, "07_go.png");

    // ------------------------------------------------ the polder
    ride_until(&mut app, &mut bots, 60 * 12, |_| false);
    snap(&app, dir, "08_polder_riding.png");
    // sit on Oma Riek's wheel
    {
        let r = race_mut(&mut app);
        let v = r.views[0];
        let (x, lane) = (r.riders[v].x, r.riders[v].lane);
        if let Some(o) = r.riders.iter_mut().find(|q| q.who == Who::Oma) {
            o.x = x + 24.0;
            o.lane = lane;
            o.y = lane as f32;
            o.v = 2.2;
        }
    }
    ride_until(&mut app, &mut bots, 12, |_| false);
    snap(&app, dir, "09_draft.png");
    if ride_until(&mut app, &mut bots, 60 * 40, |a| {
        let r = a.race.as_ref().unwrap();
        me(a).tuck && r.wind_for(r.views[0]).0.head() > 0.5
    }) {
        snap(&app, dir, "10_tuck_headwind.png");
    }
    // a sheep crossing right in front
    {
        let r = race_mut(&mut app);
        let v = r.views[0];
        let x = r.riders[v].x + 120.0;
        let proto = r.objs[0].clone();
        for k in 0..5 {
            let mut o = proto.clone();
            o.kind = OKind::Sheep;
            o.x = x + k as f32 * 9.0;
            o.y = -0.2 - k as f32 * 0.42;
            o.state = 1;
            o.gone = false;
            o.hit = 0;
            o.var = k as u8;
            r.objs.push(o);
        }
    }
    ride_until(&mut app, &mut bots, 40, |_| false);
    snap(&app, dir, "11_sheep_crossing.png");
    ride_until(&mut app, &mut bots, 60 * 4, |_| false);
    place(&mut app, OKind::Goose, 150.0, None);
    ride_until(&mut app, &mut bots, 100, |a| a.race.as_ref().unwrap().objs.iter().any(|o| o.kind == OKind::Goose && o.state == 1));
    ride_until(&mut app, &mut bots, 30, |_| false);
    snap(&app, dir, "12_goose_chase.png");
    place(&mut app, OKind::Food(Food::Kaas), 70.0, None);
    ride_until(&mut app, &mut bots, 60, |a| me(a).stats.eaten > 0);
    ride_until(&mut app, &mut bots, 6, |_| false);
    snap(&app, dir, "13_eat_cheese.png");
    place(&mut app, OKind::Cat, 110.0, None);
    ride_until(&mut app, &mut bots, 60, |a| me(a).z > 6.0);
    snap(&app, dir, "14_hop.png");
    // the cheese stand, a rain shower with an umbrella, a stork
    {
        let r = race_mut(&mut app);
        let v = r.views[0];
        let sx = r.stands[0].0;
        r.riders[v].x = sx - 40.0;
        r.rare = Some((Rare::Stork, sx + 140.0));
    }
    ride_until(&mut app, &mut bots, 10, |_| false);
    snap(&app, dir, "14b_cheese_stand_stork.png");
    {
        let r = race_mut(&mut app);
        let v = r.views[0];
        let x = r.riders[v].x;
        r.shower = Some((x + 60.0, x + 900.0));
    }
    place(&mut app, OKind::Umbrella, 40.0, None);
    ride_until(&mut app, &mut bots, 70, |_| false);
    snap(&app, dir, "14c_shower_umbrella.png");
    place(&mut app, OKind::Pothole, 300.0, Some(0.0));
    place(&mut app, OKind::Bollard, 330.0, Some(1.0));
    ride_until(&mut app, &mut bots, 30, |_| false);
    snap(&app, dir, "14d_look_ahead.png");
    race_mut(&mut app).shower = None;
    race_mut(&mut app).riders[0].umbrella = false;
    // a crash: a bollard at speed
    ride_until(&mut app, &mut bots, 60 * 3, |_| false);
    {
        let r = race_mut(&mut app);
        let v = r.views[0];
        r.riders[v].v = 2.4;
    }
    place(&mut app, OKind::Bollard, 30.0, None);
    let mut crash = Bot::new(BotKind::Metronome, 9);
    for _ in 0..80 {
        let pad = crash.pad(app.race.as_ref().unwrap(), 0);
        app.update([pad, Pad::default()], false);
        if me(&app).state == RState::Fly && me(&app).st == 14 {
            snap(&app, dir, "15_crash_fly.png");
        }
        if me(&app).state == RState::Swim && me(&app).st == 18 {
            snap(&app, dir, "16_crash_plons.png");
            break;
        }
    }
    ride_until(&mut app, &mut bots, 200, |a| me(a).state == RState::Climb && me(a).st == 14);
    snap(&app, dir, "17_climb_out.png");
    ride_until(&mut app, &mut bots, 90, |_| false);
    snap(&app, dir, "18_dripping.png");
    // the man with the hammer
    race_mut(&mut app).riders[0].energy = 0.3;
    let mut mash = Bot::new(BotKind::Masher, 5);
    for _ in 0..90 {
        let pad = mash.pad(app.race.as_ref().unwrap(), 0);
        app.update([pad, Pad::default()], false);
        if me(&app).hammer == 120 {
            break;
        }
    }
    snap(&app, dir, "19_man_with_the_hammer.png");
    // pedalling in rhythm, as a sheet
    let mut sheet = Vec::new();
    for k in 0..24 {
        ride_until(&mut app, &mut bots, 1, |_| false);
        if k % 3 == 0 {
            sheet.push(frame_of(&app));
        }
    }
    save_sheet(&sheet, 4, &dir.join("20_pedal_sequence.png"), 2);
    println!("wrote 20_pedal_sequence.png");
    // pause
    press(&mut app, Pad { start: true, ..Pad::default() });
    snap(&app, dir, "21_pause.png");
    press(&mut app, Pad { start: true, ..Pad::default() });
    // ride to the finish
    ride_until(&mut app, &mut bots, 60 * 200, |a| me(a).finish.is_some());
    ride_until(&mut app, &mut bots, 40, |_| false);
    snap(&app, dir, "22_finish.png");
    ride_until(&mut app, &mut bots, 60 * 30, |_| false);
    run(&mut app, 50);
    snap(&app, dir, "23_results_tally.png");
    run(&mut app, 200);
    snap(&app, dir, "24_results.png");

    // ------------------------------------------------ Amsterdam
    app.mode = Mode::One;
    app.stage = 1;
    app.scene = Scene::Intro;
    app.t = 0;
    run(&mut app, 70);
    snap(&app, dir, "25_intro_amsterdam.png");
    app.start_race(1);
    run(&mut app, COUNTDOWN as usize + 2);
    ride_until(&mut app, &mut bots, 60 * 20, |a| me(a).x > 1500.0);
    snap(&app, dir, "26_cobbles.png");
    {
        let r = race_mut(&mut app);
        let v = r.views[0];
        let sx = r.stands[0].0;
        r.riders[v].x = sx - 40.0;
        r.rare = Some((Rare::PartyBoat, sx + 150.0));
    }
    ride_until(&mut app, &mut bots, 10, |_| false);
    snap(&app, dir, "26b_herring_cart_party_boat.png");
    place(&mut app, OKind::Tourist, 90.0, None);
    ride_until(&mut app, &mut bots, 40, |_| false);
    snap(&app, dir, "27_tourist.png");
    {
        let r = race_mut(&mut app);
        let v = r.views[0];
        r.riders[v].bell_t = 0;
    }
    app.update([a, Pad::default()], false);
    ride_until(&mut app, &mut bots, 8, |_| false);
    snap(&app, dir, "28_tring_tring.png");
    place(&mut app, OKind::Rails, 60.0, None);
    ride_until(&mut app, &mut bots, 40, |a| me(a).z > 3.0);
    snap(&app, dir, "29_tram_rails.png");
    ride_until(&mut app, &mut bots, 60 * 100, |a| a.race.as_ref().unwrap().bridge.as_ref().map(|b| b.state == BState::Warn).unwrap_or(false));
    // dawdle so the barrier comes down in front of us
    for _ in 0..70 {
        let r = race_mut(&mut app);
        let v = r.views[0];
        r.riders[v].v = r.riders[v].v.min(0.9);
        app.update([Pad::default(); 2], false);
    }
    snap(&app, dir, "30_bridge_alarm.png");
    ride_until(&mut app, &mut bots, 60 * 10, |a| a.race.as_ref().unwrap().bridge.as_ref().map(|b| b.lift > 0.9).unwrap_or(false));
    ride_until(&mut app, &mut bots, 30, |_| false);
    snap(&app, dir, "31_bridge_open.png");
    ride_until(&mut app, &mut bots, 60 * 12, |a| a.race.as_ref().unwrap().tram.map(|x| x < 120.0).unwrap_or(false));
    snap(&app, dir, "32_tram.png");
    place(&mut app, OKind::Sofa, 110.0, Some(0.0));
    if let Some(o) = race_mut(&mut app).objs.last_mut() {
        o.vx = 1.2;
    }
    ride_until(&mut app, &mut bots, 24, |_| false);
    snap(&app, dir, "33_sofa_lady.png");

    // ------------------------------------------------ the dike
    app.stage = 2;
    app.scene = Scene::Intro;
    app.t = 0;
    run(&mut app, 70);
    snap(&app, dir, "34_intro_afsluitdijk.png");
    app.start_race(2);
    run(&mut app, COUNTDOWN as usize + 2);
    ride_until(&mut app, &mut bots, 60 * 25, |_| false);
    snap(&app, dir, "35_storm.png");
    {
        let r = race_mut(&mut app);
        let v = r.views[0];
        let x = r.riders[v].x;
        r.rare = Some((Rare::Kite, x + 30.0));
    }
    ride_until(&mut app, &mut bots, 10, |_| false);
    snap(&app, dir, "35b_kite_surfer.png");
    ride_until(&mut app, &mut bots, 60 * 60, |a| {
        let r = a.race.as_ref().unwrap();
        r.wind_for(r.views[0]).0.side() > 0.0 && me(a).y > 1.15
    });
    snap(&app, dir, "36_side_wind.png");
    {
        let r = race_mut(&mut app);
        let v = r.views[0];
        let sx = r.def().snackbar.unwrap_or(7000.0);
        r.riders[v].x = sx - 110.0;
    }
    ride_until(&mut app, &mut bots, 40, |_| false);
    snap(&app, dir, "37_snackbar.png");
    // lose on purpose: continue and game over
    ride_until(&mut app, &mut bots, 60 * 300, |_| false);
    run(&mut app, 400);
    if app.scene == Scene::Results {
        app.last_place = 3;
        app.t = 1000;
        press(&mut app, a);
    }
    if app.scene != Scene::Continue {
        app.scene = Scene::Continue;
        app.count = 9;
        app.t = 0;
    }
    run(&mut app, 70);
    snap(&app, dir, "38_continue.png");
    app.scene = Scene::GameOver;
    app.t = 0;
    run(&mut app, 90);
    snap(&app, dir, "39_game_over.png");
    app.scene = Scene::Champion;
    app.t = 0;
    app.score = [4210, 0];
    run(&mut app, 2);
    app.mode = Mode::One;
    // confetti comes from make_confetti; go through the real path
    let mut app_c = App::new(5);
    app_c.mode = Mode::One;
    app_c.stage = 2;
    app_c.start_race(2);
    {
        let r = race_mut(&mut app_c);
        for (k, rd) in r.riders.iter_mut().enumerate() {
            rd.x = r.length - 10.0 - k as f32 * 40.0;
            rd.v = 2.0;
        }
    }
    ride_until(&mut app_c, &mut bots, 60 * 60, |_| false);
    run(&mut app_c, 60 * 12);
    app_c.last_place = 0;
    press(&mut app_c, a);
    run(&mut app_c, 160);
    snap(&app_c, dir, "40_champion.png");

    // ------------------------------------------------ attract mode
    let mut app_d = App::new(9);
    run(&mut app_d, super::app::IDLE_DEMO as usize + 600);
    snap(&app_d, dir, "41_demo.png");

    // ------------------------------------------------ two players
    let mut app2 = App::new(7);
    run(&mut app2, 30);
    press(&mut app2, a);
    press(&mut app2, Pad { down: true, ..Pad::default() });
    press(&mut app2, a);
    run(&mut app2, 10);
    app2.update([Pad::default(), Pad { right: true, ..Pad::default() }], false);
    run(&mut app2, 10);
    snap(&app2, dir, "42_select_2p.png");
    app2.update([a, a], false);
    run(&mut app2, 50);
    snap(&app2, dir, "43_intro_2p.png");
    app2.update([a, a], false);
    run(&mut app2, COUNTDOWN as usize - 40);
    snap(&app2, dir, "44_split_countdown.png");
    ride_until(&mut app2, &mut bots, 60 * 25, |_| false);
    snap(&app2, dir, "45_split_race.png");
    ride_until(&mut app2, &mut bots, 60 * 40, |_| false);
    snap(&app2, dir, "46_split_race_2.png");
    ride_until(&mut app2, &mut bots, 60 * 300, |_| false);
    run(&mut app2, 300);
    snap(&app2, dir, "47_results_2p.png");
    app2.stage = 2;
    app2.score = [3120, 3480];
    app2.t = 10_000;
    press(&mut app2, a);
    run(&mut app2, 150);
    snap(&app2, dir, "48_champion_2p.png");

    sprite_sheets(&app.art, dir);
}

/// Every rider on every bike in every crank frame, and the props.
fn sprite_sheets(art: &Art, dir: &Path) {
    let bg = rgb(96, 128, 112);
    let mut frames = Vec::new();
    for who in ALL_WHO {
        for bike in BIKES {
            if !art.sets.iter().any(|s| s.who == who && s.bike == bike) {
                continue;
            }
            let mut f = Frame::new();
            f.clear(bg);
            f.text(4, 4, &format!("{} - {}", who.def().short, bike.def().name), rgb(248, 248, 248));
            for k in 0..8 {
                for tuck in 0..2 {
                    let x = 24 + k as i32 * 30;
                    let y = 52 + tuck * 46;
                    draw_bike_rider(&mut f, art, who, bike, x, y, k, k as f32 * 2.0, tuck == 1, false, false, Some((Cargo::Cheese, 3)), 0);
                }
            }
            draw_bike_rider(&mut f, art, who, bike, 30, 150, 0, 0.0, false, true, false, Some((Cargo::Tulips, 3)), 0);
            draw_bike_rider(&mut f, art, who, bike, 80, 150, 2, 0.0, false, false, true, Some((Cargo::Herring, 3)), 0);
            let p = art.people(who);
            let pal = rider_palette(who);
            for (k, s) in [&p.stand, &p.cheer, &p.victory, &p.fly, &p.down, &p.swim].iter().enumerate() {
                blit_person(&mut f, s, 130 + k as i32 * 22, 150, false, &pal);
            }
            frames.push(f);
        }
    }
    save_sheet(&frames, 3, &dir.join("50_sheet_riders.png"), 2);
    println!("wrote 50_sheet_riders.png");

    // zoomed: crank frames of each bike posture, to judge the pedalling
    let mut z = Frame::new();
    z.clear(bg);
    for (row, (who, bike)) in [(Who::Kees, Bike::Oma), (Who::Dennis, Bike::Race), (Who::DeVries, Bike::Bak), (Who::Anouk, Bike::Lig)].iter().enumerate() {
        for k in 0..4 {
            let x = 22 + k as i32 * 32;
            draw_bike_rider(&mut z, art, *who, *bike, x, 34 + row as i32 * 36, k * 2, k as f32 * 2.0, false, false, false, Some((Cargo::Tulips, 2)), 0);
        }
    }
    save_png(&z, &dir.join("52_zoom_pedal.png"), 6);
    println!("wrote 52_zoom_pedal.png");

    let mut f = Frame::new();
    f.clear(bg);
    let items: Vec<&Bmp> = vec![
        &art.cow,
        &art.cow_graze,
        &art.sheep[0],
        &art.sheep[1],
        &art.goose[0],
        &art.goose[1],
        &art.cat,
        &art.cat_run,
        &art.duck,
        &art.gull[0],
        &art.gull[1],
        &art.dog,
        &art.kids[0],
        &art.kids[1],
        &art.food[0],
        &art.food[1],
        &art.food[2],
        &art.food[3],
        &art.bollard,
        &art.herring,
        &art.hammer,
        &art.bell,
        &art.camera,
        &art.sofa,
        &art.tractor,
    ];
    let (mut x, mut y) = (4, 4);
    for b in items {
        if x + b.w > 252 {
            x = 4;
            y += 34;
        }
        b.draw(&mut f, x, y, &PROPS, false);
        x += b.w + 6;
    }
    for (k, c) in [Cargo::Cheese, Cargo::Tulips, Cargo::Herring, Cargo::Tuba, Cargo::Clock].iter().enumerate() {
        art.cargo_bmp(*c).draw(&mut f, 4 + k as i32 * 14, 150, &PROPS, false);
    }
    art.mill.draw(&mut f, 110, 140, &PROPS, false);
    super::scenery::big_mill(&mut f, art, 190, 214, 0.3, (0, H));
    for (k, wheel) in art.wheels.iter().enumerate() {
        for (j, w) in wheel.iter().enumerate() {
            w.draw(&mut f, 4 + j as i32 * 14, 170 + k as i32 * 13, &PROPS, false);
        }
    }
    let mut bikes = Frame::new();
    bikes.clear(bg);
    for (k, b) in art.bikes.iter().enumerate() {
        b.draw(&mut bikes, 8 + (k as i32 % 2) * 120, 8 + (k as i32 / 2) * 40, &PROPS, false);
    }
    f.rect_outline(0, 0, W, H, OUTLINE);
    save_sheet(&[f, bikes], 2, &dir.join("51_sheet_props.png"), 3);
    println!("wrote 51_sheet_props.png");
}
