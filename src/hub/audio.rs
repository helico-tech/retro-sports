//! Island sounds: footsteps, a blip, a ding on arrival, a fanfare on enter,
//! and a sunny calypso for walking around.

use crate::cart::{Bank, Sound};
use crate::engine::synth::*;

pub const STEP: usize = 0;
pub const BLIP: usize = 1;
pub const ARRIVE: usize = 2;
pub const ENTER: usize = 3;

pub const ISLAND: usize = 0;

const C: [i32; 3] = [48, 52, 55];
const AM: [i32; 3] = [45, 48, 52];
const F: [i32; 3] = [53, 57, 60];
const G: [i32; 3] = [55, 59, 62];
const DM: [i32; 3] = [50, 53, 57];

const CALYPSO: Song = Song {
    bpm: 126.0,
    chords: &[C, AM, F, G, C, AM, DM, G],
    melody: &[
        "G5 - . E5 . G5 - . C6 - - . B5 - A5 -",
        "G5 - - - E5 - - - . . E5 - D5 - C5 -",
        "A5 - . F5 . A5 - . C6 - - . D6 - C6 -",
        "B5 - - - G5 - - - . . . . . . . .",
        "G5 - . E5 . G5 - . C6 - - . E6 - D6 -",
        "C6 - - - A5 - - - E5 - G5 - A5 - . .",
        "F5 - . A5 . D6 - . C6 - . B5 . A5 - .",
        "B5 - - - D6 - - - C6 - - - - - . .",
    ],
    bass: [0, R, R, 7, R, R, 12, R, 0, R, R, 7, R, 12, 7, R],
    drums: "k.hsk.h.k.hsk.hh",
    lead_duty: 0.25,
};

fn sfx(i: usize) -> Vec<f32> {
    let mut b = Buf::new(0.4);
    match i {
        STEP => {
            b.hiss(0.0, 0.03, 0.18, 0.3, 0.001, 2.0);
            b.tone(0.0, 0.03, 180.0, 120.0, Wave::Tri, 0.2, 0.001, 1.0);
        }
        BLIP => b.tone(0.0, 0.035, 988.0, 988.0, Wave::Square(0.25), 0.13, 0.001, 0.5),
        ARRIVE => {
            b.tone(0.0, 0.06, midi_hz(84.0), midi_hz(84.0), Wave::Square(0.25), 0.14, 0.001, 0.6);
            b.tone(0.06, 0.12, midi_hz(91.0), midi_hz(91.0), Wave::Square(0.25), 0.14, 0.001, 1.0);
        }
        _ => {
            for (k, n) in [72.0, 76.0, 79.0, 84.0, 88.0].iter().enumerate() {
                b.tone(k as f32 * 0.045, 0.12, midi_hz(*n), midi_hz(*n), Wave::Square(0.25), 0.15, 0.001, 0.7);
            }
            b.hiss(0.1, 0.25, 0.25, 0.4, 0.05, 1.0);
        }
    }
    b.s
}

pub fn sounds() -> Bank {
    Bank {
        sfx: (0..4).map(|i| Sound { samples: sfx(i), volume: 0.7 }).collect(),
        music: vec![Sound { samples: render_song(&CALYPSO), volume: 0.45 }],
    }
}
