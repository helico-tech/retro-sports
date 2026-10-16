// release builds on Windows open no console window next to the game
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;
use retro_sports::console::{bank_sounds, Console, Cue};
use retro_sports::engine::gfx::{Frame, H, W};
use retro_sports::engine::input::Pad;
use retro_sports::engine::synth::wav;
use retro_sports::games::CARTS;
use std::collections::HashMap;

fn conf() -> Conf {
    Conf {
        window_title: "RETRO SPORTS".to_owned(),
        window_width: W * 3,
        window_height: H * 3,
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

const USAGE: &str = "usage: retro-sports [--shots <dir> [game]] [--<tool> <game> [n]]";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let Some(flag) = args.get(1).and_then(|a| a.strip_prefix("--")) else {
        return macroquad::Window::from_config(conf(), run());
    };
    if flag == "shots" {
        let dir = args.get(2).map(String::as_str).unwrap_or("shots");
        let only = args.get(3).map(String::as_str);
        if only.is_none() || only == Some("hub") {
            retro_sports::hub::shots(&format!("{dir}/hub"));
        }
        for c in CARTS.iter().filter(|c| only.is_none() || only == Some(c.id)) {
            (c.shots)(&format!("{dir}/{}", c.id));
        }
        return;
    }
    let cart = args.get(2).and_then(|id| CARTS.iter().find(|c| c.id == id));
    let tool = cart.and_then(|c| c.tools.iter().find(|(name, _)| *name == flag));
    match tool {
        Some((_, run_tool)) => run_tool(args.get(3).and_then(|s| s.parse().ok()).unwrap_or(100)),
        None => {
            eprintln!("{USAGE}");
            for c in CARTS {
                let tools: Vec<_> = c.tools.iter().map(|(n, _)| *n).collect();
                eprintln!("  {:<10} tools: {}", c.id, tools.join(", "));
            }
            std::process::exit(2);
        }
    }
}

fn pads() -> [Pad; 2] {
    let k = is_key_down;
    [
        Pad {
            up: k(KeyCode::Up),
            down: k(KeyCode::Down),
            left: k(KeyCode::Left),
            right: k(KeyCode::Right),
            // , . / work too, so in 2P the two left hands don't collide
            a: k(KeyCode::Z) || k(KeyCode::Comma),
            b: k(KeyCode::X) || k(KeyCode::Period),
            c: k(KeyCode::C) || k(KeyCode::Slash),
            start: k(KeyCode::Enter),
        },
        Pad {
            up: k(KeyCode::W),
            down: k(KeyCode::S),
            left: k(KeyCode::A),
            right: k(KeyCode::D),
            a: k(KeyCode::F),
            b: k(KeyCode::G),
            c: k(KeyCode::H),
            start: k(KeyCode::Tab),
        },
    ]
}

/// A game's sounds, loaded into the audio device the first time it is used.
struct Loaded {
    sfx: Vec<Option<(Sound, f32)>>,
    music: Vec<Option<(Sound, f32)>>,
}

async fn load_bank(bank: usize) -> Loaded {
    let b = bank_sounds(bank);
    let mut out = Loaded { sfx: Vec::new(), music: Vec::new() };
    for s in &b.sfx {
        out.sfx.push(load_sound_from_bytes(&wav(&s.samples)).await.ok().map(|x| (x, s.volume)));
    }
    for s in &b.music {
        out.music.push(load_sound_from_bytes(&wav(&s.samples)).await.ok().map(|x| (x, s.volume)));
    }
    out
}

async fn run() {
    let seed = (macroquad::miniquad::date::now() * 1000.0) as u64;
    let mut console = Console::new(seed);
    let mut banks: HashMap<usize, Loaded> = HashMap::new();
    let mut frame = Frame::new();
    let mut rgba = Vec::new();
    frame.to_rgba(&mut rgba);
    let tex = Texture2D::from_rgba8(W as u16, H as u16, &rgba);
    tex.set_filter(FilterMode::Nearest);

    let dt = 1.0 / 60.0;
    let mut acc = 0.0f32;
    let mut pending_esc = false;
    let mut scanlines = true;
    let mut playing: Option<Cue> = None;
    loop {
        if !banks.contains_key(&console.bank()) {
            banks.insert(console.bank(), load_bank(console.bank()).await);
            // loading takes a moment; don't fast-forward the game to catch up
            get_frame_time();
            acc = 0.0;
        }
        acc += get_frame_time().min(0.1);
        pending_esc |= is_key_pressed(KeyCode::Escape);
        if is_key_pressed(KeyCode::F1) {
            scanlines = !scanlines;
        }
        while acc >= dt {
            console.update(pads(), pending_esc);
            pending_esc = false;
            acc -= dt;
        }
        for cue in console.sfx.drain(..) {
            if let Some(Some((snd, vol))) = banks.get(&cue.bank).and_then(|b| b.sfx.get(cue.id)) {
                play_sound(snd, PlaySoundParams { looped: false, volume: *vol });
            }
        }
        let want = console.music();
        if want != playing {
            if let Some(Some((snd, _))) = playing.and_then(|p| banks.get(&p.bank).and_then(|b| b.music.get(p.id))) {
                stop_sound(snd);
            }
            if let Some(Some((snd, vol))) = want.and_then(|w| banks.get(&w.bank).and_then(|b| b.music.get(w.id))) {
                play_sound(snd, PlaySoundParams { looped: true, volume: *vol });
            }
            playing = want;
        }
        if console.quit {
            // a browser tab has nothing to quit to; stopping the loop would freeze it
            if cfg!(target_arch = "wasm32") {
                console.quit = false;
                console.hub.quit = false;
            } else {
                break;
            }
        }

        console.draw(&mut frame);
        frame.to_rgba(&mut rgba);
        tex.update_from_bytes(W as u32, H as u32, &rgba);

        clear_background(BLACK);
        let (sw, sh) = (screen_width(), screen_height());
        let scale = (sw / W as f32).min(sh / H as f32).floor().max(1.0);
        let (w, h) = (W as f32 * scale, H as f32 * scale);
        let (x, y) = (((sw - w) / 2.0).floor(), ((sh - h) / 2.0).floor());
        draw_texture_ex(&tex, x, y, WHITE, DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
        if scanlines && scale >= 3.0 {
            let thick = (scale / 3.0).floor();
            for row in 0..H {
                draw_rectangle(x, y + (row as f32 + 1.0) * scale - thick, w, thick, Color::new(0.0, 0.0, 0.0, 0.22));
            }
        }
        next_frame().await;
    }
}
