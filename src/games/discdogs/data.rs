//! The cast (four owners and their dogs), the three stages and all the
//! jokes. Every string here uses only characters of the 5x7 font.

use crate::engine::gfx::rgb;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Breed {
    Corgi,
    Collie,
    Afghan,
    Keeshond,
}

pub struct DogDef {
    pub name: &'static str,
    pub breed: Breed,
    pub breed_name: &'static str,
    /// 1..=5
    pub speed: i32,
    pub jump: i32,
    pub focus: i32,
    pub zoomies: i32,
    pub bio: &'static str,
    /// what the owner shouts when the dog snaps out of a squirrel stare
    pub good: &'static str,
    /// fur, fur shade, light (belly, muzzle), light shade, dark patches, dark shade
    pub coat: [u32; 6],
}

pub struct OwnerDef {
    pub name: &'static str,
    /// engine head style (0 spiky, 1 headband, 2 cap, 3 beanie, 4 mohawk, 5 ponytail)
    pub style: usize,
    pub skin: (u32, u32),
    pub hair: (u32, u32),
    pub shirt: (u32, u32),
    pub pants: (u32, u32),
    pub acc: (u32, u32),
    pub cap: (u32, u32),
    pub bio: &'static str,
    /// what they say on the intro card when you face them
    pub heckle: [&'static str; 2],
}

pub struct Pair {
    pub owner: OwnerDef,
    pub dog: DogDef,
    /// the pair's color on menus and in the HUD
    pub color: u32,
}

const SKIN_LIGHT: (u32, u32) = (rgb(248, 200, 160), rgb(216, 144, 104));
const SKIN_TAN: (u32, u32) = (rgb(224, 160, 104), rgb(176, 112, 64));
const SKIN_DARK: (u32, u32) = (rgb(160, 96, 56), rgb(112, 64, 32));
const SKIN_PINK: (u32, u32) = (rgb(248, 184, 160), rgb(208, 128, 112));

pub const PAIRS: [Pair; 4] = [
    Pair {
        owner: OwnerDef {
            name: "DEBBIE",
            style: 5,
            skin: SKIN_DARK,
            hair: (rgb(248, 104, 48), rgb(176, 56, 24)),
            shirt: (rgb(248, 72, 168), rgb(176, 32, 120)),
            pants: (rgb(64, 200, 232), rgb(32, 128, 184)),
            acc: (rgb(248, 232, 64), rgb(200, 160, 24)),
            cap: (rgb(248, 232, 64), rgb(200, 160, 24)),
            bio: "TEACHES AEROBICS. TO DOGS.",
            heckle: ["FEEL THE BURN,", "PUPPY!"],
        },
        dog: DogDef {
            name: "BISCUIT",
            breed: Breed::Corgi,
            breed_name: "CORGI",
            speed: 5,
            jump: 2,
            focus: 3,
            zoomies: 3,
            bio: "BUILT LOW TO THE GROUND. UNSTOPPABLE.",
            good: "GOOD GIRL!",
            coat: [rgb(232, 144, 56), rgb(176, 88, 32), rgb(248, 240, 224), rgb(208, 192, 176), rgb(144, 72, 32), rgb(104, 48, 24)],
        },
        color: rgb(248, 72, 168),
    },
    Pair {
        owner: OwnerDef {
            name: "COACH CHAD",
            style: 2,
            skin: SKIN_LIGHT,
            hair: (rgb(248, 216, 72), rgb(200, 152, 32)),
            shirt: (rgb(48, 168, 72), rgb(24, 112, 48)),
            pants: (rgb(232, 232, 240), rgb(168, 176, 200)),
            acc: (rgb(248, 248, 248), rgb(168, 176, 200)),
            cap: (rgb(232, 56, 48), rgb(152, 24, 32)),
            bio: "WHISTLES IN HIS SLEEP.",
            heckle: ["TURBO HAS A PHD.", "YOUR DOG HAS FLEAS."],
        },
        dog: DogDef {
            name: "TURBO",
            breed: Breed::Collie,
            breed_name: "BORDER COLLIE",
            speed: 4,
            jump: 4,
            focus: 3,
            zoomies: 2,
            bio: "HAS A PHD IN FRISBEE.",
            good: "GOOD BOY!",
            coat: [rgb(56, 48, 72), rgb(32, 24, 48), rgb(248, 248, 248), rgb(200, 200, 216), rgb(56, 48, 72), rgb(32, 24, 48)],
        },
        color: rgb(64, 200, 96),
    },
    Pair {
        owner: OwnerDef {
            name: "LADY PEMBERTON",
            style: 1,
            skin: SKIN_PINK,
            hair: (rgb(232, 232, 248), rgb(176, 176, 200)),
            shirt: (rgb(152, 80, 224), rgb(96, 48, 160)),
            pants: (rgb(152, 80, 224), rgb(96, 48, 160)),
            acc: (rgb(248, 216, 48), rgb(192, 144, 16)),
            cap: (rgb(248, 216, 48), rgb(192, 144, 16)),
            bio: "OWNS 14 TIARAS AND A HOVERCRAFT.",
            heckle: ["HOW QUAINT. YOUR DOG", "HAS ONLY ONE NAME."],
        },
        dog: DogDef {
            name: "SIR WOOFINGTON",
            breed: Breed::Afghan,
            breed_name: "AFGHAN HOUND",
            speed: 2,
            jump: 5,
            focus: 1,
            zoomies: 5,
            bio: "HAIR BY VIDAL BASSOON.",
            good: "GOOD SIR!",
            coat: [rgb(240, 216, 168), rgb(200, 160, 104), rgb(248, 240, 208), rgb(224, 200, 152), rgb(160, 112, 64), rgb(112, 72, 40)],
        },
        color: rgb(176, 112, 248),
    },
    Pair {
        owner: OwnerDef {
            name: "OOM JOOP",
            style: 3,
            skin: SKIN_LIGHT,
            hair: (rgb(200, 200, 208), rgb(136, 136, 152)),
            shirt: (rgb(248, 136, 32), rgb(192, 88, 16)),
            pants: (rgb(48, 72, 152), rgb(24, 40, 104)),
            acc: (rgb(248, 248, 248), rgb(168, 176, 200)),
            cap: (rgb(48, 96, 216), rgb(24, 56, 152)),
            bio: "BIKED HERE FROM DELFT. UPHILL.",
            heckle: ["KEES DOES NOT CHASE", "SQUIRRELS. KEES IS ABOVE IT."],
        },
        dog: DogDef {
            name: "KEES",
            breed: Breed::Keeshond,
            breed_name: "KEESHOND",
            speed: 3,
            jump: 3,
            focus: 5,
            zoomies: 2,
            bio: "LOVES CHEESE. FEARS NOTHING.",
            good: "GOEIE HOND!",
            coat: [rgb(168, 168, 168), rgb(112, 112, 120), rgb(224, 224, 216), rgb(176, 176, 168), rgb(56, 56, 64), rgb(32, 32, 40)],
        },
        color: rgb(248, 152, 32),
    },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Park,
    Pier,
    Bowl,
}

pub const STAGES: [Stage; 3] = [Stage::Park, Stage::Pier, Stage::Bowl];

impl Stage {
    pub fn name(self) -> &'static str {
        match self {
            Stage::Park => "SUNNY HILL PARK",
            Stage::Pier => "PIER 92",
            Stage::Bowl => "THE GRAND BOWL",
        }
    }
    pub fn tagline(self) -> &'static str {
        match self {
            Stage::Park => "SQUIRRELS: 14. DOGS: 0.",
            Stage::Pier => "THE GULLS ARE WATCHING.",
            Stage::Bowl => "60,000 FANS. ONE FRISBEE.",
        }
    }
    pub fn index(self) -> usize {
        match self {
            Stage::Park => 0,
            Stage::Pier => 1,
            Stage::Bowl => 2,
        }
    }
    /// steady wind for a toss is drawn from -max..max (wind units, see sim)
    pub fn wind_max(self) -> f32 {
        match self {
            Stage::Park => 0.5,
            Stage::Pier => 0.9,
            Stage::Bowl => 0.7,
        }
    }
}

pub const TICKER: &str = "DISC DOGS! THE WORLD'S GOODEST FRISBEE CHAMPIONSHIP  *  NO SQUIRRELS WERE HARMED. SEVERAL WERE STARED AT  *  \
SEAGULLS ARE NOT ALLOWED TO ENTER. THEY ENTER ANYWAY  *  SIR WOOFINGTON'S HAIR HAS ITS OWN AGENT  *  \
REMEMBER: A DISC IN THE POND IS A SOGGY DISC  *  KEES WOULD LIKE TO SPEAK TO THE MANAGER ABOUT THE CHEESE  *  \
SPONSORED BY BARK COLA: IT'S RUFF OUT THERE  *  ";

pub const DEMO_TIPS: [&str; 4] = [
    "TIP: HOLD C WHEN YOU LET GO FOR A FLOATER",
    "TIP: TAP RIGHT TWICE FOR ZOOMIES",
    "TIP: SQUIRREL? TAP C. GOOD BOY!",
    "TIP: FINISH THE FLIP BEFORE YOU LAND",
];

/// What the announcer says after a miss.
pub const MISS_QUIPS: [&str; 6] = [
    "THE DOG IS NOT MAD. JUST DISAPPOINTED.",
    "THAT DISC HAD A FAMILY.",
    "SOMEWHERE, A SQUIRREL IS LAUGHING.",
    "GOOD THROW. WRONG DIRECTION.",
    "THE DOG WOULD LIKE A WORD WITH YOU.",
    "PHYSICS: 1. DOG: 0.",
];

/// Where the CPU opponents and the 2P mode play: stage per ladder step.
pub fn stage_for(step: usize) -> Stage {
    STAGES[step.min(2)]
}
