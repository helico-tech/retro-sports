//! TEGENWIND! art. Props, animals and bikes are string bitmaps in color
//! roles (one shared key string, palette-swapped per use) that get a 1px
//! outline when they are built. Riders are engine `person()` sprites made
//! from cycling skeletons: feet on the pedal circle, knees solved with a
//! two-bone IK, eight crank frames per bike posture. Everything is built
//! once in `Art::new()`.

use super::data::*;
use crate::engine::gfx::{line_points, mix, rgb, shade, Frame};
use crate::engine::sprites::{self as sp, person, Palette, Skel, Sprite, ROLES};
use crate::engine::ui::OUTLINE;

// ------------------------------------------------------------------ bitmaps

/// Role keys shared by every prop bitmap: the position in this string is
/// the palette index ('.' = transparent, 'O' = outline).
pub const KEYS: &str = ".OWwgKRrYyBbPGHSCcAaLNoTt";

pub const I_OUT: u8 = 1;
pub const I_C: u8 = 16;
pub const I_CD: u8 = 17;
pub const I_A: u8 = 18;
pub const I_AD: u8 = 19;

pub type Pal = [u32; 25];

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
    rgb(72, 168, 72),
    rgb(240, 224, 176),
    rgb(248, 200, 160),
    rgb(232, 64, 48),
    rgb(152, 32, 40),
    rgb(64, 120, 216),
    rgb(32, 64, 152),
    rgb(160, 216, 248),
    rgb(48, 72, 168),
    rgb(248, 136, 24),
    rgb(232, 160, 96),
    rgb(176, 96, 48),
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
        for sy in 0..self.h {
            let py = y + sy;
            if !(0..224).contains(&py) {
                continue;
            }
            for sx in 0..self.w {
                let v = self.px[(sy * self.w + sx) as usize];
                if v == 0 {
                    continue;
                }
                let px = if flip { x + self.w - 1 - sx } else { x + sx };
                f.set(px, py, pal[v as usize]);
            }
        }
    }

    /// Draw with its bottom-centre at (x, y).
    pub fn draw_foot(&self, f: &mut Frame, x: i32, y: i32, pal: &Pal, flip: bool) {
        self.draw(f, x - self.w / 2, y - self.h + 1, pal, flip);
    }

    /// Draw only rows above screen y `clip` (things sinking in water).
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

    /// One flat color (silhouettes, shadows of far things).
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

pub const COW: [&str; 15] = [
    "..................H...H.",
    "..................KWWWK.",
    "..KKKKWWWWWWKKKWWKWWWWWK",
    ".KKKKKKWWWWWKKKKWWKWWKWW",
    ".WKKKKWWWWWWWKKWWWWWWWPP",
    "WWWWWWWWWWKKWWWWWWW.PPPP",
    "W.WWWWWWWKKKKWWWWWW..PP.",
    "W.WWWWWWWWKKWWWWWWW.....",
    "W.KKWWWWWWWWWWWKKWW.....",
    "W..KKWWWWWWWWWWKKKW.....",
    "K..WWWWWPPWWWWWWWWW.....",
    "...WW.WW.PP...WW.WW.....",
    "...WW.WW......WW.WW.....",
    "...ww.ww......ww.ww.....",
    "...KK.KK......KK.KK.....",
];

/// A cow with her head down, grazing.
pub const COW_GRAZE: [&str; 15] = [
    "........................",
    "........................",
    "..KKKKWWWWWWKKKWWKK.....",
    ".KKKKKKWWWWWKKKKWWWW....",
    ".WKKKKWWWWWWWKKWWWWWWW..",
    "WWWWWWWWWWKKWWWWWWWKWWH.",
    "W.WWWWWWWKKKKWWWWWWWWWW.",
    "W.WWWWWWWWKKWWWWWW.WWKWW",
    "W.KKWWWWWWWWWWWKKW..WWWW",
    "W..KKWWWWWWWWWWKKK..WPPP",
    "K..WWWWWPPWWWWWWWW...PP.",
    "...WW.WW.PP...WW.WW.GG..",
    "...WW.WW......WW.WW.....",
    "...ww.ww......ww.ww.....",
    "...KK.KK......KK.KK.....",
];

pub const SHEEP: [[&str; 11]; 2] = [
    [
        "..wWWwWWw.....",
        ".wWWwWWwWWw.KK",
        "wWwWWWwWWwWKKKK",
        "WWWWwWWWwWWKWKK",
        "wWwWWWWwWWwKKKP",
        "WWWwWWwWWWWwKK.",
        ".wWWWwWWWwWw...",
        "..wwwwwwwww....",
        "..K.K..K.K.....",
        "..K.K..K.K.....",
        "..K.K..K.K.....",
    ],
    [
        "..wWWwWWw.....",
        ".wWWwWWwWWw.KK",
        "wWwWWWwWWwWKKKK",
        "WWWWwWWWwWWKWKK",
        "wWwWWWWwWWwKKKP",
        "WWWwWWwWWWWwKK.",
        ".wWWWwWWWwWw...",
        "..wwwwwwwww....",
        "...KK..KK......",
        "..K..KK..K.....",
        "..K..KK..K.....",
    ],
];

pub const GOOSE: [[&str; 13]; 2] = [
    [
        "........WW...",
        ".......WWKW..",
        ".......WWWWoo",
        "........WWo..",
        "........WW...",
        ".......WW....",
        "..WW..WWW....",
        ".WWWWWWWWW...",
        "WWWwwwWWWW...",
        ".WWWwwwWWW...",
        "..WWWWWWW....",
        "....o.o......",
        "...oo.oo.....",
    ],
    [
        ".............",
        ".............",
        ".............",
        "..........oo.",
        "....W...WWKWo",
        "...WWW.WWWWoo",
        "..WWwWWWW....",
        ".WWWwwwWW....",
        "WWWwwwwWW....",
        ".WWwwwWWW....",
        "..WWWWWW.....",
        "....o.o......",
        "...oo..oo....",
    ],
];

pub const CAT: [&str; 6] = ["........yy.y", "..yYYYY.YYYY", ".YYyYYyYYKYY", "YYYYyYYYYYWY", "YyYYYYyYYYY.", ".YYYYYYYYY.."];

pub const CAT_RUN: [&str; 8] = [
    "..........y.y",
    "y.........YYY",
    ".y..yYYYYYYKY",
    "..YYYyYYyYYWY",
    "...YYYYYYYYY.",
    "...Y.Y..Y..Y.",
    "..Y...Y..Y..Y",
    ".............",
];

pub const DUCK: [&str; 6] = ["......GG.", ".....GGKo", ".....WW..", "BBBBBBBB.", ".BbBBBBb.", "..bbbbb.."];

pub const GULL: [[&str; 5]; 2] = [
    ["K.........K", "gW.......Wg", ".WW.....WW.", "..WWWWWWWoo", "...wWWWw..."],
    ["...........", "...WWWWWWoo", "..WWwWWwW..", ".WW.....WW.", "gK.......Kg"],
];

pub const DOG: [&str; 6] = ["..b...", ".bBBB.", "bBBKBB", ".BBBBK", "..BBP.", "......"];

pub const KID_HEADS: [[&str; 5]; 2] = [[".YYY.", "YYYYY", "YSSKS", ".SSS.", "..A.."], [".bbb.", "bbbbb", "bSSKS", "bSSS.", "..R.."]];

pub const STROOPWAFEL: [&str; 4] = [".yyyyyy.", "yYyYyYyy", "yyYyYyYy", ".yyyyyy."];
pub const KROKET: [&str; 4] = ["..Y.....", ".bBBBBb.", "bBtBBtBb", ".bbbbbb."];
pub const HARING: [&str; 4] = ["......w..", "wwWWWWWwg", ".gwwwwwgg", "..W.W...."];
pub const KAAS: [&str; 6] = ["....yy", "..yYYY", "yYYYYY", "YY.YYY", "YYYY.Y", "YYYYYY"];

pub const BOLLARD: [&str; 15] = [".WWW.", "WWWWW", "RRRRR", "RRRRR", "WWWWW", "WWWWW", "RRRRR", "RRRRR", "WWWWW", "WWWWW", "LLLLL", "WWWWW", "wwwww", "wwwww", "ggggg"];

pub const HERRING_ROAD: [&str; 4] = ["g.......w..", "gg.wwWWWWw.", "gwWWWWWWWKw", "g..wwwwwww."];

pub const HAMMER: [&str; 9] = ["bbbbbbb..", "bBBBBBbb.", "bBBBBBbb.", "bbbbbbb..", "...BB....", "...BB....", "...BB....", "...BB....", "...bb...."];

pub const BELL: [&str; 4] = [".www.", "wWWWw", "wWWWw", "gwwwg"];

pub const CAMERA: [&str; 3] = ["KKgKK", "KwKKK", "KKKKK"];

/// A red velvet sofa, side view (the lady who moves house by bike).
pub const SOFA: [&str; 10] = [
    "RRR............RRR",
    "RrRRRRRRRRRRRRRRrR",
    "RrRRRRRRRRRRRRRRrR",
    "RrrrrrrrrrrrrrrrrR",
    "RRRRRRRRRRRRRRRRRR",
    "RrRRrRRRrRRRrRRRrR",
    "RRRRRRRRRRRRRRRRRR",
    "rrrrrrrrrrrrrrrrrr",
    ".b..............b.",
    ".b..............b.",
];

pub const TRACTOR: [&str; 20] = [
    "..........gg..............",
    "..........gg..............",
    ".........KKKKKKKKKK.......",
    ".........K.......LK.......",
    ".........K.......LK.......",
    ".........K.......LK.......",
    ".........K.......LK.......",
    "........GGGGGGGGGGGGGG....",
    "........GGGGGGGGGGGGGGGGGG",
    ".......GGGGGGGGGGGGGGGGGGG",
    "......GGGGGGGGGgGgGgGgGGYY",
    "......GGGGGGGGGGGGGGGGGGGG",
    ".....GGGGGGGGGGGGGGGGGGGGG",
    "....KKKKKKKK...GGGGGGGGGGG",
    "...KKKKKKKKKK..KKgggKKKK..",
    "..KKKwwwwwwKKK.KKKKKKKKK..",
    "..KKwwYYYYwwKK....KKKK....",
    "..KKwwYYYYwwKK...KKwwKK...",
    "..KKKwwwwwwKKK...KKwwKK...",
    "...KKKKKKKKKK.....KKKK....",
];

/// Sail-less windmill body (a Dutch smock mill), big version.
pub const MILL: [&str; 44] = [
    "........tTTt........",
    ".......tTTTTt.......",
    "......tTTTTTTt......",
    ".....bBBBBBBBBb.....",
    "......bbbbbbbb......",
    "......gwgwgwgg......",
    "......gggggggg......",
    ".....gggggggggg.....",
    ".....ggwWgggggg.....",
    ".....ggWWgggggg.....",
    ".....gggggggggg.....",
    "....gggggggggggg....",
    "....gggggggggggg....",
    "....ggggggwWgggg....",
    "....ggggggWWgggg....",
    "...gggggggggggggg...",
    "BBBBBBBBBBBBBBBBBBBB",
    "bbbbbbbbbbbbbbbbbbbb",
    ".b.b..ggggggggg.b.b.",
    "...gggggggggggggg...",
    "...gggggggggggggg...",
    "..gggggggggggggggg..",
    "..ggggggwWgggggggg..",
    "..ggggggWWgggggggg..",
    "..gggggggggggggggg..",
    ".gggggggggggggggggg.",
    ".gggggggggggggggggg.",
    ".gggggggggggggggggg.",
    "gggggggggggggggggggg",
    "gggggggggggggggggggg",
    "BBBBBBBBBBBBBBBBBBBB",
    "BwwwBwwwBBwwwBwwwBBB",
    "BwRwBwRwBBwRwBwRwBBB",
    "BwwwBwwwBBwwwBwwwBBB",
    "BBBBBBBBBBBBBBBBBBBB",
    "BBBBBBBbbbbbBBBBBBBB",
    "BBBBBBBbRRrbBBBBBBBB",
    "BBBBBBBbRRrbBBBBBBBB",
    "BBBBBBBbRRrbBBBBBBBB",
    "BBBBBBBbRRrbBBBBBBBB",
    "BBBBBBBbRRrbBBBBBBBB",
    "BBBBBBBbRRrbBBBBBBBB",
    "BBBBBBBbRRrbBBBBBBBB",
    "bbbbbbbbbbbbbbbbbbbb",
];

/// A small far-away windmill silhouette (drawn flat).
pub const MILL_FAR: [&str; 16] = [
    "...ww...",
    "..wwww..",
    "..wwww..",
    "..wwww..",
    "..wwww..",
    ".wwwwww.",
    "wwwwwwww",
    ".wwwwww.",
    ".wwwwww.",
    ".wwwwww.",
    "wwwwwwww",
    "wwwwwwww",
    "wwwwwwww",
    "wwwwwwww",
    "wwwwwwww",
    "wwwwwwww",
];

// ------------------------------------------------------------------ riders

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Posture {
    Up,
    UpTuck,
    Race,
    RaceTuck,
    Lig,
    LigTuck,
}

pub fn posture(bike: Bike, tuck: bool) -> Posture {
    match (bike, tuck) {
        (Bike::Oma | Bike::Bak, false) => Posture::Up,
        (Bike::Oma | Bike::Bak, true) => Posture::UpTuck,
        (Bike::Race, false) => Posture::Race,
        (Bike::Race, true) => Posture::RaceTuck,
        (Bike::Lig, false) => Posture::Lig,
        (Bike::Lig, true) => Posture::LigTuck,
    }
}

/// Rider canvas: the bike's anchor (ground under the crank) sits at (12, 31).
pub const RX: i32 = 12;
pub const RY: i32 = 31;

/// Crank centre in the rider canvas.
pub fn crank_at(bike: Bike) -> (i32, i32) {
    match bike {
        Bike::Lig => (20, 22),
        _ => (12, 27),
    }
}

fn knee(hip: (f32, f32), foot: (f32, f32), a: f32, b: f32) -> (i32, i32) {
    let (dx, dy) = (foot.0 - hip.0, foot.1 - hip.1);
    let d = (dx * dx + dy * dy).sqrt().max(0.01);
    if d >= a + b - 0.01 {
        let k = a / (a + b);
        return ((hip.0 + dx * k).round() as i32, (hip.1 + dy * k).round() as i32);
    }
    let x = (a * a - b * b + d * d) / (2.0 * d);
    let h = (a * a - x * x).max(0.0).sqrt();
    let (ux, uy) = (dx / d, dy / d);
    let base = (hip.0 + ux * x, hip.1 + uy * x);
    // two choices: pick the one that points forward / up
    let p1 = (base.0 - uy * h, base.1 + ux * h);
    let p2 = (base.0 + uy * h, base.1 - ux * h);
    let score = |p: (f32, f32)| p.0 - p.1;
    let p = if score(p1) > score(p2) { p1 } else { p2 };
    (p.0.round() as i32, p.1.round() as i32)
}

/// A cycling skeleton for crank frame `k` (0..8).
pub fn ride_skel(p: Posture, k: usize) -> Skel {
    // head, neck, hip, back elbow, back hand, front elbow, front hand
    let (hd, nk, hip, be, bh, fe, fh, leg) = match p {
        Posture::Up => ((12, 5), (11, 10), (8, 19), (13, 15), (17, 17), (14, 15), (18, 17), 6.5),
        Posture::UpTuck => ((15, 8), (13, 12), (8, 19), (15, 16), (18, 18), (16, 16), (19, 18), 6.5),
        Posture::Race => ((16, 8), (14, 12), (8, 19), (16, 17), (19, 20), (17, 17), (20, 20), 6.5),
        Posture::RaceTuck => ((18, 12), (15, 15), (7, 19), (17, 19), (20, 21), (18, 19), (21, 21), 6.8),
        Posture::Lig => ((5, 12), (5, 17), (9, 24), (8, 22), (11, 25), (9, 22), (12, 25), 7.4),
        Posture::LigTuck => ((6, 14), (5, 19), (9, 24), (8, 23), (11, 25), (9, 23), (12, 25), 7.4),
    };
    let crank = if matches!(p, Posture::Lig | Posture::LigTuck) { (20.0, 22.0) } else { (12.0, 27.0) };
    let a = std::f32::consts::TAU * k as f32 / 8.0;
    let r = 3.0;
    let ff = (crank.0 + r * a.cos(), crank.1 + r * a.sin());
    let bf = (crank.0 - r * a.cos(), crank.1 - r * a.sin());
    let hipf = (hip.0 as f32, hip.1 as f32);
    let fk = knee(hipf, ff, leg, leg);
    let bk = knee(hipf, bf, leg, leg);
    let rnd = |p: (f32, f32)| (p.0.round() as i32, p.1.round() as i32);
    [hd, nk, hip, be, bh, fe, fh, bk, rnd(bf), fk, rnd(ff)]
}

/// Standing on the pedals, cranks level (a hop).
fn hop_skel(bike: Bike) -> Skel {
    match bike {
        Bike::Lig => {
            let mut s = ride_skel(Posture::Lig, 0);
            s[0].1 -= 1;
            s
        }
        Bike::Race => [(17, 5), (15, 10), (11, 17), (16, 14), (19, 18), (17, 14), (20, 18), (10, 22), (9, 27), (16, 22), (15, 27)],
        _ => [(15, 5), (14, 10), (11, 17), (15, 13), (18, 16), (16, 13), (19, 16), (10, 22), (9, 27), (16, 22), (15, 27)],
    }
}

/// One arm up: waving after the finish.
fn wave_skel(bike: Bike) -> Skel {
    let mut s = ride_skel(posture(bike, false), 2);
    if matches!(bike, Bike::Lig) {
        s[3] = (6, 11);
        s[4] = (7, 6);
    } else {
        s[0] = (12, 5);
        s[1] = (11, 10);
        s[3] = (9, 7);
        s[4] = (8, 2);
    }
    s
}

pub const STAND: Skel = [(12, 5), (12, 10), (12, 19), (8, 15), (8, 20), (16, 15), (16, 20), (10, 24), (10, 29), (14, 24), (14, 29)];
pub const VICTORY: Skel = [(12, 5), (12, 10), (12, 19), (8, 7), (4, 2), (16, 7), (20, 2), (10, 24), (9, 29), (14, 24), (15, 29)];
pub const CHEER: Skel = [(12, 5), (12, 10), (12, 19), (8, 13), (6, 9), (16, 7), (20, 2), (10, 24), (10, 29), (14, 24), (14, 29)];
pub const DIVE: Skel = [(17, 22), (13, 25), (6, 25), (19, 26), (22, 25), (19, 24), (23, 23), (2, 24), (0, 22), (2, 26), (0, 26)];
pub const DOWN: Skel = [(5, 25), (9, 27), (17, 27), (10, 23), (12, 21), (12, 30), (15, 30), (20, 24), (23, 27), (21, 27), (23, 29)];
pub const WALK: [Skel; 4] = [
    [(13, 5), (13, 10), (12, 19), (15, 14), (17, 17), (10, 15), (9, 19), (10, 24), (8, 28), (15, 24), (16, 29)],
    [(13, 5), (13, 10), (12, 19), (12, 15), (13, 19), (14, 15), (15, 19), (11, 24), (10, 29), (14, 24), (14, 29)],
    [(13, 5), (13, 10), (12, 19), (10, 15), (9, 19), (15, 14), (17, 17), (15, 24), (16, 29), (10, 24), (8, 28)],
    [(13, 5), (13, 10), (12, 19), (14, 15), (15, 19), (12, 15), (13, 19), (14, 24), (14, 29), (11, 24), (10, 29)],
];
/// Taking a photo: the front hand up at the face.
pub const PHOTO: Skel = [(12, 5), (12, 10), (12, 19), (14, 14), (17, 8), (16, 14), (18, 8), (10, 24), (10, 29), (14, 24), (14, 29)];

pub struct RiderSet {
    pub who: Who,
    pub bike: Bike,
    /// [tuck][crank frame]
    pub ride: [Vec<Sprite>; 2],
    pub hop: Sprite,
    pub wave: Sprite,
}

pub struct People {
    pub who: Who,
    pub fly: Sprite,
    pub swim: Sprite,
    pub stand: Sprite,
    pub victory: Sprite,
    pub cheer: Sprite,
    pub down: Sprite,
}

/// Tourists, spectators and the farmer: (style, palette index).
pub struct Extras {
    /// [style][walk frame 0..4, stand, photo, cheer]
    pub frames: Vec<Vec<Sprite>>,
}

pub fn rider_palette(who: Who) -> Palette {
    let d = who.def();
    let mut pal = [0u32; ROLES];
    pal[sp::OUT as usize] = OUTLINE;
    pal[sp::SKIN as usize] = d.skin.0;
    pal[sp::SKIN_D as usize] = d.skin.1;
    pal[sp::HAIR as usize] = d.hair.0;
    pal[sp::HAIR_D as usize] = d.hair.1;
    pal[sp::SHIRT as usize] = d.shirt.0;
    pal[sp::SHIRT_D as usize] = d.shirt.1;
    pal[sp::PANTS as usize] = d.pants.0;
    pal[sp::PANTS_D as usize] = d.pants.1;
    pal[sp::WHITE as usize] = rgb(248, 248, 248);
    pal[sp::ACC as usize] = d.acc.0;
    pal[sp::ACC_D as usize] = d.acc.1;
    pal[sp::SHOE as usize] = rgb(248, 248, 248);
    pal[sp::EYE as usize] = rgb(16, 16, 32);
    pal[sp::CAP as usize] = d.cap.0;
    pal[sp::CAP_D as usize] = d.cap.1;
    pal
}

/// A person in the crowd: shirt, pants, skin, hair.
pub fn extra_palette(k: usize) -> Palette {
    const SHIRTS: [(u32, u32); 6] = [
        (rgb(248, 104, 184), rgb(176, 48, 120)),
        (rgb(64, 200, 248), rgb(32, 120, 200)),
        (rgb(248, 136, 24), rgb(192, 80, 16)),
        (rgb(120, 216, 88), rgb(56, 144, 48)),
        (rgb(248, 232, 96), rgb(200, 168, 32)),
        (rgb(232, 232, 240), rgb(160, 168, 192)),
    ];
    const SKINS: [(u32, u32); 3] = [(rgb(248, 200, 160), rgb(216, 144, 104)), (rgb(224, 160, 104), rgb(176, 112, 64)), (rgb(160, 96, 56), rgb(112, 64, 32))];
    const HAIRS: [(u32, u32); 4] = [(rgb(56, 48, 72), rgb(24, 24, 40)), (rgb(248, 216, 72), rgb(200, 152, 32)), (rgb(136, 80, 32), rgb(88, 48, 16)), (rgb(216, 216, 224), rgb(160, 160, 176))];
    let s = SHIRTS[k % 6];
    let sk = SKINS[(k / 2) % 3];
    let h = HAIRS[(k / 3) % 4];
    let mut pal = [0u32; ROLES];
    pal[sp::OUT as usize] = OUTLINE;
    pal[sp::SKIN as usize] = sk.0;
    pal[sp::SKIN_D as usize] = sk.1;
    pal[sp::HAIR as usize] = h.0;
    pal[sp::HAIR_D as usize] = h.1;
    pal[sp::SHIRT as usize] = s.0;
    pal[sp::SHIRT_D as usize] = s.1;
    pal[sp::PANTS as usize] = if k % 2 == 0 { rgb(240, 216, 168) } else { rgb(72, 88, 144) };
    pal[sp::PANTS_D as usize] = if k % 2 == 0 { rgb(200, 168, 120) } else { rgb(48, 56, 104) };
    pal[sp::WHITE as usize] = rgb(248, 248, 248);
    pal[sp::ACC as usize] = rgb(248, 248, 248);
    pal[sp::ACC_D as usize] = rgb(184, 184, 208);
    pal[sp::SHOE as usize] = rgb(248, 248, 248);
    pal[sp::EYE as usize] = rgb(16, 16, 32);
    pal[sp::CAP as usize] = s.1;
    pal[sp::CAP_D as usize] = shade(s.1, 3, 4);
    pal
}

/// Ghostly blue for a rider seen through water.
pub fn wet_palette(base: &Palette) -> Palette {
    let mut p = *base;
    for (i, c) in p.iter_mut().enumerate() {
        if i != sp::OUT as usize {
            *c = mix(*c, rgb(96, 168, 232), 0.35);
        }
    }
    p
}

/// Draw a rider sprite with the bike anchor at screen (x, y).
pub fn blit_rider(f: &mut Frame, s: &Sprite, x: i32, y: i32, pal: &Palette, clip: i32) {
    for sy in 0..s.h {
        let py = y - RY + sy;
        if py >= clip || py < 0 {
            continue;
        }
        for sx in 0..s.w {
            let v = s.px[(sy * s.w + sx) as usize];
            if v != sp::T {
                f.set(x - RX + sx, py, pal[v as usize]);
            }
        }
    }
}

/// Draw any person sprite with its feet at (x, y) (engine origin 12, 30).
pub fn blit_person(f: &mut Frame, s: &Sprite, x: i32, y: i32, flip: bool, pal: &Palette) {
    sp::blit(f, s, x, y, flip, pal);
}

/// The same, but only the rows above screen y `clip` (someone in water).
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

// ------------------------------------------------------------------ bikes

/// Geometry of one bike, in bike coords (ground under the crank = 0, 0).
pub struct BikeGeo {
    pub rear: (i32, i32),
    pub rear_r: i32,
    pub front: (i32, i32),
    pub front_r: i32,
    pub crank: (i32, i32),
    /// top of the rack (where cargo sits), and its width
    pub rack: (i32, i32),
    pub rack_w: i32,
    /// where the bell sits
    pub bell: (i32, i32),
}

pub fn geo(b: Bike) -> BikeGeo {
    match b {
        Bike::Oma => BikeGeo { rear: (-10, -5), rear_r: 5, front: (10, -5), front_r: 5, crank: (0, -4), rack: (-15, -12), rack_w: 10, bell: (5, -16) },
        Bike::Race => BikeGeo { rear: (-10, -5), rear_r: 5, front: (10, -5), front_r: 5, crank: (0, -4), rack: (-13, -13), rack_w: 7, bell: (8, -12) },
        Bike::Bak => BikeGeo { rear: (-9, -5), rear_r: 5, front: (23, -4), front_r: 4, crank: (0, -4), rack: (8, -12), rack_w: 15, bell: (6, -15) },
        Bike::Lig => BikeGeo { rear: (-9, -5), rear_r: 5, front: (13, -4), front_r: 4, crank: (8, -9), rack: (-15, -12), rack_w: 8, bell: (1, -8) },
    }
}

/// Build a bike frame (no wheels, no crank arm) as a role bitmap. The
/// anchor (ground under the crank) is at (`BIKE_AX`, `BIKE_AY`).
pub const BIKE_AX: i32 = 18;
pub const BIKE_AY: i32 = 20;

pub fn build_bike(b: Bike) -> Bmp {
    let mut m = Bmp::new(46, 24);
    let p = |x: i32, y: i32| (BIKE_AX + x, BIKE_AY + y);
    let g = geo(b);
    const C: u8 = I_C;
    const CD: u8 = I_CD;
    const MET: u8 = 3; // w
    const MD: u8 = 4; // g
    const BLK: u8 = 5; // K
    const LEA: u8 = 11; // b
    let cr = p(g.crank.0, g.crank.1);
    let ra = p(g.rear.0, g.rear.1);
    let fa = p(g.front.0, g.front.1);
    match b {
        Bike::Oma => {
            // fenders over both wheels
            for (c, _) in [(ra, 0), (fa, 0)] {
                for k in 0..=10 {
                    let a = std::f32::consts::PI * (1.0 + k as f32 / 10.0);
                    m.set(c.0 + (a.cos() * 6.5).round() as i32, c.1 + (a.sin() * 6.5).round() as i32, CD);
                }
            }
            let seat = p(-4, -11);
            m.line(cr, ra, 2, C); // chain stay (under the chain guard)
            m.line(seat, ra, 1, C);
            m.line(cr, seat, 2, C);
            // step-through double tubes
            m.line(cr, p(3, -7), 2, C);
            m.line(p(3, -7), p(7, -12), 2, C);
            m.line(p(2, -5), p(5, -9), 1, CD);
            // head tube and fork
            m.line(p(7, -14), p(8, -10), 2, C);
            m.line(p(8, -10), fa, 1, MD);
            // handlebar: stem up, bars swept back to the grips
            m.line(p(7, -14), p(6, -16), 1, MET);
            m.line(p(6, -16), p(3, -15), 1, MET);
            m.set(p(3, -15).0, p(3, -15).1 + 1, BLK);
            m.set(p(2, -15).0, p(2, -15).1, BLK);
            // chain guard (dichte kettingkast)
            m.line(p(0, -4), p(-9, -5), 3, BLK);
            m.line(p(1, -3), p(-8, -4), 1, 4);
            // rack and lights
            m.line(p(-15, -11), p(-6, -11), 1, MET);
            m.line(p(-14, -11), p(-10, -6), 1, MD);
            m.stamp(p(-7, -14).0, p(-7, -14).1, &["bbbbb", ".bbb."]);
            m.line(p(-4, -12), seat, 1, MD);
            m.stamp(p(8, -14).0, p(8, -14).1, &["WY", "WW"]);
            m.set(p(-16, -10).0, p(-16, -10).1, 6);
        }
        Bike::Race => {
            let seat = p(-4, -12);
            let head = p(8, -12);
            m.line(cr, ra, 1, C);
            m.line(seat, ra, 1, C);
            m.line(cr, seat, 2, C);
            m.line(seat, head, 2, C);
            m.line(cr, p(7, -9), 2, C);
            m.line(head, p(8, -9), 2, C);
            m.line(p(8, -9), fa, 1, MD);
            // drop bars
            m.line(p(8, -13), p(10, -13), 1, BLK);
            m.line(p(10, -13), p(11, -11), 1, BLK);
            m.line(p(11, -11), p(9, -9), 1, BLK);
            // saddle and post
            m.line(p(-4, -12), p(-4, -13), 1, MD);
            m.stamp(p(-7, -14).0, p(-7, -14).1, &["KKKKK"]);
            // tiny seat-post rack for the cargo
            m.line(p(-13, -12), p(-7, -12), 1, MD);
            m.line(p(-5, -12), p(-7, -12), 1, MD);
            // white stripes on the frame
            m.set(p(2, -12).0, p(2, -12).1, 2);
            m.set(p(-1, -8).0, p(-1, -8).1, 2);
        }
        Bike::Bak => {
            // the box in front, plank wood with a colored rim
            let x0 = p(6, -12);
            for y in 0..8 {
                for x in 0..17 {
                    let wood = if (y / 2) % 2 == 0 { 10 } else { 23 };
                    m.set(x0.0 + x - (y > 5) as i32 * (x == 0) as i32, x0.1 + y, wood);
                }
            }
            for x in 0..17 {
                m.set(x0.0 + x, x0.1, C);
                m.set(x0.0 + x, x0.1 + 7, CD);
            }
            m.line(p(23, -4), p(21, -5), 1, MD);
            // long low frame under the box
            m.line(cr, p(23, -4), 2, CD);
            m.line(cr, ra, 2, C);
            let seat = p(-4, -11);
            m.line(seat, ra, 1, C);
            m.line(cr, seat, 2, C);
            m.line(p(3, -6), p(5, -13), 2, C);
            // steering bar
            m.line(p(5, -13), p(5, -15), 1, MET);
            m.line(p(4, -15), p(7, -15), 1, BLK);
            m.stamp(p(-7, -13).0, p(-7, -13).1, &["bbbbb", ".bbb."]);
            m.line(p(-4, -11), p(-4, -12), 1, MD);
            m.line(p(0, -4), p(-8, -5), 2, BLK);
            m.set(p(-14, -8).0, p(-14, -8).1, 6);
        }
        Bike::Lig => {
            // long low boom from the rear wheel to the crank in front
            m.line(ra, p(-2, -6), 2, C);
            m.line(p(-2, -6), cr, 2, C);
            m.line(p(-2, -6), fa, 1, MD);
            m.line(p(6, -7), fa, 1, C);
            // reclined mesh seat
            m.line(p(-5, -5), p(-9, -16), 2, BLK);
            m.line(p(-4, -6), p(-8, -16), 1, LEA);
            m.line(p(-9, -15), ra, 1, MD);
            // under-seat steering
            m.line(p(-1, -6), p(0, -8), 1, MET);
            m.line(p(0, -8), p(2, -7), 1, BLK);
            // a safety flag on a whip pole
            m.line(p(-11, -14), p(-13, -23), 1, MET);
            m.stamp(p(-13, -23).0, p(-13, -23).1, &["oooo", "ooo.", "oo.."]);
            m.line(p(-16, -11), p(-11, -11), 1, MD);
        }
    }
    let _ = (C, LEA);
    m.outline();
    m
}

/// A spinning wheel: tire, rim, spokes, hub. Frame k of 4 turns 15 degrees.
pub fn build_wheel(r: i32, k: usize, thin: bool) -> Bmp {
    let s = 2 * r + 3;
    let c = r + 1;
    let mut m = Bmp::new(s, s);
    for y in 0..s {
        for x in 0..s {
            let d = (((x - c) * (x - c) + (y - c) * (y - c)) as f32).sqrt();
            let v = if d <= r as f32 + 0.5 && d > r as f32 - if thin { 0.6 } else { 1.1 } {
                5
            } else if d <= r as f32 - 0.6 && d > r as f32 - 1.6 {
                3
            } else {
                0
            };
            if v != 0 {
                m.set(x, y, v);
            }
        }
    }
    for sp in 0..6 {
        let a = (sp as f32 * 60.0 + k as f32 * 15.0).to_radians();
        let e = (c as f32 + a.cos() * (r as f32 - 1.6), c as f32 + a.sin() * (r as f32 - 1.6));
        line_points(c, c, e.0.round() as i32, e.1.round() as i32, |x, y| {
            if m.get(x, y) == 0 {
                m.set(x, y, 4);
            }
        });
    }
    m.set(c, c, 2);
    m.outline();
    m
}

/// A cargo piece, about 8x5.
pub fn cargo_rows(c: Cargo) -> &'static [&'static str] {
    match c {
        Cargo::Cheese => &[".yYYYYy.", "yYYYYYYy", "yYYYYYYy", ".yyyyyy."],
        Cargo::Tulips => &[".R.Y.R..", "RRYYRR..", ".G.G.G..", "tTTTTt..", ".tTTt..."],
        Cargo::Herring => &["g...g.g.", "BBBBBBBB", "BbBBbBBb", "BBBBBBBB"],
        Cargo::Tuba => &["..yYYy..", ".yY..Yy.", "yY.yy.Yy", ".YyYYyY.", "...YY..."],
        Cargo::Clock => &[".BBBB.", "BwwwwB", "BwKwwB", "BwwKwB", "BBBBBB", "BbYYbB", "BbbbbB"],
    }
}

/// Everything, built once.
pub struct Art {
    pub sets: Vec<RiderSet>,
    pub people: Vec<People>,
    pub extras: Extras,
    pub bikes: Vec<Bmp>,
    /// [r4 thin, r4, r5 thin, r5][frame]
    pub wheels: Vec<Vec<Bmp>>,
    pub cargo: Vec<Bmp>,
    pub cow: Bmp,
    pub cow_graze: Bmp,
    pub sheep: [Bmp; 2],
    pub goose: [Bmp; 2],
    pub cat: Bmp,
    pub cat_run: Bmp,
    pub duck: Bmp,
    pub gull: [Bmp; 2],
    pub dog: Bmp,
    pub kids: [Bmp; 2],
    pub food: [Bmp; 4],
    pub bollard: Bmp,
    pub herring: Bmp,
    pub hammer: Bmp,
    pub bell: Bmp,
    pub camera: Bmp,
    pub sofa: Bmp,
    pub tractor: Bmp,
    pub mill: Bmp,
    pub mill_far: Bmp,
}

fn bikes_of(who: Who) -> Vec<Bike> {
    match who {
        Who::Kees | Who::Anouk => BIKES.to_vec(),
        // a rival could also ride for the CPU "Kees" in the demo; rivals keep their own
        w => vec![w.def().bike],
    }
}

impl Art {
    pub fn new() -> Self {
        let mut sets = Vec::new();
        for who in ALL_WHO {
            let style = who.def().style;
            for bike in bikes_of(who) {
                let ride = [false, true].map(|tuck| (0..8).map(|k| person(&ride_skel(posture(bike, tuck), k), style, false, false)).collect::<Vec<_>>());
                sets.push(RiderSet { who, bike, ride, hop: person(&hop_skel(bike), style, false, false), wave: person(&wave_skel(bike), style, false, false) });
            }
        }
        // extra riders on the road: the lady with the sofa rides an omafiets
        let people = ALL_WHO
            .iter()
            .map(|&who| {
                let st = who.def().style;
                People {
                    who,
                    fly: person(&DIVE, st, true, false),
                    swim: person(&VICTORY, st, false, false),
                    stand: person(&STAND, st, false, false),
                    victory: person(&VICTORY, st, false, false),
                    cheer: person(&CHEER, st, false, false),
                    down: person(&DOWN, st, true, true),
                }
            })
            .collect();
        let extras = Extras {
            frames: (0..6)
                .map(|st| {
                    let mut v: Vec<Sprite> = WALK.iter().map(|k| person(k, st, false, false)).collect();
                    v.push(person(&STAND, st, false, false));
                    v.push(person(&PHOTO, st, false, false));
                    v.push(person(&CHEER, st, false, false));
                    v.push(person(&VICTORY, st, false, false));
                    v
                })
                .collect(),
        };
        let wheels = [(4, true), (4, false), (5, true), (5, false)].iter().map(|&(r, thin)| (0..4).map(|k| build_wheel(r, k, thin)).collect()).collect();
        Art {
            sets,
            people,
            extras,
            bikes: BIKES.iter().map(|&b| build_bike(b)).collect(),
            wheels,
            cargo: [Cargo::Cheese, Cargo::Tulips, Cargo::Herring, Cargo::Tuba, Cargo::Clock].iter().map(|&c| Bmp::art(cargo_rows(c))).collect(),
            cow: Bmp::art(&COW),
            cow_graze: Bmp::art(&COW_GRAZE),
            sheep: [Bmp::art(&SHEEP[0]), Bmp::art(&SHEEP[1])],
            goose: [Bmp::art(&GOOSE[0]), Bmp::art(&GOOSE[1])],
            cat: Bmp::art(&CAT),
            cat_run: Bmp::art(&CAT_RUN),
            duck: Bmp::art(&DUCK),
            gull: [Bmp::art(&GULL[0]), Bmp::art(&GULL[1])],
            dog: Bmp::art(&DOG),
            kids: [Bmp::art(&KID_HEADS[0]), Bmp::art(&KID_HEADS[1])],
            food: [Bmp::art(&STROOPWAFEL), Bmp::art(&KROKET), Bmp::art(&HARING), Bmp::art(&KAAS)],
            bollard: Bmp::art(&BOLLARD),
            herring: Bmp::art(&HERRING_ROAD),
            hammer: Bmp::art(&HAMMER),
            bell: Bmp::art(&BELL),
            camera: Bmp::art(&CAMERA),
            sofa: Bmp::art(&SOFA),
            tractor: Bmp::art(&TRACTOR),
            mill: Bmp::art(&MILL),
            mill_far: Bmp::parse(&MILL_FAR),
        }
    }

    pub fn set(&self, who: Who, bike: Bike) -> &RiderSet {
        self.sets.iter().find(|s| s.who == who && s.bike == bike).or_else(|| self.sets.iter().find(|s| s.who == who)).unwrap()
    }

    pub fn people(&self, who: Who) -> &People {
        &self.people[who.index()]
    }

    pub fn wheel(&self, r: i32, thin: bool, frame: usize) -> &Bmp {
        let i = match (r, thin) {
            (4, true) => 0,
            (4, false) => 1,
            (_, true) => 2,
            _ => 3,
        };
        &self.wheels[i][frame % 4]
    }

    pub fn cargo_bmp(&self, c: Cargo) -> &Bmp {
        &self.cargo[c as usize]
    }
}

impl Default for Art {
    fn default() -> Self {
        Self::new()
    }
}
