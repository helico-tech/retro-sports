//! TEGENWIND! data: the four bikes, the riders and the three stages.

use crate::engine::gfx::rgb;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bike {
    Oma,
    Race,
    Bak,
    Lig,
}

pub const BIKES: [Bike; 4] = [Bike::Oma, Bike::Race, Bike::Bak, Bike::Lig];

pub struct BikeDef {
    pub name: &'static str,
    /// three short lines (19 characters fit)
    pub bio: [&'static str; 3],
    /// push per stroke (multiplier)
    pub power: f32,
    /// air drag (multiplier)
    pub drag: f32,
    /// air drag multiplier when tucked
    pub tuck: f32,
    /// rolling resistance (multiplier)
    pub roll: f32,
    /// hop take-off speed
    pub hop: f32,
    /// cargo points per piece
    pub cargo_pts: i32,
    /// pieces on the rack at the start
    pub cargo_n: i32,
    /// chance a piece stays on after a bump
    pub cargo_grip: f32,
    /// frames between strokes (the cadence limit)
    pub gap: i32,
    /// SPD, WIND, CARGO, HOP for the select screen, 1..=5
    pub stats: [i32; 4],
}

impl Bike {
    pub fn def(self) -> &'static BikeDef {
        &BIKE_DEFS[self as usize]
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

pub const BIKE_DEFS: [BikeDef; 4] = [
    BikeDef {
        name: "OMAFIETS",
        bio: ["28 KG DUTCH STEEL.", "STOLEN 6 TIMES,", "CAME BACK 7 TIMES."],
        power: 1.0,
        drag: 1.08,
        tuck: 0.66,
        roll: 1.0,
        hop: 2.9,
        cargo_pts: 150,
        cargo_n: 4,
        cargo_grip: 0.8,
        gap: 10,
        stats: [3, 2, 4, 3],
    },
    BikeDef {
        name: "RACEFIETS",
        bio: ["LIGHTER THAN", "DENNIS'S EGO.", "CARGO? WHAT CARGO?"],
        power: 0.96,
        drag: 0.95,
        tuck: 0.72,
        roll: 0.85,
        hop: 3.3,
        cargo_pts: 150,
        cargo_n: 3,
        cargo_grip: 0.25,
        gap: 10,
        stats: [5, 4, 1, 4],
    },
    BikeDef {
        name: "BAKFIETS",
        bio: ["FITS 2 KIDS, A DOG", "AND A SOFA. HOPS", "LIKE A FRIDGE."],
        power: 1.02,
        drag: 1.04,
        tuck: 0.72,
        roll: 1.05,
        hop: 2.1,
        cargo_pts: 250,
        cargo_n: 6,
        cargo_grip: 0.9,
        gap: 10,
        stats: [2, 2, 5, 1],
    },
    BikeDef {
        name: "LIGFIETS",
        bio: ["LIE BACK, LET THE", "WIND GO OVER YOU.", "CAN'T SEE HEDGES."],
        power: 0.98,
        drag: 0.74,
        tuck: 0.92,
        roll: 1.0,
        hop: 2.4,
        cargo_pts: 150,
        cargo_n: 4,
        cargo_grip: 0.6,
        gap: 10,
        stats: [3, 5, 3, 1],
    },
];

// ------------------------------------------------------------------ riders

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Who {
    Kees,
    Anouk,
    Oma,
    Dennis,
    DeVries,
}

pub const RIVALS: [Who; 3] = [Who::Oma, Who::Dennis, Who::DeVries];
pub const ALL_WHO: [Who; 5] = [Who::Kees, Who::Anouk, Who::Oma, Who::Dennis, Who::DeVries];

pub struct CharDef {
    pub name: &'static str,
    pub short: &'static str,
    pub style: usize,
    pub skin: (u32, u32),
    pub hair: (u32, u32),
    pub shirt: (u32, u32),
    pub pants: (u32, u32),
    pub acc: (u32, u32),
    pub cap: (u32, u32),
    /// frame color of the bike
    pub frame: (u32, u32),
    /// the bike a rival always rides
    pub bike: Bike,
    /// marker / map dot color
    pub color: u32,
    pub tagline: &'static str,
    /// things they shout during the race
    pub heckles: &'static [&'static str],
    // personality
    pub power: f32,
    /// how much of the wind reaches them (Oma Riek: hardly any)
    pub wind_feel: f32,
    /// power multiplier in a headwind / a tailwind
    pub head_power: f32,
    pub tail_power: f32,
    /// air drag behind another rider
    pub draft: f32,
    /// how keen they are to sit on a wheel, 0..1
    pub draft_seek: f32,
    /// how keen they are to tuck, 0..1
    pub tuck_seek: f32,
    pub never_bonks: bool,
}

const SKIN_LIGHT: (u32, u32) = (rgb(248, 200, 160), rgb(216, 144, 104));
const SKIN_PINK: (u32, u32) = (rgb(248, 184, 152), rgb(208, 128, 104));
const SKIN_TAN: (u32, u32) = (rgb(224, 160, 104), rgb(176, 112, 64));

impl Who {
    pub fn def(self) -> &'static CharDef {
        &CHARS[self as usize]
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

pub const CHARS: [CharDef; 5] = [
    CharDef {
        name: "KEES",
        short: "KEES",
        style: 0,
        skin: SKIN_LIGHT,
        hair: (rgb(248, 216, 72), rgb(200, 152, 32)),
        shirt: (rgb(248, 128, 24), rgb(192, 80, 16)),
        pants: (rgb(56, 80, 168), rgb(32, 48, 112)),
        acc: (rgb(248, 248, 248), rgb(168, 176, 200)),
        cap: (rgb(248, 128, 24), rgb(192, 80, 16)),
        frame: (rgb(232, 64, 48), rgb(152, 32, 40)),
        bike: Bike::Oma,
        color: rgb(248, 232, 64),
        tagline: "CYCLED TO SCHOOL AGAINST THE WIND. BOTH WAYS.",
        heckles: &["HUP!", "LEKKER!"],
        power: 1.0,
        wind_feel: 1.0,
        head_power: 1.0,
        tail_power: 1.0,
        draft: 0.55,
        draft_seek: 0.6,
        tuck_seek: 0.7,
        never_bonks: false,
    },
    CharDef {
        name: "ANOUK",
        short: "ANOUK",
        style: 5,
        skin: SKIN_TAN,
        hair: (rgb(136, 72, 32), rgb(88, 40, 16)),
        shirt: (rgb(64, 184, 240), rgb(32, 112, 192)),
        pants: (rgb(232, 232, 240), rgb(160, 168, 192)),
        acc: (rgb(248, 104, 160), rgb(184, 48, 112)),
        cap: (rgb(248, 104, 160), rgb(184, 48, 112)),
        frame: (rgb(56, 184, 120), rgb(24, 112, 72)),
        bike: Bike::Race,
        color: rgb(96, 224, 248),
        tagline: "HAS NEVER ONCE WAITED FOR A GREEN LIGHT.",
        heckles: &["HUP!", "LEKKER!"],
        power: 1.0,
        wind_feel: 1.0,
        head_power: 1.0,
        tail_power: 1.0,
        draft: 0.55,
        draft_seek: 0.6,
        tuck_seek: 0.7,
        never_bonks: false,
    },
    CharDef {
        name: "OMA RIEK",
        short: "OMA RIEK",
        style: 3,
        skin: SKIN_PINK,
        hair: (rgb(216, 216, 224), rgb(160, 160, 176)),
        shirt: (rgb(144, 80, 168), rgb(96, 48, 120)),
        pants: (rgb(104, 104, 120), rgb(64, 64, 80)),
        acc: (rgb(232, 232, 216), rgb(176, 168, 152)),
        cap: (rgb(200, 56, 88), rgb(136, 32, 56)),
        frame: (rgb(48, 48, 56), rgb(24, 24, 32)),
        bike: Bike::Oma,
        color: rgb(200, 120, 232),
        tagline: "HAS NOT STOPPED PEDALLING SINCE 1953.",
        heckles: &["...", "...", "..."],
        power: 0.92,
        wind_feel: 0.35,
        head_power: 1.0,
        tail_power: 0.96,
        draft: 0.6,
        draft_seek: 0.1,
        tuck_seek: 0.0,
        never_bonks: true,
    },
    CharDef {
        name: "DENNIS",
        short: "DENNIS",
        style: 2,
        skin: SKIN_LIGHT,
        hair: (rgb(136, 80, 32), rgb(88, 48, 16)),
        shirt: (rgb(200, 248, 48), rgb(128, 184, 24)),
        pants: (rgb(40, 40, 56), rgb(24, 24, 32)),
        acc: (rgb(248, 72, 168), rgb(176, 32, 112)),
        cap: (rgb(248, 72, 168), rgb(176, 32, 112)),
        frame: (rgb(248, 72, 168), rgb(160, 32, 104)),
        bike: Bike::Race,
        color: rgb(176, 248, 48),
        tagline: "FULL LYCRA. HATES WIND. HATES YOU.",
        heckles: &["OUT OF MY WAY!", "AERO, BABY!", "STRAVA SEGMENT!", "PFFF, WIND..."],
        power: 1.02,
        wind_feel: 1.0,
        head_power: 0.9,
        tail_power: 1.12,
        draft: 0.55,
        draft_seek: 0.5,
        tuck_seek: 0.9,
        never_bonks: false,
    },
    CharDef {
        name: "FAMILIE DE VRIES",
        short: "DE VRIES",
        style: 4,
        skin: SKIN_LIGHT,
        hair: (rgb(232, 136, 48), rgb(168, 88, 24)),
        shirt: (rgb(248, 216, 40), rgb(192, 152, 16)),
        pants: (rgb(72, 104, 72), rgb(40, 64, 40)),
        acc: (rgb(48, 120, 216), rgb(24, 72, 152)),
        cap: (rgb(248, 216, 40), rgb(192, 152, 16)),
        frame: (rgb(48, 120, 216), rgb(24, 72, 152)),
        bike: Bike::Bak,
        color: rgb(248, 176, 48),
        tagline: "TWO KIDS, ONE DOG, ZERO SHAME. DRAFTS LIKE A PRO.",
        heckles: &["ZIJN WE ER AL?", "WOEF!", "IK MOET PLASSEN!", "PAPA, HARDER!"],
        power: 1.02,
        wind_feel: 1.0,
        head_power: 1.0,
        tail_power: 1.0,
        draft: 0.38,
        draft_seek: 1.0,
        tuck_seek: 0.4,
        never_bonks: false,
    },
];

// ------------------------------------------------------------------ stages

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Wind {
    /// headwind strength
    Head(f32),
    /// tailwind strength
    Tail(f32),
    /// side wind toward the canal
    Side(f32),
}

impl Wind {
    /// Headwind component in px/frame (negative = from behind).
    pub fn head(self) -> f32 {
        match self {
            Wind::Head(s) => s,
            Wind::Tail(s) => -s,
            Wind::Side(s) => 0.3 * s,
        }
    }
    pub fn side(self) -> f32 {
        match self {
            Wind::Side(s) => s,
            _ => 0.0,
        }
    }
    pub fn strength(self) -> f32 {
        match self {
            Wind::Head(s) | Wind::Tail(s) | Wind::Side(s) => s,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Wind::Head(_) => "TEGENWIND",
            Wind::Tail(_) => "WIND MEE",
            Wind::Side(_) => "ZIJWIND",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Venue {
    Polder,
    Amsterdam,
    Dijk,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cargo {
    Cheese,
    Tulips,
    Herring,
    Tuba,
    Clock,
}

impl Cargo {
    pub fn name(self) -> &'static str {
        match self {
            Cargo::Cheese => "CHEESE WHEELS",
            Cargo::Tulips => "TULIP POTS",
            Cargo::Herring => "CRATES OF HERRING",
            Cargo::Tuba => "TUBAS",
            Cargo::Clock => "GRANDFATHER CLOCKS",
        }
    }
    pub fn one(self) -> &'static str {
        match self {
            Cargo::Cheese => "CHEESE",
            Cargo::Tulips => "TULIPS",
            Cargo::Herring => "HERRING",
            Cargo::Tuba => "TUBA",
            Cargo::Clock => "CLOCK",
        }
    }
}

/// Things that can be placed on the path, with a weight per stage.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Spawn {
    Pothole,
    Bollard,
    Herring,
    Cat,
    Geese,
    Sheep,
    Cow,
    Tourist,
    Rails,
    Tractor,
    Sofa,
    Food,
}

pub struct StageDef {
    pub name: &'static str,
    pub venue: Venue,
    pub tagline: [&'static str; 2],
    pub tip: [&'static str; 2],
    /// sections: (length in px, wind)
    pub sections: &'static [(f32, Wind)],
    /// (from, to) x ranges of cobbles
    pub cobbles: &'static [(f32, f32)],
    pub spawns: &'static [(Spawn, i32)],
    /// gap between spawns (min, max) in px
    pub spacing: (i32, i32),
    pub cargo: Cargo,
    /// x of the drawbridge, if any
    pub bridge: Option<f32>,
    /// x of the lonely snack bar, if any
    pub snackbar: Option<f32>,
    /// the CPU's skill on this stage in 1P
    pub cpu_skill: f32,
}

impl StageDef {
    pub fn length(&self) -> f32 {
        self.sections.iter().map(|s| s.0).sum()
    }
    /// The wind at route position x, and the index of its section.
    pub fn wind_at(&self, x: f32) -> (Wind, usize) {
        let mut acc = 0.0;
        for (i, &(len, w)) in self.sections.iter().enumerate() {
            acc += len;
            if x < acc {
                return (w, i);
            }
        }
        let last = self.sections.len() - 1;
        (self.sections[last].1, last)
    }
    pub fn section_start(&self, i: usize) -> f32 {
        self.sections[..i].iter().map(|s| s.0).sum()
    }
    pub fn on_cobbles(&self, x: f32) -> bool {
        self.cobbles.iter().any(|&(a, b)| x >= a && x < b)
    }
}

pub const STAGES: [StageDef; 3] = [
    StageDef {
        name: "DE POLDER",
        venue: Venue::Polder,
        tagline: ["LIGHT HEADWIND. MOSTLY COWS.", "SOME SHEEP. A FEW GEESE. NO HILLS."],
        tip: ["DRAFT: SIT RIGHT BEHIND A RIDER", "IN YOUR LANE. LESS WIND, FREE SPEED!"],
        sections: &[(2200.0, Wind::Head(0.5)), (2600.0, Wind::Tail(0.6)), (2400.0, Wind::Side(0.7)), (3000.0, Wind::Head(0.8)), (2200.0, Wind::Tail(0.5))],
        cobbles: &[],
        spawns: &[
            (Spawn::Pothole, 5),
            (Spawn::Bollard, 3),
            (Spawn::Cat, 2),
            (Spawn::Geese, 3),
            (Spawn::Sheep, 2),
            (Spawn::Cow, 2),
            (Spawn::Tractor, 1),
            (Spawn::Food, 4),
        ],
        spacing: (230, 380),
        cargo: Cargo::Cheese,
        bridge: None,
        snackbar: None,
        cpu_skill: 0.45,
    },
    StageDef {
        name: "AMSTERDAM",
        venue: Venue::Amsterdam,
        tagline: ["CANALS, COBBLES AND TRAMS.", "AND TOURISTS. SO MANY TOURISTS."],
        tip: ["RING THE BELL (Z) AT TOURISTS.", "HOP (X) OVER TRAM RAILS!"],
        sections: &[(2600.0, Wind::Head(0.4)), (3000.0, Wind::Side(0.5)), (3200.0, Wind::Tail(0.6)), (3600.0, Wind::Head(0.7))],
        cobbles: &[(1400.0, 2600.0), (5600.0, 6800.0), (10200.0, 11400.0)],
        spawns: &[
            (Spawn::Pothole, 3),
            (Spawn::Bollard, 3),
            (Spawn::Herring, 2),
            (Spawn::Cat, 1),
            (Spawn::Tourist, 6),
            (Spawn::Rails, 3),
            (Spawn::Sofa, 1),
            (Spawn::Food, 4),
        ],
        spacing: (210, 340),
        cargo: Cargo::Tulips,
        bridge: Some(7600.0),
        snackbar: None,
        cpu_skill: 0.66,
    },
    StageDef {
        name: "AFSLUITDIJK",
        venue: Venue::Dijk,
        tagline: ["32 KM OF DIKE. STORM FORCE 9.", "ONE LONELY SNACK BAR. GOOD LUCK."],
        tip: ["STORM! HOLD C TO TUCK. IN SIDE", "WIND, KEEP AWAY FROM THE WATER!"],
        sections: &[(2800.0, Wind::Head(1.0)), (2800.0, Wind::Side(1.0)), (3200.0, Wind::Head(1.3)), (2000.0, Wind::Tail(0.8)), (3400.0, Wind::Head(1.2))],
        cobbles: &[],
        spawns: &[
            (Spawn::Pothole, 3),
            (Spawn::Herring, 3),
            (Spawn::Geese, 2),
            (Spawn::Sheep, 3),
            (Spawn::Bollard, 2),
            (Spawn::Food, 3),
        ],
        spacing: (240, 400),
        cargo: Cargo::Herring,
        bridge: None,
        snackbar: Some(7100.0),
        cpu_skill: 0.8,
    },
];

/// Points for 1st..4th place.
pub const PLACE_PTS: [i32; 4] = [1000, 600, 300, 100];

pub fn ordinal(place: usize) -> &'static str {
    ["1E", "2E", "3E", "4E"][place.min(3)]
}

/// The ticker on the title screen.
pub const TICKER: &str = "EVERY DUTCH CHILD CYCLES TO SCHOOL AGAINST THE WIND. BOTH WAYS.      \
OMA RIEK HAS NOT STOPPED PEDALLING SINCE 1953.      \
HOLLAND HAS MORE BIKES THAN PEOPLE. MOST OF THEM ARE IN A CANAL.      \
DO NOT FEED THE GEESE. THE GEESE WILL FEED ON YOU.      \
THE AFSLUITDIJK: 32 KM, ZERO CORNERS, ONE SNACK BAR.      \
LEKKER BEZIG!      ";
