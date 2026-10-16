//! STALEN ZENUWEN! art. Props, animals and bikes are string bitmaps in
//! color roles that get a 1px outline when they are built (the `Bmp` code
//! is a copy of Tegenwind's: games do not depend on each other). People
//! are engine `person()` sprites from our own skeletons: walking, running,
//! picking a lock, acting casual, riding (eight crank frames), swimming.
//! Everything is built once in `Art::new()`.

use super::data::*;
use crate::engine::gfx::{line_points, mix, rgb, shade, Frame, H};
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

/// The krul: Amsterdam's green iron urinal. Feet show under the screen.
pub const KRUL: [&str; 27] = [
    "......dd......",
    ".....dDDd.....",
    "...ddDDDDdd...",
    "..dDDDDDDDDd..",
    ".dDGDDDDDDDDd.",
    ".ddddddddddd..",
    "......dd......",
    "......dd......",
    "..dDDDDDDDDd..",
    ".dDGDDGDDGDDd.",
    ".dDDDDDDDDDDd.",
    ".dDGDDDDDDDDd.",
    ".dDGDDDDDDDDd.",
    ".dDDDDDDDDDDd.",
    ".dDDDDDDDDDDd.",
    ".dDGDDDDDDDDd.",
    ".dDGDDDDDDDDd.",
    ".dDDDDDDDDDDd.",
    ".dDDDDDDDDDDd.",
    ".dDGDDDDDDDDd.",
    ".dDDDDDDDDDDd.",
    ".dDGDDGDDGDDd.",
    ".ddddddddddd..",
    "..d........d..",
    "..d........d..",
    "..d........d..",
    ".dd........dd.",
];

pub const GLASBAK: [&str; 17] = [
    ".....dddd.....",
    "...ddDDDDdd...",
    "..dDGGDDDDDd..",
    ".dDGDDDDDDDDd.",
    ".dDGDKKKKDDDd.",
    ".dDDDDDDDDDDd.",
    "dDGDDDDDDDDDDd",
    "dDDDWWWWWWDDDd",
    "dDDDWKWKWWDDDd",
    "dDDDWWWWWWDDDd",
    "dDGDDDDDDDDDDd",
    "dDDDDDDDDDDDDd",
    "dDDDDDDDDDDDDd",
    "dDDDDDDDDDDDDd",
    "dddddddddddddd",
    ".gg........gg.",
    ".gg........gg.",
];

pub const TRUNK: [&str; 34] = [
    "..bb.bb.",
    "...bBb..",
    "b..bBb..",
    ".bbBBb.b",
    "...BBbb.",
    "...BBb..",
    "...BBb..",
    "..bBBb..",
    "..BBBb..",
    "..BtBb..",
    "..BBBb..",
    "..BBtb..",
    "..BBBb..",
    "..BtBb..",
    "..BBBb..",
    "..BBBb..",
    "..BBtb..",
    "..BBBb..",
    "..BtBb..",
    "..BBBb..",
    "..BBBb..",
    "..BBtb..",
    "..BBBb..",
    "..BtBb..",
    "..BBBb..",
    "..BBBb..",
    "..BBBbb.",
    ".bBBtBb.",
    ".BBBBBb.",
    "bBBBBBBb",
    "gggggggg",
    "gwwwwwwg",
    "gggggggg",
    "........",
];

/// An Amsterdammertje: the brown bollard with three crosses.
pub const BOLLARD: [&str; 10] = [".bb.", "bBBb", "bBBb", "bBBb", "bYYb", "bBBb", "bYYb", "bBBb", "bYYb", "bBBb"];

pub const HERON: [&str; 22] = [
    "..KK.....",
    ".wwKK....",
    ".wwwKyyy.",
    "..ww.....",
    "..ww.....",
    "..w......",
    ".ww......",
    ".www.....",
    "gwwwg....",
    "ggwwgg...",
    "gggwggg..",
    "ggggggg..",
    ".gggggg..",
    "..ggggg..",
    "...gggg..",
    "....gg...",
    "....y....",
    "....y....",
    "....y....",
    "...y.y...",
    "...y.y...",
    "..yy.yy..",
];

/// A heron in flight: neck tucked in an S, legs trailing, wings up and down.
pub const HERON_FLY: [[&str; 12]; 2] = [
    [
        "..........gg................",
        ".........gggg...............",
        "........ggggg...............",
        ".......gggggg...............",
        "......ggggggg.......KK......",
        "......gggggg.......wwKyyyy..",
        "yy...gwwwwwwwgg...ww........",
        ".yyyywwwwwwwwwwwwww.........",
        "......gwwwwwwwgg............",
        "........gggg................",
        "............................",
        "............................",
    ],
    [
        "............................",
        "............................",
        "............................",
        "....................KK......",
        "...................wwKyyyy..",
        "..................ww........",
        "yy...gwwwwwwwgg..ww.........",
        ".yyyywwwwwwwwwwwww..........",
        "......ggggggggg.............",
        ".......gggggg...............",
        "........ggggg...............",
        ".........ggg................",
    ],
];

/// Bruno, the chief's dog.
pub const DOG: [[&str; 10]; 2] = [
    [
        ".........bb.....",
        "........bBBb....",
        "........bBKBBK..",
        "b.......BBBBBBP.",
        ".bBBBBBBBBBB....",
        ".BBbbbbbbBBb....",
        ".BbBBBBBBBb.....",
        ".B.b....b.B.....",
        ".B.b....b.B.....",
        ".K.K....K.K.....",
    ],
    [
        ".........bb.....",
        "........bBBb....",
        "........bBKBBK..",
        "bb......BBBBBB..",
        ".bBBBBBBBBBB....",
        ".BBbbbbbbBBb....",
        ".BbBBBBBBBb.....",
        "..Bb...bB.......",
        ".B..b.b..B......",
        "K....K....K.....",
    ],
];

/// The houseboat cat.
pub const CAT: [&str; 7] = ["y.........Y.Y", "y.........YYY", ".y........YKYK", ".yYYyYYyYYYYYP", "..YYYYYYYYYY..", "..YyYYyYYYYY..", "..Y.Y....Y.Y.."];

/// A coot (meerkoet) on its nest of junk.
pub const COOT: [&str; 6] = ["....KK..", "...KKWK.", "KKKKKKK.", "KKKKKKK.", ".KKKKK..", "........"];

pub const CUTTER: [&str; 6] = ["RR..........gg", "RRRRR......gw.", "..RRRRRgggwg..", "..RRRRRgggwg..", "RRRRR......gw.", "RR..........gg"];

pub const TICKET: [&str; 9] = ["WWWWWWW", "WKKKKKW", "WWWWWWW", "WKKKWKW", "WWWWWWW", "WKKKKWW", "WWWWWWW", "WRRRRRW", "WWWWWWW"];

pub const COFFEE: [&str; 5] = ["W.W..", ".W.W.", "wwww.", "TTTTw", "TTTT."];

pub const PHONE: [&str; 4] = ["KKK", "KLK", "KLK", "KKK"];

// ------------------------------------------------------------------ bikes

/// Geometry of one bike, in bike coords (ground under the crank = 0, 0).
pub struct BikeGeo {
    pub rear: (i32, i32),
    pub front: (i32, i32),
    pub rear_r: i32,
    pub front_r: i32,
    pub crank: (i32, i32),
}

pub fn geo(k: Kind) -> BikeGeo {
    match k {
        Kind::Race => BikeGeo { rear: (-10, -5), rear_r: 5, front: (10, -5), front_r: 5, crank: (0, -4) },
        Kind::Bak => BikeGeo { rear: (-9, -5), rear_r: 5, front: (23, -4), front_r: 4, crank: (0, -4) },
        _ => BikeGeo { rear: (-10, -5), rear_r: 5, front: (10, -5), front_r: 5, crank: (0, -4) },
    }
}

/// Bike frames are built into a 46x24 bitmap with the anchor (ground under
/// the crank) at (`BIKE_AX`, `BIKE_AY`).
pub const BIKE_AX: i32 = 18;
pub const BIKE_AY: i32 = 20;

pub fn build_bike(k: Kind) -> Bmp {
    let mut m = Bmp::new(46, 24);
    let p = |x: i32, y: i32| (BIKE_AX + x, BIKE_AY + y);
    let g = geo(k);
    const C: u8 = I_C;
    const CD: u8 = I_CD;
    const MET: u8 = 3;
    const MD: u8 = 4;
    const BLK: u8 = 5;
    let cr = p(g.crank.0, g.crank.1);
    let ra = p(g.rear.0, g.rear.1);
    let fa = p(g.front.0, g.front.1);
    match k {
        Kind::Oma | Kind::Gold | Kind::Wreck => {
            let wreck = k == Kind::Wreck;
            // fenders over both wheels (a wreck lost the front one)
            for (c, front) in [(ra, false), (fa, true)] {
                if wreck && front {
                    continue;
                }
                for s in 0..=10 {
                    let a = std::f32::consts::PI * (1.0 + s as f32 / 10.0);
                    m.set(c.0 + (a.cos() * 6.5).round() as i32, c.1 + (a.sin() * 6.5).round() as i32, CD);
                }
            }
            let seat = p(-4, -11);
            m.line(cr, ra, 2, C);
            m.line(seat, ra, 1, C);
            m.line(cr, seat, 2, C);
            m.line(cr, p(3, -7), 2, C);
            m.line(p(3, -7), p(7, -12), 2, C);
            m.line(p(2, -5), p(5, -9), 1, CD);
            m.line(p(7, -14), p(8, -10), 2, C);
            m.line(p(8, -10), fa, 1, MD);
            if wreck {
                // bent bars, no grips
                m.line(p(7, -14), p(5, -17), 1, MD);
                m.line(p(5, -17), p(2, -16), 1, MD);
            } else {
                m.line(p(7, -14), p(6, -16), 1, MET);
                m.line(p(6, -16), p(3, -15), 1, MET);
                m.set(p(3, -15).0, p(3, -15).1 + 1, BLK);
                m.set(p(2, -15).0, p(2, -15).1, BLK);
                // chain guard (dichte kettingkast)
                m.line(p(0, -4), p(-9, -5), 3, BLK);
                m.line(p(1, -3), p(-8, -4), 1, 4);
                // rack, lamp, a crate on the front for the omafiets
                m.line(p(-15, -11), p(-6, -11), 1, MET);
                m.line(p(-14, -11), p(-10, -6), 1, MD);
                m.stamp(p(8, -14).0, p(8, -14).1, &["WY", "WW"]);
                m.set(p(-16, -10).0, p(-16, -10).1, 6);
            }
            // saddle
            m.stamp(p(-7, -14).0, p(-7, -14).1, if wreck { &["b.bbb", ".bbb."] } else { &["bbbbb", ".bbb."] });
            m.line(p(-4, -12), seat, 1, MD);
        }
        Kind::Race => {
            let seat = p(-4, -12);
            let head = p(8, -12);
            m.line(cr, ra, 1, C);
            m.line(seat, ra, 1, C);
            m.line(cr, seat, 2, C);
            m.line(seat, head, 2, C);
            m.line(cr, p(7, -9), 2, C);
            m.line(head, p(8, -9), 2, C);
            m.line(p(8, -9), fa, 1, MD);
            m.line(p(8, -13), p(10, -13), 1, BLK);
            m.line(p(10, -13), p(11, -11), 1, BLK);
            m.line(p(11, -11), p(9, -9), 1, BLK);
            m.line(p(-4, -12), p(-4, -13), 1, MD);
            m.stamp(p(-7, -14).0, p(-7, -14).1, &["KKKKK"]);
            m.set(p(2, -12).0, p(2, -12).1, 2);
            m.set(p(-1, -8).0, p(-1, -8).1, 2);
        }
        Kind::Bak => {
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
            m.line(cr, p(23, -4), 2, CD);
            m.line(cr, ra, 2, C);
            let seat = p(-4, -11);
            m.line(seat, ra, 1, C);
            m.line(cr, seat, 2, C);
            m.line(p(3, -6), p(5, -13), 2, C);
            m.line(p(5, -13), p(5, -15), 1, MET);
            m.line(p(4, -15), p(7, -15), 1, BLK);
            m.stamp(p(-7, -13).0, p(-7, -13).1, &["bbbbb", ".bbb."]);
            m.line(p(-4, -11), p(-4, -12), 1, MD);
            m.line(p(0, -4), p(-8, -5), 2, BLK);
            m.set(p(-14, -8).0, p(-14, -8).1, 6);
        }
    }
    m.outline();
    m
}

/// A wheel: tire, rim, spokes, hub. Frame k of 4 turns 15 degrees.
pub fn build_wheel(r: i32, k: usize, thin: bool, bent: bool) -> Bmp {
    let s = 2 * r + 3;
    let c = r + 1;
    let mut m = Bmp::new(s, s);
    for y in 0..s {
        for x in 0..s {
            // a bent rim (achtje: the Dutch figure-eight wheel)
            let squash = if bent { 1.0 + 0.18 * (((x - c) as f32) / r as f32) } else { 1.0 };
            let (fx, fy) = ((x - c) as f32, (y - c) as f32 * squash);
            let d = (fx * fx + fy * fy).sqrt();
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
    for spoke in 0..6 {
        let a = (spoke as f32 * 60.0 + k as f32 * 15.0).to_radians();
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

pub fn bike_pal(kind: Kind, color: usize) -> Pal {
    match kind {
        Kind::Gold => pal_with((rgb(248, 216, 64), rgb(200, 144, 24)), (rgb(248, 248, 168), rgb(248, 216, 64))),
        Kind::Wreck => {
            let mut p = pal_with((rgb(168, 96, 56), rgb(112, 56, 32)), (rgb(136, 88, 56), rgb(96, 56, 32)));
            p[3] = rgb(176, 120, 88);
            p[4] = rgb(128, 88, 64);
            p
        }
        _ => {
            let c = BIKE_COLORS[color % BIKE_COLORS.len()];
            pal_with(c, (rgb(64, 120, 216), rgb(32, 64, 152)))
        }
    }
}

/// The BOA's dark blue service bike.
pub fn boa_bike_pal() -> Pal {
    pal_with((rgb(40, 48, 96), rgb(24, 24, 64)), (rgb(216, 248, 48), rgb(144, 184, 24)))
}

// ------------------------------------------------------------------ people

/// Rider canvas: the bike's anchor (ground under the crank) sits at (12, 31).
pub const RX: i32 = 12;
pub const RY: i32 = 31;

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
    let p1 = (base.0 - uy * h, base.1 + ux * h);
    let p2 = (base.0 + uy * h, base.1 - ux * h);
    let score = |p: (f32, f32)| p.0 - p.1;
    let p = if score(p1) > score(p2) { p1 } else { p2 };
    (p.0.round() as i32, p.1.round() as i32)
}

/// A cycling skeleton for crank frame `k` (0..8): upright or bent over.
pub fn ride_skel(race: bool, k: usize) -> Skel {
    let (hd, nk, hip, be, bh, fe, fh) =
        if race { ((16, 8), (14, 12), (8, 19), (16, 17), (19, 20), (17, 17), (20, 20)) } else { ((12, 5), (11, 10), (8, 19), (13, 15), (17, 17), (14, 15), (18, 17)) };
    let crank = (12.0, 27.0);
    let a = std::f32::consts::TAU * k as f32 / 8.0;
    let r = 3.0;
    let ff = (crank.0 + r * a.cos(), crank.1 + r * a.sin());
    let bf = (crank.0 - r * a.cos(), crank.1 - r * a.sin());
    let hipf = (hip.0 as f32, hip.1 as f32);
    let fk = knee(hipf, ff, 6.5, 6.5);
    let bk = knee(hipf, bf, 6.5, 6.5);
    let rnd = |p: (f32, f32)| (p.0.round() as i32, p.1.round() as i32);
    [hd, nk, hip, be, bh, fe, fh, bk, rnd(bf), fk, rnd(ff)]
}

pub const STAND: Skel = [(12, 5), (12, 10), (12, 19), (8, 15), (8, 20), (16, 15), (16, 20), (10, 24), (10, 29), (14, 24), (14, 29)];
pub const WALK: [Skel; 4] = [
    [(13, 5), (13, 10), (12, 19), (15, 14), (17, 17), (10, 15), (9, 19), (10, 24), (8, 28), (15, 24), (16, 29)],
    [(13, 5), (13, 10), (12, 19), (12, 15), (13, 19), (14, 15), (15, 19), (11, 24), (10, 29), (14, 24), (14, 29)],
    [(13, 5), (13, 10), (12, 19), (10, 15), (9, 19), (15, 14), (17, 17), (15, 24), (16, 29), (10, 24), (8, 28)],
    [(13, 5), (13, 10), (12, 19), (14, 15), (15, 19), (12, 15), (13, 19), (14, 24), (14, 29), (11, 24), (10, 29)],
];
pub const RUN: [Skel; 4] = [
    [(14, 6), (13, 11), (12, 19), (15, 14), (18, 12), (10, 15), (8, 18), (9, 24), (6, 27), (16, 23), (17, 29)],
    [(14, 5), (13, 10), (12, 18), (12, 15), (13, 19), (14, 15), (15, 19), (11, 23), (9, 27), (14, 23), (13, 29)],
    [(14, 6), (13, 11), (12, 19), (10, 15), (8, 18), (16, 14), (19, 12), (16, 23), (17, 29), (9, 24), (6, 27)],
    [(14, 5), (13, 10), (12, 18), (14, 15), (15, 19), (12, 15), (13, 19), (14, 23), (13, 29), (11, 23), (9, 27)],
];
/// Kneeling at a lock, hands on it; the second frame moves the hands.
pub const PICK: [Skel; 2] = [
    [(14, 11), (13, 16), (10, 23), (15, 20), (19, 24), (16, 20), (20, 23), (14, 29), (7, 29), (16, 23), (17, 29)],
    [(15, 11), (14, 16), (10, 23), (16, 21), (20, 25), (17, 21), (21, 25), (14, 29), (7, 29), (16, 23), (17, 29)],
];
/// Hands in the pockets, head up, whistling.
pub const CASUAL: Skel = [(11, 4), (12, 9), (12, 19), (9, 14), (10, 19), (15, 14), (14, 19), (10, 24), (10, 29), (14, 24), (14, 29)];
/// Phone up at arm's length.
pub const SELFIE: Skel = [(12, 5), (12, 10), (12, 19), (9, 15), (9, 20), (16, 9), (20, 5), (10, 24), (10, 29), (14, 24), (14, 29)];
/// Both arms up: the bike goes over the head onto the boat.
pub const TOSS: Skel = [(12, 5), (12, 10), (12, 19), (10, 6), (12, 1), (15, 6), (17, 1), (10, 24), (9, 29), (15, 24), (16, 29)];
/// Sitting on the ground, caught.
pub const SIT: Skel = [(13, 15), (13, 20), (11, 27), (15, 22), (18, 25), (16, 22), (19, 25), (16, 26), (20, 29), (17, 27), (21, 29)];
/// Treading water: head and hands up.
pub const SWIM: Skel = [(12, 11), (12, 16), (12, 25), (8, 14), (5, 11), (16, 14), (19, 11), (10, 29), (9, 31), (14, 29), (15, 31)];
/// Hands on the quay, pulling up.
pub const CLIMB: Skel = [(12, 8), (12, 13), (12, 22), (9, 9), (8, 4), (15, 9), (16, 4), (10, 27), (10, 31), (14, 27), (14, 31)];
pub const DIVE: Skel = [(17, 22), (13, 25), (6, 25), (19, 26), (22, 25), (19, 24), (23, 23), (2, 24), (0, 22), (2, 26), (0, 26)];
pub const VICTORY: Skel = [(12, 5), (12, 10), (12, 19), (8, 7), (4, 2), (16, 7), (20, 2), (10, 24), (9, 29), (14, 24), (15, 29)];
pub const CHEER: Skel = [(12, 5), (12, 10), (12, 19), (8, 13), (6, 9), (16, 7), (20, 2), (10, 24), (10, 29), (14, 24), (14, 29)];
pub const PHOTO: Skel = [(12, 5), (12, 10), (12, 19), (14, 14), (17, 8), (16, 14), (18, 8), (10, 24), (10, 29), (14, 24), (14, 29)];
/// Writing a fine: notebook at the chest.
pub const WRITE: Skel = [(13, 6), (12, 10), (12, 19), (14, 15), (16, 14), (15, 15), (18, 13), (10, 24), (10, 29), (14, 24), (14, 29)];
/// Whistle at the mouth, other arm pointing.
pub const WHISTLE: Skel = [(12, 5), (12, 10), (12, 19), (14, 12), (15, 7), (16, 11), (21, 9), (10, 24), (9, 29), (14, 24), (15, 29)];
/// Coffee cup at the mouth.
pub const SIP: Skel = [(12, 5), (12, 10), (12, 19), (8, 15), (8, 20), (15, 13), (15, 8), (10, 24), (10, 29), (14, 24), (14, 29)];

/// Every pose for one head style.
pub struct People {
    pub walk: [Sprite; 4],
    pub run: [Sprite; 4],
    pub stand: Sprite,
    pub pick: [Sprite; 2],
    pub casual: Sprite,
    pub selfie: Sprite,
    pub toss: Sprite,
    pub sit: Sprite,
    pub swim: Sprite,
    pub climb: Sprite,
    pub fall: Sprite,
    pub victory: Sprite,
    pub cheer: Sprite,
    pub photo: Sprite,
    pub write: Sprite,
    pub whistle: Sprite,
    pub sip: Sprite,
    /// [race posture][crank frame]
    pub ride: [Vec<Sprite>; 2],
}

impl People {
    fn new(style: usize) -> Self {
        let p = |k: &Skel| person(k, style, false, false);
        People {
            walk: WALK.map(|k| p(&k)),
            run: RUN.map(|k| p(&k)),
            stand: p(&STAND),
            pick: PICK.map(|k| p(&k)),
            casual: p(&CASUAL),
            selfie: p(&SELFIE),
            toss: p(&TOSS),
            sit: p(&SIT),
            swim: p(&SWIM),
            climb: p(&CLIMB),
            fall: person(&DIVE, style, true, false),
            victory: p(&VICTORY),
            cheer: p(&CHEER),
            photo: p(&PHOTO),
            write: p(&WRITE),
            whistle: p(&WHISTLE),
            sip: p(&SIP),
            ride: [false, true].map(|race| (0..8).map(|k| p(&ride_skel(race, k))).collect()),
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

pub fn thief_palette(who: Who) -> Palette {
    let d = who.def();
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

/// Navy uniform, a fluorescent stripe across the chest, a navy cap.
pub fn boa_palette(k: usize) -> Palette {
    const SKINS: [(u32, u32); 4] = [(rgb(248, 200, 160), rgb(216, 144, 104)), (rgb(160, 96, 56), rgb(112, 64, 32)), (rgb(224, 160, 104), rgb(176, 112, 64)), (rgb(248, 208, 176), rgb(216, 152, 120))];
    const HAIRS: [(u32, u32); 4] = [(rgb(56, 48, 72), rgb(24, 24, 40)), (rgb(24, 24, 32), rgb(8, 8, 16)), (rgb(136, 80, 32), rgb(88, 48, 16)), (rgb(216, 216, 224), rgb(160, 160, 176))];
    let mut pal = base_pal();
    let s = SKINS[k % 4];
    let h = HAIRS[(k / 2 + k) % 4];
    pal[sp::SKIN as usize] = s.0;
    pal[sp::SKIN_D as usize] = s.1;
    pal[sp::HAIR as usize] = h.0;
    pal[sp::HAIR_D as usize] = h.1;
    pal[sp::SHIRT as usize] = rgb(40, 56, 112);
    pal[sp::SHIRT_D as usize] = rgb(24, 32, 72);
    pal[sp::PANTS as usize] = rgb(32, 32, 48);
    pal[sp::PANTS_D as usize] = rgb(16, 16, 32);
    pal[sp::ACC as usize] = rgb(216, 248, 48);
    pal[sp::ACC_D as usize] = rgb(32, 32, 48);
    pal[sp::CAP as usize] = rgb(32, 40, 88);
    pal[sp::CAP_D as usize] = rgb(16, 24, 56);
    pal[sp::WHITE as usize] = rgb(216, 248, 48);
    pal
}

/// The BOA player in 2P: the same uniform, a red cap band so you can tell.
pub fn boa_player_palette(p: usize) -> Palette {
    let mut pal = boa_palette(p * 3);
    pal[sp::CAP as usize] = if p == 0 { rgb(232, 64, 48) } else { rgb(64, 168, 232) };
    pal[sp::CAP_D as usize] = shade(pal[sp::CAP as usize], 2, 3);
    pal
}

/// Pedestrians: tourists (0..100) and the orange crowd (100..).
pub fn ped_palette(k: usize) -> Palette {
    const SHIRTS: [(u32, u32); 6] = [
        (rgb(248, 104, 184), rgb(176, 48, 120)),
        (rgb(64, 200, 248), rgb(32, 120, 200)),
        (rgb(248, 232, 96), rgb(200, 168, 32)),
        (rgb(120, 216, 88), rgb(56, 144, 48)),
        (rgb(232, 232, 240), rgb(160, 168, 192)),
        (rgb(200, 72, 72), rgb(136, 40, 48)),
    ];
    const SKINS: [(u32, u32); 3] = [(rgb(248, 200, 160), rgb(216, 144, 104)), (rgb(224, 160, 104), rgb(176, 112, 64)), (rgb(160, 96, 56), rgb(112, 64, 32))];
    const HAIRS: [(u32, u32); 4] = [(rgb(56, 48, 72), rgb(24, 24, 40)), (rgb(248, 216, 72), rgb(200, 152, 32)), (rgb(136, 80, 32), rgb(88, 48, 16)), (rgb(216, 216, 224), rgb(160, 160, 176))];
    let orange = k >= 100;
    let k = k % 100;
    let s = if orange { [(rgb(248, 136, 24), rgb(192, 80, 16)), (rgb(248, 160, 48), rgb(208, 96, 24))][k % 2] } else { SHIRTS[k % 6] };
    let sk = SKINS[(k / 2) % 3];
    let h = HAIRS[(k / 3) % 4];
    let mut pal = base_pal();
    pal[sp::SKIN as usize] = sk.0;
    pal[sp::SKIN_D as usize] = sk.1;
    pal[sp::HAIR as usize] = if orange && k.is_multiple_of(3) { rgb(248, 136, 24) } else { h.0 };
    pal[sp::HAIR_D as usize] = if orange && k.is_multiple_of(3) { rgb(192, 80, 16) } else { h.1 };
    pal[sp::SHIRT as usize] = s.0;
    pal[sp::SHIRT_D as usize] = s.1;
    let jeans = k.is_multiple_of(2);
    pal[sp::PANTS as usize] = if orange { rgb(248, 248, 248) } else if jeans { rgb(72, 88, 144) } else { rgb(240, 216, 168) };
    pal[sp::PANTS_D as usize] = if orange { rgb(200, 200, 216) } else if jeans { rgb(48, 56, 104) } else { rgb(200, 168, 120) };
    pal[sp::ACC as usize] = if orange { rgb(200, 48, 48) } else { rgb(248, 248, 248) };
    pal[sp::ACC_D as usize] = rgb(184, 184, 208);
    pal[sp::CAP as usize] = if orange { rgb(248, 136, 24) } else { s.1 };
    pal[sp::CAP_D as usize] = if orange { rgb(192, 80, 16) } else { shade(s.1, 3, 4) };
    pal
}

/// Sjaak the fence: striped shirt, black cap, sunglasses (the eyes).
pub fn sjaak_palette() -> Palette {
    let mut pal = base_pal();
    pal[sp::SKIN as usize] = rgb(232, 176, 136);
    pal[sp::SKIN_D as usize] = rgb(192, 128, 96);
    pal[sp::HAIR as usize] = rgb(40, 32, 32);
    pal[sp::HAIR_D as usize] = rgb(16, 16, 16);
    pal[sp::SHIRT as usize] = rgb(232, 232, 240);
    pal[sp::SHIRT_D as usize] = rgb(48, 64, 128);
    pal[sp::PANTS as usize] = rgb(48, 64, 128);
    pal[sp::PANTS_D as usize] = rgb(32, 40, 88);
    pal[sp::ACC as usize] = rgb(48, 64, 128);
    pal[sp::ACC_D as usize] = rgb(32, 40, 88);
    pal[sp::CAP as usize] = rgb(32, 32, 40);
    pal[sp::CAP_D as usize] = rgb(16, 16, 24);
    pal[sp::EYE as usize] = rgb(8, 8, 8);
    pal
}

/// Wet clothes: a little bluer.
pub fn wet_palette(base: &Palette) -> Palette {
    let mut p = *base;
    for (i, c) in p.iter_mut().enumerate() {
        if i != sp::OUT as usize {
            *c = mix(*c, rgb(96, 168, 232), 0.3);
        }
    }
    p
}

/// Draw a person sprite with the feet at (x, y), flipped to face left.
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

/// A rider sprite with the bike anchor at screen (x, y).
pub fn blit_rider(f: &mut Frame, s: &Sprite, x: i32, y: i32, flip: bool, pal: &Palette) {
    for sy in 0..s.h {
        let py = y - RY + sy;
        for sx in 0..s.w {
            let v = s.px[(sy * s.w + sx) as usize];
            if v != sp::T {
                let px = if flip { x + RX - sx } else { x - RX + sx };
                f.set(px, py, pal[v as usize]);
            }
        }
    }
}

/// Everything, built once.
pub struct Art {
    /// one set of poses per head style
    pub people: Vec<People>,
    pub bikes: Vec<Bmp>,
    /// [r4 thin, r4, r5 thin, r5, r5 bent][frame]
    pub wheels: Vec<Vec<Bmp>>,
    pub krul: Bmp,
    pub glasbak: Bmp,
    pub trunk: Bmp,
    pub bollard: Bmp,
    pub heron: Bmp,
    pub heron_fly: [Bmp; 2],
    pub dog: [Bmp; 2],
    pub cat: Bmp,
    pub coot: Bmp,
    pub cutter: Bmp,
    pub ticket: Bmp,
    pub coffee: Bmp,
    pub phone: Bmp,
}

impl Art {
    pub fn new() -> Self {
        let wheels = [(4, true, false), (4, false, false), (5, true, false), (5, false, false), (5, false, true)]
            .iter()
            .map(|&(r, thin, bent)| (0..4).map(|k| build_wheel(r, k, thin, bent)).collect())
            .collect();
        Art {
            people: (0..sp::HAIR_STYLES).map(People::new).collect(),
            bikes: KINDS.iter().map(|&k| build_bike(k)).collect(),
            wheels,
            krul: Bmp::art(&KRUL),
            glasbak: Bmp::art(&GLASBAK),
            trunk: Bmp::art(&TRUNK),
            bollard: Bmp::art(&BOLLARD),
            heron: Bmp::art(&HERON),
            heron_fly: [Bmp::art(&HERON_FLY[0]), Bmp::art(&HERON_FLY[1])],
            dog: [Bmp::art(&DOG[0]), Bmp::art(&DOG[1])],
            cat: Bmp::art(&CAT),
            coot: Bmp::art(&COOT),
            cutter: Bmp::art(&CUTTER),
            ticket: Bmp::art(&TICKET),
            coffee: Bmp::art(&COFFEE),
            phone: Bmp::parse(&PHONE),
        }
    }

    pub fn style(&self, s: usize) -> &People {
        &self.people[s % self.people.len()]
    }

    pub fn wheel(&self, r: i32, thin: bool, bent: bool, frame: usize) -> &Bmp {
        let i = match (r, thin, bent) {
            (_, _, true) => 4,
            (4, true, _) => 0,
            (4, false, _) => 1,
            (_, true, _) => 2,
            _ => 3,
        };
        &self.wheels[i][frame % 4]
    }

    pub fn bike(&self, k: Kind) -> &Bmp {
        &self.bikes[k as usize]
    }
}

impl Default for Art {
    fn default() -> Self {
        Self::new()
    }
}
