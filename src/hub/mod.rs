//! The game selector: Helico Island. (First version: a plain list.)

use crate::cart::{Bank, Sound};
use crate::engine::gfx::*;
use crate::engine::input::{edge, Pad};
use crate::engine::ui::*;
use crate::games::CARTS;

pub struct Hub {
    pub cursor: usize,
    pub t: u32,
    pub sfx: Vec<usize>,
    pub start: Option<usize>,
    pub quit: bool,
    prev: [Pad; 2],
}

impl Hub {
    pub fn new(_seed: u64) -> Self {
        Hub { cursor: 0, t: 0, sfx: Vec::new(), start: None, quit: false, prev: [Pad::default(); 2] }
    }
    pub fn came_back(&mut self, i: usize) {
        self.cursor = i;
    }
    pub fn update(&mut self, pads: [Pad; 2], esc: bool) {
        self.t += 1;
        let e = edge(pads[0], self.prev[0]);
        self.prev = pads;
        if esc {
            self.quit = true;
            return;
        }
        if e.down {
            self.cursor = (self.cursor + 1) % CARTS.len();
        }
        if e.up {
            self.cursor = (self.cursor + CARTS.len() - 1) % CARTS.len();
        }
        if e.a || e.start {
            self.start = Some(self.cursor);
        }
    }
    pub fn music(&self) -> Option<usize> {
        None
    }
    pub fn draw(&self, f: &mut Frame) {
        f.clear(rgb(16, 16, 48));
        f.logo(128, 16, "RETRO SPORTS", 2, &SUNSET, OUTLINE, LOGO_SHADOW);
        for (i, c) in CARTS.iter().enumerate() {
            let col = if i == self.cursor { rgb(248, 224, 72) } else { rgb(200, 200, 216) };
            f.text_center(128, 70 + i as i32 * 14, c.title, col, OUTLINE, true);
        }
    }
}

pub fn sounds() -> Bank {
    Bank { sfx: vec![Sound { samples: vec![0.0; 10], volume: 0.0 }], music: Vec::new() }
}

pub fn shots(dir: &str) {
    std::fs::create_dir_all(dir).unwrap();
    let mut f = Frame::new();
    Hub::new(1).draw(&mut f);
    crate::engine::png::save_png(&f, &std::path::Path::new(dir).join("01_island.png"), 3);
}
