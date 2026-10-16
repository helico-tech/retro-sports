//! DISC DOGS! sound effects and songs, made with the engine synth.

use super::sim::Sfx;
use crate::engine::synth::*;

pub const ALL_SFX: [Sfx; 16] = [
    Sfx::Throw,
    Sfx::Bark,
    Sfx::Chomp,
    Sfx::Flip,
    Sfx::Oof,
    Sfx::Splash,
    Sfx::Squawk,
    Sfx::Chitter,
    Sfx::Gust,
    Sfx::Tick,
    Sfx::Ding,
    Sfx::Whistle,
    Sfx::Cheer,
    Sfx::Blip,
    Sfx::Confirm,
    Sfx::WahWah,
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Track {
    Title,
    Play,
    Results,
    Champion,
}

pub const ALL_TRACKS: [Track; 4] = [Track::Title, Track::Play, Track::Results, Track::Champion];

/// A tone with vibrato (for whistles, squawks and the sad trombone).
#[allow(clippy::too_many_arguments)]
fn vib(b: &mut Buf, start: f32, dur: f32, f0: f32, f1: f32, depth: f32, rate: f32, w: Wave, vol: f32) {
    let i0 = (start * SR) as usize;
    let n = (dur * SR) as usize;
    let mut ph = 0.0f32;
    for k in 0..n {
        let t = k as f32 / SR;
        let u = k as f32 / n as f32;
        let f = (f0 + (f1 - f0) * u) * (1.0 + depth * (t * rate * std::f32::consts::TAU).sin());
        ph += f / SR;
        let env = (t / 0.01).min(1.0) * ((n - k) as f32 / 600.0).min(1.0);
        if let Some(x) = b.s.get_mut(i0 + k) {
            *x += osc(w, ph) * vol * env;
        }
    }
}

pub fn sfx_samples(s: Sfx) -> Vec<f32> {
    let mut b = Buf::new(match s {
        Sfx::Cheer => 1.6,
        Sfx::Splash | Sfx::Gust | Sfx::WahWah => 1.0,
        Sfx::Whistle | Sfx::Squawk | Sfx::Oof => 0.6,
        _ => 0.35,
    });
    match s {
        Sfx::Throw => {
            b.hiss(0.0, 0.22, 0.4, 0.3, 0.06, 1.2);
            b.tone(0.0, 0.12, 900.0, 300.0, Wave::Square(0.5), 0.08, 0.002, 1.0);
        }
        Sfx::Bark => {
            // "arf!"
            b.tone(0.0, 0.11, 560.0, 320.0, Wave::Square(0.3), 0.22, 0.004, 0.8);
            b.tone(0.0, 0.11, 1120.0, 640.0, Wave::Square(0.5), 0.06, 0.004, 1.2);
            b.hiss(0.0, 0.06, 0.25, 0.6, 0.002, 1.5);
        }
        Sfx::Chomp => {
            b.tone(0.0, 0.05, 260.0, 140.0, Wave::Square(0.5), 0.25, 0.001, 1.0);
            b.hiss(0.0, 0.03, 0.4, 0.7, 0.001, 2.0);
            b.tone(0.07, 0.05, 220.0, 120.0, Wave::Square(0.5), 0.22, 0.001, 1.0);
            b.hiss(0.07, 0.03, 0.35, 0.7, 0.001, 2.0);
            b.tone(0.13, 0.08, 784.0, 1046.0, Wave::Square(0.25), 0.12, 0.002, 0.8);
        }
        Sfx::Flip => {
            b.tone(0.0, 0.25, 260.0, 1100.0, Wave::Tri, 0.3, 0.005, 0.6);
            b.hiss(0.0, 0.25, 0.25, 0.25, 0.08, 1.0);
        }
        Sfx::Oof => {
            b.tone(0.0, 0.35, 240.0, 70.0, Wave::Square(0.5), 0.2, 0.002, 0.8);
            b.tone(0.0, 0.2, 140.0, 50.0, Wave::Tri, 0.5, 0.001, 1.0);
            b.hiss(0.0, 0.12, 0.6, 0.5, 0.001, 2.0);
            // little birds
            for k in 0..3 {
                b.tone(0.25 + k as f32 * 0.09, 0.05, 2400.0, 2800.0, Wave::Square(0.25), 0.04, 0.002, 0.5);
            }
        }
        Sfx::Splash => {
            b.hiss(0.0, 0.7, 0.7, 0.5, 0.005, 2.2);
            b.hiss(0.0, 0.25, 0.4, 0.15, 0.002, 1.0);
            for k in 0..5 {
                let st = 0.12 + k as f32 * 0.09;
                b.tone(st, 0.05, 500.0 + k as f32 * 90.0, 900.0 + k as f32 * 120.0, Wave::Sine, 0.12, 0.002, 1.0);
            }
        }
        Sfx::Squawk => {
            vib(&mut b, 0.0, 0.16, 1000.0, 1350.0, 0.08, 28.0, Wave::Square(0.3), 0.12);
            vib(&mut b, 0.18, 0.28, 1300.0, 750.0, 0.1, 24.0, Wave::Square(0.3), 0.12);
            b.hiss(0.0, 0.4, 0.15, 0.8, 0.01, 1.0);
        }
        Sfx::Chitter => {
            for k in 0..7 {
                let f0 = 1800.0 + (k % 3) as f32 * 260.0;
                b.tone(k as f32 * 0.035, 0.022, f0, f0 * 1.2, Wave::Square(0.25), 0.18, 0.001, 0.8);
            }
        }
        Sfx::Gust => {
            let n = (0.95 * SR) as usize;
            let mut y = 0.0;
            for k in 0..n {
                let t = k as f32 / SR;
                let u = k as f32 / n as f32;
                let r = b.noise();
                let lp = 0.03 + 0.06 * (u * std::f32::consts::PI).sin();
                y += (r - y) * lp;
                let env = (u * std::f32::consts::PI).sin().powf(1.5);
                b.s[k] += y * 2.2 * env * (1.0 + 0.3 * (t * 9.0).sin());
            }
        }
        Sfx::Tick => b.tone(0.0, 0.018, 1600.0, 1500.0, Wave::Square(0.5), 0.08, 0.001, 1.0),
        Sfx::Ding => {
            b.tone(0.0, 0.33, 1760.0, 1760.0, Wave::Sine, 0.25, 0.001, 1.5);
            b.tone(0.0, 0.25, 2637.0, 2637.0, Wave::Sine, 0.12, 0.001, 2.0);
            b.tone(0.0, 0.06, 3520.0, 3520.0, Wave::Square(0.25), 0.05, 0.001, 1.0);
        }
        Sfx::Whistle => {
            // the owner's two-finger whistle: wheet-whoo
            vib(&mut b, 0.0, 0.18, 1400.0, 2600.0, 0.004, 30.0, Wave::Sine, 0.22);
            vib(&mut b, 0.2, 0.3, 2500.0, 1600.0, 0.006, 30.0, Wave::Sine, 0.22);
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
        Sfx::Blip => b.tone(0.0, 0.035, 1175.0, 1175.0, Wave::Square(0.25), 0.15, 0.001, 0.5),
        Sfx::Confirm => {
            for (k, n) in [72.0, 79.0, 84.0].iter().enumerate() {
                b.tone(k as f32 * 0.05, 0.08, midi_hz(*n), midi_hz(*n), Wave::Square(0.25), 0.17, 0.001, 0.6);
            }
        }
        Sfx::WahWah => {
            // the sad trombone
            for (k, n) in [62.0f32, 61.0, 60.0].iter().enumerate() {
                vib(&mut b, k as f32 * 0.2, 0.19, midi_hz(*n), midi_hz(*n), 0.006, 6.0, Wave::Square(0.4), 0.2);
            }
            vib(&mut b, 0.6, 0.38, midi_hz(59.0), midi_hz(57.5), 0.03, 7.0, Wave::Square(0.4), 0.2);
        }
    }
    b.s
}

// ---------------------------------------------------------------- music

const C: [i32; 3] = [48, 52, 55];
const AM: [i32; 3] = [45, 48, 52];
const F: [i32; 3] = [53, 57, 60];
const G: [i32; 3] = [55, 59, 62];
const D: [i32; 3] = [50, 54, 57];
const EM: [i32; 3] = [52, 55, 59];
const DM: [i32; 3] = [50, 53, 57];
const BB: [i32; 3] = [46, 50, 53];

pub const TITLE: Song = Song {
    bpm: 138.0,
    chords: &[C, AM, F, G, C, AM, F, G],
    melody: &[
        "C5 - E5 G5 - E5 C6 - B5 - G5 - E5 - G5 -",
        "A5 - - E5 - C5 A4 - B4 - C5 - E5 - - -",
        "F5 - A5 C6 - A5 F5 - G5 - A5 - C6 - A5 -",
        "G5 - - - D5 - G5 - B5 - - - . . . .",
        "C5 - E5 G5 - E5 C6 - D6 - C6 - B5 - G5 -",
        "A5 - C6 - A5 - E5 - G5 - - - E5 - C5 -",
        "F5 - - A5 - - C6 - D6 - C6 - A5 - F5 -",
        "G5 - B5 - D6 - - - C6 - - - . . . .",
    ],
    bass: [0, R, 12, R, 7, R, 12, R, 0, R, 12, R, 7, R, 12, 7],
    drums: "k.hhs.hkk.hhs.hh",
    lead_duty: 0.25,
};

pub const PLAY: Song = Song {
    bpm: 152.0,
    chords: &[G, D, EM, C, G, D, C, D],
    melody: &[
        "D5 - G5 - B5 - G5 - D6 - B5 - G5 - A5 -",
        "F#5 - A5 - D6 - - - C6 - A5 - F#5 - D5 -",
        "E5 - G5 - B5 - - E6 D6 - B5 - G5 - - -",
        "E5 - G5 - C6 - E6 - D6 - C6 - B5 - A5 -",
        "B5 - - B5 A5 - G5 - D5 - G5 - B5 - D6 -",
        "C6 - - - A5 - F#5 - A5 - D6 - - - . .",
        "E6 - D6 - C6 - B5 - A5 - G5 - E5 - C5 -",
        "D5 - F#5 - A5 - C6 - D6 - - - . . . .",
    ],
    bass: [0, R, 12, R, 0, 0, 12, R, 0, R, 12, R, 7, R, 12, R],
    drums: "k.h.s.hkk.h.s.hh",
    lead_duty: 0.5,
};

pub const RESULTS: Song = Song {
    bpm: 112.0,
    chords: &[F, DM, BB, C],
    melody: &[
        "A5 - - C6 - - A5 - G5 - F5 - - - . .",
        "F5 - - A5 - - D6 - C6 - A5 - - - . .",
        "D6 - - - C6 - A#5 - A5 - - - F5 - G5 -",
        "E5 - - - G5 - - - C6 - - - . . . .",
    ],
    bass: [0, R, R, 7, R, R, 12, R, 0, R, R, 7, R, 10, 12, R],
    drums: "k...s..hk.h.s..h",
    lead_duty: 0.125,
};

pub const CHAMP: Song = Song {
    bpm: 120.0,
    chords: &[C, F, G, C],
    melody: &[
        "G4 - C5 - E5 - G5 - - - E5 - G5 - - -",
        "A5 - - - C6 - - - F6 - - - E6 - D6 -",
        "D6 - - - B5 - G5 - D6 - - - E6 - F6 -",
        "E6 - - - - - - - C6 - - - - - - -",
    ],
    bass: [0, R, R, 0, R, R, 0, R, 7, R, R, 7, R, R, 12, R],
    drums: "k...s..kk...s.hh",
    lead_duty: 0.25,
};

pub fn song(t: Track) -> &'static Song {
    match t {
        Track::Title => &TITLE,
        Track::Play => &PLAY,
        Track::Results => &RESULTS,
        Track::Champion => &CHAMP,
    }
}

pub fn music_samples(t: Track) -> Vec<f32> {
    render_song(song(t))
}
