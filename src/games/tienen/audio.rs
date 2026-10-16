//! TIENEN! sound effects and songs, made with the engine synth: a bouncy
//! schoolyard title tune, a fast street-football play song, a comic march
//! for the billenbrand and a victory fanfare.

use super::world::Sfx;
use crate::engine::synth::*;

pub const ALL_SFX: [Sfx; 16] = [
    Sfx::Blip,
    Sfx::Confirm,
    Sfx::Kick,
    Sfx::Header,
    Sfx::Bounce,
    Sfx::Post,
    Sfx::Net,
    Sfx::Catch,
    Sfx::Punch,
    Sfx::Whistle,
    Sfx::Cheer,
    Sfx::Groan,
    Sfx::Honk,
    Sfx::Bark,
    Sfx::Ouch,
    Sfx::Whiff,
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Track {
    Title,
    Play,
    Brand,
    Fanfare,
}

pub const ALL_TRACKS: [Track; 4] = [Track::Title, Track::Play, Track::Brand, Track::Fanfare];

/// A struck piece of metal: a few inharmonic partials with a long decay.
fn strike(b: &mut Buf, t: f32, f0: f32, dur: f32, vol: f32) {
    for (k, (m, v)) in [(1.0f32, 1.0f32), (2.76, 0.45), (5.4, 0.25), (1.5, 0.3)].iter().enumerate() {
        let w = if k == 0 { Wave::Sine } else { Wave::Tri };
        b.tone(t, dur * if k == 0 { 1.0 } else { 0.6 }, f0 * m, f0 * m, w, vol * v, 0.001, 1.4);
    }
}

/// A shouted vowel with a pitch glide (a kid's voice).
fn shout(b: &mut Buf, st: f32, du: f32, f0: f32, f1: f32, formants: (f32, f32), vol: f32) {
    let i0 = (st * SR) as usize;
    let n = (du * SR) as usize;
    let (mut p1, mut p2, mut p3) = (0.0f32, 0.0f32, 0.0f32);
    for k in 0..n {
        let u = k as f32 / n as f32;
        let f = f0 + (f1 - f0) * u;
        p1 += f / SR;
        p2 += formants.0 / SR;
        p3 += formants.1 / SR;
        let pulse = osc(Wave::Square(0.15), p1);
        let env = (u / 0.08).min(1.0) * (1.0 - u).powf(0.7);
        if let Some(x) = b.s.get_mut(i0 + k) {
            *x += (pulse * 0.14 + osc(Wave::Sine, p2) * pulse.abs() * 0.12 + osc(Wave::Sine, p3) * pulse.abs() * 0.05) * env * vol;
        }
    }
}

pub fn sfx_samples(s: Sfx) -> Vec<f32> {
    let mut b = Buf::new(match s {
        Sfx::Cheer => 1.5,
        Sfx::Groan => 0.9,
        Sfx::Whistle | Sfx::Honk => 0.8,
        Sfx::Net | Sfx::Ouch => 0.6,
        Sfx::Blip => 0.06,
        _ => 0.4,
    });
    match s {
        Sfx::Blip => b.tone(0.0, 0.035, 1046.0, 1046.0, Wave::Square(0.25), 0.15, 0.001, 0.5),
        Sfx::Confirm => {
            for (k, n) in [76.0, 81.0, 84.0].iter().enumerate() {
                b.tone(k as f32 * 0.05, 0.08, midi_hz(*n), midi_hz(*n), Wave::Square(0.25), 0.17, 0.001, 0.6);
            }
        }
        Sfx::Kick => {
            // TOK: a boot on a ball
            b.tone(0.0, 0.09, 190.0, 60.0, Wave::Sine, 0.75, 0.001, 1.2);
            b.hiss(0.0, 0.025, 0.5, 0.85, 0.0005, 2.0);
            b.tone(0.0, 0.02, 900.0, 500.0, Wave::Square(0.4), 0.08, 0.0005, 1.5);
        }
        Sfx::Header => {
            // BONK: hollow, a bit higher
            b.tone(0.0, 0.12, 330.0, 150.0, Wave::Tri, 0.5, 0.001, 1.0);
            b.tone(0.0, 0.06, 660.0, 440.0, Wave::Sine, 0.2, 0.001, 1.5);
            b.hiss(0.0, 0.02, 0.3, 0.7, 0.0005, 2.0);
        }
        Sfx::Bounce => {
            b.tone(0.0, 0.07, 140.0, 70.0, Wave::Sine, 0.45, 0.001, 1.2);
            b.hiss(0.0, 0.02, 0.2, 0.5, 0.0005, 2.0);
        }
        Sfx::Post => {
            // PING: the iron goal rings
            b.tone(0.0, 0.04, 200.0, 80.0, Wave::Sine, 0.4, 0.001, 1.0);
            strike(&mut b, 0.0, 880.0, 0.38, 0.22);
            strike(&mut b, 0.005, 1320.0, 0.3, 0.1);
        }
        Sfx::Net => {
            // the ball in the net: a swish
            let n = (0.45 * SR) as usize;
            let mut y = 0.0;
            for k in 0..n {
                let u = k as f32 / n as f32;
                let r = b.noise();
                y += (r - y) * (0.25 - 0.18 * u);
                b.s[k] += y * 0.9 * (1.0 - u).powf(1.5) * (u / 0.03).min(1.0);
            }
            b.tone(0.0, 0.08, 120.0, 60.0, Wave::Sine, 0.3, 0.001, 1.0);
        }
        Sfx::Catch => {
            // KLETS: the ball in two hands
            b.hiss(0.0, 0.05, 0.7, 0.75, 0.0005, 2.5);
            b.tone(0.0, 0.08, 160.0, 90.0, Wave::Sine, 0.5, 0.001, 1.0);
        }
        Sfx::Punch => {
            b.tone(0.0, 0.1, 220.0, 70.0, Wave::Sine, 0.7, 0.001, 1.0);
            b.hiss(0.0, 0.04, 0.6, 0.9, 0.0005, 2.0);
            b.tone(0.03, 0.08, 500.0, 900.0, Wave::Tri, 0.12, 0.002, 1.0);
        }
        Sfx::Whistle => {
            // a gym teacher's whistle: a trill, then a long one
            for (st, du) in [(0.0f32, 0.1f32), (0.15, 0.5)] {
                let i0 = (st * SR) as usize;
                let n = (du * SR) as usize;
                let mut ph = 0.0f32;
                for k in 0..n {
                    let t = k as f32 / SR;
                    let f = 2700.0 + if (t * 32.0 * std::f32::consts::TAU).sin() > 0.0 { 170.0 } else { 0.0 };
                    ph += f / SR;
                    let env = (t / 0.01).min(1.0) * if k > n - 400 { (n - k) as f32 / 400.0 } else { 1.0 };
                    b.s[i0 + k] += osc(Wave::Square(0.5), ph) * 0.1 * env;
                }
            }
        }
        Sfx::Cheer => {
            // kids yelling: high noise and a few "JAAA"s
            let n = (1.4 * SR) as usize;
            let mut y = 0.0;
            let mut y2 = 0.0;
            for k in 0..n {
                let t = k as f32 / SR;
                let u = k as f32 / n as f32;
                let r = b.noise();
                let lp = 0.2 + 0.1 * (t * 17.0).sin() * (t * 9.0).sin();
                y += (r - y) * lp;
                y2 += (y - y2) * 0.5;
                let env = (t / 0.1).min(1.0) * (1.0 - u).powf(1.6);
                b.s[k] += (y - y2 * 0.5) * 0.6 * env;
            }
            shout(&mut b, 0.0, 0.5, 520.0, 600.0, (900.0, 1500.0), 0.9);
            shout(&mut b, 0.08, 0.45, 450.0, 520.0, (800.0, 1300.0), 0.7);
            shout(&mut b, 0.5, 0.5, 560.0, 480.0, (850.0, 1400.0), 0.6);
        }
        Sfx::Groan => {
            // OOOOOH: everybody knows who goes in goal
            shout(&mut b, 0.0, 0.85, 330.0, 220.0, (450.0, 800.0), 1.0);
            shout(&mut b, 0.03, 0.8, 280.0, 190.0, (420.0, 760.0), 0.8);
        }
        Sfx::Honk => {
            // TOET TOET: a small car's horn, two notes at once
            for st in [0.0f32, 0.28] {
                for f in [392.0f32, 494.0] {
                    b.tone(st, 0.22, f, f, Wave::Square(0.5), 0.09, 0.005, 1.0);
                }
            }
        }
        Sfx::Bark => {
            // KEF KEF: a poodle
            for st in [0.0f32, 0.16] {
                shout(&mut b, st, 0.1, 900.0, 650.0, (1300.0, 2400.0), 1.0);
                b.hiss(st, 0.03, 0.15, 0.6, 0.0005, 2.0);
            }
        }
        Sfx::Ouch => {
            // a slap, then AUUU
            b.hiss(0.0, 0.04, 0.9, 0.8, 0.0005, 2.5);
            b.tone(0.0, 0.06, 200.0, 100.0, Wave::Sine, 0.6, 0.001, 1.0);
            shout(&mut b, 0.06, 0.5, 520.0, 700.0, (700.0, 1100.0), 1.0);
        }
        Sfx::Whiff => {
            // a leg through the air
            let n = (0.22 * SR) as usize;
            let mut y = 0.0;
            for k in 0..n {
                let u = k as f32 / n as f32;
                let r = b.noise();
                y += (r - y) * (0.05 + 0.25 * (u * std::f32::consts::PI).sin());
                b.s[k] += y * 0.9 * (u * std::f32::consts::PI).sin();
            }
        }
    }
    b.s
}

// ---------------------------------------------------------------- music

const F: [i32; 3] = [53, 57, 60];
const BB: [i32; 3] = [46, 50, 53];
const C: [i32; 3] = [48, 52, 55];
const DM: [i32; 3] = [50, 53, 57];
const GM: [i32; 3] = [55, 58, 62];
const G: [i32; 3] = [55, 59, 62];
const D: [i32; 3] = [50, 54, 57];
const EM: [i32; 3] = [52, 55, 59];
const CC: [i32; 3] = [48, 52, 55];
const A7: [i32; 3] = [57, 61, 64];
const EB: [i32; 3] = [51, 55, 58];

/// Bouncy and cheeky, like a skipping rope rhyme.
const TITLE: Song = Song {
    bpm: 132.0,
    chords: &[F, F, BB, C, F, DM, GM, C],
    melody: &[
        "C5 . F5 . A5 . F5 . C6 - A5 . F5 . . .",
        "G5 . A5 . A#5 . A5 . G5 - F5 - . . . .",
        "D5 . F5 . A#5 . D6 . C6 - A#5 . A5 . G5 .",
        "E5 . G5 . C6 . G5 . E5 - D5 - C5 . . .",
        "C5 . F5 . A5 . F5 . C6 - D6 . C6 . A5 .",
        "D6 - - . C6 . A5 . F5 . A5 . D6 . . .",
        "A#5 . A5 . G5 . A#5 . D6 - C6 . A#5 . G5 .",
        "E5 . G5 . C6 . E6 . F6 - - - . . . .",
    ],
    bass: [0, R, 7, R, 12, R, 7, R, 0, R, 7, R, 12, 7, 5, R],
    drums: "k.h.s.h.k.k.s.hh",
    lead_duty: 0.25,
};

/// Running after the ball, all afternoon.
const PLAY: Song = Song {
    bpm: 150.0,
    chords: &[G, G, CC, D, EM, CC, D, G],
    melody: &[
        "G5 . B5 . D6 . B5 . G5 . D5 . G5 . B5 .",
        "A5 . G5 . F#5 . G5 . A5 - - . . . . .",
        "E5 . G5 . C6 . G5 . E5 . C5 . E5 . G5 .",
        "F#5 . A5 . D6 . C6 . B5 . A5 . F#5 . D5 .",
        "E5 . G5 . B5 . E6 . D6 . B5 . G5 . E5 .",
        "C6 - . C6 B5 . A5 . G5 . E5 . C5 . . .",
        "D5 . F#5 . A5 . D6 . C6 . A5 . F#5 . A5 .",
        "G5 - - . D5 . G5 . B5 - - - . . . .",
    ],
    bass: [0, R, 12, 0, R, 12, 0, R, 0, R, 12, 0, 7, R, 5, R],
    drums: "k.hsk.hsk.hsk.sh",
    lead_duty: 0.125,
};

/// The billenbrand: a slow, stupid march.
const BRAND: Song = Song {
    bpm: 96.0,
    chords: &[BB, BB, EB, F, BB, EB, F, BB],
    melody: &[
        "A#4 - . A#4 D5 - . D5 F5 - - - D5 - . .",
        "A#4 - . A#4 D5 - . D5 F5 - D5 - A#4 - . .",
        "G5 - . G5 G5 - F5 - D#5 - - - C5 - . .",
        "A5 - . A5 G5 - F5 - C5 - - - . . . .",
        "A#4 - . A#4 D5 - . D5 F5 - - - A#5 - . .",
        "G5 - - - D#5 - - - C5 - D#5 - G5 - . .",
        "F5 - . F5 G5 - A5 - C6 - - - A5 - . .",
        "A#5 - . . F5 . . . A#4 - - - . . . .",
    ],
    bass: [0, R, R, R, 7, R, R, R, 0, R, R, R, 7, R, R, R],
    drums: "k...s...k...s.s.",
    lead_duty: 0.5,
};

const FANFARE: Song = Song {
    bpm: 124.0,
    chords: &[CC, F, A7, DM, G, CC],
    melody: &[
        "C5 - E5 - G5 - C6 - - - G5 - C6 - E6 -",
        "F6 - - - C6 - A5 - F5 - - - A5 - C6 -",
        "C#6 - - - E6 - - - A6 - - - G6 - E6 -",
        "F6 - - - D6 - A5 - F5 - A5 - D6 - F6 -",
        "G6 - - - F6 - D6 - B5 - G5 - B5 - D6 -",
        "C6 - - - G5 - E5 - C6 - - - - - - -",
    ],
    bass: [0, R, R, 0, R, R, 0, R, 7, R, R, 7, R, R, 12, R],
    drums: "k...s..kk...s.hh",
    lead_duty: 0.5,
};

pub fn music_samples(t: Track) -> Vec<f32> {
    render_song(match t {
        Track::Title => &TITLE,
        Track::Play => &PLAY,
        Track::Brand => &BRAND,
        Track::Fanfare => &FANFARE,
    })
}
