//! TRAIL MIX! sounds and songs, made with the engine synth: a yodelling
//! title song, a driving race song and an alphorn fanfare.

use super::race::Sfx;
use crate::engine::synth::*;

pub const ALL_SFX: [Sfx; 16] = [
    Sfx::Step,
    Sfx::Jump,
    Sfx::Splash,
    Sfx::Cowbell,
    Sfx::Baa,
    Sfx::Whistle,
    Sfx::Crowd,
    Sfx::Gulp,
    Sfx::Trip,
    Sfx::Slide,
    Sfx::Ding,
    Sfx::Heart,
    Sfx::Blip,
    Sfx::Confirm,
    Sfx::Horn,
    Sfx::Moo,
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Track {
    Title,
    Race,
    Win,
}

pub const ALL_TRACKS: [Track; 3] = [Track::Title, Track::Race, Track::Win];

pub fn volume(s: Sfx) -> f32 {
    match s {
        Sfx::Step => 0.25,
        Sfx::Crowd | Sfx::Slide => 0.5,
        Sfx::Heart => 0.7,
        _ => 0.8,
    }
}

pub fn sfx_samples(s: Sfx) -> Vec<f32> {
    let mut b = Buf::new(match s {
        Sfx::Crowd => 1.5,
        Sfx::Baa | Sfx::Moo | Sfx::Horn | Sfx::Slide => 0.8,
        Sfx::Ding | Sfx::Cowbell | Sfx::Splash => 0.5,
        _ => 0.3,
    });
    match s {
        Sfx::Step => {
            b.hiss(0.0, 0.04, 0.35, 0.25, 0.002, 2.0);
            b.tone(0.0, 0.05, 110.0, 60.0, Wave::Tri, 0.3, 0.001, 1.5);
        }
        Sfx::Jump => b.tone(0.0, 0.13, 300.0, 720.0, Wave::Square(0.5), 0.1, 0.002, 1.0),
        Sfx::Splash => {
            b.hiss(0.0, 0.4, 0.7, 0.75, 0.01, 1.4);
            for (k, f0) in [900.0f32, 1300.0, 700.0, 1600.0].iter().enumerate() {
                b.tone(0.05 + k as f32 * 0.06, 0.05, *f0, f0 * 1.6, Wave::Sine, 0.12, 0.002, 1.0);
            }
        }
        Sfx::Cowbell => {
            for st in [0.0f32, 0.16] {
                b.tone(st, 0.3, 540.0, 540.0, Wave::Square(0.5), 0.08, 0.001, 2.5);
                b.tone(st, 0.3, 800.0, 800.0, Wave::Square(0.5), 0.07, 0.001, 2.5);
            }
        }
        Sfx::Baa => {
            // a goat: buzzy, wobbling, sliding down
            let n = (0.62 * SR) as usize;
            let mut ph = 0.0f32;
            for k in 0..n {
                let t = k as f32 / SR;
                let u = k as f32 / n as f32;
                let f = 470.0 - 90.0 * u + 14.0 * (t * 38.0).sin();
                ph += f / SR;
                let trem = 0.6 + 0.4 * (t * 26.0 * std::f32::consts::TAU).sin().abs();
                let env = (t / 0.03).min(1.0) * (1.0 - u).powf(0.6);
                b.s[k] += osc(Wave::Square(0.3), ph) * 0.26 * env * trem;
            }
        }
        Sfx::Whistle => {
            b.tone(0.0, 0.18, 2900.0, 2500.0, Wave::Sine, 0.25, 0.005, 0.6);
            b.tone(0.0, 0.18, 2900.0, 2500.0, Wave::Square(0.5), 0.03, 0.005, 0.6);
        }
        Sfx::Crowd => {
            let n = (1.4 * SR) as usize;
            let (mut y, mut y2) = (0.0, 0.0);
            for k in 0..n {
                let t = k as f32 / SR;
                let u = k as f32 / n as f32;
                let r = b.noise();
                let lp = 0.12 + 0.08 * (t * 13.0).sin() * (t * 7.0).sin();
                y += (r - y) * lp;
                y2 += (y - y2) * 0.5;
                let env = (t / 0.2).min(1.0) * (1.0 - u).powf(1.3);
                b.s[k] += (y - y2 * 0.5) * 1.3 * env;
            }
            for st in [0.1f32, 0.35, 0.6, 0.85] {
                b.tone(st, 0.18, 560.0, 560.0, Wave::Square(0.5), 0.03, 0.001, 2.0);
                b.tone(st, 0.18, 830.0, 830.0, Wave::Square(0.5), 0.025, 0.001, 2.0);
            }
        }
        Sfx::Gulp => {
            b.tone(0.0, 0.08, 220.0, 120.0, Wave::Sine, 0.5, 0.002, 1.0);
            b.tone(0.11, 0.09, 200.0, 100.0, Wave::Sine, 0.5, 0.002, 1.0);
        }
        Sfx::Trip => {
            b.tone(0.0, 0.16, 170.0, 40.0, Wave::Tri, 0.7, 0.001, 1.0);
            b.hiss(0.0, 0.12, 0.5, 0.5, 0.001, 2.0);
            b.tone(0.12, 0.15, 600.0, 200.0, Wave::Square(0.25), 0.07, 0.002, 1.0);
        }
        Sfx::Slide => b.hiss(0.0, 0.75, 0.45, 0.22, 0.15, 1.2),
        Sfx::Ding => {
            b.tone(0.0, 0.45, 1568.0, 1568.0, Wave::Sine, 0.25, 0.001, 2.0);
            b.tone(0.0, 0.3, 3136.0, 3136.0, Wave::Sine, 0.08, 0.001, 2.0);
            b.tone(0.0, 0.2, 2093.0, 2093.0, Wave::Square(0.25), 0.04, 0.001, 2.0);
        }
        Sfx::Heart => {
            b.tone(0.0, 0.09, 70.0, 45.0, Wave::Sine, 0.8, 0.002, 1.2);
            b.tone(0.14, 0.08, 64.0, 42.0, Wave::Sine, 0.6, 0.002, 1.2);
        }
        Sfx::Blip => b.tone(0.0, 0.035, 1046.0, 1046.0, Wave::Square(0.25), 0.15, 0.001, 0.5),
        Sfx::Confirm => {
            for (k, n) in [74.0, 81.0, 86.0].iter().enumerate() {
                b.tone(k as f32 * 0.05, 0.08, midi_hz(*n), midi_hz(*n), Wave::Square(0.25), 0.17, 0.001, 0.6);
            }
        }
        Sfx::Horn => {
            // an alphorn-ish start horn: two low notes a fifth apart
            b.tone(0.0, 0.7, 196.0, 196.0, Wave::Square(0.5), 0.12, 0.05, 0.4);
            b.tone(0.0, 0.7, 293.7, 293.7, Wave::Square(0.4), 0.08, 0.06, 0.4);
            b.tone(0.0, 0.7, 98.0, 98.0, Wave::Tri, 0.35, 0.05, 0.4);
        }
        Sfx::Moo => {
            let n = (0.75 * SR) as usize;
            let mut ph = 0.0f32;
            let mut lp = 0.0f32;
            for k in 0..n {
                let t = k as f32 / SR;
                let u = k as f32 / n as f32;
                let f = 125.0 - 30.0 * u + 3.0 * (t * 9.0).sin();
                ph += f / SR;
                let x = osc(Wave::Square(0.35), ph);
                lp += (x - lp) * (0.05 + 0.1 * (u * 3.0).min(1.0) * (1.0 - u));
                let env = (t / 0.12).min(1.0) * (1.0 - u).powf(0.5);
                b.s[k] += lp * 0.5 * env;
            }
        }
    }
    b.s
}

// ---------------------------------------------------------------- music

const D: [i32; 3] = [50, 54, 57];
const G: [i32; 3] = [55, 59, 62];
const A: [i32; 3] = [57, 61, 64];
const AM: [i32; 3] = [57, 60, 64];
const F: [i32; 3] = [53, 57, 60];
const C: [i32; 3] = [48, 52, 55];
const E: [i32; 3] = [52, 56, 59];

/// Alpine and heroic, with yodel flips between low and high notes.
const TITLE: Song = Song {
    bpm: 124.0,
    chords: &[D, G, A, D, D, G, A, D],
    melody: &[
        "A4 - D5 - F#5 - A5 - D6 - - - A5 - F#5 -",
        "G5 - B5 - D6 - B5 - G5 - - - D5 - - -",
        "A5 - C#6 - E6 - C#6 - A5 - E5 - C#5 - A4 -",
        "D5 - F#5 A5 D6 - - - - - . . A4 - C#5 -",
        "D5 A5 D5 A5 F#5 D6 F#5 D6 A5 - - - F#5 - D5 -",
        "B4 G5 B4 G5 D5 B5 D5 B5 G5 - - - D5 - B4 -",
        "C#5 A5 C#5 A5 E5 C#6 E5 C#6 A5 - G5 - F#5 - E5 -",
        "D5 - - - A4 - - - D5 - - - . . . .",
    ],
    bass: [0, R, 7, R, 12, R, 7, R, 0, R, 7, R, 12, R, 7, R],
    drums: "k...h.h.k...h.h.",
    lead_duty: 0.25,
};

const RACE: Song = Song {
    bpm: 152.0,
    chords: &[AM, F, C, G, AM, F, G, E],
    melody: &[
        "A5 - . A5 . C6 - . E6 - D6 - C6 - B5 -",
        "C6 - - - A5 - - - F5 - - - . . . .",
        "G5 - . G5 . C6 - . E6 - D6 - C6 - D6 -",
        "B5 - - - G5 - - - D5 - - - . . . .",
        "E6 - . E6 . D6 - . C6 - B5 - A5 - C6 -",
        "A5 - - - F5 - A5 - C6 - - - A5 - - -",
        "B5 - . B5 . D6 - . G6 - F6 - E6 - D6 -",
        "E6 - - - B5 - - - G#5 - - - E5 - - -",
    ],
    bass: [0, 0, 12, 0, 0, 0, 12, 0, 0, 0, 12, 0, 7, 7, 10, 12],
    drums: "k.hhs.hkk.hks.hh",
    lead_duty: 0.5,
};

/// An alphorn fanfare: slow, hollow, the natural notes of a long horn.
const WIN: Song = Song {
    bpm: 100.0,
    chords: &[C, F, G, C],
    melody: &[
        "G4 - - - C5 - - - E5 - - - G5 - - -",
        "A5 - - - - - F5 - C5 - - - F5 - - -",
        "G5 - - - B5 - - - D6 - - - B5 - G5 -",
        "C6 - - - - - - - - - - - . . . .",
    ],
    bass: [0, R, R, R, 7, R, R, R, 0, R, R, R, 7, R, R, R],
    drums: "k.......s...k...",
    lead_duty: 0.5,
};

pub fn music_samples(t: Track) -> Vec<f32> {
    render_song(match t {
        Track::Title => &TITLE,
        Track::Race => &RACE,
        Track::Win => &WIN,
    })
}
