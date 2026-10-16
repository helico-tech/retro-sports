//! TEGENWIND! sound effects and songs, made with the engine synth: a
//! street-organ title tune, a polka to race to, a storm shanty for the
//! dike and a fanfare.

use super::race::Sfx;
use crate::engine::synth::*;

pub const ALL_SFX: [Sfx; 16] = [
    Sfx::Bell,
    Sfx::Tick,
    Sfx::Gust,
    Sfx::Splash,
    Sfx::Moo,
    Sfx::Honk,
    Sfx::Crash,
    Sfx::Hop,
    Sfx::Rattle,
    Sfx::Chomp,
    Sfx::Whistle,
    Sfx::TramBell,
    Sfx::BridgeAlarm,
    Sfx::Cheer,
    Sfx::Blip,
    Sfx::Confirm,
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Track {
    Title,
    Polka,
    Storm,
    Fanfare,
}

pub const ALL_TRACKS: [Track; 4] = [Track::Title, Track::Polka, Track::Storm, Track::Fanfare];

/// A struck bell: a few inharmonic partials with a long decay.
fn strike(b: &mut Buf, t: f32, f0: f32, dur: f32, vol: f32) {
    for (k, (m, v)) in [(1.0f32, 1.0f32), (2.76, 0.45), (5.4, 0.25), (1.5, 0.3)].iter().enumerate() {
        let w = if k == 0 { Wave::Sine } else { Wave::Tri };
        b.tone(t, dur * if k == 0 { 1.0 } else { 0.6 }, f0 * m, f0 * m, w, vol * v, 0.001, 1.4);
    }
}

pub fn sfx_samples(s: Sfx) -> Vec<f32> {
    let mut b = Buf::new(match s {
        Sfx::Cheer | Sfx::Gust => 1.6,
        Sfx::Moo => 1.0,
        Sfx::Splash | Sfx::TramBell | Sfx::Whistle | Sfx::Crash => 0.8,
        Sfx::BridgeAlarm => 0.75,
        Sfx::Tick => 0.05,
        _ => 0.4,
    });
    match s {
        Sfx::Bell => {
            // tring tring!
            for st in [0.0, 0.13] {
                strike(&mut b, st, 2350.0, 0.22, 0.16);
                b.tone(st, 0.03, 4200.0, 4000.0, Wave::Square(0.25), 0.04, 0.0005, 1.0);
            }
        }
        Sfx::Tick => {
            // the chain clicking over a sprocket
            b.hiss(0.0, 0.012, 0.18, 0.85, 0.0005, 2.0);
            b.tone(0.0, 0.018, 1500.0, 900.0, Wave::Square(0.3), 0.05, 0.0005, 2.0);
        }
        Sfx::Gust => {
            // whoooosh: dark noise swelling and falling
            let n = (1.5 * SR) as usize;
            let mut y = 0.0;
            for k in 0..n {
                let u = k as f32 / n as f32;
                let r = b.noise();
                let lp = 0.02 + 0.06 * (u * std::f32::consts::PI).sin();
                y += (r - y) * lp;
                let env = (u * std::f32::consts::PI).sin().powf(1.5);
                b.s[k] += y * 2.4 * env;
            }
        }
        Sfx::Splash => {
            // PLONS! a drop of the body, then bubbles
            b.tone(0.0, 0.18, 320.0, 70.0, Wave::Tri, 0.55, 0.001, 1.0);
            b.hiss(0.0, 0.5, 0.6, 0.45, 0.002, 1.6);
            for (k, t) in [0.25f32, 0.33, 0.4, 0.5, 0.58].iter().enumerate() {
                let f0 = 500.0 + k as f32 * 90.0;
                b.tone(*t, 0.05, f0, f0 * 1.8, Wave::Sine, 0.12, 0.002, 1.0);
            }
        }
        Sfx::Moo => {
            // BOEEE: a low tone with a slow wobble and a breathy edge
            let n = (0.9 * SR) as usize;
            let mut ph = 0.0f32;
            let mut ph2 = 0.0f32;
            for k in 0..n {
                let t = k as f32 / SR;
                let u = k as f32 / n as f32;
                let f = 128.0 + 22.0 * (u * 3.0).min(1.0) - 30.0 * u * u + 3.0 * (t * 7.0).sin();
                ph += f / SR;
                ph2 += f * 2.01 / SR;
                let env = (t / 0.12).min(1.0) * (1.0 - u).powf(0.7);
                b.s[k] += (osc(Wave::Square(0.3), ph) * 0.18 + osc(Wave::Tri, ph2) * 0.2) * env;
            }
            b.hiss(0.0, 0.9, 0.05, 0.3, 0.1, 1.0);
        }
        Sfx::Honk => {
            // GAK GAK: nasal squawks with a pitch drop
            for st in [0.0, 0.16] {
                b.tone(st, 0.12, 520.0, 380.0, Wave::Square(0.12), 0.22, 0.003, 0.6);
                b.tone(st, 0.12, 1040.0, 760.0, Wave::Square(0.3), 0.06, 0.003, 0.6);
                b.hiss(st, 0.1, 0.08, 0.5, 0.002, 1.0);
            }
        }
        Sfx::Crash => {
            // metal and spokes going everywhere
            b.hiss(0.0, 0.35, 0.7, 0.6, 0.001, 1.8);
            b.tone(0.0, 0.2, 180.0, 50.0, Wave::Tri, 0.6, 0.001, 1.0);
            strike(&mut b, 0.02, 820.0, 0.4, 0.14);
            strike(&mut b, 0.09, 1130.0, 0.3, 0.12);
            b.tone(0.15, 0.25, 2200.0, 1900.0, Wave::Square(0.5), 0.03, 0.001, 1.5);
        }
        Sfx::Hop => b.tone(0.0, 0.12, 240.0, 640.0, Wave::Square(0.5), 0.11, 0.002, 1.0),
        Sfx::Rattle => {
            // cobbles under thin tyres
            for k in 0..9 {
                let t = k as f32 * 0.035 + if k % 2 == 0 { 0.0 } else { 0.008 };
                b.hiss(t, 0.02, 0.32, 0.7, 0.0005, 2.0);
                b.tone(t, 0.02, 260.0, 180.0, Wave::Tri, 0.12, 0.0005, 2.0);
            }
        }
        Sfx::Chomp => {
            for st in [0.0, 0.12] {
                b.hiss(st, 0.07, 0.5, 0.25, 0.002, 1.4);
                b.tone(st, 0.06, 180.0, 110.0, Wave::Square(0.5), 0.15, 0.002, 1.0);
            }
            b.tone(0.24, 0.1, 700.0, 980.0, Wave::Sine, 0.12, 0.002, 1.0);
        }
        Sfx::Whistle => {
            for (st, du) in [(0.0f32, 0.12f32), (0.17, 0.4)] {
                let i0 = (st * SR) as usize;
                let n = (du * SR) as usize;
                let mut ph = 0.0f32;
                for k in 0..n {
                    let t = k as f32 / SR;
                    let f = 2600.0 + if (t * 32.0 * std::f32::consts::TAU).sin() > 0.0 { 170.0 } else { 0.0 };
                    ph += f / SR;
                    let env = (t / 0.01).min(1.0) * if k > n - 400 { (n - k) as f32 / 400.0 } else { 1.0 };
                    b.s[i0 + k] += osc(Wave::Square(0.5), ph) * 0.11 * env;
                }
            }
        }
        Sfx::TramBell => {
            // the Amsterdam tram: a deep double ding
            strike(&mut b, 0.0, 760.0, 0.6, 0.2);
            strike(&mut b, 0.2, 760.0, 0.55, 0.18);
        }
        Sfx::BridgeAlarm => {
            // bells on the bridge, ringing fast
            for k in 0..6 {
                strike(&mut b, k as f32 * 0.11, if k % 2 == 0 { 1320.0 } else { 1180.0 }, 0.16, 0.1);
            }
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
            // and a toeter (vuvuzela-ish party horn)
            b.tone(0.3, 0.35, 233.0, 233.0, Wave::Square(0.4), 0.05, 0.03, 0.4);
            b.tone(0.75, 0.3, 262.0, 262.0, Wave::Square(0.4), 0.05, 0.03, 0.4);
        }
        Sfx::Blip => b.tone(0.0, 0.035, 1046.0, 1046.0, Wave::Square(0.25), 0.15, 0.001, 0.5),
        Sfx::Confirm => {
            for (k, n) in [79.0, 84.0, 88.0].iter().enumerate() {
                b.tone(k as f32 * 0.05, 0.08, midi_hz(*n), midi_hz(*n), Wave::Square(0.25), 0.17, 0.001, 0.6);
            }
        }
    }
    b.s
}

// ---------------------------------------------------------------- music

const C: [i32; 3] = [48, 52, 55];
const F: [i32; 3] = [53, 57, 60];
const G: [i32; 3] = [55, 59, 62];
const D: [i32; 3] = [50, 54, 57];
const AM: [i32; 3] = [57, 60, 64];
const E: [i32; 3] = [52, 56, 59];
const FL: [i32; 3] = [41, 45, 48];

/// Street organ: jaunty, reedy lead, oom-pah bass.
const TITLE: Song = Song {
    bpm: 116.0,
    chords: &[C, G, C, F, C, G, C, C],
    melody: &[
        "E5 - G5 - C6 - G5 - E5 - G5 - C6 - - -",
        "D6 - B5 - G5 - B5 - D6 - - - B5 - - -",
        "C6 - E6 - D6 C6 B5 A5 G5 - - - E5 - - -",
        "F5 - A5 - C6 - A5 - F5 - A5 - C6 - D6 -",
        "E6 - - - C6 - - - G5 - A5 - G5 - E5 -",
        "D5 - G5 - B5 - D6 - F6 - E6 - D6 - B5 -",
        "C6 - E6 - G6 - E6 - C6 - G5 - E5 - G5 -",
        "C6 - - - - - - - . . . . . . . .",
    ],
    bass: [0, R, R, R, 7, R, R, R, 12, R, R, R, 7, R, R, R],
    drums: "k..hs..hk..hs..h",
    lead_duty: 0.125,
};

/// The race: a polka, oom-pah on the eighths.
const POLKA: Song = Song {
    bpm: 144.0,
    chords: &[G, G, D, D, G, C, D, G],
    melody: &[
        "B5 - D6 - B5 - G5 - B5 - D6 - G6 - - -",
        "F#6 - E6 - D6 - B5 - D6 - - - . . . .",
        "A5 - C6 - A5 - F#5 - A5 - C6 - F#6 - - -",
        "E6 - D6 - C6 - A5 - D6 - - - . . . .",
        "B5 - D6 - G6 - D6 - B5 - G5 - B5 - D6 -",
        "E6 - G6 - E6 - C6 - E6 - - - D6 - C6 -",
        "B5 - A5 - G5 - F#5 - A5 - - - F#5 - D5 -",
        "G5 - - - B5 - - - G5 - - - . . . .",
    ],
    bass: [0, R, R, R, 7, R, R, R, 0, R, R, R, 7, R, R, R],
    drums: "k.s.k.s.k.s.k.s.",
    lead_duty: 0.25,
};

/// The dike in a storm: a minor-key shanty.
const STORM: Song = Song {
    bpm: 138.0,
    chords: &[AM, AM, F, G, AM, F, E, E],
    melody: &[
        "A5 - - - E5 - A5 - C6 - B5 - A5 - E5 -",
        "A5 - - - - - G5 - A5 - B5 - C6 - - -",
        "C6 - - - A5 - C6 - F6 - E6 - D6 - C6 -",
        "B5 - - - G5 - - - D6 - - - B5 - - -",
        "E6 - - - D6 - C6 - B5 - C6 - A5 - - -",
        "F5 - A5 - C6 - F6 - E6 - - - C6 - - -",
        "B5 - - - G#5 - B5 - E6 - - - D6 - B5 -",
        "G#5 - - - B5 - - - E5 - - - . . . .",
    ],
    bass: [0, R, 0, R, 12, R, 0, R, 0, R, 0, R, 12, R, 7, R],
    drums: "k.hhs.hhk.hhs.hk",
    lead_duty: 0.5,
};

const FANFARE: Song = Song {
    bpm: 120.0,
    chords: &[C, FL, G, C],
    melody: &[
        "C5 - E5 - G5 - C6 - - - G5 - C6 - - -",
        "A5 - - - C6 - - - F6 - - - E6 - D6 -",
        "D6 - - - B5 - G5 - D6 - - - E6 - F6 -",
        "E6 - - - - - - - C6 - - - - - - -",
    ],
    bass: [0, R, R, 0, R, R, 0, R, 7, R, R, 7, R, R, 12, R],
    drums: "k...s..kk...s.hh",
    lead_duty: 0.5,
};

pub fn music_samples(t: Track) -> Vec<f32> {
    render_song(match t {
        Track::Title => &TITLE,
        Track::Polka => &POLKA,
        Track::Storm => &STORM,
        Track::Fanfare => &FANFARE,
    })
}
