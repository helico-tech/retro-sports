//! TRAIL MIX! Trail running up and down Mount Helico, against Gerald the goat.

pub mod app;
pub mod audio;
pub mod course;
pub mod headless;
pub mod race;
pub mod render;
pub mod runners;
pub mod scenery;
pub mod sprites;

use crate::cart::{Bank, Cart, Game, Sound};
use crate::engine::gfx::Frame;
use crate::engine::input::Pad;

pub const CART: Cart = Cart {
    id: "trailmix",
    title: "TRAIL MIX!",
    sport: "TRAIL RUNNING",
    blurb: "THE ULTRA TRAIL OF MOUNT HELICO. BEWARE OF GERALD.",
    new: |seed| Box::new(app::App::new(seed)),
    sounds,
    shots: headless::shots,
    tools: &[("sim", |n| headless::sim(n as usize)), ("probe", headless::probe)],
};

fn sounds() -> Bank {
    Bank {
        sfx: audio::ALL_SFX.iter().map(|&s| Sound { samples: audio::sfx_samples(s), volume: audio::volume(s) }).collect(),
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
