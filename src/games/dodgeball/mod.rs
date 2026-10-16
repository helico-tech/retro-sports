//! DODGE OR DIE! 3 vs 3 beach dodgeball.

pub mod app;
pub mod audio;
pub mod court;
pub mod game;
pub mod headless;
pub mod render;
pub mod teams;

use crate::cart::{Bank, Cart, Game, Sound};
use crate::engine::gfx::Frame;
use crate::engine::input::Pad;

pub const CART: Cart = Cart {
    id: "dodgeball",
    title: "DODGE OR DIE!",
    sport: "DODGEBALL",
    blurb: "IF YOU CAN DODGE A WRENCH, YOU CAN DODGE A BALL.",
    new: |seed| Box::new(app::App::new(seed)),
    sounds,
    shots: headless::shots,
    tools: &[("sim", |n| headless::sim(n as usize)), ("probe", headless::probe)],
};

fn sounds() -> Bank {
    use game::Sfx;
    let vol = |s: Sfx| match s {
        Sfx::Cheer => 0.5,
        Sfx::Bounce | Sfx::Pickup => 0.6,
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
