//! STALEN ZENUWEN! Steal a bike on the Amsterdam canals while the BOA's
//! watch: pick locks, act casual, hide in the krul, and throw your loot on
//! Sjaak's boat.

pub mod app;
pub mod art;
pub mod audio;
pub mod bot;
pub mod data;
pub mod headless;
pub mod probe;
pub mod render;
pub mod scenery;
pub mod world;

use crate::cart::{Bank, Cart, Game, Sound};
use crate::engine::gfx::Frame;
use crate::engine::input::Pad;

pub const CART: Cart = Cart {
    id: "zenuwen",
    title: "STALEN ZENUWEN!",
    sport: "BIKE LIFTING",
    blurb: "AMSTERDAM'S FASTEST HANDS. MIND THE BOA'S.",
    new: |seed| Box::new(app::App::new(seed)),
    sounds,
    shots: headless::shots,
    tools: &[("sim", probe::sim), ("probe", probe::probe)],
};

fn sounds() -> Bank {
    use world::Sfx;
    let vol = |s: Sfx| match s {
        Sfx::Click | Sfx::Saw => 0.45,
        Sfx::Cheer => 0.5,
        _ => 0.8,
    };
    Bank {
        sfx: audio::ALL_SFX.iter().map(|&s| Sound { samples: audio::sfx_samples(s), volume: vol(s) }).collect(),
        music: audio::ALL_TRACKS.iter().map(|&t| Sound { samples: audio::music_samples(t), volume: 0.45 }).collect(),
    }
}

impl Game for app::App {
    fn update(&mut self, pads: [Pad; 2], esc: bool) {
        app::App::update(self, pads, esc);
    }
    fn draw(&self, f: &mut Frame) {
        app::App::draw(self, f);
    }
    fn take_sfx(&mut self) -> Vec<usize> {
        self.sfx.drain(..).filter_map(|s| audio::ALL_SFX.iter().position(|&x| x == s)).collect()
    }
    fn music(&self) -> Option<usize> {
        app::App::music(self).and_then(|t| audio::ALL_TRACKS.iter().position(|&x| x == t))
    }
    fn wants_exit(&self) -> bool {
        self.quit
    }
}
