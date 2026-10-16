//! STALEN ZENUWEN! sound effects and songs, made with the engine synth: a
//! sneaky title tune, a tiptoe play song, a chase and a fanfare.

use super::world::Sfx;
use crate::engine::synth::*;

pub const ALL_SFX: [Sfx; 16] = [
    Sfx::Blip,
    Sfx::Confirm,
    Sfx::Click,
    Sfx::Saw,
    Sfx::Clank,
    Sfx::Krak,
    Sfx::Unlock,
    Sfx::Whistle,
    Sfx::Hey,
    Sfx::Toss,
    Sfx::Cash,
    Sfx::Splash,
    Sfx::Bell,
    Sfx::Busted,
    Sfx::Squawk,
    Sfx::Cheer,
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Track {
    Title,
    Sneak,
    Chase,
    Fanfare,
}

pub const ALL_TRACKS: [Track; 4] = [Track::Title, Track::Sneak, Track::Chase, Track::Fanfare];

/// A struck piece of metal: a few inharmonic partials with a long decay.
fn strike(b: &mut Buf, t: f32, f0: f32, dur: f32, vol: f32) {
    for (k, (m, v)) in [(1.0f32, 1.0f32), (2.76, 0.45), (5.4, 0.25), (1.5, 0.3)].iter().enumerate() {
        let w = if k == 0 { Wave::Sine } else { Wave::Tri };
        b.tone(t, dur * if k == 0 { 1.0 } else { 0.6 }, f0 * m, f0 * m, w, vol * v, 0.001, 1.4);
    }
}

pub fn sfx_samples(s: Sfx) -> Vec<f32> {
    let mut b = Buf::new(match s {
        Sfx::Cheer => 1.6,
        Sfx::Busted => 1.3,
        Sfx::Splash | Sfx::Whistle | Sfx::Cash | Sfx::Krak => 0.8,
        Sfx::Click => 0.06,
        _ => 0.45,
    });
    match s {
        Sfx::Blip => b.tone(0.0, 0.035, 1046.0, 1046.0, Wave::Square(0.25), 0.15, 0.001, 0.5),
        Sfx::Confirm => {
            for (k, n) in [79.0, 84.0, 88.0].iter().enumerate() {
                b.tone(k as f32 * 0.05, 0.08, midi_hz(*n), midi_hz(*n), Wave::Square(0.25), 0.17, 0.001, 0.6);
            }
        }
        Sfx::Click => {
            // a pick in a small lock
            b.tone(0.0, 0.02, 3200.0, 2600.0, Wave::Square(0.3), 0.08, 0.0005, 2.0);
            b.hiss(0.0, 0.015, 0.15, 0.9, 0.0005, 2.0);
        }
        Sfx::Saw => {
            // a hacksaw on a chain: a rasp that rises
            let n = (0.18 * SR) as usize;
            let mut y = 0.0;
            for k in 0..n {
                let u = k as f32 / n as f32;
                let r = b.noise();
                y += (r - y) * (0.3 + 0.4 * u);
                let buzz = if (k / (40 - (u * 20.0) as usize).max(8)).is_multiple_of(2) { 1.0 } else { 0.4 };
                b.s[k] += y * 0.5 * buzz * (u * std::f32::consts::PI).sin();
            }
        }
        Sfx::Clank => {
            // KLENG: steel on steel
            strike(&mut b, 0.0, 640.0, 0.35, 0.22);
            strike(&mut b, 0.01, 1530.0, 0.25, 0.14);
            b.hiss(0.0, 0.05, 0.4, 0.8, 0.0005, 2.0);
        }
        Sfx::Krak => {
            // the bolt cutter: a thump, a crack and the pieces ringing
            b.tone(0.0, 0.15, 160.0, 50.0, Wave::Sine, 0.7, 0.001, 1.0);
            b.hiss(0.0, 0.08, 0.9, 0.95, 0.0005, 2.5);
            strike(&mut b, 0.03, 2100.0, 0.5, 0.12);
            strike(&mut b, 0.12, 1700.0, 0.4, 0.08);
            strike(&mut b, 0.2, 2600.0, 0.3, 0.06);
        }
        Sfx::Unlock => {
            // ka-klik, and a little chime: free!
            b.tone(0.0, 0.03, 900.0, 500.0, Wave::Square(0.4), 0.15, 0.0005, 1.5);
            b.tone(0.05, 0.04, 1300.0, 700.0, Wave::Square(0.4), 0.15, 0.0005, 1.5);
            for (k, n) in [84.0, 88.0, 91.0, 96.0].iter().enumerate() {
                b.tone(0.1 + k as f32 * 0.05, 0.12, midi_hz(*n), midi_hz(*n), Wave::Square(0.125), 0.1, 0.001, 0.8);
            }
        }
        Sfx::Whistle => {
            // the BOA's whistle: a trill
            for (st, du) in [(0.0f32, 0.12f32), (0.17, 0.45)] {
                let i0 = (st * SR) as usize;
                let n = (du * SR) as usize;
                let mut ph = 0.0f32;
                for k in 0..n {
                    let t = k as f32 / SR;
                    let f = 2900.0 + if (t * 30.0 * std::f32::consts::TAU).sin() > 0.0 { 190.0 } else { 0.0 };
                    ph += f / SR;
                    let env = (t / 0.01).min(1.0) * if k > n - 400 { (n - k) as f32 / 400.0 } else { 1.0 };
                    b.s[i0 + k] += osc(Wave::Square(0.5), ph) * 0.1 * env;
                }
            }
        }
        Sfx::Hey => {
            // HE!: a shouted vowel, falling
            let n = (0.22 * SR) as usize;
            let (mut p1, mut p2, mut p3) = (0.0f32, 0.0f32, 0.0f32);
            for k in 0..n {
                let u = k as f32 / n as f32;
                let f0 = 260.0 - 80.0 * u;
                p1 += f0 / SR;
                p2 += 530.0 / SR;
                p3 += 1850.0 / SR;
                let pulse = osc(Wave::Square(0.15), p1);
                let env = (u / 0.08).min(1.0) * (1.0 - u).powf(0.8);
                b.s[k] += (pulse * 0.16 + osc(Wave::Sine, p2) * pulse.abs() * 0.12 + osc(Wave::Sine, p3) * pulse.abs() * 0.05) * env;
            }
        }
        Sfx::Toss => {
            // whoosh, up and over
            let n = (0.4 * SR) as usize;
            let mut y = 0.0;
            for k in 0..n {
                let u = k as f32 / n as f32;
                let r = b.noise();
                y += (r - y) * (0.04 + 0.2 * (u * std::f32::consts::PI).sin());
                b.s[k] += y * 1.2 * (u * std::f32::consts::PI).sin();
            }
            b.tone(0.3, 0.1, 140.0, 70.0, Wave::Tri, 0.3, 0.001, 1.0);
        }
        Sfx::Cash => {
            // ka-ching
            b.hiss(0.0, 0.04, 0.3, 0.7, 0.0005, 2.0);
            b.tone(0.0, 0.05, 300.0, 200.0, Wave::Square(0.5), 0.15, 0.001, 1.0);
            strike(&mut b, 0.06, 2637.0, 0.6, 0.1);
            strike(&mut b, 0.1, 3520.0, 0.55, 0.09);
            for k in 0..5 {
                strike(&mut b, 0.15 + k as f32 * 0.04, 4200.0 + k as f32 * 300.0, 0.12, 0.03);
            }
        }
        Sfx::Splash => {
            b.tone(0.0, 0.18, 320.0, 70.0, Wave::Tri, 0.55, 0.001, 1.0);
            b.hiss(0.0, 0.5, 0.6, 0.45, 0.002, 1.6);
            for (k, t) in [0.25f32, 0.33, 0.4, 0.5, 0.58].iter().enumerate() {
                let f0 = 500.0 + k as f32 * 90.0;
                b.tone(*t, 0.05, f0, f0 * 1.8, Wave::Sine, 0.12, 0.002, 1.0);
            }
        }
        Sfx::Bell => {
            for st in [0.0, 0.13] {
                strike(&mut b, st, 2350.0, 0.22, 0.16);
                b.tone(st, 0.03, 4200.0, 4000.0, Wave::Square(0.25), 0.04, 0.0005, 1.0);
            }
        }
        Sfx::Busted => {
            // a stamp on the fine, then the sad trombone
            b.tone(0.0, 0.1, 120.0, 40.0, Wave::Sine, 0.7, 0.001, 1.0);
            b.hiss(0.0, 0.05, 0.5, 0.5, 0.0005, 2.0);
            for (k, (n0, n1)) in [(62.0f32, 61.0f32), (61.0, 60.0), (60.0, 59.0), (59.0, 55.0)].iter().enumerate() {
                let st = 0.2 + k as f32 * 0.24;
                let du = if k == 3 { 0.5 } else { 0.22 };
                let i0 = (st * SR) as usize;
                let n = (du * SR) as usize;
                let mut ph = 0.0f32;
                for j in 0..n {
                    let u = j as f32 / n as f32;
                    let f = midi_hz(n0 + (n1 - n0) * u) * (1.0 + 0.01 * (j as f32 / SR * 30.0).sin());
                    ph += f / SR;
                    let env = (j as f32 / SR / 0.02).min(1.0) * (1.0 - u).powf(0.5);
                    if let Some(x) = b.s.get_mut(i0 + j) {
                        *x += osc(Wave::Square(0.3), ph) * 0.12 * env;
                    }
                }
            }
        }
        Sfx::Squawk => {
            // FRAAANK: the heron's harsh croak
            let n = (0.35 * SR) as usize;
            let mut ph = 0.0f32;
            for k in 0..n {
                let u = k as f32 / n as f32;
                let f = 420.0 - 160.0 * u + 30.0 * (k as f32 / SR * 60.0).sin();
                ph += f / SR;
                let env = (u / 0.05).min(1.0) * (1.0 - u).powf(0.6);
                let r = b.noise();
                b.s[k] += (osc(Wave::Square(0.2), ph) * 0.2 + r * 0.06) * env;
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
            b.tone(0.3, 0.35, 233.0, 233.0, Wave::Square(0.4), 0.05, 0.03, 0.4);
            b.tone(0.75, 0.3, 262.0, 262.0, Wave::Square(0.4), 0.05, 0.03, 0.4);
        }
    }
    b.s
}

// ---------------------------------------------------------------- music

const DM: [i32; 3] = [50, 53, 57];
const GM: [i32; 3] = [55, 58, 62];
const A: [i32; 3] = [57, 61, 64];
const BB: [i32; 3] = [46, 50, 53];
const AM: [i32; 3] = [57, 60, 64];
const E: [i32; 3] = [52, 56, 59];
const F: [i32; 3] = [53, 57, 60];
const EM: [i32; 3] = [52, 55, 59];
const CC: [i32; 3] = [48, 52, 55];
const D: [i32; 3] = [50, 54, 57];
const B: [i32; 3] = [47, 51, 54];
const G: [i32; 3] = [55, 59, 62];
const FL: [i32; 3] = [41, 45, 48];

/// Sneaky, chromatic, on tiptoes.
const TITLE: Song = Song {
    bpm: 108.0,
    chords: &[DM, DM, GM, A, DM, BB, A, A],
    melody: &[
        "D5 . . F5 . . G#5 A5 - - . . . . . .",
        "D6 . C6 . A5 . F5 . G#5 - A5 - . . . .",
        "G5 . . A#5 . . C#6 D6 - - . . . . . .",
        "E6 . D6 . C#6 . A5 . E5 - - - . . . .",
        "D5 . . F5 . . G#5 A5 - - . . F5 . A5 .",
        "D6 - . . C6 . A#5 . A5 . G5 . F5 . . .",
        "E5 . G5 . A#5 . C#6 . E6 - - - D6 . C#6 .",
        "D6 - - - . . A5 . D5 - - - . . . .",
    ],
    bass: [0, R, R, 3, 7, R, R, 5, 0, R, R, 3, 7, R, 6, R],
    drums: "k..h.hk.s..h.hs.",
    lead_duty: 0.25,
};

/// Walking through the street, trying to look normal.
const SNEAK: Song = Song {
    bpm: 124.0,
    chords: &[AM, AM, DM, E, AM, F, E, AM],
    melody: &[
        "A4 . C5 . E5 . . . D#5 . E5 . . . . .",
        "A5 . . . G5 . E5 . C5 . . . . . . .",
        "D5 . F5 . A5 . . . G#5 . A5 . . . . .",
        "B5 . . . G#5 . E5 . B4 . . . . . . .",
        "A4 . C5 . E5 . A5 . G5 . E5 . C5 . . .",
        "F5 . A5 . C6 . . . B5 . A5 . F5 . . .",
        "E5 . G#5 . B5 . E6 . D6 . B5 . G#5 . . .",
        "A5 - . . E5 . . . A4 - . . . . . .",
    ],
    bass: [0, R, R, R, 7, R, R, R, 0, R, R, R, 7, R, 5, R],
    drums: "k.h.s.h.k.h.s.hh",
    lead_duty: 0.125,
};

/// RENNEN!
const CHASE: Song = Song {
    bpm: 168.0,
    chords: &[EM, EM, CC, D, EM, EM, CC, B],
    melody: &[
        "E5 E5 . E5 G5 . E5 . B5 - A5 - G5 - F#5 -",
        "E5 - . . B4 . E5 . G5 . F#5 . E5 . D5 .",
        "C5 C5 . C5 E5 . G5 . C6 - B5 - A5 - G5 -",
        "F#5 - . . A5 . F#5 . D5 . E5 . F#5 . A5 .",
        "E5 E5 . E5 G5 . E5 . B5 - A5 - G5 - B5 -",
        "E6 - D6 - B5 - G5 - E5 - G5 - B5 - D6 -",
        "C6 - . C6 B5 - . B5 A5 - . A5 G5 - F#5 -",
        "F#5 - - - B5 - - - D#5 - - - F#5 - - -",
    ],
    bass: [0, 0, 12, 0, 0, 12, 0, 12, 0, 0, 12, 0, 0, 12, 7, 12],
    drums: "k.hsk.hsk.hsk.ss",
    lead_duty: 0.5,
};

const FANFARE: Song = Song {
    bpm: 120.0,
    chords: &[CC, FL, G, CC],
    melody: &[
        "C5 - E5 - G5 - C6 - - - G5 - C6 - E6 -",
        "F6 - - - E6 - D6 - C6 - - - A5 - - -",
        "B5 - - - D6 - - - G6 - - - F6 - D6 -",
        "E6 - - - G6 - - - C6 - - - - - - -",
    ],
    bass: [0, R, R, 0, R, R, 0, R, 7, R, R, 7, R, R, 12, R],
    drums: "k...s..kk...s.hh",
    lead_duty: 0.5,
};

pub fn music_samples(t: Track) -> Vec<f32> {
    render_song(match t {
        Track::Title => &TITLE,
        Track::Sneak => &SNEAK,
        Track::Chase => &CHASE,
        Track::Fanfare => &FANFARE,
    })
}
