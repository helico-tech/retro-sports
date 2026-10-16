//! STALEN ZENUWEN! data: the thieves, bike kinds and their locks, the
//! BOA's and the three stages along the canals.

use crate::engine::gfx::rgb;

// ------------------------------------------------------------------ thieves

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Who {
    Joost,
    Harrie,
    Sanne,
    Prof,
}

pub const THIEVES: [Who; 4] = [Who::Joost, Who::Harrie, Who::Sanne, Who::Prof];

pub struct ThiefDef {
    pub name: &'static str,
    pub short: &'static str,
    pub bio: [&'static str; 3],
    /// SNEAK, HANDS, LEGS, BIKE (1..5)
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
}

impl Who {
    pub fn def(self) -> &'static ThiefDef {
        &THIEF_DEFS[self.index()]
    }
    pub fn index(self) -> usize {
        THIEVES.iter().position(|&w| w == self).unwrap_or(0)
    }
    pub fn sneak(self) -> i32 {
        self.def().stats[0]
    }
    pub fn hands(self) -> i32 {
        self.def().stats[1]
    }
    pub fn legs(self) -> i32 {
        self.def().stats[2]
    }
    pub fn wheels(self) -> i32 {
        self.def().stats[3]
    }
    pub fn walk_speed(self) -> f32 {
        0.86 + 0.07 * self.legs() as f32
    }
    pub fn run_speed(self) -> f32 {
        1.3 + 0.1 * self.legs() as f32
    }
    /// How fast BOA's get suspicious of this one.
    pub fn sus_mul(self) -> f32 {
        1.3 - 0.12 * self.sneak() as f32
    }
    pub fn ride_mul(self) -> f32 {
        0.85 + 0.05 * self.wheels() as f32
    }
    /// Taps for a ring lock.
    pub fn ring_taps(self) -> i32 {
        9 - self.hands()
    }
    /// Saw strokes for a chain.
    pub fn chain_strokes(self) -> i32 {
        22 - 2 * self.hands()
    }
    /// Frames for one swing of the U-lock needle, and the green zone width.
    pub fn u_period(self) -> f32 {
        56.0 + 9.0 * self.hands() as f32
    }
    pub fn u_zone(self) -> f32 {
        0.13 + 0.025 * self.hands() as f32
    }
}

pub const THIEF_DEFS: [ThiefDef; 4] = [
    ThiefDef {
        name: "JOOST",
        short: "JOOST",
        bio: ["STUDENT. HIS BIKE", "WAS STOLEN 7 TIMES.", "PAYBACK TIME."],
        stats: [2, 3, 5, 3],
        style: 3,
        skin: (rgb(248, 200, 160), rgb(216, 144, 104)),
        hair: (rgb(200, 136, 56), rgb(136, 80, 32)),
        shirt: (rgb(72, 168, 88), rgb(40, 112, 56)),
        pants: (rgb(72, 88, 144), rgb(48, 56, 104)),
        acc: (rgb(248, 232, 96), rgb(200, 168, 32)),
        cap: (rgb(232, 64, 48), rgb(152, 32, 40)),
        color: rgb(120, 216, 88),
    },
    ThiefDef {
        name: "OPA HARRIE",
        short: "OPA",
        bio: ["78 YEARS OLD.", "NOBODY SUSPECTS", "OPA. NOBODY."],
        stats: [5, 4, 1, 2],
        style: 2,
        skin: (rgb(248, 208, 176), rgb(216, 152, 120)),
        hair: (rgb(216, 216, 224), rgb(160, 160, 176)),
        shirt: (rgb(176, 152, 112), rgb(128, 104, 72)),
        pants: (rgb(104, 96, 88), rgb(72, 64, 56)),
        acc: (rgb(176, 48, 48), rgb(120, 32, 32)),
        cap: (rgb(120, 112, 96), rgb(80, 72, 64)),
        color: rgb(216, 184, 136),
    },
    ThiefDef {
        name: "SANNE",
        short: "SANNE",
        bio: ["BIKE COURIER.", "RIDES ANYTHING.", "CAN'T OPEN A JAR."],
        stats: [3, 1, 3, 5],
        style: 5,
        skin: (rgb(224, 160, 104), rgb(176, 112, 64)),
        hair: (rgb(40, 32, 48), rgb(16, 16, 32)),
        shirt: (rgb(248, 104, 184), rgb(176, 48, 120)),
        pants: (rgb(48, 48, 64), rgb(24, 24, 40)),
        acc: (rgb(64, 216, 248), rgb(32, 136, 200)),
        cap: (rgb(64, 216, 248), rgb(32, 136, 200)),
        color: rgb(248, 104, 184),
    },
    ThiefDef {
        name: "DE PROF",
        short: "PROF",
        bio: ["RETIRED LOCKSMITH", "OPENS ANYTHING.", "RIDES LIKE A SOFA."],
        stats: [3, 5, 2, 1],
        style: 0,
        skin: (rgb(160, 96, 56), rgb(112, 64, 32)),
        hair: (rgb(56, 48, 72), rgb(24, 24, 40)),
        shirt: (rgb(88, 64, 136), rgb(56, 40, 96)),
        pants: (rgb(64, 64, 80), rgb(40, 40, 56)),
        acc: (rgb(248, 248, 248), rgb(184, 184, 208)),
        cap: (rgb(88, 64, 136), rgb(56, 40, 96)),
        color: rgb(168, 136, 232),
    },
];

// ------------------------------------------------------------------ bikes

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Kind {
    Oma,
    Race,
    Bak,
    Gold,
    Wreck,
}

pub const KINDS: [Kind; 5] = [Kind::Oma, Kind::Race, Kind::Bak, Kind::Gold, Kind::Wreck];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lock {
    None,
    Ring,
    Chain,
    U,
    /// a chain and then a U-lock (the golden bike)
    Double,
}

impl Lock {
    pub fn name(self) -> &'static str {
        match self {
            Lock::None => "GEEN SLOT",
            Lock::Ring => "RINGSLOT",
            Lock::Chain => "KETTINGSLOT",
            Lock::U => "BEUGELSLOT",
            Lock::Double => "DUBBEL SLOT",
        }
    }
}

pub struct KindDef {
    pub name: &'static str,
    /// cash when delivered
    pub value: i32,
    /// riding speed in px per frame
    pub speed: f32,
    pub lock: Lock,
}

impl Kind {
    pub fn def(self) -> &'static KindDef {
        &KIND_DEFS[self as usize]
    }
}

pub const KIND_DEFS: [KindDef; 5] = [
    KindDef { name: "OMAFIETS", value: 60, speed: 2.0, lock: Lock::Ring },
    KindDef { name: "RACEFIETS", value: 120, speed: 2.7, lock: Lock::U },
    KindDef { name: "BAKFIETS", value: 150, speed: 1.6, lock: Lock::Chain },
    KindDef { name: "GOUDEN FIETS", value: 500, speed: 2.3, lock: Lock::Double },
    KindDef { name: "ROESTBAK", value: 10, speed: 1.55, lock: Lock::None },
];

/// Frame colors of parked bikes (frame, dark).
pub const BIKE_COLORS: [(u32, u32); 8] = [
    (rgb(40, 40, 56), rgb(16, 16, 32)),
    (rgb(48, 88, 160), rgb(24, 48, 104)),
    (rgb(200, 48, 48), rgb(128, 24, 32)),
    (rgb(72, 136, 80), rgb(40, 88, 48)),
    (rgb(232, 232, 240), rgb(160, 168, 192)),
    (rgb(48, 160, 136), rgb(24, 104, 88)),
    (rgb(144, 96, 176), rgb(96, 56, 128)),
    (rgb(64, 168, 184), rgb(32, 112, 128)),
];

// ------------------------------------------------------------------ BOA's

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoaKind {
    Foot,
    Bike,
    /// the chief, with his dog
    Chief,
}

pub struct BoaSpec {
    pub name: &'static str,
    pub kind: BoaKind,
    /// patrol route along the street, and the depth they walk at
    pub x0: f32,
    pub x1: f32,
    pub y: f32,
    /// how far they see, px
    pub vision: f32,
    /// frames between glances back (on average)
    pub look_every: u32,
    /// takes coffee breaks at the kiosk
    pub coffee: bool,
}

// ------------------------------------------------------------------ stages

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Venue {
    Gracht,
    Station,
    Koningsdag,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Spot {
    /// an elm on the quay: hide behind it, blocks views
    Tree,
    /// the green iron urinal
    Krul,
    /// the glass recycling bin
    Glasbak,
    /// a street organ (Koningsdag)
    Organ,
    /// coffee kiosk: blocks views, BOA's take breaks here
    Kiosk,
}

impl Spot {
    pub fn hides(self) -> bool {
        !matches!(self, Spot::Kiosk)
    }
    /// Half the width of what it blocks, in depth px.
    pub fn block(self) -> f32 {
        match self {
            Spot::Tree => 5.0,
            Spot::Krul | Spot::Glasbak => 7.0,
            Spot::Organ => 7.0,
            Spot::Kiosk => 9.0,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Spot::Tree => "IEP",
            Spot::Krul => "KRUL",
            Spot::Glasbak => "GLASBAK",
            Spot::Organ => "DRAAIORGEL",
            Spot::Kiosk => "KOFFIE",
        }
    }
}

pub struct StageDef {
    pub name: &'static str,
    pub venue: Venue,
    pub tagline: [&'static str; 2],
    pub tip: [&'static str; 2],
    pub len: f32,
    /// seconds
    pub time: u32,
    pub order: &'static [Kind],
    /// parked bikes besides the order: (kind, count)
    pub bikes: &'static [(Kind, usize)],
    /// chance an omafiets has a chain instead of a ring lock
    pub oma_chain: f32,
    pub boas: &'static [BoaSpec],
    /// hide spots and blockers: (kind, x, y)
    pub spots: &'static [(Spot, f32, f32)],
    pub peds: usize,
    /// the orange crowd (Koningsdag)
    pub crowd: usize,
    pub herons: usize,
    pub cutters: u32,
    /// the BOA who heckles on the intro card
    pub boss: &'static str,
    pub heckles: &'static [&'static str],
}

pub const STAGES: [StageDef; 3] = [
    StageDef {
        name: "PRINSENGRACHT",
        venue: Venue::Gracht,
        tagline: ["A QUIET MORNING ON THE CANAL.", "HENK IS ON DUTY. AND ON COFFEE."],
        tip: ["TAP Z AT A RING LOCK. DON'T RUN.", "C: ACT CASUAL, OR HIDE BY A TREE."],
        len: 1024.0,
        time: 120,
        order: &[Kind::Oma, Kind::Oma, Kind::Race, Kind::Bak],
        bikes: &[(Kind::Oma, 8), (Kind::Race, 2), (Kind::Bak, 2), (Kind::Wreck, 2)],
        oma_chain: 0.1,
        boas: &[
            BoaSpec { name: "HENK", kind: BoaKind::Foot, x0: 110.0, x1: 640.0, y: 26.0, vision: 74.0, look_every: 700, coffee: true },
            BoaSpec { name: "INGRID", kind: BoaKind::Foot, x0: 560.0, x1: 980.0, y: 40.0, vision: 78.0, look_every: 560, coffee: false },
        ],
        spots: &[
            (Spot::Tree, 90.0, 59.0),
            (Spot::Tree, 250.0, 59.0),
            (Spot::Kiosk, 360.0, 3.0),
            (Spot::Tree, 430.0, 59.0),
            (Spot::Krul, 560.0, 58.0),
            (Spot::Glasbak, 700.0, 3.0),
            (Spot::Tree, 790.0, 59.0),
            (Spot::Tree, 950.0, 59.0),
        ],
        peds: 6,
        crowd: 0,
        herons: 2,
        cutters: 2,
        boss: "HANDHAVER HENK",
        heckles: &["IK ZIE ALLES!", "EVEN KOFFIE...", "NIET OP MIJN GRACHT!"],
    },
    StageDef {
        name: "CENTRAAL STATION",
        venue: Venue::Station,
        tagline: ["10,000 BIKES BY THE WATER.", "NOBODY WILL MISS ONE. RIGHT?"],
        tip: ["HIDE IN THE KRUL. NOBODY LOOKS THERE.", "FATIMA RIDES A BIKE. DON'T RACE HER."],
        len: 1152.0,
        time: 150,
        order: &[Kind::Race, Kind::Bak, Kind::Oma, Kind::Race, Kind::Oma],
        bikes: &[(Kind::Oma, 10), (Kind::Race, 4), (Kind::Bak, 2), (Kind::Wreck, 3)],
        oma_chain: 0.3,
        boas: &[
            BoaSpec { name: "RUUD", kind: BoaKind::Foot, x0: 80.0, x1: 600.0, y: 22.0, vision: 82.0, look_every: 480, coffee: false },
            BoaSpec { name: "PETRA", kind: BoaKind::Foot, x0: 540.0, x1: 1100.0, y: 44.0, vision: 82.0, look_every: 420, coffee: true },
            BoaSpec { name: "FATIMA", kind: BoaKind::Bike, x0: 40.0, x1: 1110.0, y: 33.0, vision: 70.0, look_every: 900, coffee: false },
        ],
        spots: &[
            (Spot::Tree, 120.0, 59.0),
            (Spot::Krul, 300.0, 58.0),
            (Spot::Glasbak, 420.0, 3.0),
            (Spot::Tree, 520.0, 59.0),
            (Spot::Kiosk, 690.0, 3.0),
            (Spot::Krul, 820.0, 58.0),
            (Spot::Tree, 960.0, 59.0),
            (Spot::Glasbak, 1060.0, 3.0),
        ],
        peds: 10,
        crowd: 0,
        herons: 2,
        cutters: 2,
        boss: "BOA FATIMA",
        heckles: &["STOPPEN! ...AUB.", "IK BEN SNELLER.", "NIET OP DE STOEP!"],
    },
    StageDef {
        name: "KONINGSDAG",
        venue: Venue::Koningsdag,
        tagline: ["KING'S DAY. THE WHOLE CITY IS ORANGE.", "THE MAYOR'S GOLDEN BIKE IS PARKED HERE."],
        tip: ["LOSE THEM IN THE ORANGE CROWD.", "BRUNO THE DOG SNIFFS OUT HIDERS."],
        len: 1280.0,
        time: 150,
        order: &[Kind::Gold, Kind::Bak, Kind::Race, Kind::Oma, Kind::Race],
        bikes: &[(Kind::Oma, 10), (Kind::Race, 3), (Kind::Bak, 2), (Kind::Wreck, 2)],
        oma_chain: 0.4,
        boas: &[
            BoaSpec { name: "RUUD", kind: BoaKind::Foot, x0: 60.0, x1: 600.0, y: 24.0, vision: 86.0, look_every: 360, coffee: false },
            BoaSpec { name: "PETRA", kind: BoaKind::Foot, x0: 700.0, x1: 1240.0, y: 42.0, vision: 86.0, look_every: 360, coffee: false },
            BoaSpec { name: "FATIMA", kind: BoaKind::Bike, x0: 40.0, x1: 1240.0, y: 32.0, vision: 72.0, look_every: 800, coffee: false },
            BoaSpec { name: "DE BOER", kind: BoaKind::Chief, x0: 480.0, x1: 860.0, y: 46.0, vision: 100.0, look_every: 300, coffee: false },
            BoaSpec { name: "SANDER", kind: BoaKind::Foot, x0: 880.0, x1: 1250.0, y: 20.0, vision: 82.0, look_every: 480, coffee: false },
            BoaSpec { name: "MIRJAM", kind: BoaKind::Foot, x0: 300.0, x1: 820.0, y: 14.0, vision: 80.0, look_every: 420, coffee: true },
        ],
        spots: &[
            (Spot::Tree, 100.0, 59.0),
            (Spot::Krul, 260.0, 58.0),
            (Spot::Organ, 400.0, 18.0),
            (Spot::Tree, 540.0, 59.0),
            (Spot::Kiosk, 640.0, 3.0),
            (Spot::Tree, 760.0, 59.0),
            (Spot::Glasbak, 900.0, 3.0),
            (Spot::Krul, 1020.0, 58.0),
            (Spot::Tree, 1160.0, 59.0),
        ],
        peds: 6,
        crowd: 22,
        herons: 1,
        cutters: 2,
        boss: "HOOFD-BOA DE BOER",
        heckles: &["BRUNO! ZOEK!", "OP KONINGSDAG?!", "BLIJF VAN DIE FIETS!"],
    },
];

/// Fine per catch, in euro.
pub const FINE: i32 = 95;
/// 2P: fining someone who did nothing.
pub const COMPLAINT: i32 = 50;
pub const MAX_FINES: u32 = 3;

pub const TICKER: &str = "AMSTERDAM HAS MORE BIKES THAN PEOPLE. DO THE MATH.      \
EVERY YEAR 15,000 BIKES ARE FISHED OUT OF THE CANALS.      \
YOUR BIKE WAS STOLEN. AGAIN. TIME TO TAKE ONE BACK.      \
BOA = BUITENGEWOON OPSPORINGSAMBTENAAR. SAY THAT FAST.      \
NEVER HIDE IN A KRUL ON A HOT DAY.      \
SJAAK BUYS ANYTHING WITH TWO WHEELS. NO QUESTIONS.      \
HELICO SOFT DOES NOT ENDORSE BIKE THEFT. MOSTLY.      ";

pub fn ordinal(place: usize) -> &'static str {
    ["1ST", "2ND", "3RD", "4TH"][place.min(3)]
}
