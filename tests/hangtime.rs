use retro_sports::engine::gfx::Frame;
use retro_sports::engine::input::Pad;
use retro_sports::games::hangtime::ai::Style;
use retro_sports::games::hangtime::app::{App, Scene};
use retro_sports::games::hangtime::audio;
use retro_sports::games::hangtime::course::{Venue, VENUES};
use retro_sports::games::hangtime::flight::*;
use retro_sports::games::hangtime::headless;

fn human_race(venue: Venue, seed: u64) -> Race {
    Race::new(venue, &[(0, Ctrl::Human(0), 0.0, Style::Normal)], seed)
}

/// Put pilot 0 in the air at (x, y), flying right at trim.
fn airborne(r: &mut Race, x: f32, y: f32) {
    let p = &mut r.pilots[0];
    p.state = State::Fly;
    p.x = x;
    p.y = y;
    p.v = TRIM;
    p.face = 1.0;
    p.start_at = 0;
}

#[test]
fn cpu_flights_always_end() {
    for venue in VENUES {
        for seed in 1..=4u64 {
            let entries: Vec<Entry> = vec![
                (0, Ctrl::Cpu, -0.3, Style::Normal),
                (1, Ctrl::Cpu, 0.9, Style::Normal),
                (2, Ctrl::Cpu, 0.5, Style::Dive),
                (3, Ctrl::Cpu, 0.5, Style::Cautious),
            ];
            let r = headless::fly(venue, &entries, seed);
            assert!(r.finished(), "{:?} seed {seed}: race did not end", venue);
            for p in &r.pilots {
                assert!(p.landing.is_some());
                assert!(p.clock < 60 * 60 * 6, "{:?}: pilot {} flew too long", venue, p.def);
            }
            // a skilled CPU makes the hoops and lands in the field
            let good = &r.pilots[1];
            assert!(good.gates_made() >= 3, "{:?} seed {seed}: skilled CPU made {} gates", venue, good.gates_made());
        }
    }
}

#[test]
fn thermals_give_lift() {
    for venue in VENUES {
        let r = human_race(venue, 3);
        for th in &r.course.thermals {
            let g = r.course.ground_at(th.x);
            let y = (g + th.top) / 2.0;
            assert!(r.course.thermal_lift(th.x, y, 0) > 0.2, "{:?}: weak thermal at {}", venue, th.x);
            assert!(r.course.thermal_lift(th.x + th.w * 3.0, y, 0) <= 0.0);
        }
        // fly through the core of the first thermal: the variometer goes up
        let th = r.course.thermals[1];
        let mut r = human_race(venue, 3);
        let g = r.course.ground_at(th.x);
        airborne(&mut r, th.x - 20.0, (g + th.top) / 2.0);
        let y0 = r.pilots[0].y;
        for _ in 0..40 {
            r.update(&[Pad::default(); 2]);
        }
        assert!(r.pilots[0].y > y0 + 4.0, "{:?}: no climb in the core ({} -> {})", venue, y0, r.pilots[0].y);
        assert!(r.pilots[0].vario > 0.1);
    }
}

#[test]
fn stalls_lose_height() {
    let mut calm = human_race(Venue::Dunes, 5);
    let mut pull = human_race(Venue::Dunes, 5);
    // over the flat polder, far from any lift
    for r in [&mut calm, &mut pull] {
        airborne(r, 2900.0, 260.0);
    }
    let up = Pad { up: true, ..Pad::default() };
    for _ in 0..240 {
        calm.update(&[Pad::default(); 2]);
        pull.update(&[up, Pad::default()]);
    }
    assert!(pull.pilots[0].stalls >= 1, "holding up never stalled");
    assert!(pull.pilots[0].y < calm.pilots[0].y - 8.0, "stalling should cost height: {} vs {}", pull.pilots[0].y, calm.pilots[0].y);
}

#[test]
fn launch_run_hop_and_tumble() {
    // a good run: alternate left and right, X in the zone
    let mut r = human_race(Venue::Alps, 1);
    let mut k = 0;
    while r.pilots[0].state != State::Fly && k < 600 {
        let p = &r.pilots[0];
        let (z0, _) = r.course.launch.zone();
        let mut pad = Pad::default();
        if p.state == State::Run {
            if k % 6 == 0 {
                pad.left = true;
            } else if k % 6 == 3 {
                pad.right = true;
            }
            pad.b = p.x > z0 + 6.0 && k % 2 == 0;
        }
        r.update(&[pad, Pad::default()]);
        k += 1;
    }
    assert_eq!(r.pilots[0].state, State::Fly);
    assert!(r.pilots[0].launch_v > 0.7, "slow launch {}", r.pilots[0].launch_v);
    // X right away: a hop
    let mut r = human_race(Venue::Alps, 1);
    let mut hopped = false;
    for k in 0..400 {
        let b = r.pilots[0].state == State::Run && k % 2 == 0;
        r.update(&[Pad { b, ..Pad::default() }, Pad::default()]);
        hopped |= matches!(r.pilots[0].state, State::Hop(_));
    }
    assert!(hopped);
    // never X: tumble off the edge
    let mut r = human_race(Venue::Alps, 1);
    let mut wobbled = false;
    for _ in 0..900 {
        r.update(&[Pad::default(); 2]);
        wobbled |= matches!(r.pilots[0].state, State::Wobble(_));
    }
    assert!(wobbled);
}

#[test]
fn flare_on_the_cross_is_a_bullseye() {
    let mut r = human_race(Venue::Dunes, 2);
    // keep the sheep off the cross for this one
    for s in r.sheep.iter_mut() {
        s.x = r.course.field.1 - 10.0;
    }
    let tx = r.course.target;
    let g = r.course.ground_at(tx);
    airborne(&mut r, tx - 70.0, g + 12.0);
    let mut flared = false;
    for _ in 0..400 {
        let p = &r.pilots[0];
        let b = p.state == State::Fly && p.agl(&r.course) < 6.0 && !flared;
        flared |= b;
        r.update(&[Pad { b, ..Pad::default() }, Pad::default()]);
        if r.pilots[0].landing.is_some() {
            break;
        }
    }
    let p = &r.pilots[0];
    assert!(matches!(p.landing, Some(Landing::Soft | Landing::Perfect)), "landing {:?}", p.landing);
    assert!(p.landing_pts >= 150, "landed {:.0} px from the cross", (p.land_x - tx).abs());
}

#[test]
fn no_flare_is_a_belly_flop_and_diving_in_crashes() {
    let mut r = human_race(Venue::Dunes, 2);
    let g = r.course.ground_at(3000.0);
    airborne(&mut r, 3000.0, g + 10.0);
    for _ in 0..300 {
        r.update(&[Pad::default(); 2]);
    }
    assert_eq!(r.pilots[0].landing, Some(Landing::Belly));
    let mut r = human_race(Venue::Dunes, 2);
    airborne(&mut r, 3000.0, g + 60.0);
    let down = Pad { down: true, c: true, ..Pad::default() };
    for _ in 0..400 {
        r.update(&[down, Pad::default()]);
    }
    assert_eq!(r.pilots[0].landing, Some(Landing::Crash));
}

#[test]
fn races_are_deterministic() {
    let entries: Vec<Entry> = (0..4).map(|k| (k, Ctrl::Cpu, 0.5, Style::Normal)).collect();
    let a = headless::fly(Venue::Canyon, &entries, 77);
    let b = headless::fly(Venue::Canyon, &entries, 77);
    for (p, q) in a.pilots.iter().zip(b.pilots.iter()) {
        assert_eq!(p.total(), q.total());
        assert_eq!(p.clock, q.clock);
        assert_eq!(p.land_x, q.land_x);
    }
}

#[test]
fn every_scene_draws() {
    let mut app = App::new(3);
    let mut f = Frame::new();
    app.prepare_stage();
    for scene in [Scene::Title, Scene::Select, Scene::Intro, Scene::Play, Scene::Results, Scene::Continue, Scene::GameOver, Scene::Champion] {
        app.scene = scene;
        app.draw(&mut f);
    }
    app.start_demo();
    for _ in 0..600 {
        app.update([Pad::default(); 2], false);
        app.draw(&mut f);
    }
}

#[test]
fn sounds_render() {
    for s in audio::ALL_SFX {
        assert!(!audio::sfx_samples(s).is_empty());
    }
    for t in audio::ALL_TRACKS {
        let wav = retro_sports::engine::synth::wav(&audio::music_samples(t));
        assert!(wav.len() > 44 + 44100);
    }
}
