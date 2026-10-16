//! What a game must provide to sit in the collection: a `Game` that runs at
//! 60 Hz on two pads and draws into a `Frame`, and a `Cart` entry with its
//! name, its sounds and its headless tools.

use crate::engine::gfx::Frame;
use crate::engine::input::Pad;

pub trait Game {
    /// One 60 Hz tick. `esc` is true when Esc was pressed since the last tick.
    fn update(&mut self, pads: [Pad; 2], esc: bool);
    fn draw(&self, f: &mut Frame);
    /// Sound effects started since the last call, as indexes into `Bank::sfx`.
    fn take_sfx(&mut self) -> Vec<usize>;
    /// The song that should play now, as an index into `Bank::music`.
    fn music(&self) -> Option<usize>;
    /// The player left the game (Esc on its title): back to the selector.
    fn wants_exit(&self) -> bool;
}

/// Generated audio samples (44.1 kHz mono) with the volume to play them at.
pub struct Sound {
    pub samples: Vec<f32>,
    pub volume: f32,
}

/// All sounds of one game. Music always loops.
pub struct Bank {
    pub sfx: Vec<Sound>,
    pub music: Vec<Sound>,
}

/// A headless tool: its name on the command line and what it runs with `n`.
pub type Tool = (&'static str, fn(n: u32));

/// One game in the collection.
pub struct Cart {
    /// Short id for the command line, e.g. "dodgeball".
    pub id: &'static str,
    /// The game's name as on the box, e.g. "DODGE OR DIE!".
    pub title: &'static str,
    /// The sport, e.g. "DODGEBALL".
    pub sport: &'static str,
    /// One whimsical line for the selector.
    pub blurb: &'static str,
    pub new: fn(seed: u64) -> Box<dyn Game>,
    pub sounds: fn() -> Bank,
    /// Writes scripted PNG screenshots of every screen into a directory.
    pub shots: fn(dir: &str),
    /// Extra headless tools, e.g. ("sim", run n CPU matches).
    pub tools: &'static [Tool],
}
