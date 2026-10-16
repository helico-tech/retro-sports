//! TIENEN! rules and flow.

use retro_sports::engine::gfx::Frame;
use retro_sports::engine::input::Pad;
use retro_sports::games::tienen::app::{App, Mode, Scene};
use retro_sports::games::tienen::data::*;
use retro_sports::games::tienen::probe;
use retro_sports::games::tienen::world::*;

const IDLE: [Pad; 2] = [Pad { up: false, down: false, left: false, right: false, a: false, b: false, c: false, start: false }; 2];

fn roster(venue: usize) -> [(usize, Ctrl, f32); 4] {
    let v = &VENUES[venue];
    [(0, Ctrl::Cpu, 0.7), (v.kids[0], Ctrl::Cpu, v.skill), (v.kids[1], Ctrl::Cpu, v.skill), (v.kids[2], Ctrl::Cpu, v.skill)]
}

/// A world in open play, the keeper holding nothing, everybody out of the way.
fn open_play(seed: u64) -> World {
    let mut w = World::new(0, roster(0), seed);
    for _ in 0..60 * 120 {
        if w.phase == Phase::Play {
            break;
        }
        w.update(&IDLE);
    }
    assert_eq!(w.phase, Phase::Play, "the latjetrap must end in play");
    // no cars, no poodle
    w.next_car = u32::MAX;
    w.next_dog = u32::MAX;
    w.ball.held = false;
    w.ball.owner = None;
    let kp = w.keeper;
    for (k, kid) in w.kids.iter_mut().enumerate() {
        kid.act = Act::Idle;
        kid.z = 0.0;
        kid.vz = 0.0;
        kid.cool = 0;
        if k != kp {
            kid.x = 230.0;
            kid.y = 10.0 + k as f32 * 15.0;
        }
    }
    w
}

fn shooter(w: &World) -> usize {
    (0..4).find(|&k| k != w.keeper).unwrap()
}

/// A shot from just in front of the goal, keeper far away in the corner.
fn shot_on_goal(w: &mut World, kind: Kind, bounced: bool) -> usize {
    let s = shooter(w);
    let kp = w.keeper;
    w.kids[kp].x = 140.0;
    w.kids[kp].y = 70.0;
    let b = &mut w.ball;
    b.x = GOAL_X + 6.0;
    b.y = goal_mid();
    b.z = 10.0;
    b.vx = -3.0;
    b.vy = 0.0;
    b.vz = 0.0;
    b.air = !bounced;
    b.from = Some(s);
    b.last_field = Some(s);
    b.last_touch = Some(s);
    b.shot = Some(Shot { kind, by: s, valid: true, bounced });
    s
}

fn run_until(w: &mut World, n: u32, f: impl Fn(&World) -> bool) -> bool {
    for _ in 0..n {
        w.update(&IDLE);
        if f(w) {
            return true;
        }
    }
    false
}

#[test]
fn the_first_keeper_starts_with_eleven() {
    for seed in 1..6 {
        let w = open_play(seed);
        assert_eq!(w.kids[w.keeper].pts, 11);
        assert_eq!(w.kids.iter().filter(|k| k.pts == 10).count(), 3);
    }
}

#[test]
fn a_goal_takes_its_value_off_the_keeper() {
    for (kind, v) in [(Kind::Volley, 1), (Kind::Header, 2), (Kind::Heel, 2), (Kind::Bicycle, 5)] {
        let mut w = open_play(3);
        let kp = w.keeper;
        w.kids[kp].pts = 9;
        shot_on_goal(&mut w, kind, false);
        assert!(run_until(&mut w, 10, |w| w.phase == Phase::Goal), "{:?} must score", kind);
        assert_eq!(w.kids[kp].pts, 9 - v);
        assert_eq!(w.keeper, kp, "the keeper stays in goal after a goal");
    }
}

#[test]
fn exactly_zero_loses() {
    let mut w = open_play(4);
    let kp = w.keeper;
    w.kids[kp].pts = 2;
    shot_on_goal(&mut w, Kind::Header, false);
    assert!(run_until(&mut w, 10, |w| w.phase == Phase::Over));
    assert_eq!(w.loser, Some(kp));
    assert_eq!(w.kids[kp].pts, 0);
}

#[test]
fn overshooting_goes_back_to_ten_minus_the_rest() {
    let mut w = open_play(5);
    let kp = w.keeper;
    w.kids[kp].pts = 2;
    shot_on_goal(&mut w, Kind::Bicycle, false);
    assert!(run_until(&mut w, 10, |w| w.phase == Phase::Goal));
    // 2 - 5 = -3: back to 7
    assert_eq!(w.kids[kp].pts, 7);
    assert_eq!(w.loser, None);
}

#[test]
fn a_bounce_into_the_goal_puts_the_shooter_in_goal() {
    let mut w = open_play(6);
    let kp = w.keeper;
    let pts = w.kids[kp].pts;
    let s = shot_on_goal(&mut w, Kind::Volley, true);
    assert!(run_until(&mut w, 10, |w| w.phase == Phase::Wissel));
    assert_eq!(w.keeper, s);
    assert_eq!(w.why, Some(Why::Bounced));
    assert_eq!(w.kids[kp].pts, pts, "no points off for a bounced goal");
}

#[test]
fn a_shot_without_a_pass_through_the_air_does_not_count() {
    let mut w = open_play(7);
    let kp = w.keeper;
    let pts = w.kids[kp].pts;
    let s = shot_on_goal(&mut w, Kind::Volley, false);
    w.ball.shot = Some(Shot { kind: Kind::Volley, by: s, valid: false, bounced: false });
    run_until(&mut w, 10, |w| w.phase != Phase::Play);
    assert_eq!(w.kids[kp].pts, pts);
    assert_eq!(w.keeper, kp);
}

#[test]
fn a_ball_landing_outside_puts_the_last_field_player_in_goal() {
    let mut w = open_play(8);
    let s = shooter(&w);
    let b = &mut w.ball;
    b.x = FIELD_X1 + 10.0;
    b.y = 30.0;
    b.z = 20.0;
    b.vx = 0.0;
    b.vy = 0.0;
    b.vz = 0.0;
    b.last_field = Some(s);
    b.air = true;
    // in the air outside the lines: still in play
    w.update(&IDLE);
    assert_eq!(w.phase, Phase::Play);
    assert!(run_until(&mut w, 120, |w| w.phase == Phase::Wissel));
    assert_eq!(w.keeper, s);
    assert_eq!(w.why, Some(Why::Out));
}

#[test]
fn a_catch_in_the_area_puts_the_last_field_player_in_goal() {
    let mut w = open_play(9);
    let s = shooter(&w);
    let kp = w.keeper;
    w.kids[kp].x = GOAL_X + 8.0;
    w.kids[kp].y = goal_mid();
    w.kids[kp].act = Act::Catch;
    w.kids[kp].at = 0;
    let b = &mut w.ball;
    b.x = GOAL_X + 12.0;
    b.y = goal_mid();
    b.z = 0.0;
    b.vx = -0.5;
    b.vy = 0.0;
    b.vz = 0.0;
    b.last_field = Some(s);
    assert!(run_until(&mut w, 10, |w| w.phase == Phase::Wissel));
    assert_eq!(w.keeper, s);
    assert_eq!(w.why, Some(Why::Caught));
}

#[test]
fn only_a_keeper_on_one_point_catches_outside_the_area() {
    for (pts, caught) in [(3, false), (1, true)] {
        let mut w = open_play(10);
        let s = shooter(&w);
        let kp = w.keeper;
        w.kids[kp].pts = pts;
        w.kids[kp].x = AREA_X + 30.0;
        w.kids[kp].y = goal_mid();
        w.kids[kp].act = Act::Catch;
        w.kids[kp].at = 0;
        let b = &mut w.ball;
        b.x = AREA_X + 34.0;
        b.y = goal_mid();
        b.z = 0.0;
        b.vx = -0.5;
        b.vy = 0.0;
        b.vz = 0.0;
        b.last_field = Some(s);
        let got = run_until(&mut w, 10, |w| w.phase == Phase::Wissel);
        assert_eq!(got, caught, "keeper on {} points", pts);
    }
}

#[test]
fn shot_kinds_follow_the_height_the_facing_and_the_jump() {
    // on the ground: foot or chest
    assert_eq!(shot_kind(-1, false, 5.0, false), Some(Kind::Volley));
    assert_eq!(shot_kind(1, false, 5.0, false), Some(Kind::Heel));
    assert_eq!(shot_kind(-1, false, 16.0, false), Some(Kind::Chest));
    assert_eq!(shot_kind(-1, false, 26.0, false), None, "a header needs a jump");
    // in a jump: the head, or a bicycle kick with your back to the goal
    assert_eq!(shot_kind(-1, true, 16.0, false), Some(Kind::Header));
    assert_eq!(shot_kind(-1, true, 26.0, false), Some(Kind::Header));
    assert_eq!(shot_kind(1, true, 20.0, false), Some(Kind::Bicycle));
    assert_eq!(shot_kind(1, true, 6.0, true), Some(Kind::Bicycle), "Lieke reaches lower");
    assert_eq!(shot_kind(1, true, 6.0, false), Some(Kind::Heel));
    assert_eq!(shot_kind(-1, true, 40.0, false), None);
}

#[test]
fn field_kids_stay_out_of_the_keepers_area() {
    let mut w = open_play(12);
    let s = shooter(&w);
    w.kids[s].x = AREA_X + 10.0;
    w.kids[s].ctrl = Ctrl::Human(0);
    let left = Pad { left: true, ..Pad::default() };
    for _ in 0..60 {
        w.update(&[left, Pad::default()]);
    }
    assert!(w.kids[s].x > AREA_X, "a field kid walked into the area");
}

#[test]
fn a_keeper_cannot_mash_his_hands_out() {
    let mut w = open_play(13);
    let kp = w.keeper;
    w.kids[kp].ctrl = Ctrl::Human(0);
    w.ball.x = 200.0;
    w.ball.y = 36.0;
    w.ball.z = 0.0;
    let a = Pad { a: true, ..Pad::default() };
    // press, release, press again while the first catch still recovers
    w.update(&[a, Pad::default()]);
    assert_eq!(w.kids[kp].act, Act::Catch);
    for _ in 0..KEEPER_WINDOW + 2 {
        w.update(&IDLE);
    }
    w.update(&[a, Pad::default()]);
    assert_eq!(w.kids[kp].act, Act::Catch);
    assert!(w.kids[kp].at > KEEPER_WINDOW, "the second press must not restart the catch");
}

#[test]
fn a_jumping_keeper_does_not_catch() {
    let mut w = open_play(14);
    let kp = w.keeper;
    w.kids[kp].z = 10.0;
    assert!(!w.may_catch(kp));
    w.kids[kp].z = 0.0;
    w.kids[kp].x = GOAL_X + 8.0;
    w.kids[kp].y = goal_mid();
    assert!(w.may_catch(kp));
}

#[test]
fn nobody_can_keep_the_ball_forever() {
    let mut w = open_play(15);
    let s = shooter(&w);
    w.kids[s].ctrl = Ctrl::Human(0);
    w.kids[s].x = 150.0;
    w.kids[s].y = 36.0;
    w.ball.x = 145.0;
    w.ball.y = 36.0;
    w.ball.z = 0.0;
    w.ball.vx = 0.0;
    w.ball.vy = 0.0;
    w.ball.owner = Some(s);
    // a human who just stands on the ball
    assert!(run_until(&mut w, 60 * 6, |w| w.ball.owner != Some(s)), "the ball never left his feet");
}

#[test]
fn a_dinner_tie_goes_against_the_keeper() {
    let mut w = open_play(16);
    let kp = w.keeper;
    for k in w.kids.iter_mut() {
        k.pts = 4;
    }
    w.play_t = MAX_GAME - 1;
    w.update(&IDLE);
    assert_eq!(w.loser, Some(kp));
}

#[test]
fn the_mums_end_a_game_that_takes_too_long() {
    let mut w = open_play(11);
    w.kids[2].pts = 3;
    let lowest = (0..4).min_by_key(|&k| w.kids[k].pts).unwrap();
    w.play_t = MAX_GAME - 1;
    w.update(&IDLE);
    assert_eq!(w.phase, Phase::Over);
    assert!(w.dinner);
    assert_eq!(w.loser, Some(lowest));
}

#[test]
fn cpu_games_end_and_are_deterministic() {
    for venue in 0..3 {
        for seed in 1..4u64 {
            let a = probe::play(venue, roster(venue), seed);
            let b = probe::play(venue, roster(venue), seed);
            assert!(a.loser < 4, "venue {} seed {}: no loser", venue, seed);
            assert_eq!((a.loser, a.secs as i32, a.goals), (b.loser, b.secs as i32, b.goals));
        }
    }
}

#[test]
fn the_billenbrand_ends_the_game() {
    let mut w = World::new(1, roster(1), 21);
    assert!(run_until(&mut w, 60 * 60 * 9, |w| w.finished()));
    let lo = w.loser.unwrap();
    assert_eq!(w.brand.shooters.len(), 3);
    assert!(!w.brand.shooters.contains(&lo));
}

#[test]
fn an_idle_human_still_gets_to_the_end() {
    // 1P with nobody touching the pad: the game, the billenbrand and the
    // results all go by themselves
    let mut app = App::new(3);
    app.mode = Mode::One;
    app.cursor = [1, 0];
    app.stage = 0;
    app.start_play();
    let mut f = Frame::new();
    let mut n = 0;
    while app.scene == Scene::Play {
        app.update(IDLE, false);
        n += 1;
        if n % 301 == 0 {
            app.draw(&mut f);
        }
        assert!(n < 60 * 60 * 9, "the game never ended");
    }
    assert_eq!(app.scene, Scene::Results);
    let mut m = 0;
    while app.scene == Scene::Results {
        app.update(IDLE, false);
        m += 1;
        assert!(m < 60 * 30, "the results never ended");
    }
    assert!(matches!(app.scene, Scene::Continue | Scene::Intro));
}

#[test]
fn every_scene_draws() {
    let mut app = App::new(9);
    let mut f = Frame::new();
    for s in [Scene::Title, Scene::Select, Scene::Intro, Scene::Results, Scene::Continue, Scene::GameOver, Scene::Champion] {
        app.scene = s;
        app.draw(&mut f);
    }
    for mode in [Mode::One, Mode::Two] {
        app.mode = mode;
        app.start_play();
        for _ in 0..60 * 30 {
            app.update(IDLE, false);
        }
        app.draw(&mut f);
    }
}

#[test]
#[ignore]
fn balance_report() {
    probe::sim(40);
    probe::probe(40);
}

#[test]
#[ignore]
fn seat_report() {
    use retro_sports::games::tienen::bot::BotKind;
    for (venue, v) in VENUES.iter().enumerate() {
        let mut agg = [[0i64; 6]; 4];
        for seed in 0..40u64 {
            let roster = [(PICKS[seed as usize % 4], Ctrl::Human(0), 0.75), (v.kids[0], Ctrl::Cpu, v.skill), (v.kids[1], Ctrl::Cpu, v.skill), (v.kids[2], Ctrl::Cpu, v.skill)];
            let mut w = World::new(venue, roster, 900 + seed);
            w.auto = [Some(BotKind::Decent), None];
            let mut kt = [0i64; 4];
            for _ in 0..60 * 60 * 6 {
                w.update(&IDLE);
                if w.phase == Phase::Play {
                    kt[w.keeper] += 1;
                }
                if matches!(w.phase, Phase::Over | Phase::Brand | Phase::Done) {
                    break;
                }
            }
            for k in 0..4 {
                let s = &w.kids[k].stats;
                agg[k][0] += s.wissels as i64;
                agg[k][1] += s.conceded as i64;
                agg[k][2] += s.scored as i64;
                agg[k][3] += s.saves as i64;
                agg[k][4] += kt[k] / 60;
                agg[k][5] += s.shots as i64;
            }
        }
        println!("{}: per seat (wissels, conceded, scored, saves, secs keeping, shots) / 40 games", v.name);
        for (k, a) in agg.iter().enumerate() {
            println!("  seat {}: {:?}", k, a.map(|x| x as f32 / 40.0));
        }
    }
}
