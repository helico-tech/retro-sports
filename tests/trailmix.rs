use retro_sports::engine::gfx::Frame;
use retro_sports::engine::input::Pad;
use retro_sports::games::trailmix::app::{App, Mode, Scene};
use retro_sports::games::trailmix::audio;
use retro_sports::games::trailmix::headless::{self, lineup, Bot, BotKind, GOAT_FAC};
use retro_sports::games::trailmix::race::*;
use retro_sports::games::trailmix::runners::GERALD;
use std::time::Instant;

fn run(race: &mut Race) {
    while !race.finished() && race.t < RACE_CAP {
        race.update(&[Pad::default(); 2]);
        race.sfx.clear();
    }
}

#[test]
fn cpu_races_always_finish() {
    for (stage, &fac) in GOAT_FAC.iter().enumerate() {
        for seed in 1..=4u64 {
            let mut racers = vec![(GERALD, Ctrl::Goat(fac))];
            for k in 0..3 {
                racers.push(((seed as usize + k) % 4, Ctrl::Cpu(Strategy::Smart, 0.3 + 0.2 * k as f32)));
            }
            let mut race = Race::new(stage, &racers, seed);
            run(&mut race);
            assert!(race.finished(), "stage {stage} seed {seed} did not finish");
            assert!(race.t < RACE_CAP, "stage {stage} seed {seed} hit the time cap");
            let secs = race.result_time(race.ranking()[0]) as f32 / 60.0;
            assert!((100.0..240.0).contains(&secs), "stage {stage}: winner took {secs:.0} s");
        }
    }
}

#[test]
fn sprinting_everything_loses_to_pacing() {
    for stage in 0..3 {
        let (mut smart, mut sprint, mut smart_wins) = (0u64, 0u64, 0);
        for seed in 0..8u64 {
            let def = seed as usize % 4;
            let racers = [(def, Ctrl::Cpu(Strategy::Smart, 0.7)), (def, Ctrl::Cpu(Strategy::Sprinter, 0.7))];
            let mut race = Race::new(stage, &racers, seed + 1);
            run(&mut race);
            smart += race.result_time(0) as u64;
            sprint += race.result_time(1) as u64;
            if race.result_time(0) < race.result_time(1) {
                smart_wins += 1;
            }
            assert!(race.runners[1].stats.bonks > race.runners[0].stats.bonks, "the sprinter should bonk more");
        }
        assert!(smart < sprint, "stage {stage}: pacing should beat sprinting");
        assert!(smart_wins >= 7, "stage {stage}: pacing won only {smart_wins} of 8");
    }
}

#[test]
fn a_human_who_never_presses_anything_still_finishes() {
    for stage in 0..3 {
        let mut race = Race::new(stage, &lineup(stage, 2, Ctrl::Human(0)), 7);
        run(&mut race);
        assert!(race.finished());
        assert!(race.runners[3].finished.is_some(), "stage {stage}: idle human did not finish");
        assert_ne!(race.ranking()[0], 3, "an idle human should not win");
    }
}

#[test]
fn bunny_hopping_is_not_an_exploit() {
    let stage = 0;
    let play = |kind: BotKind| -> u32 {
        let mut race = Race::new(stage, &lineup(stage, 1, Ctrl::Human(0)), 3);
        let mut bot = Bot::new(kind, 3);
        while !race.finished() && race.t < RACE_CAP {
            let p = bot.pad(&race, 3);
            race.update(&[p, Pad::default()]);
            race.sfx.clear();
        }
        race.result_time(3)
    };
    assert!(play(BotKind::Decent) < play(BotKind::Hopper));
    assert!(play(BotKind::Decent) < play(BotKind::Sprinter));
}

#[test]
fn races_are_deterministic() {
    let go = || {
        let mut race = Race::new(1, &lineup(1, 0, Ctrl::Cpu(Strategy::Smart, 0.6)), 99);
        run(&mut race);
        (0..race.runners.len()).map(|i| race.result_time(i)).collect::<Vec<_>>()
    };
    assert_eq!(go(), go());
}

#[test]
fn the_ladder_reaches_the_champion_screen() {
    let mut app = App::new(4);
    app.mode = Mode::One;
    app.cursor[0] = 0;
    let mut bots = Bot::new(BotKind::Decent, 8);
    // title -> menu -> select -> intro
    let press = |app: &mut App, p: Pad| {
        app.update([p, Pad::default()], false);
        app.update([Pad::default(); 2], false);
    };
    let a = Pad { a: true, ..Pad::default() };
    for _ in 0..30 {
        app.update([Pad::default(); 2], false);
    }
    press(&mut app, a);
    press(&mut app, a);
    press(&mut app, a);
    for _ in 0..20 {
        app.update([Pad::default(); 2], false);
    }
    assert_eq!(app.scene, Scene::Intro);
    let mut frames = 0;
    while app.scene != Scene::Champion {
        let p = match (&app.scene, &app.race) {
            (Scene::Play, Some(race)) => bots.pad(race, race.views[0]),
            (Scene::Results, _) if frames % 2 == 0 => {
                // let the bot qualify even on a bad day
                app.qualified = true;
                a
            }
            (Scene::Intro, _) | (Scene::Continue, _) if frames % 2 == 0 => a,
            _ => Pad::default(),
        };
        app.update([p, Pad::default()], false);
        frames += 1;
        assert!(frames < 60 * 60 * 15, "ladder took too long, stuck in {:?}", app.scene);
    }
    let mut f = Frame::new();
    app.draw(&mut f);
}

#[test]
fn every_scene_draws() {
    let mut app = App::new(3);
    let mut f = Frame::new();
    for scene in [Scene::Title, Scene::Select, Scene::Intro, Scene::Results, Scene::Continue, Scene::GameOver, Scene::Champion] {
        app.scene = scene;
        app.draw(&mut f);
    }
    for mode in [Mode::One, Mode::Two] {
        app.mode = mode;
        app.scene = Scene::Select;
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

#[test]
fn new_is_fast_and_frames_are_cheap() {
    let t0 = Instant::now();
    let mut app = App::new(1);
    let new_ms = t0.elapsed().as_secs_f64() * 1000.0;
    // 2P split screen is the heaviest frame
    app.mode = Mode::Two;
    app.cursor = [0, 1];
    app.start_race(&[(2, Ctrl::Cpu(Strategy::Smart, 0.5)), (GERALD, Ctrl::Goat(0.9)), (1, Ctrl::Human(1)), (0, Ctrl::Human(0))]);
    let mut f = Frame::new();
    for _ in 0..300 {
        app.update([Pad { right: true, ..Pad::default() }; 2], false);
    }
    let t1 = Instant::now();
    for _ in 0..120 {
        app.update([Pad::default(); 2], false);
        app.draw(&mut f);
    }
    let frame_ms = t1.elapsed().as_secs_f64() * 1000.0 / 120.0;
    println!("App::new {new_ms:.1} ms, 2P frame {frame_ms:.2} ms");
    if !cfg!(debug_assertions) {
        assert!(new_ms < 60.0, "App::new took {new_ms:.1} ms");
        assert!(frame_ms < 4.0, "a 2P frame took {frame_ms:.2} ms");
    }
}

#[test]
#[ignore]
fn sim_dev() {
    headless::sim(40);
}

#[test]
#[ignore]
fn probe_dev() {
    headless::probe(10);
}

#[test]
#[ignore]
fn trace_dev() {
    headless::trace(1);
}
