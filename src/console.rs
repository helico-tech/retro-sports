//! The console: the selector (hub) and the game that is running. Turns each
//! game's sound indexes into cues with a bank number, so the window layer
//! can load each game's sounds the first time it is played.

use crate::cart::{Bank, Game};
use crate::engine::gfx::Frame;
use crate::engine::input::{any, Pad};
use crate::engine::rng::Rng;
use crate::games::CARTS;
use crate::hub::Hub;

/// Bank 0 is the selector; game `i` in `CARTS` is bank `i + 1`.
pub const HUB_BANK: usize = 0;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Cue {
    pub bank: usize,
    pub id: usize,
}

pub fn bank_sounds(bank: usize) -> Bank {
    if bank == HUB_BANK { crate::hub::sounds() } else { (CARTS[bank - 1].sounds)() }
}

pub struct Console {
    pub hub: Hub,
    pub game: Option<(usize, Box<dyn Game>)>,
    pub quit: bool,
    pub sfx: Vec<Cue>,
    rng: Rng,
    /// After switching between hub and game, ignore buttons until all are up,
    /// so the key that picked a game does not also press start in it.
    hold: bool,
}

impl Console {
    pub fn new(seed: u64) -> Self {
        Console { hub: Hub::new(seed), game: None, quit: false, sfx: Vec::new(), rng: Rng(seed | 1), hold: false }
    }

    /// The sound bank in use now.
    pub fn bank(&self) -> usize {
        self.game.as_ref().map_or(HUB_BANK, |(i, _)| i + 1)
    }

    pub fn update(&mut self, pads: [Pad; 2], esc: bool) {
        if self.hold && (any(pads[0]) || any(pads[1])) {
            return self.update_inner([Pad::default(); 2], esc);
        }
        self.hold = false;
        self.update_inner(pads, esc)
    }

    fn update_inner(&mut self, pads: [Pad; 2], esc: bool) {
        if let Some((i, g)) = &mut self.game {
            let bank = *i + 1;
            g.update(pads, esc);
            self.sfx.extend(g.take_sfx().into_iter().map(|id| Cue { bank, id }));
            if g.wants_exit() {
                let i = *i;
                self.game = None;
                self.hub.came_back(i);
                self.hold = true;
            }
            return;
        }
        self.hub.update(pads, esc);
        self.sfx.extend(self.hub.sfx.drain(..).map(|id| Cue { bank: HUB_BANK, id }));
        if let Some(i) = self.hub.start.take() {
            let seed = self.rng.next() as u64 | 1;
            self.game = Some((i, (CARTS[i].new)(seed)));
            self.hold = true;
        }
        self.quit = self.hub.quit;
    }

    pub fn draw(&self, f: &mut Frame) {
        match &self.game {
            Some((_, g)) => g.draw(f),
            None => self.hub.draw(f),
        }
    }

    pub fn music(&self) -> Option<Cue> {
        match &self.game {
            Some((i, g)) => g.music().map(|id| Cue { bank: i + 1, id }),
            None => self.hub.music().map(|id| Cue { bank: HUB_BANK, id }),
        }
    }
}
