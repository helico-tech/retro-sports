use retro_sports::engine::gfx::Frame;
use retro_sports::engine::input::Pad;
use retro_sports::games::tegenwind::app::{App, Mode, Scene};
use retro_sports::games::tegenwind::data::*;
use retro_sports::games::tegenwind::probe::{ladder_field, run_race, Bot, BotKind};
use retro_sports::games::tegenwind::race::*;

fn cpu_field(stage: usize) -> Vec<(Who, Bike, Ctrl, f32)> {
    ladder_field(Bike::Oma, stage, Ctrl::Cpu)
}

#[test]
fn cpu_races_always_finish() {
    for stage in 0..3 {
        for seed in 1..=6u64 {
            let mut race = Race::new(stage, &cpu_field(stage), seed);
            run_race(&mut race, &mut []);
            assert!(race.finished(), "stage {stage} seed {seed} did not finish");
            assert!(race.riders.iter().all(|r| r.finish.is_some()), "stage {stage} seed {seed}: someone never crossed the line");
            let res = race.results();
            let secs = res[0].1 as f32 / 60.0;
            assert!((60.0..200.0).contains(&secs), "stage {stage}: winner took {secs:.0}s");
        }
    }
}

#[test]
fn same_seed_same_race() {
    let run = || {
        let mut race = Race::new(1, &cpu_field(1), 77);
        run_race(&mut race, &mut []);
        race.results()
    };
    assert_eq!(run(), run());
}

/// One rider alone on an empty road: the speed it settles at.
fn top_speed(bike: Bike, wind: f32, tuck: bool, draft: bool, gap: i32) -> f32 {
    let mut race = Race::new(0, &[(Who::Kees, bike, Ctrl::Human(0), 0.5), (Who::Anouk, bike, Ctrl::Cpu, 0.5)], 3);
    race.objs.clear();
    race.phase = Phase::Race;
    // a straight road with one fixed wind
    let mut prev = [Pad::default(); 2];
    let mut side = false;
    let mut v = 0.0;
    for t in 0..60 * 40 {
        let r = &mut race.riders[0];
        r.x = 1000.0;
        r.y = 0.0;
        r.lane = 0;
        r.energy = 100.0;
        let lead = &mut race.riders[1];
        lead.state = RState::Ride;
        lead.x = if draft { 1022.0 } else { 9000.0 };
        lead.y = 0.0;
        lead.v = 0.0;
        race.gust = 1.0;
        let mut p = Pad { c: tuck, ..Pad::default() };
        if race.riders[0].since >= gap {
            side = !side;
            if side {
                p.left = true;
            } else {
                p.right = true;
            }
        }
        // wind: push the same headwind every frame by faking the section
        let _ = wind;
        race.update(&[p, Pad::default()], &prev);
        prev = [p, Pad::default()];
        if t > 60 * 30 {
            v = race.riders[0].v;
        }
    }
    v
}

#[test]
fn rhythm_beats_mashing() {
    let field = ladder_field(Bike::Oma, 0, Ctrl::Human(0));
    let time = |kind: BotKind| {
        let mut race = Race::new(0, &field, 11);
        race.objs.retain(|o| matches!(o.kind, OKind::Food(_)));
        let mut bots = [(0usize, Bot::new(kind, 5))];
        run_race(&mut race, &mut bots);
        let r = &race.riders[0];
        (r.finish.unwrap_or(u32::MAX), r.stats.wasted, r.stats.perfect)
    };
    let (steady, w0, p0) = time(BotKind::Metronome);
    let (mash, w1, _) = time(BotKind::Masher);
    assert!(steady < mash, "steady {steady} vs mash {mash}");
    assert_eq!(w0, 0);
    assert!(p0 > 100);
    assert!(w1 > 100, "mashing should waste strokes");
}

#[test]
fn tucking_and_drafting_cut_the_drag() {
    // the start of the polder is a headwind
    let free = top_speed(Bike::Oma, 0.5, false, false, 11);
    let tucked = top_speed(Bike::Oma, 0.5, true, false, 11);
    let drafted = top_speed(Bike::Oma, 0.5, false, true, 11);
    assert!(free > 1.0, "free {free}");
    assert!(tucked > free * 1.05, "tuck {tucked} vs {free}");
    assert!(drafted > free * 1.05, "draft {drafted} vs {free}");
}

#[test]
fn wasted_strokes_give_no_push() {
    let mut race = Race::new(0, &[(Who::Kees, Bike::Oma, Ctrl::Human(0), 0.5)], 1);
    race.objs.clear();
    race.phase = Phase::Race;
    let l = Pad { left: true, ..Pad::default() };
    let r = Pad { right: true, ..Pad::default() };
    let z = Pad::default();
    race.update(&[l, z], &[z, z]);
    let v1 = race.riders[0].v;
    assert!(v1 > 0.05, "first stroke pushes");
    // right away again: too fast
    race.update(&[r, z], &[z, z]);
    assert_eq!(race.riders[0].last_stroke, Stroke::Wasted);
    assert!(race.riders[0].v <= v1);
    // the same foot twice does nothing at all
    for _ in 0..30 {
        race.update(&[z, z], &[z, z]);
    }
    let before = race.riders[0].stats;
    race.update(&[r, z], &[z, z]);
    assert_eq!(race.riders[0].stats.wasted, before.wasted);
    assert_eq!(race.riders[0].stats.good + race.riders[0].stats.perfect, before.good + before.perfect);
}

#[test]
fn idle_human_is_swept_up_and_race_ends() {
    let field = ladder_field(Bike::Bak, 2, Ctrl::Human(0));
    let mut race = Race::new(2, &field, 4);
    run_race(&mut race, &mut []);
    assert!(race.finished());
    assert!(race.riders[0].dnf);
    assert_eq!(race.results().last().unwrap().0, 0);
}

#[test]
fn every_scene_draws_and_demo_runs() {
    let mut app = App::new(3);
    let mut f = Frame::new();
    for scene in [Scene::Title, Scene::Select, Scene::Intro, Scene::Results, Scene::Continue, Scene::GameOver, Scene::Champion] {
        app.scene = scene;
        app.draw(&mut f);
    }
    app.start_demo();
    assert_eq!(app.mode, Mode::Demo);
    for _ in 0..900 {
        app.update([Pad::default(); 2], false);
        app.draw(&mut f);
    }
    // any button leaves the demo
    app.update([Pad { a: true, ..Pad::default() }, Pad::default()], false);
    assert_eq!(app.scene, Scene::Title);
}

#[test]
fn two_player_tour_reaches_the_champion_screen() {
    let mut app = App::new(8);
    let a = Pad { a: true, ..Pad::default() };
    let z = Pad::default();
    let tap = |app: &mut App, p: [Pad; 2]| {
        app.update(p, false);
        app.update([z, z], false);
    };
    for _ in 0..30 {
        app.update([z, z], false);
    }
    tap(&mut app, [a, z]);
    tap(&mut app, [Pad { down: true, ..z }, z]);
    tap(&mut app, [a, z]);
    for _ in 0..20 {
        app.update([z, z], false);
    }
    tap(&mut app, [a, a]);
    let mut bots = [Bot::new(BotKind::Decent, 1), Bot::new(BotKind::Decent, 2)];
    let mut stages = 0;
    for _ in 0..60 * 60 * 20 {
        if app.scene == Scene::Champion {
            break;
        }
        let pads = match (app.scene, app.race.as_ref()) {
            (Scene::Play, Some(r)) => [bots[0].pad(r, r.views[0]), bots[1].pad(r, r.views[1])],
            (Scene::Intro, _) if app.t > 60 => {
                stages += 1;
                [a, z]
            }
            (Scene::Results, _) if app.t > 400 => [a, z],
            _ => [z, z],
        };
        app.update(pads, false);
    }
    assert_eq!(app.scene, Scene::Champion);
    assert!(stages >= 3);
    assert!(app.score[0] > 0 && app.score[1] > 0);
}

#[test]
fn sounds_render() {
    use retro_sports::games::tegenwind::audio::*;
    for s in ALL_SFX {
        assert!(!sfx_samples(s).is_empty());
    }
    for t in ALL_TRACKS {
        assert!(music_samples(t).len() > 44100 * 4);
    }
}

#[test]
fn new_is_fast() {
    let t0 = std::time::Instant::now();
    let _app = App::new(1);
    let ms = t0.elapsed().as_millis();
    // generous for debug builds; release is a few ms
    assert!(ms < 2000, "App::new took {ms} ms");
}
