//! Sprites, built once when the game starts: glider wings rendered from a
//! small 3D model (pitch and turn frames, a sail pattern per pilot), the
//! pilots via `engine::sprites::person` with our own skeletons, and the
//! animals and props as string bitmaps.

use super::pilots::{Sail, EAGLE, PILOTS};
use crate::engine::gfx::{line_points, mix, rgb, shade, Frame, H, W};
use crate::engine::sprites::{self as sp, person, Skel};
use crate::engine::ui::OUTLINE;

/// Transparent pixel in a `Pic`.
pub const CLEAR: u32 = 0xFF00_0000;

/// A sprite of color roles with its own origin.
#[derive(Clone)]
pub struct Spr {
    pub w: i32,
    pub h: i32,
    pub ox: i32,
    pub oy: i32,
    pub px: Vec<u8>,
}

impl Spr {
    pub fn new(w: i32, h: i32, ox: i32, oy: i32) -> Self {
        Spr { w, h, ox, oy, px: vec![0; (w * h) as usize] }
    }
    pub fn get(&self, x: i32, y: i32) -> u8 {
        if x >= 0 && y >= 0 && x < self.w && y < self.h { self.px[(y * self.w + x) as usize] } else { 0 }
    }
    pub fn set(&mut self, x: i32, y: i32, v: u8) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = v;
        }
    }
    /// 1px outline (role `out`) around everything that is not clear.
    pub fn outline(&mut self, out: u8) {
        let src = self.px.clone();
        let at = |x: i32, y: i32| if x >= 0 && y >= 0 && x < self.w && y < self.h { src[(y * self.w + x) as usize] } else { 0 };
        for y in 0..self.h {
            for x in 0..self.w {
                if at(x, y) == 0 && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| {
                    let v = at(x + dx, y + dy);
                    v != 0 && v != out
                }) {
                    self.px[(y * self.w + x) as usize] = out;
                }
            }
        }
    }
}

/// Draw a role sprite with its origin at (x, y).
pub fn blit(f: &mut Frame, s: &Spr, x: i32, y: i32, flip: bool, pal: &[u32]) {
    let x0 = x - if flip { s.w - 1 - s.ox } else { s.ox };
    let y0 = y - s.oy;
    if x0 > W || y0 > H || x0 + s.w < 0 || y0 + s.h < 0 {
        return;
    }
    for sy in 0..s.h {
        let py = y0 + sy;
        if !(0..H).contains(&py) {
            continue;
        }
        for sx in 0..s.w {
            let v = s.px[(sy * s.w + sx) as usize];
            if v == 0 {
                continue;
            }
            let px = if flip { x0 + s.w - 1 - sx } else { x0 + sx };
            if (0..W).contains(&px) {
                f.px[(py * W + px) as usize] = pal[v as usize];
            }
        }
    }
}

/// Draw a role sprite in one flat color (silhouettes, ghosts).
pub fn blit_flat(f: &mut Frame, s: &Spr, x: i32, y: i32, flip: bool, c: u32) {
    let x0 = x - if flip { s.w - 1 - s.ox } else { s.ox };
    let y0 = y - s.oy;
    for sy in 0..s.h {
        for sx in 0..s.w {
            if s.px[(sy * s.w + sx) as usize] != 0 {
                let px = if flip { x0 + s.w - 1 - sx } else { x0 + sx };
                f.set(px, y0 + sy, c);
            }
        }
    }
}

/// A sprite with fixed colors (`CLEAR` is transparent).
#[derive(Clone)]
pub struct Pic {
    pub w: i32,
    pub h: i32,
    pub ox: i32,
    pub oy: i32,
    pub px: Vec<u32>,
}

impl Pic {
    pub fn new(w: i32, h: i32, ox: i32, oy: i32) -> Self {
        Pic { w, h, ox, oy, px: vec![CLEAR; (w * h) as usize] }
    }
    pub fn set(&mut self, x: i32, y: i32, c: u32) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = c;
        }
    }
    pub fn get(&self, x: i32, y: i32) -> u32 {
        if x >= 0 && y >= 0 && x < self.w && y < self.h { self.px[(y * self.w + x) as usize] } else { CLEAR }
    }
    /// From string rows; `map` gives the color of each letter ('.' is clear).
    pub fn from_rows(rows: &[&str], map: &[(u8, u32)], ox: i32, oy: i32, outline: bool) -> Pic {
        let w = rows.iter().map(|r| r.len()).max().unwrap_or(0) as i32 + 2;
        let h = rows.len() as i32 + 2;
        let mut p = Pic::new(w, h, ox + 1, oy + 1);
        for (y, r) in rows.iter().enumerate() {
            for (x, ch) in r.bytes().enumerate() {
                if let Some(&(_, c)) = map.iter().find(|m| m.0 == ch) {
                    p.set(x as i32 + 1, y as i32 + 1, c);
                }
            }
        }
        if outline {
            p.outline(OUTLINE);
        }
        p
    }
    pub fn outline(&mut self, c: u32) {
        let src = self.px.clone();
        let at = |x: i32, y: i32| if x >= 0 && y >= 0 && x < self.w && y < self.h { src[(y * self.w + x) as usize] } else { CLEAR };
        for y in 0..self.h {
            for x in 0..self.w {
                if at(x, y) == CLEAR && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| at(x + dx, y + dy) != CLEAR && at(x + dx, y + dy) != c) {
                    self.px[(y * self.w + x) as usize] = c;
                }
            }
        }
    }
}

impl Pic {
    pub fn hline(&mut self, x0: i32, x1: i32, y: i32, c: u32) {
        for x in x0.min(x1)..=x0.max(x1) {
            self.set(x, y, c);
        }
    }
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: u32) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }
    pub fn dither(&mut self, x: i32, y: i32, w: i32, h: i32, a: u32, b: u32) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, if (xx + yy) & 1 == 0 { a } else { b });
            }
        }
    }
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: u32) {
        line_points(x0, y0, x1, y1, |x, y| self.set(x, y, c));
    }
    pub fn thick(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, size: i32, c: u32) {
        let lo = -(size / 2);
        line_points(x0, y0, x1, y1, |x, y| {
            for dy in lo..lo + size {
                for dx in lo..lo + size {
                    self.set(x + dx, y + dy, c);
                }
            }
        });
    }
    pub fn ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, c: u32) {
        if rx <= 0 || ry <= 0 {
            return;
        }
        for dy in -ry..=ry {
            let t = 1.0 - (dy as f32 / (ry as f32 + 0.5)).powi(2);
            if t < 0.0 {
                continue;
            }
            let w = ((rx as f32 + 0.5) * t.sqrt()) as i32;
            self.hline(cx - w, cx + w, cy + dy, c);
        }
    }
    /// Scanline polygon fill (even-odd).
    pub fn poly(&mut self, pts: &[(i32, i32)], c: u32) {
        if pts.len() < 3 {
            return;
        }
        let miny = pts.iter().map(|p| p.1).min().unwrap().max(0);
        let maxy = pts.iter().map(|p| p.1).max().unwrap().min(self.h - 1);
        let mut xs: Vec<i32> = Vec::with_capacity(16);
        for y in miny..=maxy {
            xs.clear();
            let fy = y as f32 + 0.5;
            for i in 0..pts.len() {
                let (x0, y0) = pts[i];
                let (x1, y1) = pts[(i + 1) % pts.len()];
                let (fy0, fy1) = (y0 as f32, y1 as f32);
                if (fy0 <= fy && fy < fy1) || (fy1 <= fy && fy < fy0) {
                    let t = (fy - fy0) / (fy1 - fy0);
                    xs.push((x0 as f32 + t * (x1 - x0) as f32).round() as i32);
                }
            }
            xs.sort_unstable();
            for pair in xs.chunks(2) {
                if pair.len() == 2 {
                    self.hline(pair[0], pair[1] - 1, y, c);
                }
            }
        }
    }
    /// Recolor every non-clear pixel with `f`.
    pub fn map(&mut self, f: impl Fn(i32, i32, u32) -> u32) {
        for y in 0..self.h {
            for x in 0..self.w {
                let i = (y * self.w + x) as usize;
                if self.px[i] != CLEAR {
                    self.px[i] = f(x, y, self.px[i]);
                }
            }
        }
    }
}

pub fn draw_pic(f: &mut Frame, p: &Pic, x: i32, y: i32, flip: bool) {
    let x0 = x - if flip { p.w - 1 - p.ox } else { p.ox };
    let y0 = y - p.oy;
    if x0 > W || y0 > H || x0 + p.w < 0 || y0 + p.h < 0 {
        return;
    }
    for sy in 0..p.h {
        let py = y0 + sy;
        if !(0..H).contains(&py) {
            continue;
        }
        let row = (py * W) as usize;
        for sx in 0..p.w {
            let c = p.px[(sy * p.w + sx) as usize];
            if c == CLEAR {
                continue;
            }
            let px = if flip { x0 + p.w - 1 - sx } else { x0 + sx };
            if (0..W).contains(&px) {
                f.px[row + px as usize] = c;
            }
        }
    }
}

/// Like `draw_pic`, upside down (a sheep in orbit).
pub fn draw_pic_vflip(f: &mut Frame, p: &Pic, x: i32, y: i32, flip: bool) {
    let x0 = x - if flip { p.w - 1 - p.ox } else { p.ox };
    let y0 = y - (p.h - 1 - p.oy);
    for sy in 0..p.h {
        for sx in 0..p.w {
            let c = p.px[((p.h - 1 - sy) * p.w + sx) as usize];
            if c != CLEAR {
                let px = if flip { x0 + p.w - 1 - sx } else { x0 + sx };
                f.set(px, y0 + sy, c);
            }
        }
    }
}

// ------------------------------------------------------------------ gliders

// glider color roles
pub const G_OUT: u8 = 1;
pub const G_A: u8 = 2;
pub const G_B: u8 = 3;
pub const G_C: u8 = 4;
pub const G_AD: u8 = 5;
pub const G_BD: u8 = 6;
pub const G_CD: u8 = 7;
pub const G_TUBE: u8 = 8;
pub const G_TUBED: u8 = 9;
pub const G_WIRE: u8 = 10;
pub const G_ROLES: usize = 11;

pub fn glider_palette(def: usize) -> [u32; G_ROLES] {
    let s = PILOTS[def].sail;
    [
        0,
        OUTLINE,
        s[0],
        s[1],
        s[2],
        shade(s[0], 3, 4),
        shade(s[1], 3, 4),
        shade(s[2], 3, 4),
        rgb(224, 224, 240),
        rgb(136, 144, 168),
        rgb(176, 184, 200),
    ]
}

/// Pitch frames (degrees, nose up positive) and turn frames (yaw degrees).
pub const PITCHES: [f32; 7] = [-36.0, -24.0, -12.0, 0.0, 12.0, 24.0, 36.0];
pub const YAWS: [f32; 4] = [0.0, -30.0, -60.0, -90.0];

fn sail_band(pat: Sail, x: f32, z: f32) -> u8 {
    let az = z.abs();
    match pat {
        Sail::Sunburst => {
            let a = az.atan2(18.5 - x);
            match ((a / 0.23) as i32) % 3 {
                0 => G_A,
                1 => G_B,
                _ => G_C,
            }
        }
        Sail::Zigzag => {
            let v = 17.0 - x;
            let tri = ((v * 0.35).rem_euclid(2.0) - 1.0).abs() * 4.0;
            match (((az + tri) / 5.0) as i32) % 3 {
                0 => G_A,
                1 => G_B,
                _ => G_C,
            }
        }
        Sail::Flowers => {
            let (cx, cz) = ((x / 7.0).floor(), (z / 7.0).floor());
            let (fx, fz) = (cx * 7.0 + 3.5 + if (cz as i32) % 2 == 0 { 1.5 } else { -1.5 }, cz * 7.0 + 3.5);
            let d = ((x - fx) * (x - fx) + (z - fz) * (z - fz)).sqrt();
            if d < 1.2 {
                G_C
            } else if d < 2.8 && ((cx + cz) as i32).rem_euclid(2) == 0 {
                G_B
            } else {
                G_A
            }
        }
        Sail::Chevron => {
            let v = 17.0 - x - az * 0.55;
            match ((v / 5.0) as i32).rem_euclid(3) {
                0 => G_A,
                1 => G_B,
                _ => G_C,
            }
        }
    }
}

fn dark(r: u8) -> u8 {
    match r {
        G_A => G_AD,
        G_B => G_BD,
        G_C => G_CD,
        x => x,
    }
}

/// Render a wing. Origin is the hang point.
pub fn glider(pat: Sail, pitch_deg: f32, yaw_deg: f32) -> Spr {
    let (w, h) = (64, 52);
    let (ox, oy) = (32, 28);
    let mut s = Spr::new(w, h, ox, oy);
    let (sp_, cp) = pitch_deg.to_radians().sin_cos();
    let (sy, cy) = yaw_deg.to_radians().sin_cos();
    let tf = |p: [f32; 3]| -> [f32; 3] {
        let (x, y, z) = (p[0], p[1], p[2]);
        let x1 = x * cp - y * sp_;
        let y1 = x * sp_ + y * cp;
        let x2 = x1 * cy + z * sy;
        let z2 = -x1 * sy + z * cy;
        [x2, -y1 + z2 * 0.30, z2]
    };
    // the airframe (x forward, y up, z toward the viewer)
    let n = [18.0, 1.0, 0.0];
    let tn = [-7.0, 0.5, 24.0];
    let tf_ = [-7.0, 0.5, -24.0];
    let k = [-13.0, 2.0, 0.0];
    let mn = [-8.5, 1.0, 11.5];
    let mf = [-8.5, 1.0, -11.5];
    let post = [-1.0, 9.0, 0.0];
    let base = [-1.0, 2.0, 0.0];
    let scr = |p: [f32; 3]| {
        let q = tf(p);
        (q[0] + ox as f32, q[1] + oy as f32, q[2])
    };
    // kingpost and wires peek out above the sail
    let line = |s: &mut Spr, a: [f32; 3], b: [f32; 3], v: u8| {
        let (pa, pb) = (scr(a), scr(b));
        line_points(pa.0.round() as i32, pa.1.round() as i32, pb.0.round() as i32, pb.1.round() as i32, |x, y| s.set(x, y, v));
    };
    line(&mut s, post, n, G_WIRE);
    line(&mut s, post, k, G_WIRE);
    line(&mut s, post, tn, G_WIRE);
    line(&mut s, post, tf_, G_WIRE);
    line(&mut s, base, post, G_TUBE);
    // sail triangles, far ones first
    let mut tris = [[n, tn, mn], [n, mn, k], [n, k, mf], [n, mf, tf_]];
    tris.sort_by(|a, b| {
        let za = tf(a[0])[2] + tf(a[1])[2] + tf(a[2])[2];
        let zb = tf(b[0])[2] + tf(b[1])[2] + tf(b[2])[2];
        za.partial_cmp(&zb).unwrap()
    });
    for tri in &tris {
        let p: Vec<(f32, f32, f32)> = tri.iter().map(|&q| scr(q)).collect();
        // facing: light from the upper front
        let (ax, ay) = (p[1].0 - p[0].0, p[1].1 - p[0].1);
        let (bx, by) = (p[2].0 - p[0].0, p[2].1 - p[0].1);
        let cross = ax * by - ay * bx;
        if cross.abs() < 0.5 {
            continue;
        }
        let a3 = tri.map(|q| tf(q));
        let lit = {
            let u = [a3[1][0] - a3[0][0], a3[1][1] - a3[0][1], a3[1][2] - a3[0][2]];
            let v = [a3[2][0] - a3[0][0], a3[2][1] - a3[0][1], a3[2][2] - a3[0][2]];
            let nz = u[0] * v[1] - u[1] * v[0];
            let ny = u[2] * v[0] - u[0] * v[2];
            let nx = u[1] * v[2] - u[2] * v[1];
            let len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.001);
            // light toward the viewer and from above the screen
            ((nz * 0.7 - ny * 0.5 + nx * 0.2) / len).abs() > 0.45
        };
        let minx = p.iter().map(|q| q.0).fold(f32::MAX, f32::min).floor() as i32;
        let maxx = p.iter().map(|q| q.0).fold(f32::MIN, f32::max).ceil() as i32;
        let miny = p.iter().map(|q| q.1).fold(f32::MAX, f32::min).floor() as i32;
        let maxy = p.iter().map(|q| q.1).fold(f32::MIN, f32::max).ceil() as i32;
        for yy in miny..=maxy {
            for xx in minx..=maxx {
                let (fx, fy) = (xx as f32 + 0.5, yy as f32 + 0.5);
                let w0 = ((p[1].0 - fx) * (p[2].1 - fy) - (p[1].1 - fy) * (p[2].0 - fx)) / cross;
                let w1 = ((p[2].0 - fx) * (p[0].1 - fy) - (p[2].1 - fy) * (p[0].0 - fx)) / cross;
                let w2 = 1.0 - w0 - w1;
                if w0 < -0.02 || w1 < -0.02 || w2 < -0.02 {
                    continue;
                }
                let mx = w0 * tri[0][0] + w1 * tri[1][0] + w2 * tri[2][0];
                let mz = w0 * tri[0][2] + w1 * tri[1][2] + w2 * tri[2][2];
                let mut r = sail_band(pat, mx, mz);
                if !lit {
                    r = dark(r);
                }
                // the sail darkens toward the trailing edge
                if mx < -9.0 + mz.abs() * 0.1 && (xx + yy) % 2 == 0 {
                    r = dark(r);
                }
                s.set(xx, yy, r);
            }
        }
    }
    // tubes: keel and leading edges
    line(&mut s, n, k, G_TUBED);
    let near_first = tf(tn)[2] < tf(tf_)[2];
    let (first, second) = if near_first { (tn, tf_) } else { (tf_, tn) };
    line(&mut s, n, first, G_TUBED);
    line(&mut s, n, second, G_TUBE);
    s.outline(G_OUT);
    s
}

/// The control frame's two uprights and base bar for a yaw (screen offsets
/// from the hang point): apex, near bar end, far bar end.
pub fn aframe(yaw_deg: f32, hand: (i32, i32)) -> [(i32, i32); 3] {
    let (sy, cy) = yaw_deg.to_radians().sin_cos();
    let bar = |z: f32| {
        let x = hand.0 as f32;
        let x2 = x * cy + z * sy;
        let z2 = -x * sy + z * cy;
        ((x2).round() as i32, (hand.1 as f32 + z2 * 0.3).round() as i32)
    };
    [(0, 0), bar(7.0), bar(-7.0)]
}

// ------------------------------------------------------------------ pilots

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PPose {
    Prone,
    Prone2,
    Tuck,
    Upright,
    Flare,
    Stand,
    Run1,
    Run2,
    Run3,
    Run4,
    Walk1,
    Walk2,
    Paddle1,
    Paddle2,
    Sit,
    Down,
    Cheer,
}

pub const ALL_PPOSES: [PPose; 17] = [
    PPose::Prone,
    PPose::Prone2,
    PPose::Tuck,
    PPose::Upright,
    PPose::Flare,
    PPose::Stand,
    PPose::Run1,
    PPose::Run2,
    PPose::Run3,
    PPose::Run4,
    PPose::Walk1,
    PPose::Walk2,
    PPose::Paddle1,
    PPose::Paddle2,
    PPose::Sit,
    PPose::Down,
    PPose::Cheer,
];

impl PPose {
    pub fn prone(self) -> bool {
        matches!(self, PPose::Prone | PPose::Prone2 | PPose::Tuck)
    }
    /// Holds a glider (the wing is drawn above).
    pub fn carries(self) -> bool {
        !matches!(self, PPose::Paddle1 | PPose::Paddle2 | PPose::Sit | PPose::Down | PPose::Cheer)
    }
}

const fn arms(k: Skel, be: (i32, i32), bh: (i32, i32), fe: (i32, i32), fh: (i32, i32)) -> Skel {
    [k[0], k[1], k[2], be, bh, fe, fh, k[7], k[8], k[9], k[10]]
}

const RUN1: Skel = [(14, 6), (13, 11), (12, 19), (15, 14), (18, 12), (10, 15), (8, 18), (9, 24), (6, 27), (16, 23), (17, 29)];
const RUN2: Skel = [(14, 5), (13, 10), (12, 18), (12, 15), (13, 19), (14, 15), (15, 19), (11, 23), (9, 27), (14, 23), (13, 29)];
const RUN3: Skel = [(14, 6), (13, 11), (12, 19), (10, 15), (8, 18), (16, 14), (19, 12), (16, 23), (17, 29), (9, 24), (6, 27)];
const RUN4: Skel = [(14, 5), (13, 10), (12, 18), (14, 15), (15, 19), (12, 15), (13, 19), (14, 23), (13, 29), (11, 23), (9, 27)];

/// Skeleton, lying flat, head on its side.
fn skel(p: PPose) -> (Skel, bool, bool) {
    let hold = |k: Skel| arms(k, (15, k[1].1 + 4), (17, k[1].1 + 1), (16, k[1].1 + 5), (18, k[1].1 + 2));
    match p {
        PPose::Prone => ([(19, 14), (16, 16), (8, 17), (17, 20), (17, 24), (16, 21), (16, 25), (3, 17), (0, 16), (3, 18), (0, 18)], true, false),
        PPose::Prone2 => ([(19, 14), (16, 16), (8, 17), (17, 20), (17, 24), (16, 21), (16, 25), (3, 18), (0, 19), (3, 19), (0, 20)], true, false),
        PPose::Tuck => ([(20, 15), (17, 17), (9, 17), (15, 20), (18, 21), (14, 21), (17, 22), (4, 17), (0, 17), (4, 18), (0, 18)], true, false),
        PPose::Upright => ([(14, 5), (13, 10), (12, 19), (15, 14), (17, 11), (16, 15), (18, 12), (11, 24), (9, 29), (13, 24), (12, 29)], false, false),
        PPose::Flare => ([(13, 6), (13, 11), (12, 19), (16, 9), (19, 5), (17, 10), (20, 6), (11, 24), (10, 29), (14, 24), (15, 29)], false, false),
        PPose::Stand => (hold([(12, 5), (12, 10), (12, 19), (0, 0), (0, 0), (0, 0), (0, 0), (10, 24), (10, 29), (14, 24), (14, 29)]), false, false),
        PPose::Run1 => (hold(RUN1), false, false),
        PPose::Run2 => (hold(RUN2), false, false),
        PPose::Run3 => (hold(RUN3), false, false),
        PPose::Run4 => (hold(RUN4), false, false),
        PPose::Walk1 => (hold([(13, 7), (12, 11), (12, 19), (0, 0), (0, 0), (0, 0), (0, 0), (10, 24), (9, 29), (14, 24), (15, 29)]), false, false),
        PPose::Walk2 => (hold([(13, 6), (12, 10), (12, 19), (0, 0), (0, 0), (0, 0), (0, 0), (12, 24), (11, 29), (13, 24), (13, 29)]), false, false),
        PPose::Paddle1 => ([(13, 8), (12, 13), (12, 21), (16, 17), (19, 20), (9, 16), (6, 19), (11, 26), (10, 30), (13, 26), (13, 30)], false, false),
        PPose::Paddle2 => ([(13, 8), (12, 13), (12, 21), (9, 16), (6, 19), (16, 17), (19, 20), (11, 26), (10, 30), (13, 26), (13, 30)], false, false),
        PPose::Sit => ([(11, 15), (11, 20), (10, 27), (8, 23), (7, 28), (14, 23), (16, 26), (15, 23), (19, 29), (16, 24), (20, 29)], false, false),
        PPose::Down => ([(5, 25), (9, 27), (17, 27), (10, 23), (12, 21), (12, 30), (15, 30), (20, 24), (23, 27), (21, 27), (23, 29)], true, true),
        PPose::Cheer => ([(12, 5), (12, 10), (12, 19), (8, 7), (4, 2), (16, 7), (20, 2), (10, 24), (9, 29), (14, 24), (15, 29)], false, false),
    }
}

/// Canvas point that sits on the ground / is the pilot's position.
fn anchor(p: PPose) -> (i32, i32) {
    if p.prone() { (11, 25) } else { (12, 30) }
}

/// Hang point (wing origin) relative to the pilot position.
pub fn hang(p: PPose) -> (i32, i32) {
    if p.prone() {
        (0, -20)
    } else {
        (0, -29)
    }
}

/// Front hand relative to the pilot position (where the control bar is).
pub fn hand(p: PPose) -> (i32, i32) {
    let (k, _, _) = skel(p);
    let a = anchor(p);
    (k[6].0 - a.0, k[6].1 - a.1)
}

fn pilot_sprite(def: usize, pose: PPose) -> Spr {
    let d = &PILOTS[def];
    let (k, flat, side) = skel(pose);
    let s = person(&k, d.style, flat, side);
    let a = anchor(pose);
    let mut o = Spr { w: s.w, h: s.h, ox: a.0, oy: a.1, px: s.px.clone() };
    let hd = k[0];
    let in_head = |x: i32, y: i32| !side && x >= hd.0 - 5 && x < hd.0 + 5 && y >= hd.1 - 5 && y < hd.1 + 5;
    if pose.prone() {
        // a harness cocoon instead of bare legs
        for y in 0..o.h {
            for x in 0..k[2].0 {
                let v = o.get(x, y);
                let nv = match v {
                    sp::SKIN | sp::ACC | sp::WHITE => sp::PANTS,
                    sp::SKIN_D | sp::ACC_D => sp::PANTS_D,
                    _ => v,
                };
                o.set(x, y, nv);
            }
        }
    }
    if def == EAGLE {
        // yellow talons everywhere but the head
        for y in 0..o.h {
            for x in 0..o.w {
                if in_head(x, y) {
                    continue;
                }
                let v = o.get(x, y);
                if v == sp::SKIN {
                    o.set(x, y, sp::ACC);
                } else if v == sp::SKIN_D {
                    o.set(x, y, sp::ACC_D);
                }
            }
        }
        if !side {
            // hooked beak and sunglasses
            let (hx, hy) = (hd.0 - 5, hd.1 - 5);
            let mut add = |x: i32, y: i32, v: u8| o.set(hx + x, hy + y, v);
            add(9, 5, sp::ACC);
            add(9, 6, sp::ACC);
            add(10, 6, sp::ACC);
            add(10, 7, sp::ACC_D);
            add(9, 7, sp::ACC_D);
            for x in 5..=8 {
                add(x, 4, sp::EYE);
            }
            add(6, 5, sp::EYE);
            add(7, 5, sp::EYE);
            add(8, 5, sp::WHITE);
        }
        o.outline(sp::OUT);
    }
    o
}

/// A prone pilot seen from the front (the middle of a turn).
fn front_pilot(def: usize) -> Spr {
    let rows: [&str; 12] = if def == EAGLE {
        [
            "............",
            "...HHHHHH...",
            "..HHHHHHHH..",
            "..HKKHHKKH..",
            "..HKWHHKWH..",
            "..HHHAAHHH..",
            "...HHAAHH...",
            "....HaaH....",
            "..TTTTTTTT..",
            ".TTTTTTTTTT.",
            "AA.TTTTTT.AA",
            "A...tttt...A",
        ]
    } else {
        let d = &PILOTS[def];
        match d.style {
            2 => [
                "....CCCC....",
                "...CCCCCC...",
                "..CCWCCCCC..",
                "..cccccccc..",
                "..sSSSSSSs..",
                "..SKSSSSKS..",
                "..SSSssSSS..",
                "...SSSSSS...",
                "..TTAAAATT..",
                ".TTTTTTTTTT.",
                "SS.TTTTTT.SS",
                "S...tttt...S",
            ],
            1 => [
                "....HHHH....",
                "...HHHHHH...",
                "..HHHHHHHH..",
                "..CCCCCCCC..",
                ".HHSSSSSSHH.",
                ".HSKSSSSKSH.",
                ".HSSSssSSSH.",
                ".H.SSSSSS.H.",
                "..TTAAAATT..",
                ".TTTTTTTTTT.",
                "SS.TTTTTT.SS",
                "S...tttt...S",
            ],
            _ => [
                "....HHHH....",
                "...HHHHHH...",
                "..HHHHHHHH..",
                "..HHhhhhHH..",
                "..HSSSSSSH..",
                "..SKWSSWKS..",
                "..SSSssSSS..",
                "...SSSSSS...",
                "..TTAAAATT..",
                ".TTTTTTTTTT.",
                "SS.TTTTTT.SS",
                "S...tttt...S",
            ],
        }
    };
    let mut s = Spr::new(14, 14, 7, 12);
    for (y, r) in rows.iter().enumerate() {
        for (x, ch) in r.bytes().enumerate() {
            let v = match ch {
                b'S' => sp::SKIN,
                b's' => sp::SKIN_D,
                b'H' => sp::HAIR,
                b'h' => sp::HAIR_D,
                b'T' => sp::SHIRT,
                b't' => sp::SHIRT_D,
                b'C' => sp::CAP,
                b'c' => sp::CAP_D,
                b'W' => sp::WHITE,
                b'K' => sp::EYE,
                b'A' => sp::ACC,
                b'a' => sp::ACC_D,
                _ => 0,
            };
            s.set(x as i32 + 1, y as i32 + 1, v);
        }
    }
    s.outline(sp::OUT);
    s
}

// ------------------------------------------------------------------ critters

const WOOL: u32 = rgb(248, 248, 240);
const WOOL_D: u32 = rgb(200, 200, 208);
const INK: u32 = rgb(40, 32, 48);

fn sheep_pic(rows: &[&str]) -> Pic {
    Pic::from_rows(rows, &[(b'W', WOOL), (b'w', WOOL_D), (b'K', INK), (b'E', rgb(248, 248, 248)), (b'P', rgb(248, 160, 176))], 7, 9, true)
}

fn cow_pic(rows: &[&str]) -> Pic {
    Pic::from_rows(
        rows,
        &[(b'W', rgb(248, 248, 240)), (b'w', rgb(200, 200, 208)), (b'B', INK), (b'P', rgb(248, 160, 176)), (b'Y', rgb(232, 216, 168)), (b'E', rgb(248, 248, 248))],
        9,
        12,
        true,
    )
}

/// Birds: (body, dark, light, beak/legs).
fn bird_pic(rows: &[&str], cols: [u32; 4], ox: i32, oy: i32) -> Pic {
    Pic::from_rows(rows, &[(b'W', cols[0]), (b'K', cols[1]), (b'G', cols[2]), (b'R', cols[3])], ox, oy, true)
}

pub struct Critters {
    pub sheep: [Pic; 5],
    pub cow: [Pic; 2],
    pub stork: [Pic; 3],
    pub stork_bundle: [Pic; 3],
    pub vulture: [Pic; 3],
    pub goose: [Pic; 2],
    pub gull: [Pic; 2],
}

impl Critters {
    fn new() -> Self {
        let sheep = [
            // stand
            sheep_pic(&[
                "...wWWWw......",
                ".wWWWWWWWw.KK.",
                "wWWWWWWWWWKKKK",
                "wWWWWWWWWWKEKK",
                "wWWWWWWWWWwKKP",
                ".wwWWWWWWww...",
                "..K.K...K.K...",
                "..K.K...K.K...",
            ]),
            // walk
            sheep_pic(&[
                "...wWWWw......",
                ".wWWWWWWWw.KK.",
                "wWWWWWWWWWKKKK",
                "wWWWWWWWWWKEKK",
                "wWWWWWWWWWwKKP",
                ".wwWWWWWWww...",
                ".K..K..K...K..",
                "K....K.K....K.",
            ]),
            // graze
            sheep_pic(&[
                "..............",
                "...wWWWw......",
                ".wWWWWWWWw....",
                "wWWWWWWWWWw...",
                "wWWWWWWWWWKKK.",
                ".wwWWWWWWwKEKK",
                "..K.K...K.KKKP",
                "..K.K...K.K...",
            ]),
            // run
            sheep_pic(&[
                "....wWWWw.....",
                "..wWWWWWWWwKK.",
                ".wWWWWWWWWKKKK",
                "wWWWWWWWWWKEKK",
                ".wWWWWWWWwwKKP",
                "..wwWWWWww....",
                ".K.K....K..K..",
                "K...K..K....K.",
            ]),
            // dizzy (lying)
            sheep_pic(&[
                "..............",
                "..............",
                "...wWWWWw.....",
                ".wWWWWWWWWw...",
                "wWWWWWWWWWWKKK",
                "wWWWWWWWWWKEEK",
                ".wwKKwwKKww.KP",
                "..............",
            ]),
        ];
        let cow = [
            cow_pic(&[
                "..............YY..",
                "...............BBY",
                ".BBWWWWBBBWWWWBBEB",
                "BBBWWWWBBBWWWWWBBB",
                "BWWWBBWWWWWBBWWWPP",
                "BWWWBBWWWWWBBWW.PP",
                ".WWWWWWWWWWWWWW...",
                "..WPWW.....WWW....",
                "..B.B......B.B....",
                "..B.B......B.B....",
            ]),
            cow_pic(&[
                "..................",
                "..............YY..",
                ".BBWWWWBBBWWWWBBBY",
                "BBBWWWWBBBWWWWWBEB",
                "BWWWBBWWWWWBBWWBBB",
                "BWWWBBWWWWWBBWWPPP",
                ".WWWWWWWWWWWWWW.PP",
                "..WPWW.....WWW....",
                "..B.B......B.B....",
                "..B.B......B.B....",
            ]),
        ];
        let stork_c = [rgb(248, 248, 248), rgb(40, 40, 56), rgb(200, 200, 216), rgb(248, 96, 48)];
        let stork_rows: [[&str; 9]; 3] = [
            [
                ".......KKKWW..........",
                "......KKKKWWWW........",
                ".......KKKWWWWW.......",
                "RR.....WWWWWWWWW......",
                "RRRRRWWWWWWWWWWWWWWRRR",
                "......WWWWWWWWW....R..",
                ".......KKKWWWW........",
                "......KKKKWWW.........",
                ".......KKKW...........",
            ],
            [
                ".....KKKK.............",
                ".....KKKKWW...........",
                "......KKKWWW..........",
                ".......KKWWWW.........",
                "RR.....WWWWWWWWW......",
                "RRRRRWWWWWWWWWWWWWWRRR",
                "......WWWWWWWWW....R..",
                ".........KKWW.........",
                "......................",
            ],
            [
                "......................",
                "......................",
                ".........KKWW.........",
                "RR.....WWWWWWWWW......",
                "RRRRRWWWWWWWWWWWWWWRRR",
                "......WWWWWWWWW....R..",
                ".......KKWWWW.........",
                "......KKKWWW..........",
                ".....KKKKWW...........",
            ],
        ];
        let stork = stork_rows.map(|r| bird_pic(&r, stork_c, 11, 4));
        let bundle_rows: [[&str; 13]; 3] = stork_rows.map(|r| {
            [r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], ".....................K", "....................GG", "...................GPPG", "....................GG"]
        });
        let bundle = bundle_rows.map(|r| {
            Pic::from_rows(
                &r,
                &[(b'W', stork_c[0]), (b'K', stork_c[1]), (b'G', rgb(240, 240, 248)), (b'R', stork_c[3]), (b'P', rgb(248, 200, 160))],
                11,
                4,
                true,
            )
        });
        let vult_c = [rgb(104, 72, 48), rgb(56, 40, 32), rgb(232, 168, 160), rgb(232, 200, 120)];
        let vulture = [
            bird_pic(
                &[
                    "......K.K.KWW.........",
                    "......KKKKWWWWW.......",
                    ".......KKWWWWWW.......",
                    "GG.....WWWWWWWWW......",
                    "GGWWWWWWWWWWWWWWWWGGR.",
                    "......WWWWWWWWW...GG..",
                    ".......KKWWWWW........",
                    "......KKKKWWW.........",
                    "......K.K.KW..........",
                ],
                vult_c,
                11,
                4,
            ),
            bird_pic(
                &[
                    "....K.K.K.............",
                    ".....KKKKWW...........",
                    "......KKKWWW..........",
                    ".......KKWWWW.........",
                    "GG.....WWWWWWWWW......",
                    "GGWWWWWWWWWWWWWWWWGGR.",
                    "......WWWWWWWWW...GG..",
                    ".........KKWW.........",
                    "......................",
                ],
                vult_c,
                11,
                4,
            ),
            bird_pic(
                &[
                    "......................",
                    "......................",
                    ".........KKWW.........",
                    "GG.....WWWWWWWWW......",
                    "GGWWWWWWWWWWWWWWWWGGR.",
                    "......WWWWWWWWW...GG..",
                    ".......KKWWWW.........",
                    "......KKKWWW..........",
                    "....K.K.K.............",
                ],
                vult_c,
                11,
                4,
            ),
        ];
        let goose_c = [rgb(168, 152, 128), rgb(40, 40, 48), rgb(232, 224, 208), rgb(64, 56, 48)];
        let goose = [
            bird_pic(&["..WW.........", "...WWW.......", "....WWW...KK.", "KWWWWWWKKKKGK", "..WWGGGG.....", "............."], goose_c, 6, 3),
            bird_pic(&[".............", ".............", "..........KK.", "KWWWWWWKKKKGK", "..WWWWGG.....", "..WWW........", ".WW.........."], goose_c, 6, 3),
        ];
        let gull_c = [rgb(248, 248, 248), rgb(120, 128, 144), rgb(200, 208, 224), rgb(248, 200, 48)];
        let gull = [
            bird_pic(&["K.......K", ".WW...WW.", "...WWWR..", "........."], gull_c, 4, 2),
            bird_pic(&[".........", "...WWWR..", ".WW...WW.", "K.......K"], gull_c, 4, 2),
        ];
        Critters { sheep, cow, stork, stork_bundle: bundle, vulture, goose, gull }
    }
}

// ------------------------------------------------------------------ bank

pub struct Wings {
    pub pitch: Vec<Spr>,
    pub yaw: Vec<Spr>,
}

pub struct Art {
    /// [pilot][pose]
    pub pilots: Vec<Vec<Spr>>,
    pub fronts: Vec<Spr>,
    pub wings: Vec<Wings>,
    pub critters: Critters,
}

impl Art {
    pub fn new() -> Self {
        let pilots = (0..PILOTS.len()).map(|d| ALL_PPOSES.iter().map(|&p| pilot_sprite(d, p)).collect()).collect();
        let fronts = (0..PILOTS.len()).map(front_pilot).collect();
        let wings = PILOTS
            .iter()
            .map(|d| Wings {
                pitch: PITCHES.iter().map(|&p| glider(d.pattern, p, 0.0)).collect(),
                yaw: YAWS.iter().map(|&y| glider(d.pattern, 0.0, y)).collect(),
            })
            .collect();
        Art { pilots, fronts, wings, critters: Critters::new() }
    }
    pub fn pilot(&self, def: usize, pose: PPose) -> &Spr {
        let i = ALL_PPOSES.iter().position(|&p| p == pose).unwrap_or(0);
        &self.pilots[def][i]
    }
    /// Wing frame for a pitch (-1..1 game units scaled to degrees).
    pub fn wing_pitch(&self, def: usize, deg: f32) -> &Spr {
        let i = PITCHES.iter().enumerate().min_by(|a, b| (a.1 - deg).abs().partial_cmp(&(b.1 - deg).abs()).unwrap()).map(|(i, _)| i).unwrap_or(3);
        &self.wings[def].pitch[i]
    }
}

impl Default for Art {
    fn default() -> Self {
        Self::new()
    }
}

/// Shade a color toward another for haze (far layers).
pub fn haze(c: u32, sky: u32, t: f32) -> u32 {
    mix(c, sky, t)
}
