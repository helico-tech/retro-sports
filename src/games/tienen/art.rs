//! TIENEN! art. Props and animals are string bitmaps in color roles that
//! get a 1px outline when they are built (the `Bmp` code is a copy of
//! Stalen Zenuwen's: games do not depend on each other). Kids are engine
//! `person()` sprites from our own skeletons: running, volleys, heels,
//! headers, bicycle kicks, keeper dives, and bending over for the
//! billenbrand. Everything is built once in `Art::new()`.

use super::data::*;
use crate::engine::gfx::{line_points, rgb, Frame, H};
use crate::engine::sprites::{self as sp, person, Palette, Skel, Sprite, ROLES};
use crate::engine::ui::OUTLINE;

// ------------------------------------------------------------------ bitmaps

/// Role keys shared by every prop bitmap: the position in this string is
/// the palette index ('.' = transparent, 'O' = outline).
pub const KEYS: &str = ".OWwgKRrYyBbPGHSCcAaLNoTtDdMmE";

pub const I_OUT: u8 = 1;
pub const I_C: u8 = 16;
pub const I_CD: u8 = 17;
pub const I_A: u8 = 18;
pub const I_AD: u8 = 19;

pub type Pal = [u32; 30];

/// The default prop palette. C/c (frame) and A/a (accent) get swapped.
pub const PROPS: Pal = [
    0,
    OUTLINE,
    rgb(248, 248, 248),
    rgb(200, 200, 216),
    rgb(136, 136, 160),
    rgb(48, 48, 64),
    rgb(216, 48, 48),
    rgb(144, 24, 40),
    rgb(248, 216, 48),
    rgb(216, 136, 32),
    rgb(160, 104, 56),
    rgb(104, 64, 32),
    rgb(240, 160, 168),
    rgb(96, 176, 88),
    rgb(240, 224, 176),
    rgb(248, 200, 160),
    rgb(232, 64, 48),
    rgb(152, 32, 40),
    rgb(64, 120, 216),
    rgb(32, 64, 152),
    rgb(160, 216, 248),
    rgb(40, 48, 96),
    rgb(248, 136, 24),
    rgb(232, 160, 96),
    rgb(176, 96, 48),
    rgb(48, 112, 72),
    rgb(24, 64, 48),
    rgb(112, 128, 152),
    rgb(72, 80, 104),
    rgb(248, 248, 168),
];

pub fn pal_with(frame: (u32, u32), acc: (u32, u32)) -> Pal {
    let mut p = PROPS;
    p[I_C as usize] = frame.0;
    p[I_CD as usize] = frame.1;
    p[I_A as usize] = acc.0;
    p[I_AD as usize] = acc.1;
    p
}

#[derive(Clone)]
pub struct Bmp {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
}

impl Bmp {
    pub fn new(w: i32, h: i32) -> Self {
        Bmp { w, h, px: vec![0; (w * h) as usize] }
    }

    pub fn parse(rows: &[&str]) -> Self {
        let h = rows.len() as i32;
        let w = rows.iter().map(|r| r.len()).max().unwrap_or(0) as i32;
        let mut b = Bmp::new(w, h);
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let v = KEYS.find(ch).unwrap_or(0) as u8;
                b.px[y * w as usize + x] = v;
            }
        }
        b
    }

    /// Parsed and outlined.
    pub fn art(rows: &[&str]) -> Self {
        let mut b = Bmp::parse(rows).padded(1);
        b.outline();
        b
    }

    pub fn padded(&self, p: i32) -> Self {
        let mut b = Bmp::new(self.w + 2 * p, self.h + 2 * p);
        for y in 0..self.h {
            for x in 0..self.w {
                b.set(x + p, y + p, self.get(x, y));
            }
        }
        b
    }

    pub fn get(&self, x: i32, y: i32) -> u8 {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize]
        } else {
            0
        }
    }

    pub fn set(&mut self, x: i32, y: i32, v: u8) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = v;
        }
    }

    pub fn line(&mut self, a: (i32, i32), b: (i32, i32), size: i32, v: u8) {
        let lo = -(size / 2);
        let hi = lo + size;
        line_points(a.0, a.1, b.0, b.1, |x, y| {
            for dy in lo..hi {
                for dx in lo..hi {
                    self.set(x + dx, y + dy, v);
                }
            }
        });
    }

    pub fn stamp(&mut self, x: i32, y: i32, rows: &[&str]) {
        for (ry, row) in rows.iter().enumerate() {
            for (rx, ch) in row.chars().enumerate() {
                let v = KEYS.find(ch).unwrap_or(0) as u8;
                if v != 0 {
                    self.set(x + rx as i32, y + ry as i32, v);
                }
            }
        }
    }

    /// 1px outline around everything that is not transparent.
    pub fn outline(&mut self) {
        let src = self.px.clone();
        let at = |x: i32, y: i32| -> u8 {
            if x >= 0 && y >= 0 && x < self.w && y < self.h {
                src[(y * self.w + x) as usize]
            } else {
                0
            }
        };
        for y in 0..self.h {
            for x in 0..self.w {
                if at(x, y) != 0 {
                    continue;
                }
                if [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| {
                    let v = at(x + dx, y + dy);
                    v != 0 && v != I_OUT
                }) {
                    self.px[(y * self.w + x) as usize] = I_OUT;
                }
            }
        }
    }

    /// Draw with the top-left corner at (x, y).
    pub fn draw(&self, f: &mut Frame, x: i32, y: i32, pal: &Pal, flip: bool) {
        self.draw_clip(f, x, y, pal, flip, H);
    }

    /// Draw with its bottom-centre at (x, y).
    pub fn draw_foot(&self, f: &mut Frame, x: i32, y: i32, pal: &Pal, flip: bool) {
        self.draw(f, x - self.w / 2, y - self.h + 1, pal, flip);
    }

    /// Draw only rows above screen y `clip` (things in water).
    pub fn draw_clip(&self, f: &mut Frame, x: i32, y: i32, pal: &Pal, flip: bool, clip: i32) {
        for sy in 0..self.h {
            let py = y + sy;
            if py >= clip || py < 0 {
                continue;
            }
            for sx in 0..self.w {
                let v = self.px[(sy * self.w + sx) as usize];
                if v != 0 {
                    let px = if flip { x + self.w - 1 - sx } else { x + sx };
                    f.set(px, py, pal[v as usize]);
                }
            }
        }
    }

    /// One flat color (silhouettes, reflections).
    pub fn draw_flat(&self, f: &mut Frame, x: i32, y: i32, c: u32, flip: bool) {
        for sy in 0..self.h {
            for sx in 0..self.w {
                if self.px[(sy * self.w + sx) as usize] != 0 {
                    let px = if flip { x + self.w - 1 - sx } else { x + sx };
                    f.set(px, y + sy, c);
                }
            }
        }
    }
}


// ------------------------------------------------------------------ props

pub const BALL: [[&str; 6]; 2] = [
    [".WWKW.", "WKKWWW", "WWWWKW", "KWWKKW", "WWKWWW", ".WWWW."],
    [".WKWW.", "WWWWKW", "KKWWWW", "WWWKKW", "WKWWWW", ".WWKW."],
];

/// A small hatchback with a grumpy driver, facing right (C/c = body).
pub const CAR: [&str; 24] = [
    "..................CCCCCCCCCCCCCCCCCC..........................",
    ".................CLLLLLLLLLcLLLLLLLLLC........................",
    "................CLLLLLLLLLLcLLLLLLLLLLC.......................",
    "...............CLLLLLLLLLLLcLLLLLHHLLLLC......................",
    "..............CLLLLLLLLLLLLcLLLLHHHHLLLLC.....................",
    ".............CLLLLLLLLLLLLLcLLLLSSKSLLLLLC....................",
    "............CLLLLLLLLLLLLLLcLLLLSSSSLLLLLLC...................",
    "...........CLLLLLLLLLLLLLLLcLLLLATAALLLLLLLC..................",
    "..CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC........",
    ".CCCCCCCCCCCCCCCCCCCCCCCCCCcCCCCCCCCCCCCCCCCCCCCCCCCCCCC......",
    "CCCCCCCCCCCCCCCCCCCCCCCCCCCcCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC....",
    "CCCCCCCCCCCCCCCCCCCCCCCCCCCcCCCCCCCCCCCCCCCCCCCCCCCCCCCCYYY...",
    "RCCCCCCCCCCCCCCCKKCCCCCCCCCcCCCCCCCCKKCCCCCCCCCCCCCCCCCCYYYw..",
    "RCCCCCCCCCCCCCCCCCCCCCCCCCCcCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCw..",
    "RCCCCCCCCCCCCCCCCCCCCCCCCCCcCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCw.",
    "wCCCCCCCCCCCCCCCCCCCCCCCCCCcCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCw.",
    "wwcccccKKKKKKKccccccccccccccccccccccccccccKKKKKKKcccccccccwwww",
    ".wcccKKKwwwwwKKKcccccccccccccccccccccccccKKKwwwwwKKKcccccccww.",
    "....KKwwgggggwwKK.......................KKwwgggggwwKK.........",
    "....KwwgggKgggwwK.......................KwwgggKgggwwK.........",
    "....KwwggKKKggwwK.......................KwwggKKKggwwK.........",
    "....KwwgggKgggwwK.......................KwwgggKgggwwK.........",
    "....KKwwgggggwwKK.......................KKwwgggggwwKK.........",
    "......KKKKKKKKK...........................KKKKKKKKK...........",
];

/// FIFI the poodle, facing right: two run frames. A pink bow (P) and
/// collar, pom-poms on the tail and legs.
pub const DOG: [[&str; 14]; 2] = [
    [
        "............PP.....",
        "...........PWWW....",
        "..........WWWWWW...",
        "..........WWKWWWWw.",
        "..........WWWWWWW..",
        "...WW......PPPWW...",
        "..WWWW....WWWWW....",
        "...WW.WWWWWWWWWW...",
        "......WWWWWWWWWW...",
        "......WWWWWWWWWW...",
        ".......WW....WW....",
        ".......w......w....",
        "......WWW....WWW...",
        "...................",
    ],
    [
        "............PP.....",
        "...........PWWW....",
        "..........WWWWWW...",
        "..........WWKWWWWw.",
        "..........WWWWWWW..",
        "..WW.......PPPWW...",
        ".WWWW.....WWWWW....",
        "..WW..WWWWWWWWWW...",
        "......WWWWWWWWWW...",
        "......WWWWWWWWWW...",
        "......WW......WW...",
        ".....w.........w...",
        "....WWW.......WWW..",
        "...................",
    ],
];

/// Pigeons: standing, pecking, two wing frames.
pub const PIGEON: [[&str; 6]; 4] = [
    ["....gg..", "...gKgy.", "gggDgg..", ".gwwgg..", "..ggg...", "...y.y.."],
    ["........", "........", "ggggg...", ".gwwggg.", "..gggKgy", "...y.y.."],
    ["..g.....", "..gg....", ".gggg...", "gggDggKy", ".gwwgg..", "........"],
    ["........", "........", "gggDggKy", ".gwwgg..", "gggg....", "g......."],
];

/// The keeper's glove (HUD).
pub const GLOVE: [&str; 7] = [".A.A.A.", ".AAAAA.", "AAAAAAA", "AAAAAAa", ".AAAAa.", ".WWWW..", ".WWWW.."];

/// The ice cream van, facing right.
pub const VAN: [&str; 17] = [
    "...........HH..............",
    "..........HTTH.............",
    "...........yy..............",
    "...........yy..............",
    ".PPPPPPPPPPPPPPPPPPPP......",
    ".PWWWWWWWWWWWWWWWWWWP......",
    ".PWLLLLLLLLLLLLLLLLWPPPP...",
    ".PWLLLLLLLLLLLLLLLLWPLLLP..",
    ".PWWWWWWWWWWWWWWWWWWPLLLLP.",
    ".PPPPPPPPPPPPPPPPPPPPPPPPPP",
    ".PPWPPWPPWPPWPPWPPWPPPPPPPY",
    ".PPPPPPPPPPPPPPPPPPPPPPPPPP",
    "wwPPKKKPPPPPPPPPPPPPKKKPPww",
    "..PKKwKKPPPPPPPPPPPKKwKKP..",
    "...KwgwK...........KwgwK...",
    "...KKwKK...........KKwKK...",
    "....KKK.............KKK....",
];

/// A garden gnome (Mevrouw Knorr's).
pub const GNOME: [&str; 10] = ["..C...", "..CC..", ".CCCC.", ".CCCCc", "..SS..", ".WWWW.", "AWWWWA", ".AAAA.", ".A..A.", "BB..BB"];

/// A sunflower.
pub const SUNFLOWER: [&str; 12] =
    [".Y.Y.", "YYYYY", "YbbbY", "YbbbY", "YYYYY", ".Y.Y.", "..G..", ".GG..", "..G..", "..GG.", "..G..", "..G.."];

// ------------------------------------------------------------------ kids

const STAND: Skel = [(12, 5), (12, 10), (12, 19), (8, 15), (8, 20), (16, 15), (16, 20), (10, 24), (10, 29), (14, 24), (14, 29)];
const RUN: [Skel; 4] = [
    [(14, 6), (13, 11), (12, 19), (15, 14), (18, 12), (10, 15), (8, 18), (9, 24), (6, 27), (16, 23), (17, 29)],
    [(14, 5), (13, 10), (12, 18), (12, 15), (13, 19), (14, 15), (15, 19), (11, 23), (9, 27), (14, 23), (13, 29)],
    [(14, 6), (13, 11), (12, 19), (10, 15), (8, 18), (16, 14), (19, 12), (16, 23), (17, 29), (9, 24), (6, 27)],
    [(14, 5), (13, 10), (12, 18), (14, 15), (15, 19), (12, 15), (13, 19), (14, 23), (13, 29), (11, 23), (9, 27)],
];
const JUMP: Skel = [(12, 4), (12, 9), (12, 18), (8, 12), (5, 10), (16, 9), (19, 6), (15, 21), (11, 25), (17, 20), (14, 25)];
const VICTORY: Skel = [(12, 5), (12, 10), (12, 19), (8, 7), (4, 2), (16, 7), (20, 2), (10, 24), (9, 29), (14, 24), (15, 29)];
const CHEER: Skel = [(12, 5), (12, 10), (12, 19), (8, 15), (7, 20), (16, 7), (19, 2), (10, 24), (9, 29), (14, 24), (15, 29)];
const READY: Skel = [(10, 8), (11, 13), (12, 21), (16, 17), (19, 17), (17, 13), (20, 12), (9, 25), (7, 29), (16, 25), (17, 29)];
const HIGH: Skel = [(12, 5), (12, 10), (12, 19), (14, 6), (16, 2), (16, 6), (19, 2), (10, 24), (9, 29), (14, 24), (15, 29)];
const HOLD: Skel = [(12, 5), (12, 10), (12, 19), (14, 15), (17, 15), (16, 16), (18, 15), (10, 24), (9, 29), (14, 24), (15, 29)];
const PUNCH: Skel = [(14, 5), (13, 10), (12, 19), (9, 14), (7, 17), (17, 7), (21, 4), (9, 24), (6, 28), (16, 24), (18, 29)];
const ROLL: Skel = [(16, 10), (14, 14), (11, 20), (11, 16), (10, 19), (17, 20), (20, 26), (8, 25), (6, 29), (15, 25), (16, 29)];
const DOWN: Skel = [(5, 25), (9, 27), (17, 27), (10, 23), (12, 21), (12, 30), (15, 30), (20, 24), (23, 27), (21, 27), (23, 29)];
const DIVE: Skel = [(17, 22), (13, 25), (6, 25), (19, 26), (22, 25), (19, 24), (23, 23), (2, 24), (0, 22), (2, 26), (0, 26)];
/// volley: the front leg out at knee height, then the follow-through
const KICK: [Skel; 2] = [
    [(10, 5), (10, 10), (11, 19), (6, 13), (3, 12), (14, 14), (16, 17), (10, 24), (10, 29), (16, 21), (21, 19)],
    [(9, 6), (10, 11), (11, 19), (6, 14), (3, 13), (14, 14), (16, 17), (10, 24), (10, 29), (17, 18), (21, 13)],
];
/// heel: the front leg flicks back
const HEEL: Skel = [(14, 6), (13, 11), (12, 19), (9, 15), (8, 19), (16, 15), (18, 18), (12, 24), (12, 29), (9, 24), (4, 22)];
const HEADER: Skel = [(16, 5), (14, 10), (11, 19), (9, 14), (6, 17), (11, 15), (9, 18), (9, 24), (8, 29), (13, 24), (14, 29)];
const HEADER_JUMP: Skel = [(16, 4), (14, 9), (11, 18), (9, 12), (5, 13), (10, 13), (6, 15), (13, 22), (9, 25), (14, 21), (12, 26)];
/// chest: leaning back, arms out
const CHEST: Skel = [(10, 5), (11, 10), (12, 19), (7, 12), (4, 14), (16, 12), (19, 14), (10, 24), (9, 29), (14, 24), (15, 29)];
/// bicycle kick: falling back, then flat with the leg over the head
const BIKE: [Skel; 2] = [
    [(7, 9), (9, 13), (14, 19), (6, 13), (3, 11), (11, 15), (13, 12), (16, 24), (18, 29), (18, 15), (17, 9)],
    [(4, 20), (8, 19), (16, 16), (7, 23), (4, 26), (10, 24), (9, 28), (19, 20), (22, 24), (16, 9), (10, 4)],
];
const SULK: Skel = [(13, 8), (12, 12), (12, 20), (10, 16), (10, 21), (14, 16), (14, 21), (11, 25), (11, 29), (13, 25), (13, 29)];
/// billenbrand: bent over, the bottom out
const BENT: Skel = [(19, 17), (16, 15), (9, 15), (15, 19), (15, 23), (17, 19), (17, 24), (9, 22), (9, 29), (11, 22), (11, 29)];
const OUCH: Skel = [(13, 4), (13, 9), (12, 17), (9, 13), (7, 17), (10, 14), (8, 18), (10, 22), (9, 26), (15, 21), (17, 24)];
const SIT: Skel = [(12, 10), (12, 15), (11, 23), (9, 19), (14, 22), (15, 19), (18, 22), (16, 23), (17, 29), (18, 23), (19, 29)];

pub struct People {
    pub stand: Sprite,
    pub run: [Sprite; 4],
    pub jump: Sprite,
    pub victory: Sprite,
    pub cheer: Sprite,
    pub ready: Sprite,
    pub high: Sprite,
    pub hold: Sprite,
    pub punch: Sprite,
    pub roll: Sprite,
    pub down: Sprite,
    pub dive: Sprite,
    pub kick: [Sprite; 2],
    pub heel: Sprite,
    pub header: Sprite,
    pub header_jump: Sprite,
    pub chest: Sprite,
    pub bike: [Sprite; 2],
    pub sulk: Sprite,
    pub bent: Sprite,
    pub ouch: Sprite,
    pub sit: Sprite,
}

/// A bigger kid: the skeleton stretched up from the feet and widened.
fn big(k: &Skel) -> Skel {
    k.map(|(x, y)| (12 + ((x - 12) as f32 * 1.1).round() as i32, 30 - ((30 - y) as f32 * 1.12).round() as i32))
}

impl People {
    fn new(style: usize) -> Self {
        People::build(style, false)
    }

    fn build(style: usize, large: bool) -> Self {
        let p = |k: &Skel| person(&if large { big(k) } else { *k }, style, false, false);
        let pf = |k: &Skel, flat: bool, side: bool| person(&if large { big(k) } else { *k }, style, flat, side);
        People {
            stand: p(&STAND),
            run: RUN.map(|k| p(&k)),
            jump: p(&JUMP),
            victory: p(&VICTORY),
            cheer: p(&CHEER),
            ready: p(&READY),
            high: p(&HIGH),
            hold: p(&HOLD),
            punch: p(&PUNCH),
            roll: p(&ROLL),
            down: pf(&DOWN, true, true),
            dive: pf(&DIVE, true, false),
            kick: KICK.map(|k| p(&k)),
            heel: p(&HEEL),
            header: p(&HEADER),
            header_jump: p(&HEADER_JUMP),
            chest: p(&CHEST),
            bike: [p(&BIKE[0]), pf(&BIKE[1], true, true)],
            sulk: p(&SULK),
            bent: p(&BENT),
            ouch: p(&OUCH),
            sit: p(&SIT),
        }
    }
}

fn base_pal() -> Palette {
    let mut pal = [0u32; ROLES];
    pal[sp::OUT as usize] = OUTLINE;
    pal[sp::WHITE as usize] = rgb(248, 248, 248);
    pal[sp::SHOE as usize] = rgb(248, 248, 248);
    pal[sp::EYE as usize] = rgb(16, 16, 32);
    pal
}

pub fn kid_palette(id: usize) -> Palette {
    let d = kid(id);
    let mut pal = base_pal();
    pal[sp::SKIN as usize] = d.skin.0;
    pal[sp::SKIN_D as usize] = d.skin.1;
    pal[sp::HAIR as usize] = d.hair.0;
    pal[sp::HAIR_D as usize] = d.hair.1;
    pal[sp::SHIRT as usize] = d.shirt.0;
    pal[sp::SHIRT_D as usize] = d.shirt.1;
    pal[sp::PANTS as usize] = d.pants.0;
    pal[sp::PANTS_D as usize] = d.pants.1;
    pal[sp::ACC as usize] = d.acc.0;
    pal[sp::ACC_D as usize] = d.acc.1;
    pal[sp::CAP as usize] = d.cap.0;
    pal[sp::CAP_D as usize] = d.cap.1;
    pal
}

/// Grown-ups in the background: a mum, the janitor, Mevrouw Knorr.
pub fn adult_palette(k: usize) -> Palette {
    let mut pal = base_pal();
    let (skin, hair, shirt, pants) = match k {
        // Timmy's mum in her dressing gown
        0 => ((rgb(248, 208, 176), rgb(216, 152, 120)), (rgb(176, 72, 40), rgb(120, 40, 24)), (rgb(240, 160, 168), rgb(200, 112, 128)), (rgb(240, 160, 168), rgb(200, 112, 128))),
        // the janitor in his grey coat
        1 => ((rgb(224, 160, 104), rgb(176, 112, 64)), (rgb(136, 136, 160), rgb(96, 96, 120)), (rgb(136, 136, 160), rgb(96, 96, 120)), (rgb(72, 80, 104), rgb(48, 56, 72))),
        // Mevrouw Knorr, purple cardigan, white perm
        _ => ((rgb(248, 216, 184), rgb(224, 168, 128)), (rgb(232, 232, 240), rgb(184, 184, 200)), (rgb(136, 72, 184), rgb(88, 40, 128)), (rgb(104, 96, 88), rgb(72, 64, 56))),
    };
    pal[sp::SKIN as usize] = skin.0;
    pal[sp::SKIN_D as usize] = skin.1;
    pal[sp::HAIR as usize] = hair.0;
    pal[sp::HAIR_D as usize] = hair.1;
    pal[sp::SHIRT as usize] = shirt.0;
    pal[sp::SHIRT_D as usize] = shirt.1;
    pal[sp::PANTS as usize] = pants.0;
    pal[sp::PANTS_D as usize] = pants.1;
    pal[sp::ACC as usize] = shirt.1;
    pal[sp::ACC_D as usize] = shirt.1;
    pal[sp::CAP as usize] = hair.0;
    pal[sp::CAP_D as usize] = hair.1;
    pal
}

/// Car colors: body and its dark.
pub const CAR_COLORS: [(u32, u32); 4] = [
    (rgb(232, 64, 48), rgb(152, 32, 40)),
    (rgb(64, 120, 216), rgb(32, 64, 152)),
    (rgb(248, 216, 48), rgb(200, 152, 32)),
    (rgb(96, 176, 88), rgb(48, 112, 72)),
];

pub fn car_pal(k: usize) -> Pal {
    pal_with(CAR_COLORS[k % 4], CAR_COLORS[k % 4])
}

/// Draw a person sprite with the feet at (x, y).
pub fn blit_person(f: &mut Frame, s: &Sprite, x: i32, y: i32, flip: bool, pal: &Palette) {
    sp::blit(f, s, x, y, flip, pal);
}

/// The same, but only the rows above screen y `clip` (behind a wall).
pub fn blit_person_clip(f: &mut Frame, s: &Sprite, x: i32, y: i32, flip: bool, pal: &Palette, clip: i32) {
    for sy in 0..s.h {
        let py = y - sp::OY + sy;
        if py >= clip || py < 0 {
            continue;
        }
        for sx in 0..s.w {
            let v = s.px[(sy * s.w + sx) as usize];
            if v != sp::T {
                let px = if flip { x + sp::OX - sx } else { x - sp::OX + sx };
                f.set(px, py, pal[v as usize]);
            }
        }
    }
}

/// Everything, built once.
pub struct Art {
    /// one set of poses per head style
    pub people: Vec<People>,
    /// Gijs, too big for the goal
    pub gijs: People,
    pub ball: [Bmp; 2],
    pub car: Bmp,
    pub dog: [Bmp; 2],
    pub pigeon: [Bmp; 4],
    pub glove: Bmp,
    pub van: Bmp,
    pub gnome: Bmp,
    pub sunflower: Bmp,
}

impl Art {
    pub fn new() -> Self {
        Art {
            people: (0..sp::HAIR_STYLES).map(People::new).collect(),
            gijs: People::build(kid(2).style, true),
            ball: [Bmp::art(&BALL[0]), Bmp::art(&BALL[1])],
            car: Bmp::art(&CAR),
            dog: [Bmp::art(&DOG[0]), Bmp::art(&DOG[1])],
            pigeon: [Bmp::art(&PIGEON[0]), Bmp::art(&PIGEON[1]), Bmp::art(&PIGEON[2]), Bmp::art(&PIGEON[3])],
            glove: Bmp::art(&GLOVE),
            van: Bmp::art(&VAN),
            gnome: Bmp::art(&GNOME),
            sunflower: Bmp::art(&SUNFLOWER),
        }
    }

    pub fn style(&self, s: usize) -> &People {
        &self.people[s % self.people.len()]
    }

    /// The poses of one kid (Gijs is bigger than the rest).
    pub fn kid(&self, id: usize) -> &People {
        if id == 2 {
            &self.gijs
        } else {
            self.style(kid(id).style)
        }
    }
}

impl Default for Art {
    fn default() -> Self {
        Self::new()
    }
}
