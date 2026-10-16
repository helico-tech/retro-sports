//! TIENEN! data: the kids, the venues and the shot kinds.

use crate::engine::gfx::rgb;

// ------------------------------------------------------------------ shots

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Kind {
    Volley,
    Chest,
    Header,
    Heel,
    Bicycle,
}

pub const KINDS: [Kind; 5] = [Kind::Volley, Kind::Chest, Kind::Header, Kind::Heel, Kind::Bicycle];

impl Kind {
    /// Points off the keeper.
    pub fn value(self) -> i32 {
        match self {
            Kind::Volley | Kind::Chest => 1,
            Kind::Header | Kind::Heel => 2,
            Kind::Bicycle => 5,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Kind::Volley => "VOLLEY",
            Kind::Chest => "BORST",
            Kind::Header => "KOPBAL",
            Kind::Heel => "HAKJE",
            Kind::Bicycle => "OMHAAL",
        }
    }
    pub fn index(self) -> usize {
        KINDS.iter().position(|&k| k == self).unwrap_or(0)
    }
    /// Ball speed off the boot (px per frame).
    pub fn speed(self) -> f32 {
        match self {
            Kind::Volley => 4.4,
            Kind::Chest => 3.2,
            Kind::Header => 3.6,
            Kind::Heel => 3.0,
            Kind::Bicycle => 4.6,
        }
    }
    /// Spread of the shot (depth, height) before stats.
    pub fn spread(self) -> (f32, f32) {
        match self {
            Kind::Volley => (4.0, 4.5),
            Kind::Chest => (5.5, 6.0),
            Kind::Header => (4.5, 5.5),
            Kind::Heel => (6.0, 6.5),
            Kind::Bicycle => (7.0, 8.0),
        }
    }
    /// Head kinds use KOP, foot kinds SCHOT.
    pub fn head(self) -> bool {
        self == Kind::Header
    }
}

// ------------------------------------------------------------------ kids

pub struct KidDef {
    pub name: &'static str,
    pub bio: [&'static str; 3],
    /// SCHOT, KOP, KEEP, SNEL (1..5)
    pub stats: [i32; 4],
    /// head style for `person()`
    pub style: usize,
    pub skin: (u32, u32),
    pub hair: (u32, u32),
    pub shirt: (u32, u32),
    pub pants: (u32, u32),
    pub acc: (u32, u32),
    pub cap: (u32, u32),
    pub color: u32,
    /// wider bicycle-kick window
    pub acrobat: bool,
}

/// The four kids you can pick.
pub const PICKS: [usize; 4] = [0, 1, 2, 3];

pub const fn pair(a: u32, b: u32) -> (u32, u32) {
    (a, b)
}

const SKIN_A: (u32, u32) = pair(rgb(248, 200, 160), rgb(216, 144, 104));
const SKIN_B: (u32, u32) = pair(rgb(224, 160, 104), rgb(176, 112, 64));
const SKIN_C: (u32, u32) = pair(rgb(168, 112, 72), rgb(120, 72, 48));
const SKIN_D: (u32, u32) = pair(rgb(248, 216, 184), rgb(224, 168, 128));

pub const KIDS: [KidDef; 13] = [
    KidDef {
        name: "KEVIN",
        bio: ["WEARS HIS ORANJE", "SHIRT IN BED.", "SHOOTS FROM ANYWHERE."],
        stats: [5, 2, 2, 4],
        style: 0,
        skin: SKIN_A,
        hair: pair(rgb(232, 200, 96), rgb(184, 144, 48)),
        shirt: pair(rgb(248, 136, 24), rgb(192, 88, 16)),
        pants: pair(rgb(248, 248, 248), rgb(184, 184, 200)),
        acc: pair(rgb(32, 72, 160), rgb(24, 40, 104)),
        cap: pair(rgb(248, 136, 24), rgb(192, 88, 16)),
        color: rgb(248, 152, 40),
        acrobat: false,
    },
    KidDef {
        name: "SAMIRA",
        bio: ["HEADS EVERYTHING.", "ALSO THINGS THAT", "AREN'T BALLS."],
        stats: [3, 5, 3, 3],
        style: 5,
        skin: SKIN_B,
        hair: pair(rgb(40, 32, 48), rgb(16, 16, 32)),
        shirt: pair(rgb(64, 120, 216), rgb(32, 64, 152)),
        pants: pair(rgb(40, 48, 96), rgb(24, 24, 64)),
        acc: pair(rgb(248, 248, 248), rgb(184, 184, 200)),
        cap: pair(rgb(232, 64, 48), rgb(152, 32, 40)),
        color: rgb(96, 160, 248),
        acrobat: false,
    },
    KidDef {
        name: "GIJS",
        bio: ["TOO BIG FOR THE", "GOAL. STANDS IN", "IT ANYWAY."],
        stats: [3, 3, 5, 1],
        style: 2,
        skin: SKIN_D,
        hair: pair(rgb(160, 96, 48), rgb(104, 64, 32)),
        shirt: pair(rgb(72, 168, 88), rgb(40, 112, 56)),
        pants: pair(rgb(72, 72, 96), rgb(48, 48, 64)),
        acc: pair(rgb(248, 216, 48), rgb(200, 152, 32)),
        cap: pair(rgb(40, 120, 64), rgb(24, 72, 40)),
        color: rgb(104, 208, 104),
        acrobat: false,
    },
    KidDef {
        name: "LIEKE",
        bio: ["3 YEARS OF BICYCLE", "KICKS. LANDED ONE.", "WEARS A HELMET NOW."],
        stats: [3, 2, 2, 5],
        style: 1,
        skin: SKIN_A,
        hair: pair(rgb(248, 232, 136), rgb(216, 184, 80)),
        shirt: pair(rgb(216, 48, 120), rgb(144, 24, 80)),
        pants: pair(rgb(40, 40, 64), rgb(24, 24, 40)),
        acc: pair(rgb(248, 248, 248), rgb(184, 184, 200)),
        cap: pair(rgb(96, 216, 232), rgb(48, 152, 176)),
        color: rgb(240, 104, 168),
        acrobat: true,
    },
    // --- de galerijflat
    KidDef {
        name: "TIMMY",
        bio: ["PLAYS IN HIS", "PYJAMAS. HIS MUM", "CALLS HIM IN SOON."],
        stats: [2, 2, 2, 3],
        style: 3,
        skin: SKIN_D,
        hair: pair(rgb(200, 136, 56), rgb(136, 80, 32)),
        shirt: pair(rgb(160, 200, 248), rgb(104, 144, 208)),
        pants: pair(rgb(160, 200, 248), rgb(104, 144, 208)),
        acc: pair(rgb(248, 248, 168), rgb(216, 200, 96)),
        cap: pair(rgb(232, 64, 48), rgb(152, 32, 40)),
        color: rgb(160, 200, 248),
        acrobat: false,
    },
    KidDef {
        name: "MO",
        bio: ["KNOWS EVERY", "CAR IN THE STREET", "BY ITS HONK."],
        stats: [3, 2, 2, 3],
        style: 4,
        skin: SKIN_C,
        hair: pair(rgb(32, 24, 24), rgb(16, 16, 16)),
        shirt: pair(rgb(232, 64, 48), rgb(152, 32, 40)),
        pants: pair(rgb(40, 48, 96), rgb(24, 24, 64)),
        acc: pair(rgb(248, 248, 248), rgb(184, 184, 200)),
        cap: pair(rgb(32, 32, 48), rgb(16, 16, 32)),
        color: rgb(232, 88, 64),
        acrobat: false,
    },
    KidDef {
        name: "JESSICA",
        bio: ["KEEPS IN HER", "MUM'S GARDENING", "GLOVES."],
        stats: [2, 2, 3, 2],
        style: 5,
        skin: SKIN_A,
        hair: pair(rgb(176, 72, 40), rgb(120, 40, 24)),
        shirt: pair(rgb(248, 216, 48), rgb(200, 152, 32)),
        pants: pair(rgb(96, 64, 128), rgb(64, 40, 88)),
        acc: pair(rgb(96, 176, 88), rgb(48, 112, 72)),
        cap: pair(rgb(96, 176, 88), rgb(48, 112, 72)),
        color: rgb(248, 216, 64),
        acrobat: false,
    },
    // --- het schoolplein
    KidDef {
        name: "DAAN",
        bio: ["GROUP 8. SAYS HE", "TRAINS WITH AJAX.", "HE DOESN'T."],
        stats: [4, 3, 3, 3],
        style: 0,
        skin: SKIN_A,
        hair: pair(rgb(104, 64, 32), rgb(72, 40, 24)),
        shirt: pair(rgb(248, 248, 248), rgb(184, 184, 200)),
        pants: pair(rgb(216, 48, 48), rgb(144, 24, 40)),
        acc: pair(rgb(216, 48, 48), rgb(144, 24, 40)),
        cap: pair(rgb(216, 48, 48), rgb(144, 24, 40)),
        color: rgb(232, 232, 240),
        acrobat: false,
    },
    KidDef {
        name: "NOOR",
        bio: ["CLASS PRESIDENT.", "KNOWS THE RULES.", "ALL OF THEM."],
        stats: [3, 4, 4, 3],
        style: 1,
        skin: SKIN_B,
        hair: pair(rgb(72, 40, 24), rgb(40, 24, 16)),
        shirt: pair(rgb(136, 72, 184), rgb(88, 40, 128)),
        pants: pair(rgb(40, 48, 96), rgb(24, 24, 64)),
        acc: pair(rgb(248, 216, 48), rgb(200, 152, 32)),
        cap: pair(rgb(248, 216, 48), rgb(200, 152, 32)),
        color: rgb(176, 112, 232),
        acrobat: false,
    },
    KidDef {
        name: "BRAM",
        bio: ["EATS HIS LUNCH", "IN GOAL. KEEPS", "WITH ONE HAND."],
        stats: [3, 3, 4, 2],
        style: 2,
        skin: SKIN_D,
        hair: pair(rgb(232, 200, 96), rgb(184, 144, 48)),
        shirt: pair(rgb(96, 176, 88), rgb(48, 112, 72)),
        pants: pair(rgb(120, 104, 72), rgb(80, 64, 48)),
        acc: pair(rgb(248, 136, 24), rgb(192, 88, 16)),
        cap: pair(rgb(32, 72, 160), rgb(24, 40, 104)),
        color: rgb(128, 200, 104),
        acrobat: false,
    },
    // --- de kooi
    KidDef {
        name: "RICARDO",
        bio: ["THE BIG BROTHER.", "HAS A MOUSTACHE.", "SORT OF."],
        stats: [5, 4, 4, 4],
        style: 4,
        skin: SKIN_B,
        hair: pair(rgb(32, 24, 24), rgb(16, 16, 16)),
        shirt: pair(rgb(32, 32, 48), rgb(16, 16, 32)),
        pants: pair(rgb(32, 32, 48), rgb(16, 16, 32)),
        acc: pair(rgb(248, 216, 48), rgb(200, 152, 32)),
        cap: pair(rgb(248, 216, 48), rgb(200, 152, 32)),
        color: rgb(248, 200, 48),
        acrobat: true,
    },
    KidDef {
        name: "JAYDEN",
        bio: ["THE TWIN WHO", "SHOOTS. OR IS", "IT THE OTHER ONE?"],
        stats: [4, 4, 3, 4],
        style: 3,
        skin: SKIN_C,
        hair: pair(rgb(32, 24, 24), rgb(16, 16, 16)),
        shirt: pair(rgb(48, 176, 168), rgb(24, 112, 112)),
        pants: pair(rgb(248, 248, 248), rgb(184, 184, 200)),
        acc: pair(rgb(232, 64, 48), rgb(152, 32, 40)),
        cap: pair(rgb(48, 176, 168), rgb(24, 112, 112)),
        color: rgb(72, 208, 200),
        acrobat: false,
    },
    KidDef {
        name: "JAYLA",
        bio: ["THE TWIN WHO", "KEEPS. NEVER", "LETS ONE IN."],
        stats: [4, 3, 5, 3],
        style: 5,
        skin: SKIN_C,
        hair: pair(rgb(32, 24, 24), rgb(16, 16, 16)),
        shirt: pair(rgb(48, 176, 168), rgb(24, 112, 112)),
        pants: pair(rgb(32, 32, 48), rgb(16, 16, 32)),
        acc: pair(rgb(248, 248, 248), rgb(184, 184, 200)),
        cap: pair(rgb(232, 64, 48), rgb(152, 32, 40)),
        color: rgb(120, 224, 216),
        acrobat: false,
    },
];

pub fn kid(i: usize) -> &'static KidDef {
    &KIDS[i % KIDS.len()]
}

pub fn schot(i: usize) -> i32 {
    kid(i).stats[0]
}
pub fn kop(i: usize) -> i32 {
    kid(i).stats[1]
}
pub fn keep(i: usize) -> i32 {
    kid(i).stats[2]
}
pub fn snel(i: usize) -> i32 {
    kid(i).stats[3]
}

// ------------------------------------------------------------------ venues

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Place {
    Flat,
    School,
    Kooi,
}

pub struct Venue {
    pub place: Place,
    pub name: &'static str,
    pub short: &'static str,
    pub tagline: [&'static str; 2],
    /// the three local kids (index into KIDS)
    pub kids: [usize; 3],
    /// CPU skill of the local kids (0..1)
    pub skill: f32,
    pub rival: &'static str,
    pub heckles: &'static [&'static str],
    pub tip: [&'static str; 2],
    /// cars cross the pitch
    pub cars: bool,
    /// the poodle
    pub dog: bool,
    /// a fence with a garden behind it (the far side)
    pub fence: bool,
    /// the ice cream van drives past
    pub van: bool,
}

pub const VENUES: [Venue; 3] = [
    Venue {
        place: Place::Flat,
        name: "GALERIJFLAT",
        short: "DE FLAT",
        tagline: ["SATURDAY MORNING BEHIND THE FLATS.", "MIND THE CARS. AND TIMMY'S MUM."],
        kids: [4, 5, 6],
        skill: 0.3,
        rival: "MO",
        heckles: &["JIJ GAAT ERIN!", "MIJN BAL, MIJN REGELS!", "AUTO? WELKE AUTO?"],
        tip: ["CROSS WITH C, SHOOT IT OUT OF THE AIR", "WITH Z. A BOUNCE IN THE GOAL: WISSEL!"],
        cars: true,
        dog: true,
        fence: false,
        van: false,
    },
    Venue {
        place: Place::School,
        name: "SCHOOLPLEIN",
        short: "SCHOOL",
        tagline: ["BREAK TIME. GROUP 8 OWNS THE GOAL.", "FIFTEEN MINUTES. NO MERCY."],
        kids: [7, 8, 9],
        skill: 0.6,
        rival: "DAAN",
        heckles: &["GROEP 3 MAG NIET MEE!", "DAT TELT NIET!", "BIJ AJAX DOEN WE DAT ZO."],
        tip: ["HEADERS AND HEELS TAKE 2 POINTS.", "TURN YOUR BACK (RIGHT) FOR A HEEL."],
        cars: false,
        dog: true,
        fence: true,
        van: false,
    },
    Venue {
        place: Place::Kooi,
        name: "DE KOOI",
        short: "DE KOOI",
        tagline: ["THE CAGE, UNDER THE LAMPS.", "THE BIG KIDS FROM THE NEXT STREET."],
        kids: [10, 11, 12],
        skill: 0.9,
        rival: "RICARDO",
        heckles: &["KEEPER IS EEN GATENKAAS!", "HEB JE HEM?!", "BILLENBRAND! BILLENBRAND!"],
        tip: ["JUMP (X) WITH YOUR BACK TO THE GOAL,", "THEN Z: OMHAAL! 5 POINTS. IF IT WORKS."],
        cars: false,
        dog: false,
        fence: false,
        van: true,
    },
];

pub const TICKER: &str = "TIENEN! - EVERYBODY STARTS WITH TEN - A BOUNCE IN THE GOAL? YOU'RE IN! - TIMMY, ETEN!!! - NOG EVEN MAM! - AUTO!!! - EXACTLY ZERO AND YOU LOSE - THE KEEPER ON 1 MAY CATCH EVERYWHERE - FIFI HAS THE BALL AGAIN - MEVROUW KNORR KEEPS EVERY BALL SINCE 1974 - IN NIJMEGEN THEY CALL IT REETREES - IN GRONINGEN KONTJERAUS - OMHAAL: 5 POINTS, 1 BRUISE - LATJETRAP DECIDES - IEN MIEN MUTTEN - ";

pub const COUNT_OUT: [&str; 14] =
    ["IEN", "MIEN", "MUTTEN", "TIEN", "POND", "GRUTTEN", "TIEN", "POND", "KAAS", "IEN", "MIEN", "MUTTEN", "IS DE", "BAAS!"];

/// Heckles at the keeper after a goal.
pub const GOAL_HECKLES: [&str; 6] = ["GATENKAAS!", "HAHA!", "ZAG JE HEM?", "PRACHTIG!", "DIE ZAT!", "NET ALS CRUIJFF!"];
/// The keeper after a goal.
pub const KEEPER_SULKS: [&str; 4] = ["ZON IN M'N OGEN!", "DAT WAS HANDS!", "GEMEEN!", "VALT NIET MEE..."];
