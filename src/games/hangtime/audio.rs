//! HANG TIME! sounds and songs, made with the engine synth: the variometer
//! beeps, wind, honks and baas, a dreamy title song, a breezy flight song
//! and a fanfare.

use super::flight::Sfx;
use crate::engine::synth::*;

pub const ALL_SFX: [Sfx; 16] = [
    Sfx::Vario1,
    Sfx::Vario2,
    Sfx::Vario3,
    Sfx::Sink,
    Sfx::Stall,
    Sfx::Whoosh,
    Sfx::Step,
    Sfx::Thud,
    Sfx::Crash,
    Sfx::Splash,
    Sfx::Honk,
    Sfx::Baa,
    Sfx::Ding,
    Sfx::Cheer,
    Sfx::Blip,
    Sfx::Confirm,
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Track {
    Title,
    Flight,
    Win,
}

pub const ALL_TRACKS: [Track; 3] = [Track::Title, Track::Flight, Track::Win];

pub fn volume(s: Sfx) -> f32 {
    match s {
        Sfx::Vario1 | Sfx::Vario2 | Sfx::Vario3 => 0.45,
        Sfx::Sink => 0.4,
        Sfx::Cheer | Sfx::Whoosh => 0.5,
        Sfx::Step => 0.6,
        _ => 0.8,
    }
}

/// A honk or a baa: a nasal pulse with wobble.
#[allow(clippy::too_many_arguments)]
fn voice(b: &mut Buf, start: f32, dur: f32, f0: f32, f1: f32, wob: f32, rate: f32, duty: f32, vol: f32) {
    let i0 = (start * SR) as usize;
    let n = (dur * SR) as usize;
    let mut ph = 0.0f32;
    for k in 0..n {
        let t = k as f32 / SR;
        let u = k as f32 / n as f32;
        let f = (f0 + (f1 - f0) * u) * (1.0 + wob * (t * rate * std::f32::consts::TAU).sin());
        ph += f / SR;
        let env = (t / 0.01).min(1.0) * (1.0 - u).powf(0.6);
        if let Some(x) = b.s.get_mut(i0 + k) {
            *x += osc(Wave::Square(duty), ph) * vol * env;
        }
    }
}

pub fn sfx_samples(s: Sfx) -> Vec<f32> {
    let mut b = Buf::new(match s {
        Sfx::Cheer => 1.6,
        Sfx::Crash | Sfx::Splash | Sfx::Baa | Sfx::Ding | Sfx::Stall => 0.8,
        Sfx::Whoosh | Sfx::Honk | Sfx::Sink => 0.5,
        _ => 0.3,
    });
    match s {
        // the classic vario: short beeps, higher and faster as you climb
        Sfx::Vario1 => b.tone(0.0, 0.07, 880.0, 900.0, Wave::Square(0.25), 0.11, 0.002, 0.4),
        Sfx::Vario2 => b.tone(0.0, 0.06, 1175.0, 1210.0, Wave::Square(0.25), 0.11, 0.002, 0.4),
        Sfx::Vario3 => {
            b.tone(0.0, 0.05, 1568.0, 1660.0, Wave::Square(0.25), 0.11, 0.002, 0.4);
            b.tone(0.0, 0.05, 784.0, 830.0, Wave::Tri, 0.08, 0.002, 0.4);
        }
        Sfx::Sink => {
            b.tone(0.0, 0.42, 196.0, 175.0, Wave::Square(0.5), 0.06, 0.03, 0.7);
            b.tone(0.0, 0.42, 98.0, 88.0, Wave::Tri, 0.2, 0.03, 0.7);
        }
        Sfx::Stall => {
            for k in 0..6 {
                let f = if k % 2 == 0 { 988.0 } else { 740.0 };
                b.tone(k as f32 * 0.09, 0.08, f, f, Wave::Square(0.5), 0.1, 0.002, 0.3);
            }
        }
        Sfx::Whoosh => {
            // wind swelling past the sail
            let n = (0.45 * SR) as usize;
            let mut y = 0.0;
            for k in 0..n {
                let u = k as f32 / n as f32;
                let r = b.noise();
                let lp = 0.04 + 0.2 * (u * std::f32::consts::PI).sin();
                y += (r - y) * lp;
                b.s[k] += y * 1.2 * (u * std::f32::consts::PI).sin();
            }
        }
        Sfx::Step => {
            b.hiss(0.0, 0.04, 0.5, 0.5, 0.001, 2.0);
            b.tone(0.0, 0.06, 140.0, 70.0, Wave::Sine, 0.5, 0.001, 1.0);
        }
        Sfx::Thud => {
            b.tone(0.0, 0.18, 150.0, 50.0, Wave::Sine, 0.8, 0.001, 1.0);
            b.hiss(0.0, 0.12, 0.4, 0.3, 0.001, 2.0);
        }
        Sfx::Crash => {
            b.hiss(0.0, 0.6, 0.8, 0.45, 0.001, 1.6);
            b.tone(0.0, 0.4, 320.0, 60.0, Wave::Square(0.5), 0.2, 0.001, 1.0);
            // a little "boing" for the comedy
            b.tone(0.3, 0.35, 260.0, 520.0, Wave::Tri, 0.25, 0.005, 1.0);
        }
        Sfx::Splash => {
            b.hiss(0.0, 0.5, 0.9, 0.7, 0.002, 1.8);
            for k in 0..5 {
                let f0 = 500.0 + k as f32 * 170.0;
                b.tone(0.12 + k as f32 * 0.07, 0.06, f0, f0 * 1.6, Wave::Sine, 0.18, 0.002, 1.0);
            }
        }
        Sfx::Honk => {
            voice(&mut b, 0.0, 0.16, 360.0, 320.0, 0.02, 30.0, 0.3, 0.16);
            voice(&mut b, 0.22, 0.18, 380.0, 330.0, 0.02, 30.0, 0.3, 0.16);
        }
        Sfx::Baa => voice(&mut b, 0.0, 0.7, 420.0, 360.0, 0.06, 18.0, 0.2, 0.16),
        Sfx::Ding => {
            b.tone(0.0, 0.7, 1319.0, 1319.0, Wave::Sine, 0.3, 0.001, 1.6);
            b.tone(0.0, 0.5, 1976.0, 1976.0, Wave::Sine, 0.15, 0.001, 2.0);
            b.tone(0.08, 0.6, 2637.0, 2637.0, Wave::Square(0.25), 0.04, 0.001, 2.0);
        }
        Sfx::Cheer => {
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
        Sfx::Blip => b.tone(0.0, 0.035, 1046.0, 1046.0, Wave::Square(0.25), 0.15, 0.001, 0.5),
        Sfx::Confirm => {
            for (k, n) in [74.0, 79.0, 86.0].iter().enumerate() {
                b.tone(k as f32 * 0.05, 0.08, midi_hz(*n), midi_hz(*n), Wave::Square(0.25), 0.17, 0.001, 0.6);
            }
        }
    }
    b.s
}

// ---------------------------------------------------------------- music

const F: [i32; 3] = [53, 57, 60];
const DM: [i32; 3] = [50, 53, 57];
const BB: [i32; 3] = [46, 50, 53];
const C: [i32; 3] = [48, 52, 55];
const G: [i32; 3] = [55, 59, 62];
const EM: [i32; 3] = [52, 55, 59];
const D: [i32; 3] = [50, 54, 57];

/// Dreamy and slow: floating over the dunes.
const TITLE: Song = Song {
    bpm: 84.0,
    chords: &[F, DM, BB, C, F, DM, BB, C],
    melody: &[
        "A5 - - - C6 - - - A5 - G5 - F5 - - -",
        "D5 - - - F5 - - - A5 - - - G5 - - -",
        "F5 - - - D5 - - - F5 - A#5 - A5 - - -",
        "G5 - - - - - E5 - C5 - - - . . . .",
        "A5 - - - C6 - - - F6 - - - E6 - D6 -",
        "C6 - - - A5 - - - F5 - - - A5 - - -",
        "A#5 - - - A5 - - - G5 - - - F5 - G5 -",
        "E5 - - - - - - - C5 - - - . . . .",
    ],
    bass: [0, R, R, R, 7, R, R, R, 12, R, R, R, 7, R, R, R],
    drums: "k...h...s...h...",
    lead_duty: 0.125,
};

/// Breezy: wind in the sail, storks overhead.
const FLIGHT: Song = Song {
    bpm: 120.0,
    chords: &[G, EM, C, D, G, EM, C, D],
    melody: &[
        "B5 - D6 - B5 - G5 - A5 - B5 - . . D5 -",
        "E5 - G5 - B5 - - - A5 - G5 - E5 - - -",
        "C6 - B5 - A5 - G5 - E5 - G5 - - - . .",
        "F#5 - - - A5 - - - D6 - C6 - A5 - - -",
        "G5 - B5 - D6 - G6 - - - D6 - B5 - - -",
        "E6 - D6 - B5 - G5 - E5 - G5 - B5 - - -",
        "C6 - - - E6 - - - D6 - C6 - B5 - A5 -",
        "A5 - - - - - F#5 - D5 - - - . . . .",
    ],
    bass: [0, R, 7, R, 12, R, 7, R, 0, R, 7, R, 12, R, 10, R],
    drums: "k.h.s.h.k.h.s.hh",
    lead_duty: 0.25,
};

/// Fanfare for the champion of the sky.
const WIN: Song = Song {
    bpm: 132.0,
    chords: &[C, F, G, C],
    melody: &[
        "C5 - E5 - G5 - C6 - - - G5 - C6 - - -",
        "A5 - - - C6 - - - F6 - - - E6 - D6 -",
        "D6 - - - B5 - G5 - D6 - - - E6 - F6 -",
        "E6 - - - - - - - C6 - - - - - - -",
    ],
    bass: [0, R, R, 0, R, R, 0, R, 7, R, R, 7, R, R, 12, R],
    drums: "k...s..kk...s.hh",
    lead_duty: 0.25,
};

pub fn music_samples(t: Track) -> Vec<f32> {
    render_song(match t {
        Track::Title => &TITLE,
        Track::Flight => &FLIGHT,
        Track::Win => &WIN,
    })
}
