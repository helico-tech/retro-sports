use retro_sports::engine::gfx::Frame;
use retro_sports::engine::input::Pad;
use retro_sports::games::zenuwen::app::{App, Mode, Scene};
use retro_sports::games::zenuwen::bot::{Bot, BotKind};
use retro_sports::games::zenuwen::data::*;
use retro_sports::games::zenuwen::probe::run_stage;
use retro_sports::games::zenuwen::world::*;

const A: Pad = Pad { up: false, down: false, left: false, right: false, a: true, b: false, c: false, start: false };

/// A quiet street: the BOA's far away, nobody walking.
fn quiet(stage: usize, who: Who) -> World {
    let mut w = World::new(stage, who, Ctrl::Human(0), None, 3, None);
    w.phase = Phase::Play;
    w.peds.clear();
    w.herons.clear();
    for b in &mut w.boas {
        b.x = 5000.0;
        b.x0 = 5000.0;
        b.x1 = 5000.0;
    }
    w
}

fn step(w: &mut World, p: Pad) {
    w.update(&[p, Pad::default()]);
}

/// Press and release.
fn tap(w: &mut World, p: Pad) {
    step(w, p);
    step(w, Pad::default());
}

/// Put the thief right next to a parked bike of `kind`, with `lock`.
fn at_bike(w: &mut World, kind: Kind, lock: Lock) -> usize {
    let i = w.bikes.iter().position(|b| b.kind == kind && b.state == BikeState::Parked).unwrap();
    w.bikes[i].lock = lock;
    w.bikes[i].lock0 = lock;
    w.thief.x = w.bikes[i].x;
    w.thief.y = w.bikes[i].y;
    i
}

#[test]
fn cpu_thief_always_finishes_a_stage() {
    for (stage, def) in STAGES.iter().enumerate() {
        for seed in 1..=4u64 {
            let r = run_stage(stage, THIEVES[seed as usize % 4], BotKind::Cpu(0.7), seed);
            assert!(r.secs > 5.0, "stage {stage} seed {seed}: over in {:.0}s", r.secs);
            assert!(r.secs <= def.time as f32 + 0.1, "stage {stage} seed {seed}: ran past the clock");
        }
    }
}

#[test]
fn same_seed_same_run() {
    let run = || {
        let r = run_stage(1, Who::Sanne, BotKind::Decent, 77);
        (r.outcome, r.secs, r.fines, r.cash, r.delivered)
    };
    assert_eq!(run(), run());
}

#[test]
fn a_ring_lock_opens_with_taps() {
    let mut w = quiet(0, Who::Joost);
    let i = at_bike(&mut w, Kind::Oma, Lock::Ring);
    tap(&mut w, A);
    assert_eq!(w.thief.state, TState::Pick);
    for _ in 0..Who::Joost.ring_taps() {
        for _ in 0..3 {
            step(&mut w, Pad::default());
        }
        tap(&mut w, A);
    }
    assert_eq!(w.thief.state, TState::Ride, "progress {}", w.bikes[i].progress);
    assert_eq!(w.thief.bike, Some(i));
    assert!(w.bikes[i].hot > 0, "a freshly stolen bike is hot");
}

#[test]
fn a_chain_needs_sawing_both_ways() {
    let mut w = quiet(0, Who::Prof);
    let i = at_bike(&mut w, Kind::Bak, Lock::Chain);
    tap(&mut w, A);
    // only left: one stroke, then nothing
    for _ in 0..30 {
        tap(&mut w, Pad { left: true, ..Pad::default() });
    }
    let one = w.bikes[i].progress;
    assert!(one > 0.0 && one < 0.2, "progress {one}");
    for k in 0..Who::Prof.chain_strokes() {
        tap(&mut w, if k % 2 == 0 { Pad { right: true, ..Pad::default() } } else { Pad { left: true, ..Pad::default() } });
    }
    assert_eq!(w.thief.state, TState::Ride);
}

#[test]
fn a_missed_u_lock_clanks_and_brings_a_boa() {
    let mut w = quiet(0, Who::Sanne);
    let i = at_bike(&mut w, Kind::Race, Lock::U);
    w.boas[0].x = w.thief.x + 90.0;
    w.boas[0].y = w.thief.y;
    w.boas[0].x0 = w.boas[0].x;
    w.boas[0].x1 = w.boas[0].x;
    w.boas[0].face_left = false;
    tap(&mut w, A);
    // press when the needle is as far from the green as it gets
    let mut pressed = false;
    for _ in 0..200 {
        let n = 0.5 + 0.5 * w.thief.needle.sin();
        if (n - w.thief.zone).abs() > 0.3 {
            tap(&mut w, A);
            pressed = true;
            break;
        }
        step(&mut w, Pad::default());
    }
    assert!(pressed);
    assert_eq!(w.thief.stats.clanks, 1);
    assert_eq!(w.bikes[i].progress, 0.0);
    assert!(w.boas[0].face_left, "the BOA turns toward the clank");
}

#[test]
fn the_cutter_opens_anything_loudly() {
    let mut w = quiet(2, Who::Joost);
    let i = at_bike(&mut w, Kind::Gold, Lock::Double);
    let x = w.thief.x;
    w.boas[0].x = x + 150.0;
    w.boas[0].y = 30.0;
    tap(&mut w, A);
    tap(&mut w, Pad { b: true, ..Pad::default() });
    assert_eq!(w.thief.state, TState::Ride);
    assert_eq!(w.thief.bike, Some(i));
    assert_eq!(w.thief.cutters, STAGES[2].cutters - 1);
    assert_eq!(w.boas[0].state, BoaState::Investigate, "everyone heard the KRAK");
}

#[test]
fn boas_see_ahead_not_behind_and_not_through_trees() {
    let w = quiet(0, Who::Joost);
    let mut b = w.boas[0].clone();
    b.x = 300.0;
    b.y = 30.0;
    b.face_left = false;
    b.state = BoaState::Patrol;
    assert!(w.sight(&b, 340.0, 30.0) > 0.0);
    assert_eq!(w.sight(&b, 260.0, 30.0), 0.0, "behind");
    assert_eq!(w.sight(&b, 300.0 + b.vision + 5.0, 30.0), 0.0, "too far");
    // a tree on the quay at x 430 (Prinsengracht)
    let (_, tx, ty) = *w.spots.iter().find(|s| s.0 == Spot::Tree && s.1 == 430.0).unwrap();
    b.x = tx - 40.0;
    b.y = ty;
    assert_eq!(w.sight(&b, tx + 20.0, ty), 0.0, "the tree is in the way");
}

#[test]
fn picking_in_view_gets_you_fined_and_three_fines_end_it() {
    let mut w = quiet(0, Who::Joost);
    at_bike(&mut w, Kind::Oma, Lock::Ring);
    let (x, y) = (w.thief.x, w.thief.y);
    let b = &mut w.boas[0];
    b.x = x + 70.0;
    b.y = y;
    b.x0 = b.x - 200.0;
    b.x1 = b.x + 200.0;
    b.face_left = true;
    b.next_look = 99999;
    tap(&mut w, A);
    let mut wondered = false;
    for _ in 0..300 {
        step(&mut w, Pad::default());
        wondered |= w.boas[0].sus > 0.35;
        if w.thief.fines > 0 {
            break;
        }
    }
    assert!(wondered, "the BOA got suspicious");
    assert_eq!(w.thief.fines, 1, "caught in the act");
    // two more
    for _ in 0..2 {
        while w.thief.state == TState::Busted {
            step(&mut w, Pad::default());
        }
        w.thief.grace = 0;
        let (x, y) = (w.thief.x, w.thief.y);
        let b = &mut w.boas[0];
        b.x = x + 6.0;
        b.y = y;
        b.state = BoaState::Chase;
        b.chase_t = 100;
        b.target = (x, y);
        step(&mut w, Pad::default());
    }
    assert_eq!(w.thief.fines, 3);
    for _ in 0..400 {
        step(&mut w, Pad::default());
    }
    assert_eq!(w.result, Some(Outcome::Busted));
}

#[test]
fn a_full_suspicion_starts_a_chase_after_the_whistle() {
    let mut w = quiet(0, Who::Joost);
    w.thief.x = 200.0;
    w.thief.y = 30.0;
    let b = &mut w.boas[0];
    b.x = 260.0;
    b.y = 30.0;
    b.x0 = 0.0;
    b.x1 = 900.0;
    b.face_left = true;
    b.next_look = 99999;
    b.sus = 1.0;
    step(&mut w, Pad::default());
    assert_eq!(w.boas[0].state, BoaState::Chase);
    assert!(w.thief.wanted > 0);
    let x0 = w.boas[0].x;
    for _ in 0..CHASE_WHISTLE - 2 {
        step(&mut w, Pad::default());
    }
    assert_eq!(w.boas[0].x, x0, "whistling first");
    for _ in 0..20 {
        step(&mut w, Pad::default());
    }
    assert!(w.boas[0].x < x0 - 10.0, "then running");
}

#[test]
fn riding_off_the_quay_sinks_the_bike() {
    let mut w = quiet(0, Who::Sanne);
    let i = at_bike(&mut w, Kind::Oma, Lock::None);
    tap(&mut w, A);
    assert_eq!(w.thief.state, TState::Ride);
    for _ in 0..140 {
        step(&mut w, Pad { down: true, ..Pad::default() });
    }
    assert_eq!(w.thief.state, TState::Swim);
    assert_eq!(w.bikes[i].state, BikeState::Sunk);
    // climb out again
    for _ in 0..60 {
        step(&mut w, Pad { up: true, ..Pad::default() });
    }
    for _ in 0..40 {
        step(&mut w, Pad::default());
    }
    assert_eq!(w.thief.state, TState::Walk);
}

#[test]
fn throwing_bikes_on_sjaaks_boat_fills_the_order() {
    let mut w = quiet(0, Who::Joost);
    let order: Vec<Kind> = w.order.iter().map(|o| o.0).collect();
    for kind in order {
        let i = at_bike(&mut w, kind, Lock::None);
        w.sjaak.x = w.bikes[i].x;
        w.sjaak.wait = 9999;
        w.thief.y = DEPTH - 2.0;
        w.bikes[i].y = DEPTH - 2.0;
        tap(&mut w, A);
        assert_eq!(w.thief.state, TState::Ride);
        tap(&mut w, A);
        assert_eq!(w.thief.state, TState::Toss);
        for _ in 0..40 {
            step(&mut w, Pad::default());
        }
    }
    assert!(w.order_done());
    assert_eq!(w.result, Some(Outcome::Cleared));
    assert!(w.cash() > 0);
}

#[test]
fn the_player_boa_pays_for_grabbing_an_innocent() {
    let mut w = World::new(STAGE_2P_TEST, Who::Joost, Ctrl::Human(0), Some(1), 5, None);
    w.phase = Phase::Play;
    let h = w.human_boa.unwrap();
    w.boas[h].x = w.thief.x + 8.0;
    w.boas[h].y = w.thief.y;
    w.update(&[Pad::default(), A]);
    assert_eq!(w.boas[h].complaints, 1);
    assert_eq!(w.thief.fines, 0);
    // on a stolen bike it sticks
    let i = at_bike(&mut w, Kind::Oma, Lock::None);
    tap(&mut w, A);
    assert_eq!(w.thief.bike, Some(i));
    w.thief.grace = 0;
    w.boas[h].x = w.thief.x + 8.0;
    w.boas[h].y = w.thief.y;
    w.boas[h].grab_t = 0;
    w.update(&[Pad::default(), A]);
    assert_eq!(w.boas[h].fines, 1);
    assert_eq!(w.thief.fines, 1);
}

const STAGE_2P_TEST: usize = retro_sports::games::zenuwen::app::STAGE_2P;

#[test]
fn every_scene_draws_in_1p_and_2p() {
    let mut f = Frame::new();
    for mode in [Mode::One, Mode::Two] {
        let mut app = App::new(4);
        app.mode = mode;
        let mut bots = [Bot::new(BotKind::Decent, 1), Bot::new(BotKind::Decent, 2)];
        app.start_play();
        let mut seen = vec![app.scene];
        for t in 0..60 * 60 * 8 {
            let mut pads = [Pad::default(); 2];
            if let Some(w) = app.world.as_ref()
                && let Ctrl::Human(p) = w.thief.ctrl {
                    pads[p.min(1)] = bots[p.min(1)].pad(w);
                }
            if app.scene != Scene::Play && t % 50 == 0 {
                pads[0].a = true;
            }
            app.update(pads, false);
            if !seen.contains(&app.scene) {
                seen.push(app.scene);
            }
            if t % 31 == 0 {
                app.draw(&mut f);
            }
            if app.scene == Scene::Title {
                break;
            }
        }
        assert!(seen.contains(&Scene::Results), "{mode:?}: never got to the results ({seen:?})");
    }
}
