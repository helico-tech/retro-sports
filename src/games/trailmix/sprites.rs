//! All of TRAIL MIX!'s art that is built once: runner poses (skeletons
//! rasterised with `engine::sprites::person()` plus painted-on details),
//! animals as string bitmaps with color roles (palette-swapped), and color
//! sprites cut from a scratch frame for props drawn with the engine's
//! primitives. Everything gets the 1px OUTLINE.

use super::runners::{Extra, RUNNERS};
use crate::engine::gfx::{rgb, Frame};
use crate::engine::sprites::{self as es, person, Skel, Sprite};
use crate::engine::ui::OUTLINE;

// ---------------------------------------------------------------- poses

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TPose {
    Run1,
    Run2,
    Run3,
    Run4,
    Up1,
    Up2,
    Up3,
    Up4,
    Down1,
    Down2,
    Down3,
    Down4,
    Sprint1,
    Sprint2,
    Sprint3,
    Sprint4,
    Walk1,
    Walk2,
    Walk3,
    Walk4,
    Hike1,
    Hike2,
    Bonk1,
    Bonk2,
    Duck1,
    Duck2,
    Jump,
    Land,
    Curl,
    Faceplant,
    GetUp,
    Glissade1,
    Glissade2,
    Dazed,
    Grab,
    HighFive,
    Ready,
    Stand,
    Victory,
    Cheer1,
    Cheer2,
    Offer,
    Tumble2,
    Tumble3,
    Tumble4,
}

pub const ALL_TPOSES: [TPose; 45] = [
    TPose::Run1,
    TPose::Run2,
    TPose::Run3,
    TPose::Run4,
    TPose::Up1,
    TPose::Up2,
    TPose::Up3,
    TPose::Up4,
    TPose::Down1,
    TPose::Down2,
    TPose::Down3,
    TPose::Down4,
    TPose::Sprint1,
    TPose::Sprint2,
    TPose::Sprint3,
    TPose::Sprint4,
    TPose::Walk1,
    TPose::Walk2,
    TPose::Walk3,
    TPose::Walk4,
    TPose::Hike1,
    TPose::Hike2,
    TPose::Bonk1,
    TPose::Bonk2,
    TPose::Duck1,
    TPose::Duck2,
    TPose::Jump,
    TPose::Land,
    TPose::Curl,
    TPose::Faceplant,
    TPose::GetUp,
    TPose::Glissade1,
    TPose::Glissade2,
    TPose::Dazed,
    TPose::Grab,
    TPose::HighFive,
    TPose::Ready,
    TPose::Stand,
    TPose::Victory,
    TPose::Cheer1,
    TPose::Cheer2,
    TPose::Offer,
    TPose::Tumble2,
    TPose::Tumble3,
    TPose::Tumble4,
];

/// Swap the two arms and the two legs of a skeleton (the other half of a stride).
fn swap(k: Skel) -> Skel {
    let mut s = k;
    s.swap(3, 5);
    s.swap(4, 6);
    s.swap(7, 9);
    s.swap(8, 10);
    s
}

/// Joints (24x32 canvas, facing right): head, neck, hip, back elbow, back
/// hand, front elbow, front hand, back knee, back foot, front knee, front foot.
pub fn skel(p: TPose) -> Skel {
    const RUN1: Skel = [(15, 6), (14, 11), (12, 19), (16, 14), (19, 12), (10, 15), (8, 18), (9, 24), (5, 27), (16, 23), (18, 29)];
    const RUN2: Skel = [(15, 5), (14, 10), (12, 18), (13, 15), (14, 19), (14, 15), (16, 18), (11, 23), (9, 28), (14, 23), (13, 29)];
    const UP1: Skel = [(17, 8), (15, 12), (11, 19), (17, 15), (20, 13), (11, 16), (9, 19), (9, 24), (6, 29), (16, 21), (17, 27)];
    const UP2: Skel = [(17, 7), (15, 11), (11, 18), (14, 15), (16, 18), (13, 15), (15, 19), (12, 23), (11, 29), (15, 22), (15, 28)];
    const DOWN1: Skel = [(12, 5), (12, 10), (12, 18), (9, 13), (5, 11), (16, 13), (19, 12), (10, 23), (6, 27), (16, 23), (19, 29)];
    const DOWN2: Skel = [(12, 5), (12, 10), (12, 18), (8, 13), (5, 12), (16, 13), (19, 11), (12, 23), (11, 29), (14, 22), (15, 28)];
    const SPRINT1: Skel = [(16, 6), (15, 11), (12, 18), (17, 13), (20, 9), (9, 15), (7, 19), (8, 22), (3, 25), (18, 21), (20, 28)];
    const SPRINT2: Skel = [(16, 5), (15, 10), (12, 17), (14, 14), (17, 16), (12, 14), (13, 18), (10, 22), (8, 28), (15, 21), (15, 27)];
    const WALK1: Skel = [(12, 5), (12, 10), (12, 19), (13, 15), (15, 19), (11, 15), (9, 19), (11, 24), (9, 29), (14, 24), (15, 29)];
    const WALK2: Skel = [(12, 5), (12, 10), (12, 19), (12, 15), (12, 20), (12, 15), (13, 20), (12, 24), (12, 29), (13, 24), (13, 29)];
    const HIKE1: Skel = [(17, 9), (15, 13), (10, 19), (16, 16), (19, 18), (13, 16), (14, 21), (8, 24), (6, 29), (15, 22), (16, 27)];
    const BONK1: Skel = [(16, 11), (14, 14), (10, 20), (14, 18), (14, 23), (15, 18), (16, 23), (11, 25), (9, 29), (14, 24), (15, 29)];
    const BONK2: Skel = [(16, 12), (14, 15), (10, 20), (14, 19), (14, 23), (15, 19), (16, 23), (12, 25), (12, 29), (13, 25), (13, 29)];
    const DUCK1: Skel = [(16, 14), (14, 17), (9, 22), (15, 20), (18, 19), (12, 21), (14, 24), (8, 26), (5, 29), (14, 25), (15, 29)];
    const DUCK2: Skel = [(16, 14), (14, 17), (9, 22), (12, 21), (14, 24), (15, 20), (18, 19), (12, 26), (11, 29), (11, 25), (9, 29)];
    match p {
        TPose::Run1 => RUN1,
        TPose::Run2 => RUN2,
        TPose::Run3 => swap(RUN1),
        TPose::Run4 => swap(RUN2),
        TPose::Up1 => UP1,
        TPose::Up2 => UP2,
        TPose::Up3 => swap(UP1),
        TPose::Up4 => swap(UP2),
        TPose::Down1 => DOWN1,
        TPose::Down2 => DOWN2,
        TPose::Down3 => {
            // arms stay out for balance, only the legs swap
            let mut k = DOWN1;
            k.swap(7, 9);
            k.swap(8, 10);
            k
        }
        TPose::Down4 => {
            let mut k = DOWN2;
            k.swap(7, 9);
            k.swap(8, 10);
            k
        }
        TPose::Sprint1 => SPRINT1,
        TPose::Sprint2 => SPRINT2,
        TPose::Sprint3 => swap(SPRINT1),
        TPose::Sprint4 => swap(SPRINT2),
        TPose::Walk1 => WALK1,
        TPose::Walk2 => WALK2,
        TPose::Walk3 => swap(WALK1),
        TPose::Walk4 => swap(WALK2),
        TPose::Hike1 => HIKE1,
        TPose::Hike2 => swap(HIKE1),
        TPose::Bonk1 => BONK1,
        TPose::Bonk2 => BONK2,
        TPose::Duck1 => DUCK1,
        TPose::Duck2 => DUCK2,
        TPose::Jump => [(15, 4), (14, 9), (12, 17), (16, 11), (19, 8), (9, 12), (6, 10), (8, 20), (3, 22), (17, 19), (20, 24)],
        TPose::Land => [(14, 6), (13, 11), (12, 19), (9, 13), (5, 12), (16, 13), (20, 12), (9, 23), (7, 28), (16, 23), (18, 28)],
        // a ball, rotated for the tumble
        TPose::Curl | TPose::Tumble2 | TPose::Tumble3 | TPose::Tumble4 => {
            [(15, 16), (13, 19), (9, 24), (13, 22), (16, 25), (15, 21), (17, 24), (13, 26), (9, 28), (14, 25), (11, 28)]
        }
        // flat on the belly, face in the dirt, feet kicking in the air
        TPose::Faceplant => [(20, 27), (17, 27), (9, 27), (19, 29), (23, 30), (20, 28), (23, 28), (5, 27), (3, 21), (6, 28), (6, 22)],
        TPose::GetUp => [(14, 12), (13, 16), (10, 23), (13, 20), (15, 24), (14, 20), (17, 24), (9, 29), (4, 30), (15, 25), (15, 30)],
        // sitting on the snow, feet first, one arm waving
        TPose::Glissade1 => [(8, 14), (8, 19), (9, 27), (5, 23), (3, 27), (12, 18), (16, 13), (15, 26), (21, 29), (14, 25), (20, 28)],
        TPose::Glissade2 => [(8, 14), (8, 19), (9, 27), (5, 23), (3, 27), (13, 17), (15, 11), (15, 26), (21, 29), (14, 25), (20, 28)],
        TPose::Dazed => [(10, 6), (11, 11), (12, 19), (10, 15), (9, 20), (14, 15), (16, 19), (11, 24), (9, 29), (14, 24), (15, 29)],
        TPose::Grab => {
            let mut k = RUN2;
            k[5] = (17, 12);
            k[6] = (21, 11);
            k
        }
        TPose::HighFive => {
            let mut k = RUN1;
            k[5] = (16, 8);
            k[6] = (18, 4);
            k
        }
        TPose::Ready => [(16, 9), (14, 13), (10, 20), (15, 16), (17, 18), (12, 17), (13, 20), (8, 25), (5, 29), (15, 24), (15, 29)],
        TPose::Stand => [(12, 5), (12, 10), (12, 19), (8, 15), (8, 20), (16, 15), (16, 20), (10, 24), (10, 29), (14, 24), (14, 29)],
        TPose::Victory => [(12, 5), (12, 10), (12, 19), (8, 7), (4, 2), (16, 7), (20, 2), (10, 24), (9, 29), (14, 24), (15, 29)],
        TPose::Cheer1 => [(12, 5), (12, 10), (12, 19), (8, 15), (8, 20), (16, 8), (17, 3), (10, 24), (10, 29), (14, 24), (14, 29)],
        TPose::Cheer2 => [(12, 6), (12, 11), (12, 19), (8, 15), (7, 20), (17, 11), (20, 8), (10, 24), (10, 29), (14, 24), (14, 29)],
        TPose::Offer => [(12, 5), (12, 10), (12, 19), (9, 15), (9, 20), (16, 14), (20, 13), (10, 24), (10, 29), (14, 24), (14, 29)],
    }
}

/// Front hand in canvas coordinates (for cowbells, items and poles).
pub fn hand(p: TPose) -> (i32, i32) {
    skel(p)[6]
}
pub fn back_hand(p: TPose) -> (i32, i32) {
    skel(p)[4]
}

fn set(s: &mut Sprite, x: i32, y: i32, v: u8) {
    if x >= 0 && y >= 0 && x < s.w && y < s.h {
        s.px[(y * s.w + x) as usize] = v;
    }
}

fn get(s: &Sprite, x: i32, y: i32) -> u8 {
    s.get(x, y)
}

/// Add a 1px outline around everything that is not already outline.
pub fn outline(s: &mut Sprite) {
    let src = s.px.clone();
    let at = |x: i32, y: i32| -> u8 {
        if x >= 0 && y >= 0 && x < s.w && y < s.h { src[(y * s.w + x) as usize] } else { es::T }
    };
    for y in 0..s.h {
        for x in 0..s.w {
            if at(x, y) != es::T {
                continue;
            }
            if [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| {
                let v = at(x + dx, y + dy);
                v != es::T && v != es::OUT
            }) {
                s.px[(y * s.w + x) as usize] = es::OUT;
            }
        }
    }
}

/// Paint a runner's trademark on top of the plain person.
fn decorate(s: &mut Sprite, k: &Skel, extra: Extra, pose: TPose) {
    let (hd, neck, hip) = (k[0], k[1], k[2]);
    let flat = matches!(pose, TPose::Faceplant);
    match extra {
        Extra::Beard => {
            // a big ginger beard under the face
            for dy in 1..=6 {
                let (a, b) = match dy {
                    1 => (1, 3),
                    2 => (-1, 4),
                    3..=4 => (-2, 4),
                    5 => (-1, 3),
                    _ => (0, 2),
                };
                for dx in a..=b {
                    let (x, y) = (hd.0 + dx, hd.1 + dy);
                    let v = get(s, x, y);
                    if v == es::EYE {
                        continue;
                    }
                    if dy == 2 && dx == 3 {
                        continue; // mouth
                    }
                    set(s, x, y, if dx == a || dy == 6 { es::HAIR_D } else { es::HAIR });
                }
            }
        }
        Extra::Vest => {
            // hydration vest: straps and a pack on the back
            if !flat {
                for t in 0..=6 {
                    let x = neck.0 + (hip.0 - neck.0) * t / 6;
                    let y = neck.1 + 2 + (hip.1 - neck.1 - 3) * t / 6;
                    for dx in [-1, 2] {
                        if get(s, x + dx, y) == es::SHIRT {
                            set(s, x + dx, y, es::ACC);
                        }
                    }
                }
                let (bx, by) = (neck.0 - 4, neck.1 + 2);
                for y in by..by + 6 {
                    for x in bx..bx + 3 {
                        let v = get(s, x, y);
                        if v == es::T || v == es::OUT || v == es::SHIRT || v == es::SHIRT_D {
                            set(s, x, y, if x == bx { es::ACC_D } else { es::ACC });
                        }
                    }
                }
            }
        }
        Extra::Socks => {
            // tall striped socks
            for &(kn, ft) in &[(k[7], k[8]), (k[9], k[10])] {
                for step in 1..=5 {
                    let x = ft.0 + (kn.0 - ft.0) * step / 8;
                    let y = ft.1 + (kn.1 - ft.1) * step / 8;
                    for dx in -1..=1 {
                        let v = get(s, x + dx, y);
                        if v == es::SKIN || v == es::SKIN_D {
                            set(s, x + dx, y, if step % 2 == 0 { es::ACC } else { es::WHITE });
                        }
                    }
                }
            }
        }
        Extra::Apron => {
            // a flowery apron over the front
            if !flat {
                for y in neck.1 + 4..hip.1 + 4 {
                    let t = (y - neck.1) as f32 / (hip.1 - neck.1).max(1) as f32;
                    let cx = neck.0 as f32 + (hip.0 - neck.0) as f32 * t;
                    for x in (cx as i32)..(cx as i32 + 4) {
                        let v = get(s, x, y);
                        if v == es::SHIRT || v == es::PANTS || v == es::SHIRT_D {
                            set(s, x, y, if (x + y) % 3 == 0 { es::ACC_D } else { es::WHITE });
                        }
                    }
                }
            }
        }
    }
    outline(s);
}

/// Rotate a sprite 90 degrees clockwise `n` times around the curl's centre.
fn rotated(s: &Sprite, n: i32) -> Sprite {
    let mut out = Sprite::new(s.w, s.h);
    let (cx, cy) = (12, 23);
    for y in 0..s.h {
        for x in 0..s.w {
            let v = s.px[(y * s.w + x) as usize];
            if v == es::T {
                continue;
            }
            let (mut dx, mut dy) = (x - cx, y - cy);
            for _ in 0..n {
                let t = dx;
                dx = -dy;
                dy = t;
            }
            set(&mut out, cx + dx, cy + dy, v);
        }
    }
    out
}

pub fn render_runner(def: usize, pose: TPose) -> Sprite {
    let d = &RUNNERS[def];
    render_person(d.style, pose, Some(d.extra))
}

pub fn render_person(style: usize, pose: TPose, extra: Option<Extra>) -> Sprite {
    let base = match pose {
        TPose::Tumble2 | TPose::Tumble3 | TPose::Tumble4 => TPose::Curl,
        p => p,
    };
    let k = skel(base);
    let mut s = person(&k, style, base == TPose::Faceplant, false);
    if let Some(e) = extra {
        decorate(&mut s, &k, e, base);
    }
    match pose {
        TPose::Tumble2 => rotated(&s, 1),
        TPose::Tumble3 => rotated(&s, 2),
        TPose::Tumble4 => rotated(&s, 3),
        _ => s,
    }
}

// ---------------------------------------------------------------- bitmaps

/// A string-bitmap sprite in color roles; `pal` maps a role to a color.
#[derive(Clone)]
pub struct Bm {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
}

pub type BPal = [u32; 24];

fn role(c: u8) -> u8 {
    match c {
        b'O' => 1,
        b'W' => 2,
        b'w' => 3,
        b'g' => 4,
        b'H' => 5,
        b'h' => 6,
        b'K' => 7,
        b'P' => 8,
        b'R' => 9,
        b'r' => 10,
        b'Y' => 11,
        b'y' => 12,
        b'B' => 13,
        b'b' => 14,
        b'G' => 15,
        b'E' => 16,
        b'L' => 17,
        b'N' => 18,
        b'n' => 19,
        b'S' => 20,
        b's' => 21,
        _ => 0,
    }
}

impl Bm {
    pub fn from(rows: &[&str]) -> Bm {
        let h = rows.len() as i32 + 2;
        let w = rows.iter().map(|r| r.len()).max().unwrap_or(0) as i32 + 2;
        let mut px = vec![0u8; (w * h) as usize];
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.bytes().enumerate() {
                px[((y as i32 + 1) * w + x as i32 + 1) as usize] = role(c);
            }
        }
        let mut b = Bm { w, h, px };
        b.outline();
        b
    }
    fn outline(&mut self) {
        let src = self.px.clone();
        let (w, h) = (self.w, self.h);
        let at = |x: i32, y: i32| if x >= 0 && y >= 0 && x < w && y < h { src[(y * w + x) as usize] } else { 0 };
        for y in 0..h {
            for x in 0..w {
                if at(x, y) == 0 && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| at(x + dx, y + dy) > 1) {
                    self.px[(y * w + x) as usize] = 1;
                }
            }
        }
    }
    /// Draw with the bottom-centre at (x, y).
    pub fn draw(&self, f: &mut Frame, x: i32, y: i32, flip: bool, pal: &BPal) {
        let ox = x - self.w / 2;
        let oy = y - self.h + 1;
        for sy in 0..self.h {
            let py = oy + sy;
            if !(0..crate::engine::gfx::H).contains(&py) {
                continue;
            }
            for sx in 0..self.w {
                let v = self.px[(sy * self.w + sx) as usize];
                if v == 0 {
                    continue;
                }
                let px = if flip { ox + self.w - 1 - sx } else { ox + sx };
                f.set(px, py, pal[v as usize]);
            }
        }
    }
    /// Squashed vertically (a flattened marmot).
    pub fn draw_squash(&self, f: &mut Frame, x: i32, y: i32, pal: &BPal, keep: i32) {
        let ox = x - self.w / 2;
        for sy in 0..self.h {
            let ty = y - (self.h - 1 - sy) * keep / self.h;
            for sx in 0..self.w {
                let v = self.px[(sy * self.w + sx) as usize];
                if v != 0 {
                    f.set(ox + sx, ty, pal[v as usize]);
                }
            }
        }
    }
}

pub fn bpal(entries: &[(u8, u32)]) -> BPal {
    let mut p = [0u32; 24];
    p[1] = OUTLINE;
    for &(c, col) in entries {
        p[role(c) as usize] = col;
    }
    p
}

// Gerald the goat, 3 frames running + a leap + a bleat + chewing.
const GOAT_RUN1: [&str; 18] = [
    "...................HHh..",
    "..................HHh...",
    "................WHh.....",
    "...............WWWWW....",
    "...............WWWKWWP..",
    "..............wWWWWWWPP.",
    ".WW...........wWWWWWW...",
    "WWw..........RRwWWBB....",
    ".WWWWWWWWWWWWRYWWBB.....",
    "WWWWWWWWWWWWWyWWWB......",
    "WWWWWWWWWWWWWWWW........",
    "wWWWWWWWWWWWWWww........",
    ".wwwwwwwwwwwwww.........",
    ".gw.ww......ww.w........",
    "gw...ww....w...ww.......",
    "g.....w...w.....w.......",
    "L.....LL.L.......L......",
    "........................",
];
const GOAT_RUN2: [&str; 18] = [
    "...................HHh..",
    "..................HHh...",
    "................WHh.....",
    "...............WWWWW....",
    "...............WWWKWWP..",
    "..............wWWWWWWPP.",
    "..............wWWWWWW...",
    "WWw..........RRwWWBB....",
    "WWWWWWWWWWWWWRYWWBB.....",
    "WWWWWWWWWWWWWyWWWB......",
    "WWWWWWWWWWWWWWWW........",
    "wWWWWWWWWWWWWWww........",
    ".wwwwwwwwwwwwww.........",
    "..ww.w......wwww........",
    "..ww.w.....ww.ww........",
    "..w..w.....w...w........",
    "..L..L.....L...L........",
    "........................",
];
const GOAT_LEAP: [&str; 18] = [
    ".....................HH.",
    "....................HHh.",
    "..................WHh...",
    ".................WWWWW..",
    ".................WWWKWWP",
    "................wWWWWWWP",
    "WW..............wWWWWW..",
    "WWw...........RRwWBB....",
    ".WWWWWWWWWWWWWRYWBB.....",
    "WWWWWWWWWWWWWWyWWB......",
    "WWWWWWWWWWWWWWWWw.......",
    "wWWWWWWWWWWWWWWwww......",
    "wwwwwwwwwwwwwww...ww....",
    "gww............w....wL..",
    "...ww...............L...",
    ".....wL.................",
    "......L.................",
    "........................",
];
const GOAT_BLEAT: [&str; 18] = [
    "..................HHh...",
    ".................HHh....",
    "...............WHh......",
    "..............WWWWW.....",
    "..............WWKWWWP...",
    ".............wWWWWWWKP..",
    ".WW..........wWWWWWPP...",
    "WWw..........RwWWBBPP...",
    ".WWWWWWWWWWWWRYWBB......",
    "WWWWWWWWWWWWWyWWB.......",
    "WWWWWWWWWWWWWWWW........",
    "wWWWWWWWWWWWWWww........",
    ".wwwwwwwwwwwwww.........",
    "..ww.w......ww.w........",
    "..ww.w......ww.w........",
    "..w..w......w..w........",
    "..L..L......L..L........",
    "........................",
];

const COW: [&str; 22] = [
    ".......................HE.....",
    "......................WWWh..E.",
    "....................WWWWWWWH..",
    "...................WWWKWWWW...",
    ".w.................WWWWWWWPP..",
    "wW.BBBBWWWWWBBBBWWWWWWWWPPP...",
    "wBBBBBBWWWWWBBBBBWWWWWWWPKP...",
    "..BBBBWWWWWWWBBBWWWWWW.PPP....",
    "..BBWWWWWWWWWWWWWWWWYY........",
    "..WWWWWBBBBWWWWWWWWYyY........",
    "..WWWWBBBBBBWWWWWWWWYY........",
    "..wWWWBBBBBWWWWWWWWw..........",
    "..wwWWWWWWWWWWWWWwww..........",
    "...wwwPPwwwwwwwwwww...........",
    "...ww.PP....www.ww............",
    "...ww.......ww..ww............",
    "...ww.......ww..ww............",
    "...ww.......ww..ww............",
    "...KK.......KK..KK............",
    "..............................",
    "..............................",
    "..............................",
];
const COW_CHEW: [&str; 22] = [
    "..............................",
    "..............................",
    "..............................",
    ".......................HE.....",
    ".w....................WWWh..E.",
    "wW.BBBBWWWWWBBBBWWWWWWWWWWH...",
    "wBBBBBBWWWWWBBBBBWWWWWWKWWW...",
    "..BBBBWWWWWWWBBBWWWWWWWWWWW...",
    "..BBWWWWWWWWWWWWWWWWYYWWWPP...",
    "..WWWWWBBBBWWWWWWWWYyYWWPPKP..",
    "..WWWWBBBBBBWWWWWWWWYY.PPPP...",
    "..wWWWBBBBBWWWWWWWWw..G.G.....",
    "..wwWWWWWWWWWWWWWwww..GGG.....",
    "...wwwPPwwwwwwwwwww...........",
    "...ww.PP....www.ww............",
    "...ww.......ww..ww............",
    "...ww.......ww..ww............",
    "...ww.......ww..ww............",
    "...KK.......KK..KK............",
    "..............................",
    "..............................",
    "..............................",
];

const MARMOT_UP: [&str; 12] = [
    "..BBB...",
    ".BBKBB..",
    ".BBBBPb.",
    "..bBBb..",
    ".BWWWB..",
    "BBWWWBB.",
    ".BWWWB..",
    ".BBBBB..",
    ".BBBBB..",
    ".bBBBb..",
    "bb..bb..",
    "........",
];
const MARMOT_SLEEP: [&str; 6] = ["....bBBBBb..", "..BBBBBBBBB.", ".BBBBBBBBKBB", "bBBBBBBBBBPb", "bbBBBBBBBBb.", ".bbbbbbbbb.."];

const DEER: [&str; 20] = [
    "......................h.h",
    "......................hhh",
    ".....................BBB.",
    "....................BBKBB",
    "....................BBBBK",
    "...................BBBB..",
    "..................BBB....",
    ".E................BBB....",
    "EEBBBBBBBBBBBBBBBBBB.....",
    ".BBBBBEBBBBBEBBBBBBB.....",
    ".BBBBBBBBBEBBBBBBBB......",
    "..BBBBBBBBBBBBBBBB.......",
    "..bbbbbbbbbbbbbbbb.......",
    "..bb..b.........bb.......",
    ".bb....b.........bb......",
    "bb......b.........bb.....",
    "K........b.........b.....",
    ".........K.........K.....",
    "........................",
    "........................",
];

const DEER2: [&str; 20] = [
    "......................h.h",
    "......................hhh",
    ".....................BBB.",
    "....................BBKBB",
    "....................BBBBK",
    "...................BBBB..",
    "..................BBB....",
    ".E................BBB....",
    "EEBBBBBBBBBBBBBBBBBB.....",
    ".BBBBBEBBBBBEBBBBBBB.....",
    ".BBBBBBBBBEBBBBBBBB......",
    "..BBBBBBBBBBBBBBBB.......",
    "..bbbbbbbbbbbbbbbbb......",
    "...bb.bb......bb.bb......",
    "....bbb........bbb.......",
    ".....K..........K........",
    "........................",
    "........................",
    "........................",
    "........................",
];

const COWBELL: [&str; 5] = [".YY.", "YYYY", "YyyY", "yyyy", ".KK."];

const ITEMS: [[&str; 7]; 4] = [
    // trail mix bag
    [".RRRR.", "RRRRRR", "RYBYBR", "RBYGYR", "RYBYBR", "RRRRRR", "......"],
    // banana
    ["....b.", "...YY.", "..YYy.", ".YYy..", "YYy...", ".y....", "......"],
    // stroopwafel
    [".BBBB.", "BbBbBB", "BBbBbB", "BbBbBB", "BBbBbB", ".BBBB.", "......"],
    // pickle jar
    [".LLLL.", "EGGGGE", "EGgGGE", "EGGgGE", "EGgGGE", "EGGGGE", ".EEEE."],
];

pub struct Art {
    /// [runner][pose]
    pub runners: Vec<Vec<Sprite>>,
    /// [head style][Cheer1, Cheer2, Offer, Stand]
    pub crowd: Vec<Vec<Sprite>>,
    pub goat: [Bm; 4],
    pub cow: [Bm; 2],
    pub marmot_up: Bm,
    pub marmot_sleep: Bm,
    pub deer: Bm,
    /// legs tucked, mid-leap
    pub deer2: Bm,
    pub cowbell: Bm,
    pub items: Vec<Bm>,
    pub goat_pal: BPal,
    pub cow_pals: [BPal; 2],
    pub marmot_pal: BPal,
    pub deer_pal: BPal,
    pub bell_pal: BPal,
    pub item_pal: BPal,
}

pub const CROWD_POSES: [TPose; 4] = [TPose::Cheer1, TPose::Cheer2, TPose::Offer, TPose::Stand];

impl Art {
    pub fn new() -> Art {
        let runners = (0..4).map(|d| ALL_TPOSES.iter().map(|&p| render_runner(d, p)).collect()).collect();
        let crowd = (0..es::HAIR_STYLES).map(|st| CROWD_POSES.iter().map(|&p| render_person(st, p, None)).collect()).collect();
        let fur = rgb(248, 248, 240);
        Art {
            runners,
            crowd,
            goat: [Bm::from(&GOAT_RUN1), Bm::from(&GOAT_RUN2), Bm::from(&GOAT_LEAP), Bm::from(&GOAT_BLEAT)],
            cow: [Bm::from(&COW), Bm::from(&COW_CHEW)],
            marmot_up: Bm::from(&MARMOT_UP),
            marmot_sleep: Bm::from(&MARMOT_SLEEP),
            deer: Bm::from(&DEER),
            deer2: Bm::from(&DEER2),
            cowbell: Bm::from(&COWBELL),
            items: ITEMS.iter().map(|r| Bm::from(r)).collect(),
            goat_pal: bpal(&[
                (b'W', fur),
                (b'w', rgb(200, 200, 208)),
                (b'g', rgb(136, 136, 152)),
                (b'H', rgb(216, 184, 120)),
                (b'h', rgb(152, 120, 72)),
                (b'K', OUTLINE),
                (b'P', rgb(248, 168, 168)),
                (b'R', rgb(232, 48, 48)),
                (b'Y', rgb(248, 216, 48)),
                (b'y', rgb(192, 144, 24)),
                (b'B', rgb(232, 232, 224)),
                (b'L', rgb(64, 56, 56)),
            ]),
            cow_pals: [
                bpal(&[
                    (b'W', rgb(248, 248, 240)),
                    (b'w', rgb(192, 192, 200)),
                    (b'B', rgb(48, 40, 48)),
                    (b'H', rgb(232, 224, 184)),
                    (b'h', rgb(168, 152, 120)),
                    (b'E', rgb(48, 40, 48)),
                    (b'K', OUTLINE),
                    (b'P', rgb(248, 176, 176)),
                    (b'Y', rgb(248, 216, 48)),
                    (b'y', rgb(184, 136, 24)),
                    (b'G', rgb(88, 176, 64)),
                ]),
                bpal(&[
                    (b'W', rgb(184, 120, 72)),
                    (b'w', rgb(136, 88, 48)),
                    (b'B', rgb(216, 176, 128)),
                    (b'H', rgb(232, 224, 184)),
                    (b'h', rgb(168, 152, 120)),
                    (b'E', rgb(112, 64, 32)),
                    (b'K', OUTLINE),
                    (b'P', rgb(232, 176, 160)),
                    (b'Y', rgb(248, 216, 48)),
                    (b'y', rgb(184, 136, 24)),
                    (b'G', rgb(88, 176, 64)),
                ]),
            ],
            marmot_pal: bpal(&[(b'B', rgb(176, 120, 72)), (b'b', rgb(120, 80, 48)), (b'W', rgb(232, 200, 152)), (b'K', OUTLINE), (b'P', rgb(64, 40, 32))]),
            deer_pal: bpal(&[(b'B', rgb(192, 120, 64)), (b'b', rgb(144, 88, 48)), (b'E', rgb(248, 240, 224)), (b'K', OUTLINE), (b'h', rgb(136, 96, 64))]),
            bell_pal: bpal(&[(b'Y', rgb(248, 216, 48)), (b'y', rgb(184, 136, 24)), (b'K', OUTLINE)]),
            item_pal: bpal(&[
                (b'R', rgb(216, 56, 48)),
                (b'Y', rgb(248, 224, 72)),
                (b'y', rgb(200, 160, 32)),
                (b'B', rgb(200, 136, 64)),
                (b'b', rgb(144, 88, 40)),
                (b'G', rgb(112, 184, 64)),
                (b'g', rgb(72, 128, 40)),
                (b'E', rgb(200, 232, 240)),
                (b'L', rgb(168, 168, 184)),
            ]),
        }
    }

    pub fn runner(&self, def: usize, p: TPose) -> &Sprite {
        let i = ALL_TPOSES.iter().position(|&q| q == p).unwrap_or(0);
        &self.runners[def.min(3)][i]
    }

    pub fn crowd(&self, style: usize, k: usize) -> &Sprite {
        &self.crowd[style % es::HAIR_STYLES][k % 4]
    }
}

impl Default for Art {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------- color sprites

/// Marker for "no pixel" in a color sprite.
pub const KEY: u32 = 0x0100_0000;

/// A sprite in plain colors (props drawn with the engine's primitives).
#[derive(Clone)]
pub struct CSprite {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u32>,
    /// anchor: bottom centre by default
    pub ax: i32,
    pub ay: i32,
}

impl CSprite {
    /// Cut a rectangle out of a frame that was cleared to KEY, and outline it.
    pub fn cut(f: &Frame, x: i32, y: i32, w: i32, h: i32, outline_it: bool) -> CSprite {
        let (w2, h2) = (w + 2, h + 2);
        let mut px = vec![KEY; (w2 * h2) as usize];
        for yy in 0..h {
            for xx in 0..w {
                let c = f.get(x + xx, y + yy);
                px[((yy + 1) * w2 + xx + 1) as usize] = c;
            }
        }
        let mut s = CSprite { w: w2, h: h2, px, ax: w2 / 2, ay: h2 - 2 };
        if outline_it {
            let src = s.px.clone();
            let at = |x: i32, y: i32| if x >= 0 && y >= 0 && x < w2 && y < h2 { src[(y * w2 + x) as usize] } else { KEY };
            for yy in 0..h2 {
                for xx in 0..w2 {
                    if at(xx, yy) == KEY && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| at(xx + dx, yy + dy) != KEY) {
                        s.px[(yy * w2 + xx) as usize] = OUTLINE;
                    }
                }
            }
        }
        s
    }
    pub fn draw(&self, f: &mut Frame, x: i32, y: i32) {
        let ox = x - self.ax;
        let oy = y - self.ay;
        for sy in 0..self.h {
            let py = oy + sy;
            if !(0..crate::engine::gfx::H).contains(&py) {
                continue;
            }
            let row = (sy * self.w) as usize;
            for sx in 0..self.w {
                let c = self.px[row + sx as usize];
                if c != KEY {
                    f.set(ox + sx, py, c);
                }
            }
        }
    }
    /// Draw only rows inside [y0, y1) of the screen.
    pub fn draw_clip(&self, f: &mut Frame, x: i32, y: i32, y0: i32, y1: i32) {
        let ox = x - self.ax;
        let oy = y - self.ay;
        for sy in 0..self.h {
            let py = oy + sy;
            if py < y0 || py >= y1 {
                continue;
            }
            let row = (sy * self.w) as usize;
            for sx in 0..self.w {
                let c = self.px[row + sx as usize];
                if c != KEY {
                    f.set(ox + sx, py, c);
                }
            }
        }
    }
}

/// A scratch frame cleared to KEY, for drawing props before cutting them out.
pub fn scratch() -> Frame {
    let mut f = Frame::new();
    f.clear(KEY);
    f
}
