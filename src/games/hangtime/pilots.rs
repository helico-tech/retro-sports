//! The four pilots: stats, bios, colors and the pattern on their sail.

use crate::engine::gfx::rgb;
use crate::engine::sprites::*;
use crate::engine::ui::OUTLINE;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sail {
    /// rays from the nose
    Sunburst,
    /// zigzag bands across the span
    Zigzag,
    /// flowers on a plain sail
    Flowers,
    /// chevrons pointing forward
    Chevron,
}

pub struct PilotDef {
    pub name: &'static str,
    pub short: &'static str,
    pub bio: [&'static str; 2],
    /// 1..=5 each; every pilot has 13 points
    pub glide: i32,
    pub climb: i32,
    pub handling: i32,
    pub nerve: i32,
    /// engine head style (see `engine::sprites::HAIR_STYLES`)
    pub style: usize,
    pub skin: (u32, u32),
    pub hair: (u32, u32),
    pub suit: (u32, u32),
    pub harness: (u32, u32),
    pub acc: (u32, u32),
    pub cap: (u32, u32),
    /// three sail colors and the pattern they make
    pub sail: [u32; 3],
    pub pattern: Sail,
    /// name and stat color on menus
    pub ui: u32,
    /// shouted when this pilot (as CPU) overtakes you
    pub heckle: &'static str,
    /// on the intro card
    pub quote: &'static str,
}

impl PilotDef {
    /// Sink rate multiplier (lower is better).
    pub fn sink_mul(&self) -> f32 {
        1.08 - 0.03 * self.glide as f32
    }
    /// Thermal and ridge lift multiplier.
    pub fn lift_mul(&self) -> f32 {
        0.92 + 0.04 * self.climb as f32
    }
    /// Frames a turn takes.
    pub fn turn_frames(&self) -> i32 {
        30 - 3 * self.handling
    }
    pub fn stall_speed(&self) -> f32 {
        0.52 - 0.012 * self.nerve as f32
    }
    /// Gravity while flaring: lower gives a wider flare window.
    pub fn flare_grav(&self) -> f32 {
        0.036 - 0.002 * self.nerve as f32
    }
    pub fn tuck_speed(&self) -> f32 {
        1.32 + 0.03 * self.nerve as f32
    }
}

const SKIN_LIGHT: (u32, u32) = (rgb(248, 200, 160), rgb(216, 144, 104));
const SKIN_TAN: (u32, u32) = (rgb(224, 160, 104), rgb(176, 112, 64));
const SKIN_DARK: (u32, u32) = (rgb(160, 96, 56), rgb(112, 64, 32));
const EAGLE_WHITE: (u32, u32) = (rgb(248, 248, 240), rgb(200, 200, 208));

pub const PILOTS: [PilotDef; 4] = [
    PilotDef {
        name: "CAPTAIN KITE",
        short: "KITE",
        bio: ["HAS NOT LANDED", "SINCE TUESDAY."],
        glide: 4,
        climb: 3,
        handling: 3,
        nerve: 3,
        style: 2,
        skin: SKIN_LIGHT,
        hair: (rgb(136, 80, 32), rgb(88, 48, 16)),
        suit: (rgb(48, 96, 224), rgb(32, 56, 152)),
        harness: (rgb(56, 64, 104), rgb(32, 40, 72)),
        acc: (rgb(248, 216, 48), rgb(192, 144, 16)),
        cap: (rgb(232, 48, 48), rgb(160, 24, 32)),
        sail: [rgb(248, 216, 48), rgb(232, 56, 48), rgb(248, 136, 32)],
        pattern: Sail::Sunburst,
        ui: rgb(248, 216, 48),
        heckle: "AHOY, GROUNDLING!",
        quote: "THE GROUND IS A MYTH.",
    },
    PilotDef {
        name: "DUNE DIRK",
        short: "DIRK",
        bio: ["TEACHES SURFING", "TO SEAGULLS."],
        glide: 3,
        climb: 2,
        handling: 5,
        nerve: 3,
        style: 1,
        skin: SKIN_TAN,
        hair: (rgb(248, 216, 72), rgb(200, 152, 32)),
        suit: (rgb(64, 216, 200), rgb(32, 144, 136)),
        harness: (rgb(232, 64, 168), rgb(160, 32, 112)),
        acc: (rgb(248, 248, 248), rgb(168, 176, 200)),
        cap: (rgb(248, 72, 168), rgb(176, 32, 112)),
        sail: [rgb(248, 72, 184), rgb(64, 224, 232), rgb(248, 248, 120)],
        pattern: Sail::Zigzag,
        ui: rgb(248, 104, 200),
        heckle: "LATER, DUDE!",
        quote: "COWABUNGA, BUT UP.",
    },
    PilotDef {
        name: "SKY GRANNY",
        short: "GRANNY",
        bio: ["KNITS A SCARF", "EVERY THERMAL."],
        glide: 3,
        climb: 5,
        handling: 2,
        nerve: 3,
        style: 5,
        skin: SKIN_DARK,
        hair: (rgb(216, 216, 232), rgb(152, 152, 176)),
        suit: (rgb(184, 104, 216), rgb(120, 64, 160)),
        harness: (rgb(96, 168, 88), rgb(56, 112, 56)),
        acc: (rgb(248, 136, 168), rgb(192, 80, 120)),
        cap: (rgb(248, 136, 168), rgb(192, 80, 120)),
        sail: [rgb(200, 160, 248), rgb(136, 232, 184), rgb(248, 248, 248)],
        pattern: Sail::Flowers,
        ui: rgb(200, 160, 248),
        heckle: "WEAR A SCARF, DEAR!",
        quote: "BACK IN MY DAY WE FLEW UPHILL.",
    },
    PilotDef {
        name: "THE EAGLE",
        short: "EAGLE",
        bio: ["YES, AN ACTUAL", "EAGLE. DON'T ASK."],
        glide: 5,
        climb: 4,
        handling: 2,
        nerve: 2,
        style: 4,
        skin: EAGLE_WHITE,
        hair: (rgb(248, 248, 248), rgb(200, 200, 216)),
        suit: (rgb(136, 80, 40), rgb(88, 48, 24)),
        harness: (rgb(104, 64, 32), rgb(64, 40, 24)),
        acc: (rgb(248, 200, 32), rgb(200, 136, 16)),
        cap: (rgb(248, 248, 248), rgb(200, 200, 216)),
        sail: [rgb(40, 40, 64), rgb(248, 200, 48), rgb(248, 248, 248)],
        pattern: Sail::Chevron,
        ui: rgb(248, 184, 48),
        heckle: "SKREEEEE!",
        quote: "SKREE. (IT MEANS GOOD LUCK.)",
    },
];

pub const EAGLE: usize = 3;

pub fn palette(p: usize) -> Palette {
    let d = &PILOTS[p];
    let mut pal = [0u32; ROLES];
    pal[OUT as usize] = OUTLINE;
    pal[SKIN as usize] = d.skin.0;
    pal[SKIN_D as usize] = d.skin.1;
    pal[HAIR as usize] = d.hair.0;
    pal[HAIR_D as usize] = d.hair.1;
    pal[SHIRT as usize] = d.suit.0;
    pal[SHIRT_D as usize] = d.suit.1;
    pal[PANTS as usize] = d.harness.0;
    pal[PANTS_D as usize] = d.harness.1;
    pal[WHITE as usize] = rgb(248, 248, 248);
    pal[ACC as usize] = d.acc.0;
    pal[ACC_D as usize] = d.acc.1;
    pal[SHOE as usize] = rgb(248, 248, 248);
    pal[EYE as usize] = if p == EAGLE { rgb(8, 8, 16) } else { rgb(16, 16, 32) };
    pal[CAP as usize] = d.cap.0;
    pal[CAP_D as usize] = d.cap.1;
    pal
}
