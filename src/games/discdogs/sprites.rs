//! DISC DOGS! art: dogs built from a small dog skeleton plus a hand-drawn
//! head per breed, the critters and props as string bitmaps, and the owners
//! as engine `person()` skeletons. Everything is drawn once in color roles
//! with a 1px outline, then palette-swapped at draw time.

use super::data::{Breed, PAIRS};
use crate::engine::gfx::{line_points, mix, rgb, shade, Frame};
use crate::engine::sprites::{self as es, person, Palette, Skel, Sprite};
use crate::engine::ui::OUTLINE;

// ------------------------------------------------------------------ role sprites

pub const T: u8 = 0;
pub const OUT: u8 = 1;
pub const FUR: u8 = 2;
pub const FUR_D: u8 = 3;
pub const LIGHT: u8 = 4;
pub const LIGHT_D: u8 = 5;
pub const DARK: u8 = 6;
pub const DARK_D: u8 = 7;
pub const EYE: u8 = 8;
pub const PINK: u8 = 9;
pub const WHITE: u8 = 10;
pub const COLLAR: u8 = 11;
pub const COLLAR_D: u8 = 12;
pub const GOLD: u8 = 13;
pub const ROLES: usize = 14;

pub type DogPal = [u32; ROLES];

/// A bitmap of color roles with an origin (the point that sits on the ground).
#[derive(Clone)]
pub struct RSprite {
    pub w: i32,
    pub h: i32,
    pub ox: i32,
    pub oy: i32,
    pub px: Vec<u8>,
}

impl RSprite {
    pub fn new(w: i32, h: i32, ox: i32, oy: i32) -> Self {
        RSprite { w, h, ox, oy, px: vec![T; (w * h) as usize] }
    }

    pub fn get(&self, x: i32, y: i32) -> u8 {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize]
        } else {
            T
        }
    }

    pub fn set(&mut self, x: i32, y: i32, v: u8) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = v;
        }
    }

    fn thick(&mut self, a: (i32, i32), b: (i32, i32), size: i32, v: u8) {
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

    fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, angle: f32, v: u8) {
        let (c, s) = (angle.cos(), angle.sin());
        let r = rx.max(ry) as i32 + 1;
        for y in (cy as i32 - r)..=(cy as i32 + r) {
            for x in (cx as i32 - r)..=(cx as i32 + r) {
                let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                let (u, w) = (dx * c + dy * s, -dx * s + dy * c);
                if (u / rx).powi(2) + (w / ry).powi(2) <= 1.0 {
                    self.set(x, y, v);
                }
            }
        }
    }

    /// Stamp a string bitmap (role letters) with its top-left at (x, y).
    pub fn stamp(&mut self, x: i32, y: i32, rows: &[&str]) {
        for (ry, row) in rows.iter().enumerate() {
            for (rx, ch) in row.bytes().enumerate() {
                let v = role(ch);
                if v != T {
                    self.set(x + rx as i32, y + ry as i32, v);
                }
            }
        }
    }

    pub fn from_rows(rows: &[&str], ox: i32, oy: i32) -> Self {
        let h = rows.len() as i32;
        let w = rows.iter().map(|r| r.len()).max().unwrap_or(0) as i32;
        let mut s = RSprite::new(w + 2, h + 2, ox + 1, oy + 1);
        s.stamp(1, 1, rows);
        s.outline();
        s
    }

    /// 1px outline around the silhouette.
    pub fn outline(&mut self) {
        let src = self.px.clone();
        let at = |x: i32, y: i32| -> u8 {
            if x >= 0 && y >= 0 && x < self.w && y < self.h {
                src[(y * self.w + x) as usize]
            } else {
                T
            }
        };
        for y in 0..self.h {
            for x in 0..self.w {
                if at(x, y) != T {
                    continue;
                }
                if [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| {
                    let v = at(x + dx, y + dy);
                    v != T && v != OUT
                }) {
                    self.px[(y * self.w + x) as usize] = OUT;
                }
            }
        }
    }

    /// Rotate 90 degrees counter-clockwise around the centre (lossless).
    pub fn rot90(&self) -> RSprite {
        let n = self.w.max(self.h);
        let mut s = RSprite::new(n, n, n / 2, n / 2);
        let (cx, cy) = (self.w / 2, self.h / 2);
        for y in 0..self.h {
            for x in 0..self.w {
                let v = self.get(x, y);
                if v == T {
                    continue;
                }
                let (dx, dy) = (x - cx, y - cy);
                s.set(n / 2 + dy, n / 2 - dx, v);
            }
        }
        s
    }
}

fn role(ch: u8) -> u8 {
    match ch {
        b'O' => OUT,
        b'F' => FUR,
        b'f' => FUR_D,
        b'L' => LIGHT,
        b'l' => LIGHT_D,
        b'D' => DARK,
        b'd' => DARK_D,
        b'K' => EYE,
        b'P' => PINK,
        b'W' => WHITE,
        b'C' => COLLAR,
        b'c' => COLLAR_D,
        b'G' => GOLD,
        _ => T,
    }
}

/// Draw a role sprite with its origin at (x, y). `flip` mirrors it.
pub fn draw(f: &mut Frame, s: &RSprite, x: i32, y: i32, flip: bool, pal: &[u32]) {
    for sy in 0..s.h {
        let py = y - s.oy + sy;
        if !(0..224).contains(&py) {
            continue;
        }
        for sx in 0..s.w {
            let v = s.px[(sy * s.w + sx) as usize];
            if v == T {
                continue;
            }
            let px = if flip { x + s.ox - sx } else { x - s.ox + sx };
            f.set(px, py, pal[v as usize]);
        }
    }
}

/// Draw only the rows above screen row `cut` (a dog in the pond).
pub fn draw_clipped(f: &mut Frame, s: &RSprite, x: i32, y: i32, flip: bool, pal: &[u32], cut: i32) {
    for sy in 0..s.h {
        let py = y - s.oy + sy;
        if py >= cut {
            break;
        }
        for sx in 0..s.w {
            let v = s.px[(sy * s.w + sx) as usize];
            if v == T {
                continue;
            }
            let px = if flip { x + s.ox - sx } else { x - s.ox + sx };
            f.set(px, py, pal[v as usize]);
        }
    }
}

/// Upside down (a dog on its back).
pub fn draw_vflip(f: &mut Frame, s: &RSprite, x: i32, y: i32, flip: bool, pal: &[u32]) {
    for sy in 0..s.h {
        for sx in 0..s.w {
            let v = s.px[(sy * s.w + sx) as usize];
            if v == T {
                continue;
            }
            let px = if flip { x + s.ox - sx } else { x - s.ox + sx };
            f.set(px, y - (s.h - 1 - sy), pal[v as usize]);
        }
    }
}

pub fn draw_scaled(f: &mut Frame, s: &RSprite, x: i32, y: i32, flip: bool, pal: &[u32], k: i32) {
    for sy in 0..s.h {
        for sx in 0..s.w {
            let v = s.px[(sy * s.w + sx) as usize];
            if v == T {
                continue;
            }
            let px = if flip { x + (s.ox - sx) * k } else { x + (sx - s.ox) * k };
            f.rect(px, y + (sy - s.oy) * k, k, k, pal[v as usize]);
        }
    }
}

// ------------------------------------------------------------------ dogs

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DogFrame {
    Stand,
    Sit,
    SitWag,
    Run0,
    Run1,
    Run2,
    Run3,
    Leap,
    Reach,
    Lie,
    Stare,
    Guilty,
    Bark,
    Happy,
    Shake0,
    Shake1,
    Curl0,
    Curl1,
    Curl2,
    Curl3,
}

pub const DOG_FRAMES: [DogFrame; 20] = [
    DogFrame::Stand,
    DogFrame::Sit,
    DogFrame::SitWag,
    DogFrame::Run0,
    DogFrame::Run1,
    DogFrame::Run2,
    DogFrame::Run3,
    DogFrame::Leap,
    DogFrame::Reach,
    DogFrame::Lie,
    DogFrame::Stare,
    DogFrame::Guilty,
    DogFrame::Bark,
    DogFrame::Happy,
    DogFrame::Shake0,
    DogFrame::Shake1,
    DogFrame::Curl0,
    DogFrame::Curl1,
    DogFrame::Curl2,
    DogFrame::Curl3,
];

#[derive(Clone, Copy, PartialEq)]
enum Head {
    Normal,
    Open,
    Sad,
}

#[derive(Clone, Copy, PartialEq)]
enum Tail {
    Stub,
    Long,
    Curl,
    Plume,
}

struct Shape {
    leg: f32,
    len: f32,
    thick: f32,
    tail: Tail,
    /// long hair hanging under the body and on the legs
    fringe: bool,
    /// fluffy ruff around the neck
    ruff: bool,
    /// lower legs in the light color
    socks: bool,
    /// dark saddle on the back
    saddle: bool,
}

fn shape(b: Breed) -> Shape {
    match b {
        Breed::Corgi => Shape { leg: 5.0, len: 13.0, thick: 7.0, tail: Tail::Stub, fringe: false, ruff: false, socks: true, saddle: false },
        Breed::Collie => Shape { leg: 8.0, len: 13.0, thick: 7.0, tail: Tail::Long, fringe: false, ruff: true, socks: true, saddle: false },
        Breed::Afghan => Shape { leg: 11.0, len: 12.0, thick: 6.0, tail: Tail::Curl, fringe: true, ruff: false, socks: false, saddle: false },
        Breed::Keeshond => Shape { leg: 6.0, len: 11.0, thick: 9.0, tail: Tail::Plume, fringe: false, ruff: true, socks: true, saddle: true },
    }
}

/// Heads facing right: (rows, neck point inside the rows).
fn head(b: Breed, h: Head) -> (&'static [&'static str], (i32, i32)) {
    match (b, h) {
        (Breed::Corgi, Head::Normal) => (
            &["F...F.....", "FF..FF....", "FPF.FPF...", "FFFFFFF...", "FFFFKFFF..", "FFFFFFLLL.", "fFFFLLLLLK", ".fLLLLLLL.", "..LLLLL..."],
            (2, 7),
        ),
        (Breed::Corgi, Head::Open) => (
            &["F...F.....", "FF..FF....", "FPF.FPF...", "FFFFFFF...", "FFFFKFFF..", "FFFFFFLLL.", "fFFFLLLLLK", ".fLLKKKK..", "..LLLPPL..", "....PP...."],
            (2, 7),
        ),
        (Breed::Corgi, Head::Sad) => (
            &["..........", "FF....FF..", "FFFFFFFF..", "FFFFFFF...", "FFFFfFFF..", "FFFFFFLLL.", "fFFFLLLLLK", ".fLLLLLLL.", "..LLLLL..."],
            (2, 7),
        ),
        (Breed::Collie, Head::Normal) => (
            &["..F.......", ".FFF.F....", ".FFFFFF...", "FFFFKFFF..", "FFFLLLLLL.", "FFLLLLLLLK", "fFLLLLLLL.", ".LLLLLL...", "..LLL....."],
            (2, 7),
        ),
        (Breed::Collie, Head::Open) => (
            &["..F.......", ".FFF.F....", ".FFFFFF...", "FFFFKFFF..", "FFFLLLLLL.", "FFLLLLLLLK", "fFLLKKKK..", ".LLLPPL...", "...PP....."],
            (2, 7),
        ),
        (Breed::Collie, Head::Sad) => (
            &["..........", "..........", "FFFFFFF...", "FFFFfFFF..", "FFFLLLLLL.", "FFLLLLLLLK", "fFLLLLLLL.", ".LLLLLL...", "..LLL....."],
            (2, 7),
        ),
        (Breed::Afghan, Head::Normal) => (
            &["..FFFF.....", ".FFFFFF....", "DFFFKFFF...", "DDFFFFFFFF.", "DDDFFFFFFDK", "DDDDFFFFF..", "DDDD.......", "DDDD.......", ".DDD.......", "..DD......."],
            (2, 5),
        ),
        (Breed::Afghan, Head::Open) => (
            &["..FFFF.....", ".FFFFFF....", "DFFFKFFF...", "DDFFFFFFFF.", "DDDFFFFFFDK", "DDDDFFKKK..", "DDDD.FPPF..", "DDDD..PP...", ".DDD.......", "..DD......."],
            (2, 5),
        ),
        (Breed::Afghan, Head::Sad) => (
            &["...........", "..FFFF.....", "DFFFFFF....", "DDFFfFFF...", "DDDFFFFFFF.", "DDDDFFFFFDK", "DDDDFFFFF..", "DDDD.......", ".DDD.......", "..DD......."],
            (2, 5),
        ),
        (Breed::Keeshond, Head::Normal) => (
            &[".F...F....", "FFF.FFF...", "FFFFFFF...", "FDKFFFF...", "FDDFFDDD..", "FFFFDDDDK.", "LFFFFDD...", "LLFFF.....", ".LLL......"],
            (2, 7),
        ),
        (Breed::Keeshond, Head::Open) => (
            &[".F...F....", "FFF.FFF...", "FFFFFFF...", "FDKFFFF...", "FFDFFDDD..", "FFFFDDDDK.", "LFFFKKKK..", "LLFFPPF...", ".LLL.PP..."],
            (2, 7),
        ),
        (Breed::Keeshond, Head::Sad) => (
            &["..........", "FF....FF..", "FFFFFFF...", "FDfFFFF...", "FDDFFDDD..", "FFFFDDDDK.", "LFFFFDD...", "LLFFF.....", ".LLL......"],
            (2, 7),
        ),
    }
}

/// One pose of the dog skeleton. Angles in degrees from straight down,
/// positive = forward. Legs: front near, front far, back near, back far.
struct DPose {
    lift: f32,
    pitch: f32,
    legs: [(f32, f32); 4],
    head: Head,
    /// head position nudge (x, y)
    hd: (i32, i32),
    tail: f32,
    sit: bool,
    lie: bool,
}

fn dpose(fr: DogFrame) -> DPose {
    let stand = [(0.0, 0.0), (6.0, 2.0), (12.0, -16.0), (4.0, -12.0)];
    let p = |lift, pitch, legs, head, hd, tail| DPose { lift, pitch, legs, head, hd, tail, sit: false, lie: false };
    match fr {
        DogFrame::Stand => p(0.0, 0.0, stand, Head::Normal, (0, 0), 30.0),
        DogFrame::Happy => p(0.0, 4.0, stand, Head::Open, (0, -1), 60.0),
        DogFrame::Bark => p(0.0, 8.0, [(10.0, 4.0), (-4.0, -2.0), (14.0, -18.0), (4.0, -10.0)], Head::Open, (1, -1), 50.0),
        DogFrame::Shake0 => p(0.0, -3.0, [(-6.0, 0.0), (4.0, 2.0), (20.0, -10.0), (8.0, -14.0)], Head::Sad, (-1, 1), 70.0),
        DogFrame::Shake1 => p(0.0, 3.0, [(6.0, 2.0), (-2.0, 0.0), (8.0, -18.0), (16.0, -10.0)], Head::Sad, (1, 0), -10.0),
        DogFrame::Stare => p(0.0, 4.0, [(70.0, -70.0), (2.0, 0.0), (14.0, -16.0), (4.0, -12.0)], Head::Normal, (1, -1), -60.0),
        DogFrame::Run0 => p(0.0, 0.0, [(-30.0, -10.0), (-20.0, -4.0), (40.0, 10.0), (30.0, 6.0)], Head::Open, (0, 0), 0.0),
        DogFrame::Run1 => p(2.0, 0.0, [(55.0, 40.0), (40.0, 30.0), (-55.0, -30.0), (-45.0, -25.0)], Head::Open, (0, 0), -10.0),
        DogFrame::Run2 => p(1.0, -6.0, [(10.0, 0.0), (25.0, 10.0), (-35.0, -10.0), (-25.0, -14.0)], Head::Open, (0, 1), -20.0),
        DogFrame::Run3 => p(0.0, 6.0, [(-35.0, -20.0), (-25.0, -14.0), (20.0, -5.0), (12.0, -10.0)], Head::Open, (0, -1), 10.0),
        DogFrame::Leap => p(3.0, 30.0, [(80.0, 120.0), (70.0, 110.0), (-70.0, -45.0), (-60.0, -40.0)], Head::Open, (1, -1), -30.0),
        DogFrame::Reach => p(3.0, -6.0, [(75.0, 65.0), (60.0, 55.0), (-50.0, -25.0), (-40.0, -20.0)], Head::Open, (1, 0), -5.0),
        DogFrame::Sit | DogFrame::SitWag | DogFrame::Guilty => DPose {
            lift: 0.0,
            pitch: 38.0,
            legs: [(0.0, 0.0), (4.0, 0.0), (95.0, -100.0), (85.0, -95.0)],
            head: if fr == DogFrame::Guilty { Head::Sad } else { Head::Normal },
            hd: if fr == DogFrame::Guilty { (1, 2) } else { (0, 0) },
            tail: if fr == DogFrame::SitWag { 80.0 } else if fr == DogFrame::Guilty { -80.0 } else { 40.0 },
            sit: true,
            lie: false,
        },
        DogFrame::Lie => DPose {
            lift: 0.0,
            pitch: 0.0,
            legs: [(88.0, 92.0), (84.0, 90.0), (90.0, -90.0), (80.0, -95.0)],
            head: Head::Sad,
            hd: (1, 4),
            tail: -70.0,
            sit: false,
            lie: true,
        },
        DogFrame::Curl0 | DogFrame::Curl1 | DogFrame::Curl2 | DogFrame::Curl3 => {
            p(0.0, -10.0, [(110.0, -40.0), (100.0, -40.0), (95.0, -50.0), (85.0, -50.0)], Head::Open, (-2, 4), -60.0)
        }
    }
}

pub const DW: i32 = 36;
pub const DH: i32 = 32;
pub const DOX: i32 = 17;
pub const DOY: i32 = 29;

fn build_dog(b: Breed, fr: DogFrame) -> RSprite {
    let sh = shape(b);
    let pose = dpose(fr);
    let mut s = RSprite::new(DW, DH, DOX, DOY);
    let ground = DOY as f32;
    let pitch = pose.pitch.to_radians();
    // body centre
    let mut cy = ground - sh.leg - sh.thick * 0.5 - pose.lift;
    let cx = DOX as f32 - 1.0;
    if pose.lie {
        cy = ground - sh.thick * 0.5 - 0.5;
    }
    if pose.sit {
        // hips on the ground, chest up
        let hip_y = ground - sh.thick * 0.45;
        cy = hip_y - (sh.len * 0.5) * pitch.sin();
    }
    let half = sh.len * 0.5;
    let hip = (cx - half * pitch.cos(), cy + half * pitch.sin());
    let sho = (cx + half * pitch.cos(), cy - half * pitch.sin());
    let ip = |p: (f32, f32)| (p.0.round() as i32, p.1.round() as i32);
    let up = sh.leg * 0.55;
    let lo = sh.leg * 0.55;
    let leg_pts = |j: (f32, f32), a1: f32, a2: f32| {
        let (r1, r2) = (a1.to_radians(), a2.to_radians());
        let knee = (j.0 + up * r1.sin(), j.1 + up * r1.cos());
        let paw = (knee.0 + lo * r2.sin(), knee.1 + lo * r2.cos());
        (knee, paw)
    };
    let joint_f = (sho.0 - 1.0, sho.1 + sh.thick * 0.3);
    let joint_b = (hip.0 + 1.0, hip.1 + sh.thick * 0.3);
    let leg = |s: &mut RSprite, j: (f32, f32), a: (f32, f32), near: bool, front: bool| {
        let (knee, paw) = leg_pts(j, a.0, a.1);
        let (main, lowc) = if near {
            (FUR, if sh.socks { LIGHT } else { FUR })
        } else {
            (FUR_D, if sh.socks { LIGHT_D } else { FUR_D })
        };
        let w_up = if sh.fringe { 3 } else if front { 2 } else { 3 };
        s.thick(ip(j), ip(knee), w_up, main);
        s.thick(ip(knee), ip(paw), 2, lowc);
        let p = ip(paw);
        s.set(p.0 + 1, p.1, lowc);
        if sh.fringe && near {
            // feathered hair on the legs
            s.thick((ip(knee).0 - 1, ip(knee).1), (p.0 - 1, p.1 - 1), 1, FUR);
        }
    };
    // far legs
    leg(&mut s, joint_f, pose.legs[1], false, true);
    leg(&mut s, joint_b, pose.legs[3], false, false);
    // tail, behind the body
    let ta = pose.tail.to_radians();
    let tb = (hip.0 - 1.0, hip.1 - sh.thick * 0.25);
    match sh.tail {
        Tail::Stub => {
            s.thick(ip(tb), ip((tb.0 - 2.0, tb.1 - 1.0 - ta.sin() * 1.5)), 3, FUR);
        }
        Tail::Long => {
            // hangs low and sweeps up at the white tip
            let mid = (tb.0 - 3.0, tb.1 + 4.0 - ta.sin() * 4.0);
            let tip = (mid.0 - 3.0, mid.1 - 1.0 - ta.sin() * 3.0);
            s.thick(ip(tb), ip(mid), 3, FUR);
            s.thick(ip(mid), ip(tip), 2, FUR);
            s.set(ip(tip).0, ip(tip).1, LIGHT);
            s.set(ip(tip).0 - 1, ip(tip).1, LIGHT);
        }
        Tail::Curl => {
            let a = (tb.0 - 2.0, tb.1 - 4.0 - ta.sin() * 2.0);
            let b2 = (a.0 + 2.0, a.1 - 3.0);
            s.thick(ip(tb), ip(a), 2, FUR);
            s.thick(ip(a), ip(b2), 2, FUR);
            s.set(ip(b2).0 + 1, ip(b2).1 + 1, FUR);
        }
        Tail::Plume => {
            // a big curl over the back
            let c = (tb.0 + 1.0, tb.1 - 4.0 - ta.sin().max(-0.3) * 2.0);
            s.ellipse(c.0, c.1, 3.5, 3.0, 0.0, FUR);
            s.ellipse(c.0 + 0.5, c.1 - 0.5, 2.0, 1.5, 0.0, LIGHT);
            s.set(ip(c).0 - 3, ip(c).1 + 2, DARK);
        }
    }
    // torso
    s.ellipse(cx, cy, half + 2.0, sh.thick * 0.5, -pitch, FUR);
    // belly and chest
    let belly = (cx + 1.0 * pitch.cos(), cy + sh.thick * 0.3);
    s.ellipse(belly.0, belly.1, half * 0.8, sh.thick * 0.22, -pitch, if sh.saddle { LIGHT } else { FUR_D });
    if b != Breed::Afghan {
        let chest = (sho.0 + 1.0, sho.1 + sh.thick * 0.15);
        s.ellipse(chest.0, chest.1, 2.5, sh.thick * 0.42, -pitch, LIGHT);
    }
    if sh.saddle {
        let top = (cx - 1.0, cy - sh.thick * 0.3);
        s.ellipse(top.0, top.1, half * 0.8, sh.thick * 0.25, -pitch, DARK);
    }
    // a little shine along the back
    if !sh.saddle && b != Breed::Collie {
        let top = (cx - 2.0, cy - sh.thick * 0.35);
        s.thick(ip((top.0 - 2.0, top.1 + 0.0)), ip((top.0 + 3.0, top.1 - 0.0)), 1, LIGHT_D);
    }
    if sh.fringe && !pose.sit && !pose.lie {
        // long silky hair hanging from the belly
        for k in 0..9 {
            let u = k as f32 / 8.0;
            let x = hip.0 + 1.0 + u * (sho.0 - hip.0 - 1.0);
            // the underside of the (pitched) body at this x
            let y0 = cy + (cx - x) * pitch.tan() + sh.thick * 0.5 - 1.0;
            let len = 2.0 + ((k * 5) % 3) as f32;
            let bend = if pose.lift > 1.0 { -1.0 } else { 0.0 };
            s.thick(ip((x, y0)), ip((x + bend, y0 + len)), 1, if k % 3 == 1 { FUR_D } else { FUR });
        }
    }
    // near legs
    leg(&mut s, joint_b, pose.legs[2], true, false);
    leg(&mut s, joint_f, pose.legs[0], true, true);
    if sh.ruff {
        let r = (sho.0 + 0.5, sho.1 - 1.0);
        s.ellipse(r.0, r.1, 2.5, sh.thick * 0.5 + 0.5, -pitch, if b == Breed::Keeshond { LIGHT } else { LIGHT });
    }
    // collar
    let neck = (sho.0 + 1.0, sho.1 - sh.thick * 0.3);
    s.thick(ip((neck.0 - 1.0, neck.1 - 1.0)), ip((neck.0, neck.1 + 2.0)), 2, COLLAR);
    s.set(ip(neck).0 + 1, ip(neck).1 + 2, GOLD);
    // head
    let (rows, (nx, ny)) = head(b, pose.head);
    let hx = ip(neck).0 - nx + 1 + pose.hd.0;
    let hy = ip(neck).1 - ny - 1 + pose.hd.1 - if b == Breed::Afghan { 2 } else { 0 };
    s.stamp(hx, hy, rows);
    s.outline();
    s
}

pub struct DogArt {
    /// [breed][frame]
    pub frames: Vec<Vec<RSprite>>,
}

impl DogArt {
    pub fn new() -> Self {
        let breeds = [Breed::Corgi, Breed::Collie, Breed::Afghan, Breed::Keeshond];
        let frames = breeds
            .iter()
            .map(|&b| {
                let mut v: Vec<RSprite> = DOG_FRAMES.iter().map(|&fr| build_dog(b, fr)).collect();
                // flip frames: the curl turned 90 degrees at a time (back flip)
                let c0 = v[16].clone();
                let mut c = recenter(&c0);
                for k in 1..4 {
                    c = c.rot90();
                    v[16 + k] = c.clone();
                }
                v[16] = recenter(&c0);
                v
            })
            .collect();
        DogArt { frames }
    }

    pub fn get(&self, pair: usize, fr: DogFrame) -> &RSprite {
        let i = DOG_FRAMES.iter().position(|&x| x == fr).unwrap_or(0);
        &self.frames[pair % 4][i]
    }
}

impl Default for DogArt {
    fn default() -> Self {
        Self::new()
    }
}

/// Crop a sprite to its content and put the origin in the middle (for spinning).
fn recenter(s: &RSprite) -> RSprite {
    let (mut x0, mut y0, mut x1, mut y1) = (s.w, s.h, 0, 0);
    for y in 0..s.h {
        for x in 0..s.w {
            if s.get(x, y) != T {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    let n = (x1 - x0 + 1).max(y1 - y0 + 1) + 1;
    let mut o = RSprite::new(n, n, n / 2, n / 2);
    let (dx, dy) = ((n - (x1 - x0 + 1)) / 2, (n - (y1 - y0 + 1)) / 2);
    for y in y0..=y1 {
        for x in x0..=x1 {
            o.set(x - x0 + dx, y - y0 + dy, s.get(x, y));
        }
    }
    o
}

/// The palette of a dog; `collar` is the side's color.
pub fn dog_pal(pair: usize, collar: u32) -> DogPal {
    let c = PAIRS[pair].dog.coat;
    let mut p = [0u32; ROLES];
    p[OUT as usize] = OUTLINE;
    p[FUR as usize] = c[0];
    p[FUR_D as usize] = c[1];
    p[LIGHT as usize] = c[2];
    p[LIGHT_D as usize] = c[3];
    p[DARK as usize] = c[4];
    p[DARK_D as usize] = c[5];
    p[EYE as usize] = rgb(16, 16, 32);
    p[PINK as usize] = rgb(248, 112, 136);
    p[WHITE as usize] = rgb(248, 248, 248);
    p[COLLAR as usize] = collar;
    p[COLLAR_D as usize] = shade(collar, 2, 3);
    p[GOLD as usize] = rgb(248, 216, 64);
    p
}

/// A soaked dog: darker and a bit blue.
pub fn wet_pal(p: &DogPal) -> DogPal {
    let mut q = *p;
    for i in [FUR, FUR_D, LIGHT, LIGHT_D, DARK, DARK_D] {
        q[i as usize] = mix(shade(p[i as usize], 4, 5), rgb(64, 96, 160), 0.2);
    }
    q
}

/// All gold (the champion statue on the podium).
pub fn gold_pal() -> DogPal {
    let mut p = [rgb(248, 200, 48); ROLES];
    p[OUT as usize] = OUTLINE;
    p[FUR_D as usize] = rgb(200, 136, 24);
    p[LIGHT as usize] = rgb(248, 240, 160);
    p[LIGHT_D as usize] = rgb(232, 200, 96);
    p[DARK as usize] = rgb(184, 120, 24);
    p[DARK_D as usize] = rgb(136, 80, 16);
    p[EYE as usize] = rgb(96, 56, 16);
    p[PINK as usize] = rgb(248, 168, 96);
    p[WHITE as usize] = rgb(248, 248, 248);
    p
}

// ------------------------------------------------------------------ critters and props

pub const GULL: [&[&str]; 3] = [
    // wings up
    &[
        "........DD..........",
        ".......DLl..........",
        "......DLLl..........",
        ".....LLLl...........",
        "....LLLl.......WW...",
        "WW.WLLWWWWWWWWWWKWGG",
        ".WWWWWWWWWWWWWWWW...",
        "...lWWWWWWWWWWl.....",
        "......P....P........",
    ],
    // gliding
    &[
        "....................",
        "....................",
        "....................",
        "..DDLLLLLLLl....WW..",
        "WW.WLLLLLLLWWWWWWKGG",
        ".WWWWWWWWWWWWWWWW...",
        "...lWWWWWWWWWWl.....",
        "......P....P........",
        "....................",
    ],
    // wings down
    &[
        "....................",
        "...............WW...",
        "WW..WWWWWWWWWWWWWKGG",
        ".WWWLLLLLLWWWWWWW...",
        "...lLLLLLlWWWWl.....",
        "....LLLLl...........",
        ".....LLl............",
        "......Dl............",
        ".......D............",
    ],
];

pub const SQUIRREL: [&[&str]; 3] = [
    // run 1: tail up
    &[
        ".FFF..........",
        "FFfFF.........",
        "FffFFF........",
        "FfFFFF....F...",
        ".FfFFF...FFF..",
        "..FfFF..FFFFF.",
        "...FFFFFFFKFFL",
        "...FFFFFFFFFL.",
        "....FFFLLLF...",
        "....FF...FF...",
    ],
    // run 2: stretched
    &[
        "..............",
        "FFF...........",
        "FfFF..........",
        "FffFF.....F...",
        ".FfFFF...FFF..",
        "..FFFFFFFFFFF.",
        "...FFFFFFFKFFL",
        "...FFFFFFFFFL.",
        "..FF.FLLLF.FF.",
        ".F.........F..",
    ],
    // sitting up, nibbling an acorn (taunt)
    &[
        "FFF...........",
        "FfFF.....F....",
        "FffFF...FFF...",
        "FfFFF..FFKFF..",
        ".FfFF..FFFFL..",
        ".FFfF.FFLLGG..",
        "..FFFFFFLLGG..",
        "...FFFFFLL....",
        "...FFFFFL.....",
        "...FF..FF.....",
    ],
];

pub const SAUSAGE: &[&str] = &[".PPPPP.", "PFFFFFP", ".PPPPP."];

pub const SLEEPER: &[&str] = &[
    "...WWWWWWW............",
    "..WlWlWlWWW...........",
    ".SWWWWWWWWWTTTTTTTPPPPDD",
    "SSWWWWWWWTTTTTTTTTPPPPDD",
    ".S...TTTTTTTTTTT.PPPP...",
];

pub const KID: &[&str] = &[
    "..CCCC..", ".CCCCCC.", "CCcHHH..", ".HSSKS..", "..SSSS..", "..TTTTS.", ".TTTTTS.", "STTTT...", "..TTT...", "..PPP...",
    "..P.P...", "..P.P...", ".DD.DD..",
];

/// Palette for the string props (gull, squirrel, sleeper, kid): roles reused as plain colors.
pub fn prop_pal(kind: &str) -> DogPal {
    let mut p = [0u32; ROLES];
    p[OUT as usize] = OUTLINE;
    p[EYE as usize] = rgb(16, 16, 32);
    p[WHITE as usize] = rgb(248, 248, 248);
    match kind {
        "gull" => {
            p[LIGHT as usize] = rgb(168, 176, 200);
            p[LIGHT_D as usize] = rgb(120, 128, 160);
            p[DARK as usize] = rgb(40, 40, 56);
            p[GOLD as usize] = rgb(248, 184, 48);
            p[PINK as usize] = rgb(248, 136, 64);
        }
        "squirrel" => {
            p[FUR as usize] = rgb(200, 104, 48);
            p[FUR_D as usize] = rgb(136, 64, 32);
            p[LIGHT as usize] = rgb(248, 224, 184);
            p[LIGHT_D as usize] = rgb(216, 176, 136);
            p[GOLD as usize] = rgb(176, 120, 64);
        }
        "sausage" => {
            p[PINK as usize] = rgb(168, 64, 48);
            p[FUR as usize] = rgb(232, 120, 88);
        }
        _ => {
            // people props: S skin is not a role here, so map letters by hand below
            p[FUR as usize] = rgb(248, 200, 160);
        }
    }
    p
}

/// People-ish props use their own letters: S skin, H hair, T shirt, P pants, D shoes, C cap, K eye.
pub fn people_rows(rows: &[&str], pal: &[(u8, u32)]) -> (Vec<Vec<u32>>, i32, i32) {
    let h = rows.len() as i32 + 2;
    let w = rows.iter().map(|r| r.len()).max().unwrap_or(0) as i32 + 2;
    let mut m = vec![vec![0u32; w as usize]; h as usize];
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.bytes().enumerate() {
            if let Some(&(_, c)) = pal.iter().find(|(k, _)| *k == ch) {
                m[y + 1][x + 1] = c | 0xFF00_0000;
            }
        }
    }
    // outline
    let src = m.clone();
    for y in 0..h as usize {
        for x in 0..w as usize {
            if src[y][x] != 0 {
                continue;
            }
            let near = [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| {
                let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                xx >= 0 && yy >= 0 && xx < w && yy < h && src[yy as usize][xx as usize] != 0
            });
            if near {
                m[y][x] = OUTLINE | 0xFF00_0000;
            }
        }
    }
    (m, w, h)
}

/// A pre-colored bitmap (alpha in the top byte) with an origin.
#[derive(Clone)]
pub struct Pix {
    pub w: i32,
    pub h: i32,
    pub ox: i32,
    pub oy: i32,
    pub px: Vec<u32>,
}

impl Pix {
    pub fn from_people(rows: &[&str], pal: &[(u8, u32)], ox: i32, oy: i32) -> Self {
        let (m, w, h) = people_rows(rows, pal);
        Pix { w, h, ox: ox + 1, oy: oy + 1, px: m.into_iter().flatten().collect() }
    }
    pub fn draw(&self, f: &mut Frame, x: i32, y: i32, flip: bool) {
        for sy in 0..self.h {
            for sx in 0..self.w {
                let c = self.px[(sy * self.w + sx) as usize];
                if c == 0 {
                    continue;
                }
                let px = if flip { x + self.ox - sx } else { x - self.ox + sx };
                f.set(px, y - self.oy + sy, c & 0xFF_FFFF);
            }
        }
    }
}

pub struct Props {
    pub gull: Vec<RSprite>,
    pub squirrel: Vec<RSprite>,
    pub sausage: RSprite,
    pub sleeper: Pix,
    pub sleeper_up: Pix,
    pub kid: Pix,
}

impl Props {
    pub fn new() -> Self {
        let gull = GULL.iter().map(|r| RSprite::from_rows(r, 10, 5)).collect();
        let squirrel = SQUIRREL.iter().map(|r| RSprite::from_rows(r, 7, 9)).collect();
        let people = [
            (b'S', rgb(248, 192, 152)),
            (b'H', rgb(120, 72, 40)),
            (b'T', rgb(56, 120, 200)),
            (b'P', rgb(64, 64, 96)),
            (b'D', rgb(96, 56, 32)),
            (b'W', rgb(232, 232, 216)),
            (b'l', rgb(144, 144, 136)),
            (b'L', rgb(248, 200, 48)),
            (b'K', rgb(16, 16, 32)),
        ];
        let sleeper = Pix::from_people(SLEEPER, &people, 11, 4);
        let up_rows: &[&str] = &[
            ".HHH..................",
            "HHHSS..WWWWWWW........",
            "HSSKS.WlWlWlWWW.......",
            ".SSSS.WWWWWWWWW.......",
            "..TTTTTTTTTTTTTTTPPPPDD",
            "..TTTTTTTTTTTTTTTPPPPDD",
        ];
        let sleeper_up = Pix::from_people(up_rows, &people, 11, 5);
        let kid_pal = [
            (b'S', rgb(232, 168, 112)),
            (b'H', rgb(56, 40, 32)),
            (b'T', rgb(248, 200, 48)),
            (b'P', rgb(56, 96, 200)),
            (b'D', rgb(232, 56, 56)),
            (b'C', rgb(232, 56, 56)),
            (b'c', rgb(160, 32, 40)),
            (b'K', rgb(16, 16, 32)),
        ];
        let kid = Pix::from_people(KID, &kid_pal, 4, 12);
        Props { gull, squirrel, sausage: RSprite::from_rows(SAUSAGE, 4, 3), sleeper, sleeper_up, kid }
    }
}

impl Default for Props {
    fn default() -> Self {
        Self::new()
    }
}

// ------------------------------------------------------------------ the disc

/// The disc seen from the side: `tilt` -1..1 for the wobble, `spin` moves the stripe.
pub fn draw_disc(f: &mut Frame, x: i32, y: i32, color: u32, tilt: i32, spin: f32) {
    let dark = shade(color, 3, 5);
    let lite = mix(color, 0xFFFFFF, 0.55);
    // rows of half-widths, top to bottom
    let rows: &[(i32, i32)] = match tilt {
        0 => &[(-3, 3), (-5, 5), (-5, 5), (-3, 3)],
        t if t > 0 => &[(1, 4), (-3, 5), (-5, 3), (-5, 0)],
        _ => &[(-4, -1), (-5, 3), (-3, 5), (0, 5)],
    };
    // outline first
    for (k, &(a, b)) in rows.iter().enumerate() {
        let yy = y - 2 + k as i32;
        f.hline(x + a - 1, x + b + 1, yy, OUTLINE);
    }
    f.hline(x + rows[0].0, x + rows[0].1, y - 3, OUTLINE);
    f.hline(x + rows[3].0, x + rows[3].1, y + 2, OUTLINE);
    let stripe = (spin as i32).rem_euclid(9) - 4;
    for (k, &(a, b)) in rows.iter().enumerate() {
        let yy = y - 2 + k as i32;
        let c = match k {
            0 => lite,
            3 => dark,
            _ => color,
        };
        f.hline(x + a, x + b, yy, c);
        if k == 1 || k == 2 {
            let sx = x + stripe;
            if sx >= x + a && sx <= x + b {
                f.set(sx, yy, rgb(248, 248, 248));
            }
        }
    }
}

// ------------------------------------------------------------------ owners

const STAND: Skel = [(12, 5), (12, 10), (12, 19), (8, 15), (8, 20), (16, 15), (16, 20), (10, 24), (10, 29), (14, 24), (14, 29)];
/// holding the disc in front of the chest, ready
const AIM: Skel = [(12, 5), (12, 10), (12, 19), (10, 15), (14, 15), (16, 14), (19, 14), (9, 24), (8, 29), (15, 24), (16, 29)];
/// backhand coil: the throwing arm across the body
const COIL: Skel = [(11, 6), (12, 11), (12, 20), (15, 15), (18, 18), (10, 15), (6, 17), (9, 24), (7, 29), (16, 25), (18, 29)];
/// release: arm out straight, front foot forward
const FLING: Skel = [(14, 6), (13, 11), (12, 20), (9, 15), (7, 19), (18, 12), (22, 11), (9, 25), (6, 29), (16, 24), (19, 29)];
/// follow-through
const FOLLOW: Skel = [(15, 7), (14, 12), (12, 20), (10, 16), (8, 20), (19, 15), (21, 19), (9, 25), (6, 29), (16, 24), (19, 29)];
const CHEER: Skel = [(12, 5), (12, 10), (12, 19), (8, 7), (4, 2), (16, 7), (20, 2), (10, 24), (9, 29), (14, 24), (15, 29)];
/// hand on the forehead
const FACEPALM: Skel = [(13, 6), (12, 11), (12, 19), (8, 15), (8, 20), (17, 11), (15, 5), (10, 24), (10, 29), (14, 24), (14, 29)];
/// two fingers in the mouth: the whistle
const WHISTLE: Skel = [(12, 5), (12, 10), (12, 19), (8, 15), (8, 20), (17, 12), (16, 8), (10, 24), (10, 29), (14, 24), (14, 29)];
/// sitting on a cooler
const SIT: Skel = [(12, 9), (12, 14), (11, 22), (8, 18), (9, 23), (15, 18), (17, 21), (16, 22), (16, 29), (17, 23), (18, 29)];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OwnerPose {
    Stand,
    Aim,
    Coil,
    Fling,
    Follow,
    Cheer,
    Facepalm,
    Whistle,
    Sit,
}

pub const OWNER_POSES: [OwnerPose; 9] = [
    OwnerPose::Stand,
    OwnerPose::Aim,
    OwnerPose::Coil,
    OwnerPose::Fling,
    OwnerPose::Follow,
    OwnerPose::Cheer,
    OwnerPose::Facepalm,
    OwnerPose::Whistle,
    OwnerPose::Sit,
];

fn skel_of(p: OwnerPose) -> Skel {
    match p {
        OwnerPose::Stand => STAND,
        OwnerPose::Aim => AIM,
        OwnerPose::Coil => COIL,
        OwnerPose::Fling => FLING,
        OwnerPose::Follow => FOLLOW,
        OwnerPose::Cheer => CHEER,
        OwnerPose::Facepalm => FACEPALM,
        OwnerPose::Whistle => WHISTLE,
        OwnerPose::Sit => SIT,
    }
}

/// Where the disc sits in the owner's hand (sprite canvas coordinates).
pub fn owner_hand(p: OwnerPose) -> (i32, i32) {
    skel_of(p)[6]
}

pub struct OwnerArt {
    /// [pair][pose]
    pub frames: Vec<Vec<Sprite>>,
}

impl OwnerArt {
    pub fn new() -> Self {
        let frames = PAIRS.iter().map(|p| OWNER_POSES.iter().map(|&q| person(&skel_of(q), p.owner.style, false, false)).collect()).collect();
        OwnerArt { frames }
    }
    pub fn get(&self, pair: usize, p: OwnerPose) -> &Sprite {
        let i = OWNER_POSES.iter().position(|&x| x == p).unwrap_or(0);
        &self.frames[pair % 4][i]
    }
}

impl Default for OwnerArt {
    fn default() -> Self {
        Self::new()
    }
}

pub fn owner_pal(pair: usize) -> Palette {
    let o = &PAIRS[pair].owner;
    let mut pal = [0u32; es::ROLES];
    pal[es::OUT as usize] = OUTLINE;
    pal[es::SKIN as usize] = o.skin.0;
    pal[es::SKIN_D as usize] = o.skin.1;
    pal[es::HAIR as usize] = o.hair.0;
    pal[es::HAIR_D as usize] = o.hair.1;
    pal[es::SHIRT as usize] = o.shirt.0;
    pal[es::SHIRT_D as usize] = o.shirt.1;
    pal[es::PANTS as usize] = o.pants.0;
    pal[es::PANTS_D as usize] = o.pants.1;
    pal[es::WHITE as usize] = rgb(248, 248, 248);
    pal[es::ACC as usize] = o.acc.0;
    pal[es::ACC_D as usize] = o.acc.1;
    pal[es::SHOE as usize] = rgb(248, 248, 248);
    pal[es::EYE as usize] = rgb(16, 16, 32);
    pal[es::CAP as usize] = o.cap.0;
    pal[es::CAP_D as usize] = o.cap.1;
    pal
}
