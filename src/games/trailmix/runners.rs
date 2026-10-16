//! The four runners, Gerald the goat, and their palettes.

use crate::engine::gfx::rgb;
use crate::engine::sprites::*;
use crate::engine::ui::OUTLINE;

/// Something painted on top of the plain `person()` sprite.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Extra {
    Beard,
    Vest,
    Socks,
    Apron,
}

pub struct RunnerDef {
    pub name: &'static str,
    pub short: &'static str,
    pub bio: &'static str,
    pub fuel: &'static str,
    /// 1..=5
    pub climb: i32,
    pub descend: i32,
    pub endurance: i32,
    pub agility: i32,
    /// flat speed multiplier (Speedy Pete is quick)
    pub speed: f32,
    pub style: usize,
    pub skin: (u32, u32),
    pub hair: (u32, u32),
    pub shirt: (u32, u32),
    pub pants: (u32, u32),
    pub acc: (u32, u32),
    pub cap: (u32, u32),
    pub extra: Extra,
}

const SKIN_LIGHT: (u32, u32) = (rgb(248, 200, 160), rgb(216, 144, 104));
const SKIN_TAN: (u32, u32) = (rgb(224, 160, 104), rgb(176, 112, 64));
const SKIN_DARK: (u32, u32) = (rgb(160, 96, 56), rgb(112, 64, 32));

pub const RUNNERS: [RunnerDef; 4] = [
    RunnerDef {
        name: "BIG BEARD BJORN",
        short: "BJORN",
        bio: "HAS NOT SHAVED SINCE 1979.",
        fuel: "FUEL: RAW ONIONS",
        climb: 4,
        descend: 2,
        endurance: 5,
        agility: 2,
        speed: 1.0,
        style: 3,
        skin: SKIN_LIGHT,
        hair: (rgb(216, 104, 40), rgb(152, 64, 24)),
        shirt: (rgb(200, 48, 48), rgb(136, 24, 32)),
        pants: (rgb(72, 88, 64), rgb(40, 56, 40)),
        acc: (rgb(248, 248, 248), rgb(176, 176, 200)),
        cap: (rgb(56, 104, 184), rgb(32, 64, 128)),
        extra: Extra::Beard,
    },
    RunnerDef {
        name: "ULLA ULTRA",
        short: "ULLA",
        bio: "RUNS TO THE SHOP. THE SHOP IS IN ITALY.",
        fuel: "FUEL: ESPRESSO",
        climb: 3,
        descend: 3,
        endurance: 4,
        agility: 3,
        speed: 1.0,
        style: 1,
        skin: SKIN_TAN,
        hair: (rgb(248, 224, 112), rgb(200, 160, 48)),
        shirt: (rgb(48, 184, 200), rgb(24, 120, 152)),
        pants: (rgb(56, 48, 88), rgb(32, 24, 56)),
        acc: (rgb(248, 120, 40), rgb(184, 72, 24)),
        cap: (rgb(248, 120, 40), rgb(184, 72, 24)),
        extra: Extra::Vest,
    },
    RunnerDef {
        name: "SPEEDY PETE",
        short: "PETE",
        bio: "MOSTLY SOCKS AND OPTIMISM.",
        fuel: "FUEL: GUMMY BEARS",
        climb: 3,
        descend: 4,
        endurance: 1,
        agility: 5,
        speed: 1.05,
        style: 2,
        skin: SKIN_DARK,
        hair: (rgb(48, 40, 56), rgb(24, 24, 40)),
        shirt: (rgb(248, 216, 48), rgb(192, 152, 24)),
        pants: (rgb(40, 64, 168), rgb(24, 40, 112)),
        acc: (rgb(232, 48, 120), rgb(160, 24, 80)),
        cap: (rgb(232, 48, 120), rgb(160, 24, 80)),
        extra: Extra::Socks,
    },
    RunnerDef {
        name: "MAMA TRAIL",
        short: "MAMA",
        bio: "SHE RAN DOWN THIS MOUNTAIN TO GIVE BIRTH. TWICE.",
        fuel: "FUEL: HER OWN SOUP",
        climb: 2,
        descend: 5,
        endurance: 3,
        agility: 3,
        speed: 1.0,
        style: 5,
        skin: SKIN_LIGHT,
        hair: (rgb(200, 200, 216), rgb(136, 136, 160)),
        shirt: (rgb(152, 72, 200), rgb(96, 40, 144)),
        pants: (rgb(48, 136, 72), rgb(24, 88, 48)),
        acc: (rgb(248, 248, 248), rgb(176, 176, 200)),
        cap: (rgb(248, 248, 248), rgb(176, 176, 200)),
        extra: Extra::Apron,
    },
];

/// Index used for Gerald in places that take a runner index.
pub const GERALD: usize = 4;

pub fn name(def: usize) -> &'static str {
    if def == GERALD { "GERALD" } else { RUNNERS[def].short }
}

pub fn palette(def: usize) -> Palette {
    let r = &RUNNERS[def.min(3)];
    let mut pal = [0u32; ROLES];
    pal[OUT as usize] = OUTLINE;
    pal[SKIN as usize] = r.skin.0;
    pal[SKIN_D as usize] = r.skin.1;
    pal[HAIR as usize] = r.hair.0;
    pal[HAIR_D as usize] = r.hair.1;
    pal[SHIRT as usize] = r.shirt.0;
    pal[SHIRT_D as usize] = r.shirt.1;
    pal[PANTS as usize] = r.pants.0;
    pal[PANTS_D as usize] = r.pants.1;
    pal[WHITE as usize] = rgb(248, 248, 248);
    pal[ACC as usize] = r.acc.0;
    pal[ACC_D as usize] = r.acc.1;
    pal[SHOE as usize] = r.acc.0;
    pal[EYE as usize] = OUTLINE;
    pal[CAP as usize] = r.cap.0;
    pal[CAP_D as usize] = r.cap.1;
    pal
}

/// Covered in mud after a faceplant: clothes go brown.
pub fn muddy(p: &Palette) -> Palette {
    let mut q = *p;
    let mud = rgb(120, 80, 40);
    let mud_d = rgb(80, 48, 24);
    q[SHIRT_D as usize] = mud_d;
    q[PANTS as usize] = mud;
    q[PANTS_D as usize] = mud_d;
    q[ACC as usize] = mud;
    q[ACC_D as usize] = mud_d;
    q[SKIN_D as usize] = mud_d;
    q
}

/// White flash (hit, pickle juice).
pub fn flash(p: &Palette) -> Palette {
    let mut q = *p;
    for (i, c) in q.iter_mut().enumerate() {
        if i != OUT as usize && i != T as usize {
            *c = rgb(248, 248, 248);
        }
    }
    q
}

/// Pickle juice face: green skin for a moment.
pub fn pickled(p: &Palette) -> Palette {
    let mut q = *p;
    q[SKIN as usize] = rgb(168, 216, 96);
    q[SKIN_D as usize] = rgb(104, 160, 56);
    q
}

/// Volunteers wear high-vis vests; spectators get random clothes.
pub fn crowd_palette(k: u32) -> Palette {
    let skins = [SKIN_LIGHT, SKIN_TAN, SKIN_DARK];
    let hairs = [(rgb(56, 48, 72), rgb(24, 24, 40)), (rgb(136, 80, 32), rgb(88, 48, 16)), (rgb(248, 216, 72), rgb(200, 152, 32)), (rgb(200, 200, 208), rgb(136, 136, 152))];
    let shirts = [
        (rgb(232, 48, 48), rgb(160, 24, 32)),
        (rgb(48, 120, 232), rgb(24, 72, 168)),
        (rgb(56, 168, 72), rgb(32, 112, 48)),
        (rgb(248, 152, 32), rgb(184, 96, 16)),
        (rgb(232, 88, 168), rgb(160, 48, 112)),
    ];
    let s = skins[(k % 3) as usize];
    let h = hairs[((k / 3) % 4) as usize];
    let sh = shirts[((k / 7) % 5) as usize];
    let mut pal = [0u32; ROLES];
    pal[OUT as usize] = OUTLINE;
    pal[SKIN as usize] = s.0;
    pal[SKIN_D as usize] = s.1;
    pal[HAIR as usize] = h.0;
    pal[HAIR_D as usize] = h.1;
    pal[SHIRT as usize] = sh.0;
    pal[SHIRT_D as usize] = sh.1;
    pal[PANTS as usize] = rgb(64, 64, 96);
    pal[PANTS_D as usize] = rgb(40, 40, 64);
    pal[WHITE as usize] = rgb(248, 248, 248);
    pal[ACC as usize] = rgb(248, 248, 248);
    pal[ACC_D as usize] = rgb(176, 176, 200);
    pal[SHOE as usize] = rgb(96, 64, 40);
    pal[EYE as usize] = OUTLINE;
    pal[CAP as usize] = sh.0;
    pal[CAP_D as usize] = sh.1;
    pal
}

pub fn volunteer_palette(k: u32) -> Palette {
    let mut pal = crowd_palette(k);
    pal[SHIRT as usize] = rgb(216, 248, 48);
    pal[SHIRT_D as usize] = rgb(160, 192, 24);
    pal[ACC as usize] = rgb(248, 136, 32);
    pal[CAP as usize] = rgb(248, 136, 32);
    pal[CAP_D as usize] = rgb(184, 88, 16);
    pal
}
