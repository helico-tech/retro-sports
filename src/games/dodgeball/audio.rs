//! Dodge or Die! sound effects and songs, made with the engine synth.

use super::game::Sfx;
use crate::engine::synth::*;

pub const ALL_SFX: [Sfx; 14] = [
    Sfx::Throw,
    Sfx::Super,
    Sfx::Hit,
    Sfx::BigHit,
    Sfx::Catch,
    Sfx::Bounce,
    Sfx::Jump,
    Sfx::Whistle,
    Sfx::Blip,
    Sfx::Confirm,
    Sfx::Ko,
    Sfx::Cheer,
    Sfx::Pickup,
    Sfx::Pass,
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Track {
    Title,
    Match,
    Champion,
}

pub const ALL_TRACKS: [Track; 3] = [Track::Title, Track::Match, Track::Champion];

pub fn sfx_samples(s: Sfx) -> Vec<f32> {
    let mut b = Buf::new(match s {
        Sfx::Cheer => 1.6,
        Sfx::Whistle | Sfx::Ko | Sfx::BigHit => 0.6,
        _ => 0.35,
    });
    match s {
        Sfx::Throw => {
            b.hiss(0.0, 0.18, 0.45, 0.35, 0.05, 1.0);
            b.tone(0.0, 0.09, 700.0, 250.0, Wave::Square(0.5), 0.1, 0.002, 1.0);
        }
        Sfx::Super => {
            for (k, n) in [72.0, 76.0, 79.0, 84.0, 88.0, 91.0].iter().enumerate() {
                b.tone(k as f32 * 0.032, 0.05, midi_hz(*n), midi_hz(*n), Wave::Square(0.25), 0.18, 0.002, 0.5);
            }
            b.hiss(0.0, 0.33, 0.4, 0.5, 0.08, 1.0);
        }
        Sfx::Hit => {
            b.hiss(0.0, 0.15, 0.7, 0.6, 0.001, 2.0);
            b.tone(0.0, 0.12, 220.0, 60.0, Wave::Square(0.5), 0.3, 0.001, 1.0);
        }
        Sfx::BigHit => {
            b.hiss(0.0, 0.4, 0.8, 0.45, 0.001, 1.5);
            b.tone(0.0, 0.32, 160.0, 40.0, Wave::Tri, 0.7, 0.001, 1.0);
            b.tone(0.0, 0.25, 110.0, 30.0, Wave::Square(0.5), 0.25, 0.001, 1.0);
        }
        Sfx::Catch => {
            b.tone(0.0, 0.05, midi_hz(88.0), midi_hz(88.0), Wave::Square(0.25), 0.2, 0.001, 0.3);
            b.tone(0.05, 0.12, midi_hz(95.0), midi_hz(95.0), Wave::Square(0.25), 0.2, 0.001, 1.0);
            b.tone(0.0, 0.05, 180.0, 90.0, Wave::Tri, 0.4, 0.001, 1.0);
        }
        Sfx::Bounce => b.tone(0.0, 0.07, 240.0, 110.0, Wave::Tri, 0.45, 0.001, 1.0),
        Sfx::Jump => b.tone(0.0, 0.12, 330.0, 760.0, Wave::Square(0.5), 0.1, 0.002, 1.0),
        Sfx::Whistle => {
            for (st, du) in [(0.0f32, 0.12f32), (0.17, 0.36)] {
                let i0 = (st * SR) as usize;
                let n = (du * SR) as usize;
                let mut ph = 0.0f32;
                for k in 0..n {
                    let t = k as f32 / SR;
                    let f = 2500.0 + if (t * 30.0 * std::f32::consts::TAU).sin() > 0.0 { 160.0 } else { 0.0 };
                    ph += f / SR;
                    let env = (t / 0.01).min(1.0) * if k > n - 400 { (n - k) as f32 / 400.0 } else { 1.0 };
                    b.s[i0 + k] += osc(Wave::Square(0.5), ph) * 0.12 * env;
                }
            }
        }
        Sfx::Blip => b.tone(0.0, 0.035, 1046.0, 1046.0, Wave::Square(0.25), 0.15, 0.001, 0.5),
        Sfx::Confirm => {
            for (k, n) in [76.0, 83.0, 88.0].iter().enumerate() {
                b.tone(k as f32 * 0.05, 0.08, midi_hz(*n), midi_hz(*n), Wave::Square(0.25), 0.17, 0.001, 0.6);
            }
        }
        Sfx::Ko => {
            let n = (0.55 * SR) as usize;
            let mut ph = 0.0;
            for k in 0..n {
                let t = k as f32 / SR;
                let u = k as f32 / n as f32;
                let f = (900.0 - 780.0 * u) * (1.0 + 0.04 * (t * 40.0).sin());
                ph += f / SR;
                b.s[k] += osc(Wave::Square(0.5), ph) * 0.15 * (1.0 - u);
            }
        }
        Sfx::Cheer => {
            // crowd: noise with a wobbling filter, plus a few whistles
            let n = (1.5 * SR) as usize;
            let mut y = 0.0;
            let mut y2 = 0.0;
            for k in 0..n {
                let t = k as f32 / SR;
                let u = k as f32 / n as f32;
                let r = b.noise();
                let lp = 0.12 + 0.08 * (t * 13.0).sin() * (t * 7.0).sin();
                y += (r - y) * lp;
                y2 += (y - y2) * 0.5;
                let env = (t / 0.25).min(1.0) * (1.0 - u).powf(1.4);
                b.s[k] += (y - y2 * 0.5) * 0.9 * env;
            }
            for (st, f0) in [(0.2f32, 1300.0f32), (0.45, 1600.0), (0.7, 1450.0)] {
                b.tone(st, 0.18, f0, f0 * 1.25, Wave::Square(0.5), 0.035, 0.03, 0.5);
            }
        }
        Sfx::Pickup => b.tone(0.0, 0.05, 520.0, 780.0, Wave::Tri, 0.35, 0.001, 0.5),
        Sfx::Pass => b.tone(0.0, 0.07, 440.0, 560.0, Wave::Square(0.125), 0.12, 0.002, 1.0),
    }
    b.s
}

// ---------------------------------------------------------------- music

const A: [i32; 3] = [57, 61, 64];
const FSM: [i32; 3] = [54, 57, 61];
const D: [i32; 3] = [50, 54, 57];
const E: [i32; 3] = [52, 56, 59];
const EM: [i32; 3] = [52, 55, 59];
const C: [i32; 3] = [48, 52, 55];
const B: [i32; 3] = [59, 63, 66];

const TITLE: Song = Song {
    bpm: 132.0,
    chords: &[A, FSM, D, E, A, FSM, D, E],
    melody: &[
        "E5 - - C#5 - E5 - A5 - - - - G#5 - F#5 -",
        "E5 - - - C#5 - - - A4 - B4 - C#5 - - -",
        "D5 - - F#5 - - A5 - - - F#5 - A5 - B5 -",
        "B5 - - - G#5 - - - E5 - - - . . . .",
        "E5 - - C#5 - E5 - A5 - - - - G#5 - F#5 -",
        "E5 - - - C#5 - E5 - F#5 - - - E5 - C#5 -",
        "D5 - F#5 - A5 - D6 - C#6 - - - B5 - A5 -",
        "B5 - - - - - G#5 - A5 - - - - - . .",
    ],
    bass: [0, R, 12, R, 0, R, 12, R, 0, R, 12, R, 7, R, 12, R],
    drums: "k.h.s.hkk.h.s.h.",
    lead_duty: 0.25,
};

const MATCH: Song = Song {
    bpm: 150.0,
    chords: &[EM, EM, C, D, EM, EM, C, B],
    melody: &[
        "E5 - . E5 . G5 - . A5 - B5 - . . A5 G5",
        "E5 - - - - - D5 - E5 - - - . . . .",
        "E5 - . E5 . G5 - . C6 - B5 - . . A5 G5",
        "A5 - - - - - F#5 - D5 - - - . . . .",
        "B5 - . B5 . A5 - . G5 - A5 - . . G5 E5",
        "G5 - - - E5 - - - D5 - E5 - . . . .",
        "C5 - E5 - G5 - C6 - B5 - - - G5 - E5 -",
        "D#5 - - - F#5 - - - B5 - - - A5 - F#5 -",
    ],
    bass: [0, 0, 12, 0, 0, 0, 12, 0, 0, 0, 12, 0, 7, 7, 10, 12],
    drums: "k.h.s.hkk.hks.hh",
    lead_duty: 0.5,
};

const CHAMP: Song = Song {
    bpm: 120.0,
    chords: &[A, D, E, A],
    melody: &[
        "A4 - C#5 - E5 - A5 - - - E5 - A5 - - -",
        "F#5 - - - A5 - - - D6 - - - C#6 - B5 -",
        "B5 - - - G#5 - E5 - B5 - - - C#6 - D6 -",
        "C#6 - - - - - - - A5 - - - - - - -",
    ],
    bass: [0, R, R, 0, R, R, 0, R, 7, R, R, 7, R, R, 12, R],
    drums: "k...s..kk...s.hh",
    lead_duty: 0.25,
};

pub fn music_samples(t: Track) -> Vec<f32> {
    render_song(match t {
        Track::Title => &TITLE,
        Track::Match => &MATCH,
        Track::Champion => &CHAMP,
    })
}
