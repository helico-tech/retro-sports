use retro_sports::engine::gfx::{glyph, Frame};
use retro_sports::engine::input::Pad;
use retro_sports::games::discdogs::app::{App, Mode, Scene};
use retro_sports::games::discdogs::audio;
use retro_sports::games::discdogs::data::*;
use retro_sports::games::discdogs::sim::*;

fn play(m: &mut Match) -> u32 {
    let mut frames = 0;
    while !m.finished() {
        m.update(&[Pad::default(); 2]);
        m.sfx.clear();
        frames += 1;
        assert!(frames < 60 * 60 * 12, "match did not finish");
    }
    frames
}

#[test]
fn cpu_matches_always_finish() {
    for seed in 1..=24u64 {
        let a = (seed % 4) as usize;
        let b = (a + 1 + (seed % 3) as usize) % 4;
        let stage = STAGES[(seed % 3) as usize];
        let skill = [0.3, 0.6, 0.9][(seed % 3) as usize];
        let mut m = Match::new([a, b], [Ctrl::Cpu, Ctrl::Cpu], [skill, skill], stage, seed);
        play(&mut m);
        let w = m.winner.expect("a winner");
        assert!(m.sides[0].results.len() >= TOSSES && m.sides[0].results.len() == m.sides[1].results.len());
        assert!(m.sides[w].total() >= m.sides[1 - w].total());
        assert!(m.stats.tosses as usize == m.sides[0].results.len() * 2);
    }
}

#[test]
fn cpu_catches_most_and_throws_far() {
    let (mut tosses, mut catches, mut yards) = (0, 0, 0);
    for seed in 1..=12u64 {
        let mut m = Match::new([(seed % 4) as usize, ((seed + 1) % 4) as usize], [Ctrl::Cpu, Ctrl::Cpu], [0.7, 0.7], STAGES[(seed % 3) as usize], seed);
        play(&mut m);
        tosses += m.stats.tosses;
        catches += m.stats.catches;
        yards += m.stats.throw_yards;
    }
    let rate = catches as f32 / tosses as f32;
    let avg = yards as f32 / tosses as f32;
    assert!((0.5..0.95).contains(&rate), "catch rate {rate}");
    assert!((20.0..70.0).contains(&avg), "average toss {avg} yards");
}

#[test]
fn idle_human_loses_but_match_finishes() {
    let mut m = Match::new([0, 1], [Ctrl::Human(0), Ctrl::Cpu], [0.5, 0.6], Stage::Park, 9);
    play(&mut m);
    // nobody throws for the idle human, so the disc goes out weakly and the dog never runs
    assert_eq!(m.winner, Some(1));
    assert_eq!(m.sides[0].total(), 0);
}

#[test]
fn scoring_sanity() {
    let r = |o, yards, perfect, shoulder, sausages| TossResult { outcome: o, yards, perfect, shoulder, sausages };
    assert_eq!(r(Outcome::Catch(CatchKind::Ground), 30, false, false, 0).points(), 30);
    assert_eq!(r(Outcome::Catch(CatchKind::Jump), 40, false, false, 0).points(), 80);
    assert_eq!(r(Outcome::Catch(CatchKind::Flip), 30, true, true, 2).points(), 90 + PERFECT_BONUS + SHOULDER_BONUS + 2 * SAUSAGE_BONUS);
    assert_eq!(r(Outcome::Catch(CatchKind::Soggy), 25, false, false, 0).points(), 25);
    assert_eq!(r(Outcome::Miss(MissKind::Seagull), 0, true, false, 3).points(), 0);
    assert_eq!(r(Outcome::Miss(MissKind::Oof), 50, true, true, 0).points(), 0);
}

fn land(angle: f32, power: f32, floater: bool, wind: f32) -> (f32, usize) {
    let mut d = Disc::new();
    d.launch(angle, power, false, floater);
    let mut pts = Vec::new();
    predict(&d, wind, &mut pts);
    (pts.last().unwrap().0, pts.len())
}

#[test]
fn disc_flies_like_a_disc() {
    // more power, farther
    assert!(land(25.0, 1.0, false, 0.0).0 > land(25.0, 0.5, false, 0.0).0 + 40.0);
    // a floater hangs longer
    assert!(land(25.0, 0.6, true, 0.0).1 > land(25.0, 0.6, false, 0.0).1 + 10);
    // the wind matters, and more so for a floater
    let tail = land(25.0, 0.7, false, 0.6).0 - land(25.0, 0.7, false, -0.6).0;
    let tail_f = land(25.0, 0.7, true, 0.6).0 - land(25.0, 0.7, true, -0.6).0;
    assert!(tail > 5.0 && tail_f > tail * 2.0, "wind {tail} floater {tail_f}");
    // a full throw stays inside the field without wind
    assert!(land(30.0, 1.0, false, 0.0).0 < FIELD_END);
}

#[test]
fn same_seed_same_match() {
    let run = || {
        let mut m = Match::new([2, 3], [Ctrl::Cpu, Ctrl::Cpu], [0.6, 0.6], Stage::Pier, 77);
        play(&mut m);
        (m.sides[0].total(), m.sides[1].total(), m.stats.frames)
    };
    assert_eq!(run(), run());
}

#[test]
fn every_scene_draws() {
    let mut app = App::new(3);
    let mut f = Frame::new();
    app.ladder = vec![1, 2, 3];
    for scene in [Scene::Title, Scene::Select, Scene::Intro, Scene::Continue, Scene::GameOver, Scene::Champion, Scene::Results] {
        app.scene = scene;
        app.draw(&mut f);
    }
    app.mode = Mode::Two;
    app.scene = Scene::Intro;
    app.draw(&mut f);
    app.start_demo();
    for _ in 0..1200 {
        app.update([Pad::default(); 2], false);
        app.draw(&mut f);
    }
}

#[test]
fn new_is_fast() {
    let t0 = std::time::Instant::now();
    let _app = App::new(1);
    let ms = t0.elapsed().as_millis();
    // well under 100 ms in release; debug builds get some slack
    let limit = if cfg!(debug_assertions) { 400 } else { 60 };
    assert!(ms < limit, "App::new took {ms} ms");
}

#[test]
fn sounds_render() {
    for s in audio::ALL_SFX {
        assert!(!audio::sfx_samples(s).is_empty());
    }
    for t in audio::ALL_TRACKS {
        let song = audio::song(t);
        for bar in song.melody {
            assert_eq!(bar.split_whitespace().count(), 16, "{t:?}: bar '{bar}' is not 16 steps");
        }
        assert_eq!(song.melody.len(), song.chords.len());
        assert!(audio::music_samples(t).len() > 44100);
    }
}

#[test]
fn all_text_fits_the_font() {
    let mut texts: Vec<&str> = vec![TICKER];
    texts.extend(DEMO_TIPS);
    texts.extend(MISS_QUIPS);
    for p in &PAIRS {
        texts.extend([p.owner.name, p.owner.bio, p.owner.heckle[0], p.owner.heckle[1]]);
        texts.extend([p.dog.name, p.dog.breed_name, p.dog.bio, p.dog.good]);
    }
    for s in STAGES {
        texts.extend([s.name(), s.tagline()]);
    }
    for k in [CatchKind::Ground, CatchKind::Rolling, CatchKind::Soggy, CatchKind::Jump, CatchKind::Flip] {
        texts.extend([k.label(), k.short()]);
    }
    for k in [MissKind::Dropped, MissKind::Oof, MissKind::Seagull, MissKind::Sea, MissKind::Hedge, MissKind::Crowd, MissKind::Sunk] {
        texts.extend([k.label(), k.short()]);
    }
    for s in texts {
        for c in s.chars() {
            assert!(glyph(c).is_some(), "'{c}' in '{s}' is not in the font");
        }
        assert!(!s.chars().any(|c| c.is_ascii_lowercase()), "lower case in '{s}'");
    }
}

#[test]
#[ignore]
fn debug_park_misses() {
    use retro_sports::games::discdogs::ai::Brain;
    let mut causes = std::collections::HashMap::new();
    for seed in 0..80u64 {
        let my = (seed % 4) as usize;
        let mut m = Match::new([my, (my + 1) % 4], [Ctrl::Human(0), Ctrl::Cpu], [0.5, 0.4], Stage::Park, 1000 + seed);
        let mut bot = Brain::human("decent", seed ^ 0xBEEF);
        let mut swam = false;
        let mut stared = false;
        let mut blanket = false;
        let mut leapt = false;
        while !m.finished() {
            let p = if m.thrower() == 0 { bot.pad(&m, 0) } else { Pad::default() };
            let before = m.sides[0].results.len();
            m.update(&[p, Pad::default()]);
            m.sfx.clear();
            if m.thrower() == 0 && m.phase == Phase::Flight {
                swam |= m.dog.state == DogState::Swim;
                stared |= m.dog.state == DogState::Stare;
                blanket |= m.dog.on_blanket.is_some();
                leapt |= m.dog.state == DogState::Air;
            }
            if m.sides[0].results.len() > before {
                let r = m.sides[0].results[before];
                if let Outcome::Miss(k) = r.outcome {
                    let key = format!("{:?} swam={} stared={} blanket={} dog={} land={:.0} dogx={:.0} air={}", k, swam, stared, blanket, my, m.disc.x, m.dog.x, m.dog.caught_air);
                    let short = format!("{:?} swam={} stared={} leapt={} over={}", k, swam, stared, leapt, m.dog.x > m.disc.x + 10.0);
                    *causes.entry(short).or_insert(0) += 1;
                    if seed < 6 { println!("{}", key); }
                }
                swam = false;
                stared = false;
                blanket = false;
                leapt = false;
            }
        }
    }
    let mut v: Vec<_> = causes.into_iter().collect();
    v.sort();
    for (k, n) in v { println!("{n:>4} {k}"); }
}

#[test]
#[ignore]
fn debug_plan() {
    for pair in 0..4 {
        let mut m = Match::new([pair, (pair + 1) % 4], [Ctrl::Cpu, Ctrl::Cpu], [0.9, 0.9], Stage::Bowl, 5);
        for _ in 0..3 {
            m.update(&[Pad::default(); 2]);
        }
    }
}

#[test]
#[ignore]
fn debug_sirw() {
    let mut m = Match::new([2, 0], [Ctrl::Cpu, Ctrl::Cpu], [0.9, 0.9], Stage::Bowl, 5);
    let mut last = Phase::Ready;
    while !m.finished() {
        m.update(&[Pad::default(); 2]);
        m.sfx.clear();
        if m.thrower() == 0 && m.phase != last {
            println!("{:?} t{} angle {} power {:.2} floater {} disc ({:.0},{:.0}) v({:.2},{:.2}) dog ({:.0},{:.0}) {:?} res {:?}", m.phase, m.phase_t, m.angle, m.power, m.floater, m.disc.x, m.disc.z, m.disc.vx, m.disc.vz, m.dog.x, m.dog.z, m.dog.state, m.result.map(|r| (r.outcome, r.yards)));
        }
        last = m.phase;
    }
}

#[test]
#[ignore]
fn debug_stare_overshoot() {
    use retro_sports::games::discdogs::ai::Brain;
    for seed in 0..80u64 {
        let my = (seed % 4) as usize;
        let mut m = Match::new([my, (my + 1) % 4], [Ctrl::Human(0), Ctrl::Cpu], [0.5, 0.4], Stage::Park, 1000 + seed);
        let mut bot = Brain::human("decent", seed ^ 0xBEEF);
        let mut log: Vec<String> = Vec::new();
        let mut stared = false;
        while !m.finished() {
            let p = if m.thrower() == 0 { bot.pad(&m, 0) } else { Pad::default() };
            let before = m.sides[0].results.len();
            m.update(&[p, Pad::default()]);
            m.sfx.clear();
            if m.thrower() == 0 && m.phase == Phase::Flight {
                stared |= m.dog.state == DogState::Stare;
                log.push(format!("t{} pad{}{}{}{} dog {:?} x{:.0} z{:.0} vx{:.2} face{} | disc x{:.0} z{:.0} | {:?} sq {:?}", m.phase_t, if p.left {"L"} else {"."}, if p.right {"R"} else {"."}, if p.b {"B"} else {"."}, if p.c {"C"} else {"."}, m.dog.state, m.dog.x, m.dog.z, m.dog.vx, m.dog.face, m.disc.x, m.disc.z, bot.debug(), m.squirrel.map(|s| (s.x as i32, s.d as i32))));
            }
            if m.sides[0].results.len() > before {
                let r = m.sides[0].results[before];
                if matches!(r.outcome, Outcome::Miss(_)) && stared && m.dog.x > m.disc.x + 10.0 {
                    for l in &log { println!("{l}"); }
                    return;
                }
                log.clear();
                stared = false;
            }
        }
    }
}

#[test]
#[ignore]
fn debug_sheet() {
    let dir = std::env::var("DD_OUT").unwrap_or("target".into());
    retro_sports::games::discdogs::headless::sheet_png(std::path::Path::new(&format!("{dir}/dogs.png")), std::path::Path::new(&format!("{dir}/props.png")));
}
