//! Contract tests every game in the collection must pass.

use retro_sports::cart::Game;
use retro_sports::console::{bank_sounds, Console};
use retro_sports::engine::gfx::Frame;
use retro_sports::engine::input::Pad;
use retro_sports::engine::rng::Rng;
use retro_sports::games::CARTS;

fn mash(rng: &mut Rng) -> Pad {
    let mut b = || rng.int(3) == 0;
    Pad { up: b(), down: b(), left: b(), right: b(), a: b(), b: b(), c: b(), start: b() }
}

fn check_cues(g: &mut dyn Game, sfx: usize, music: usize, id: &str) {
    for s in g.take_sfx() {
        assert!(s < sfx, "{id}: sfx {s} not in bank");
    }
    if let Some(m) = g.music() {
        assert!(m < music, "{id}: music {m} not in bank");
    }
}

#[test]
fn every_game_survives_idle_and_mashing() {
    for c in CARTS {
        let bank = (c.sounds)();
        assert!(!bank.sfx.is_empty() && !bank.music.is_empty(), "{}: needs sounds and music", c.id);
        for s in bank.sfx.iter().chain(bank.music.iter()) {
            assert!(!s.samples.is_empty(), "{}: empty sound", c.id);
        }
        for seed in 1..=3u64 {
            let mut g = (c.new)(seed);
            let mut rng = Rng(seed * 7919);
            let mut f = Frame::new();
            // 5 minutes idle (attract mode / demo), then 10 minutes of button mashing
            for t in 0..60 * 60 * 15 {
                let pads = if t < 60 * 60 * 5 { [Pad::default(); 2] } else { [mash(&mut rng), mash(&mut rng)] };
                g.update(pads, false);
                check_cues(g.as_mut(), bank.sfx.len(), bank.music.len(), c.id);
                if t % 97 == 0 {
                    g.draw(&mut f);
                }
            }
        }
    }
}

#[test]
fn esc_always_leads_back_to_the_selector() {
    for c in CARTS {
        let mut g = (c.new)(5);
        let mut rng = Rng(11);
        // get somewhere into the game first
        for _ in 0..60 * 40 {
            g.update([mash(&mut rng), Pad::default()], false);
        }
        let mut n = 0;
        while !g.wants_exit() {
            g.update([Pad::default(); 2], n % 30 == 0);
            n += 1;
            assert!(n < 60 * 10, "{}: Esc does not lead out of the game", c.id);
        }
    }
}

#[test]
fn console_enters_and_leaves_every_game() {
    let mut con = Console::new(3);
    let press = |con: &mut Console, p: Pad| {
        con.update([p, Pad::default()], false);
        con.update([Pad::default(); 2], false);
    };
    for i in 0..CARTS.len() {
        let settle = |con: &mut Console| {
            let mut n = 0;
            while !con.hub.ready() {
                con.update([Pad::default(); 2], false);
                n += 1;
                assert!(n < 60 * 10, "hiker never arrives");
            }
        };
        settle(&mut con);
        let mut steps = 0;
        while con.hub.cursor != i {
            press(&mut con, Pad { right: true, ..Pad::default() });
            settle(&mut con);
            steps += 1;
            assert!(steps <= CARTS.len(), "the hiker never reaches {}", CARTS[i].id);
        }
        press(&mut con, Pad { start: true, ..Pad::default() });
        for _ in 0..180 {
            con.update([Pad::default(); 2], false);
        }
        assert_eq!(con.bank(), i + 1, "did not enter {}", CARTS[i].id);
        let mut n = 0;
        while con.game.is_some() {
            con.update([Pad::default(); 2], n % 30 == 0);
            n += 1;
            assert!(n < 60 * 10);
        }
        assert!(!con.quit);
    }
    let _ = bank_sounds(0);
}
