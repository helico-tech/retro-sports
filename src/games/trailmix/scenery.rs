//! Backdrops and props. Everything static is drawn once: per stage the sky,
//! three parallax strips (far range, hills, tree line) and ground textures;
//! for the game all props and obstacles as outlined color sprites. Per frame
//! only blits, clouds, birds and fog.

use super::course::{Ground, SIGNS};
use super::sprites::{scratch, CSprite, KEY};
use crate::engine::gfx::*;
use crate::engine::rng::Rng;
use crate::engine::ui::OUTLINE;

/// Width of the wrapping parallax strips.
pub const STRIP_W: i32 = 512;

/// A strip that wraps horizontally; KEY is transparent.
pub struct Strip {
    pub h: i32,
    pub px: Vec<u32>,
}

impl Strip {
    fn new(h: i32) -> Strip {
        Strip { h, px: vec![KEY; (STRIP_W * h) as usize] }
    }
    fn set(&mut self, x: i32, y: i32, c: u32) {
        if y >= 0 && y < self.h {
            let x = x.rem_euclid(STRIP_W);
            self.px[(y * STRIP_W + x) as usize] = c;
        }
    }
    fn get(&self, x: i32, y: i32) -> u32 {
        if y >= 0 && y < self.h { self.px[(y * STRIP_W + x.rem_euclid(STRIP_W)) as usize] } else { KEY }
    }
    fn vline(&mut self, x: i32, y0: i32, y1: i32, c: u32) {
        for y in y0.max(0)..y1.min(self.h) {
            self.set(x, y, c);
        }
    }
    /// Blit with its top at screen row `top`, scrolled by `off`, clipped to rows [y0, y1).
    pub fn draw(&self, f: &mut Frame, off: i32, top: i32, y0: i32, y1: i32) {
        let off = off.rem_euclid(STRIP_W);
        for sy in 0..self.h {
            let py = top + sy;
            if py < y0 || py >= y1 || !(0..H).contains(&py) {
                continue;
            }
            let row = (sy * STRIP_W) as usize;
            let dst = (py * W) as usize;
            for x in 0..W {
                let c = self.px[row + ((x + off) % STRIP_W) as usize];
                if c != KEY {
                    f.px[dst + x as usize] = c;
                }
            }
        }
    }
}

/// 32x32 ground texture; the top face uses rows 0..TOP.
pub struct Tex {
    pub top: [u32; 32 * 12],
    /// front face, three depth shades
    pub front: [[u32; 32 * 32]; 3],
    pub edge: u32,
    pub lip: u32,
}

pub const TOP: i32 = 10;

pub struct Scenery {
    pub stage: usize,
    pub sky: Vec<u32>,
    pub sky_h: i32,
    pub far: Strip,
    pub hills: Strip,
    pub trees: Strip,
    pub tex: Vec<Tex>,
    pub clouds: Vec<CSprite>,
    pub verge: [u32; 3],
    pub haze: u32,
}

pub fn ground_index(g: Ground) -> usize {
    match g {
        Ground::Dirt => 0,
        Ground::Grass => 1,
        Ground::Rock => 2,
        Ground::Mud => 3,
        Ground::Scree => 4,
        Ground::Snow => 5,
        Ground::Water => 6,
        Ground::Road => 7,
    }
}

/// Natural rock: irregular stones (cells around random points, wrapping
/// every 32 px) with dark cracks and a lit top-left edge.
fn rock_px(x: i32, y: i32, s: u32, r: u32) -> u32 {
    let mut best = (i32::MAX, i32::MAX, 0usize);
    let mut pts = [(0i32, 0i32); 7];
    for (k, p) in pts.iter_mut().enumerate() {
        let hk = hash(k as i32, 7, s);
        *p = ((hk % 32) as i32, ((hk >> 8) % 32) as i32);
    }
    for (k, &(px, py)) in pts.iter().enumerate() {
        let mut dx = (x - px).abs();
        let mut dy = (y - py).abs();
        dx = dx.min(32 - dx);
        dy = dy.min(32 - dy);
        let d = dx * dx + dy * dy * 2;
        if d < best.0 {
            best = (d, best.0, k);
        } else if d < best.1 {
            best.1 = d;
        }
    }
    let edge = (best.1 as f32).sqrt() - (best.0 as f32).sqrt();
    let (px, py) = pts[best.2];
    let lit = (x - px) + (y - py) < -3;
    let tone = [rgb(120, 120, 144), rgb(128, 120, 136), rgb(112, 116, 140)][best.2 % 3];
    if edge < 1.0 {
        rgb(64, 64, 88)
    } else if edge < 2.0 && lit {
        rgb(168, 168, 184)
    } else if r < 3 {
        shade(tone, 7, 8)
    } else {
        tone
    }
}

fn hash(x: i32, y: i32, s: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263) ^ s.wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

impl Scenery {
    pub fn new(stage: usize) -> Scenery {
        let mut rng = Rng(0xA11CE ^ (stage as u64 * 101 + 9));
        let sky_cols: &[u32] = match stage {
            0 => &[rgb(72, 136, 216), rgb(96, 160, 224), rgb(128, 184, 232), rgb(168, 208, 232), rgb(208, 224, 216), rgb(240, 232, 192)],
            1 => &[rgb(40, 104, 208), rgb(56, 128, 224), rgb(88, 160, 240), rgb(128, 192, 248), rgb(176, 216, 248), rgb(216, 240, 248)],
            _ => &[rgb(24, 40, 120), rgb(32, 64, 160), rgb(48, 96, 192), rgb(80, 136, 216), rgb(128, 176, 232), rgb(192, 216, 240)],
        };
        let sky_h = 140;
        let mut sky = vec![0u32; (sky_h * 2) as usize];
        // two columns per row (even/odd x) give the dithered seam between bands
        let n = sky_cols.len() as i32;
        for y in 0..sky_h {
            let pos = y * n * 4 / sky_h;
            let band = (pos / 4).min(n - 1);
            let sub = pos % 4;
            let a = sky_cols[band as usize];
            let b = sky_cols[(band + 1).min(n - 1) as usize];
            for k in 0..2 {
                sky[(y * 2 + k) as usize] = if sub == 3 && (k + y) & 1 == 0 { b } else { a };
            }
        }
        let haze = sky_cols[n as usize - 1];
        let mut s = Scenery {
            stage,
            sky,
            sky_h,
            far: Strip::new(90),
            hills: Strip::new(70),
            trees: Strip::new(64),
            tex: Vec::new(),
            clouds: Vec::new(),
            verge: [0; 3],
            haze,
        };
        s.make_far(&mut rng);
        s.make_hills(&mut rng);
        s.make_trees(&mut rng);
        s.make_tex();
        s.make_clouds(&mut rng);
        s.verge = match stage {
            0 => [rgb(64, 144, 56), rgb(40, 104, 48), rgb(112, 184, 72)],
            1 => [rgb(96, 184, 72), rgb(64, 136, 56), rgb(152, 216, 96)],
            _ => [rgb(136, 136, 152), rgb(96, 96, 120), rgb(184, 184, 200)],
        };
        s
    }

    // ------------------------------------------------------------ strips

    fn make_far(&mut self, rng: &mut Rng) {
        let (body, shade_c, snow, snow_d) = match self.stage {
            0 => (rgb(120, 152, 200), rgb(104, 128, 184), rgb(232, 240, 248), rgb(184, 200, 232)),
            1 => (rgb(112, 136, 200), rgb(88, 112, 176), rgb(248, 248, 248), rgb(192, 208, 240)),
            _ => (rgb(96, 104, 168), rgb(72, 80, 144), rgb(248, 248, 248), rgb(184, 200, 232)),
        };
        let h = self.far.h;
        // peaks: a ridge line from random peaks, then fill
        let mut ridge = vec![0i32; STRIP_W as usize];
        let mut peaks: Vec<(i32, i32)> = Vec::new();
        let mut x = 0;
        while x < STRIP_W {
            peaks.push((x, 8 + rng.int(40)));
            x += 40 + rng.int(50);
        }
        // Mount Helico: the big one
        peaks.push((300, 2));
        peaks.sort();
        for xx in 0..STRIP_W {
            let mut best = h;
            for &(px, py) in &peaks {
                for wrap in [-STRIP_W, 0, STRIP_W] {
                    let d = (xx - px - wrap).abs();
                    let slope = if (xx - px - wrap) < 0 { 1.1 } else { 0.9 };
                    let y = py + (d as f32 * slope) as i32;
                    best = best.min(y);
                }
            }
            ridge[xx as usize] = best + (hash(xx, 0, 3) % 2) as i32;
        }
        for xx in 0..STRIP_W {
            let top = ridge[xx as usize];
            // the left side of each peak is in shade
            let left = ridge[((xx + 1) % STRIP_W) as usize] < top;
            for y in top..h {
                let snowy = y < top + 10 && top < 40;
                let c = if snowy {
                    if left { snow_d } else { snow }
                } else if left {
                    shade_c
                } else {
                    body
                };
                self.far.set(xx, y, c);
            }
            // snow line streaks
            if top < 40 && hash(xx, 1, 5) % 3 == 0 {
                self.far.vline(xx, top + 10, top + 13 + (hash(xx, 2, 5) % 5) as i32, if left { snow_d } else { snow });
            }
        }
        // haze at the foot
        for y in h - 14..h {
            for xx in 0..STRIP_W {
                if (xx + y) % 2 == 0 || y > h - 6 {
                    let c = self.far.get(xx, y);
                    if c != KEY {
                        self.far.set(xx, y, mix(c, rgb(200, 216, 240), 0.35));
                    }
                }
            }
        }
    }

    fn make_hills(&mut self, rng: &mut Rng) {
        let (a, b, hl) = match self.stage {
            0 => (rgb(64, 136, 96), rgb(48, 112, 88), rgb(88, 160, 104)),
            1 => (rgb(104, 176, 88), rgb(80, 152, 72), rgb(144, 200, 104)),
            _ => (rgb(112, 120, 144), rgb(88, 96, 128), rgb(152, 160, 184)),
        };
        let h = self.hills.h;
        let ph = [rng.range(0.0, 6.0), rng.range(0.0, 6.0)];
        for x in 0..STRIP_W {
            let t = x as f32 / STRIP_W as f32 * std::f32::consts::TAU;
            let y = 22.0 + 10.0 * (t * 3.0 + ph[0]).sin() + 6.0 * (t * 5.0 + ph[1]).sin() + 3.0 * (t * 11.0).sin();
            let top = y as i32;
            for yy in top..h {
                let c = if yy < top + 2 { hl } else if (yy - top) > 20 { b } else { a };
                self.hills.set(x, yy, c);
            }
            // dithered seam between the two greens
            if (x % 2) == 0 {
                self.hills.set(x, top + 20, b);
            }
        }
        // little things on the hills: chalets (meadow), rock bands (summit), tree dots (forest)
        let mut x = 10;
        while x < STRIP_W - 10 {
            let top = (0..h).find(|&yy| self.hills.get(x, yy) != KEY).unwrap_or(h);
            match self.stage {
                0 => {
                    for k in 0..3 {
                        let tx = x + k * 3;
                        let tt = (0..h).find(|&yy| self.hills.get(tx, yy) != KEY).unwrap_or(h);
                        for d in 0..5 {
                            for w in -(d / 2)..=(d / 2) {
                                self.hills.set(tx + w, tt - 4 + d, b);
                            }
                        }
                    }
                }
                1 => {
                    if rng.f() < 0.3 {
                        // tiny chalet
                        for yy in top + 2..top + 6 {
                            for xx in x..x + 6 {
                                self.hills.set(xx, yy, rgb(176, 112, 72));
                            }
                        }
                        for d in 0..3 {
                            for xx in x - 1 + d..x + 7 - d {
                                self.hills.set(xx, top + 1 - d, rgb(152, 56, 48));
                            }
                        }
                    } else {
                        for xx in x..x + 3 {
                            self.hills.set(xx, top + 4, hl);
                        }
                    }
                }
                _ => {
                    for k in 0..6 {
                        self.hills.set(x + k, top + 5 + k / 2, b);
                    }
                }
            }
            x += 14 + rng.int(20);
        }
    }

    fn make_trees(&mut self, rng: &mut Rng) {
        let h = self.trees.h;
        match self.stage {
            0 => {
                // dense forest line: overlapping pines
                let (d, m, l) = (rgb(24, 88, 64), rgb(32, 112, 72), rgb(56, 136, 80));
                for x in 0..STRIP_W {
                    self.trees.vline(x, h - 14, h, d);
                }
                let mut x = 0;
                while x < STRIP_W {
                    let th = 26 + rng.int(28);
                    let base = h - 6;
                    for k in 0..th {
                        let half = (k * 7 / th).max(1) + if k % 6 > 3 { 1 } else { 0 };
                        let y = base - th + k;
                        for w in -half..=half {
                            let c = if w > half / 2 { l } else if w < -half / 2 { d } else { m };
                            self.trees.set(x + w, y, c);
                        }
                    }
                    x += 6 + rng.int(8);
                }
            }
            1 => {
                // scattered firs and a fence line on the meadow
                let (d, m) = (rgb(32, 96, 72), rgb(48, 120, 80));
                let g = rgb(120, 192, 88);
                for x in 0..STRIP_W {
                    self.trees.vline(x, h - 8, h, g);
                    if x % 2 == 0 {
                        self.trees.set(x, h - 9, g);
                    }
                }
                let mut x = 0;
                while x < STRIP_W {
                    if rng.f() < 0.6 {
                        let th = 18 + rng.int(18);
                        for k in 0..th {
                            let half = (k * 5 / th).max(1);
                            for w in -half..=half {
                                self.trees.set(x + w, h - 8 - th + k, if w > 0 { m } else { d });
                            }
                        }
                        self.trees.vline(x, h - 8, h - 5, rgb(88, 56, 32));
                    }
                    x += 12 + rng.int(30);
                }
                for x in (0..STRIP_W).step_by(2) {
                    self.trees.set(x, h - 4, rgb(168, 128, 88));
                    if x % 10 == 0 {
                        self.trees.vline(x, h - 7, h - 3, rgb(136, 96, 64));
                    }
                }
            }
            _ => {
                // rocky outcrops with snow patches
                let (r, rd, sn) = (rgb(120, 120, 144), rgb(88, 88, 112), rgb(232, 240, 248));
                let mut x = 0;
                while x < STRIP_W {
                    let w = 10 + rng.int(26);
                    let hh = 10 + rng.int(30);
                    for k in 0..w {
                        let tt = h - hh + ((k as f32 / w as f32 - 0.5).abs() * hh as f32) as i32;
                        for y in tt..h {
                            let c = if y < tt + 2 && hh > 20 { sn } else if k < w / 3 { rd } else { r };
                            self.trees.set(x + k, y, c);
                        }
                    }
                    x += w - 3 + rng.int(14);
                }
            }
        }
    }

    fn make_clouds(&mut self, rng: &mut Rng) {
        for k in 0..3 {
            let mut f = scratch();
            let w = 30 + k * 10;
            let (c, sh) = (rgb(248, 248, 248), rgb(208, 224, 240));
            for _ in 0..5 {
                let cx = 128 + rng.int(w) - w / 2;
                let cy = 100 + rng.int(5) - 2;
                let r = 5 + rng.int(6);
                f.fill_ellipse(cx, cy + 2, r + 2, r / 2 + 2, sh);
            }
            for _ in 0..5 {
                let cx = 128 + rng.int(w) - w / 2;
                let cy = 99 + rng.int(4) - 2;
                let r = 5 + rng.int(6);
                f.fill_ellipse(cx, cy, r + 2, r / 2 + 1, c);
            }
            self.clouds.push(CSprite::cut(&f, 128 - w / 2 - 14, 86, w + 28, 22, false));
        }
    }

    // ------------------------------------------------------------ textures

    fn make_tex(&mut self) {
        let grounds = [Ground::Dirt, Ground::Grass, Ground::Rock, Ground::Mud, Ground::Scree, Ground::Snow, Ground::Water, Ground::Road];
        for (gi, g) in grounds.iter().enumerate() {
            let mut top = [0u32; 32 * 12];
            let mut front = [0u32; 32 * 32];
            let s = gi as u32 * 77 + self.stage as u32;
            let (edge, lip) = match g {
                Ground::Dirt => (rgb(216, 168, 112), rgb(96, 56, 40)),
                Ground::Grass => (rgb(160, 224, 104), rgb(72, 48, 32)),
                Ground::Rock => (rgb(216, 216, 224), rgb(72, 72, 96)),
                Ground::Mud => (rgb(152, 104, 64), rgb(64, 40, 24)),
                Ground::Scree => (rgb(200, 200, 208), rgb(72, 72, 96)),
                Ground::Snow => (rgb(248, 248, 248), rgb(112, 136, 184)),
                Ground::Water => (rgb(168, 216, 248), rgb(24, 56, 120)),
                Ground::Road => (rgb(200, 192, 176), rgb(72, 64, 64)),
            };
            for y in 0..12 {
                for x in 0..32 {
                    let hsh = hash(x, y, s);
                    let r = hsh % 16;
                    let c = match g {
                        Ground::Dirt => {
                            if r == 0 { rgb(120, 80, 56) } else if r < 3 { rgb(208, 160, 104) } else if (x + y) % 2 == 0 && y > 6 { rgb(168, 120, 80) } else { rgb(184, 136, 88) }
                        }
                        Ground::Grass => {
                            // a worn track through the grass
                            if (3..8).contains(&y) {
                                if r < 2 { rgb(144, 104, 64) } else if (x + y) % 2 == 0 && (y == 3 || y == 7) { rgb(88, 160, 64) } else { rgb(176, 136, 88) }
                            } else if r < 3 {
                                rgb(144, 208, 96)
                            } else if r < 6 {
                                rgb(72, 144, 56)
                            } else {
                                rgb(96, 176, 72)
                            }
                        }
                        Ground::Rock => {
                            let crack = (x + y * 3) % 13 == 0 || hsh % 23 == 0;
                            if crack { rgb(112, 112, 128) } else if r < 3 { rgb(192, 192, 200) } else { rgb(160, 160, 176) }
                        }
                        Ground::Mud => {
                            let puddle = ((x / 7 + y / 3) % 4 == 0) && y > 2 && y < 9;
                            if puddle { if r < 4 { rgb(184, 160, 128) } else { rgb(120, 88, 64) } } else if r < 3 { rgb(72, 48, 32) } else { rgb(104, 72, 40) }
                        }
                        Ground::Scree => {
                            if r < 5 { rgb(200, 200, 208) } else if r < 10 { rgb(144, 144, 160) } else { rgb(168, 168, 184) }
                        }
                        Ground::Snow => {
                            if r < 2 { rgb(200, 224, 248) } else if (x + y) % 2 == 0 && y > 7 { rgb(216, 232, 248) } else { rgb(240, 248, 248) }
                        }
                        Ground::Water => {
                            if (x + y * 5) % 9 == 0 { rgb(152, 208, 248) } else if (x + y) % 2 == 0 { rgb(64, 136, 216) } else { rgb(80, 152, 224) }
                        }
                        Ground::Road => {
                            // cobbles
                            let bx = (x + (y / 4) * 3) % 6;
                            let by = y % 4;
                            if bx == 0 || by == 0 { rgb(112, 104, 96) } else if r < 3 { rgb(200, 192, 176) } else { rgb(168, 160, 144) }
                        }
                    };
                    top[(y * 32 + x) as usize] = c;
                }
            }
            for y in 0..32 {
                for x in 0..32 {
                    let hsh = hash(x, y + 100, s);
                    let r = hsh % 32;
                    let strata = (y % 11) == 0 || ((y + 1) % 11 == 0 && x % 2 == 0);
                    let c = match g {
                        Ground::Dirt | Ground::Mud | Ground::Grass => {
                            let base = if *g == Ground::Mud { rgb(96, 64, 40) } else { rgb(144, 96, 64) };
                            if r == 0 || r == 1 {
                                rgb(168, 160, 152) // pebble
                            } else if r == 2 {
                                rgb(88, 64, 48)
                            } else if strata {
                                shade(base, 13, 16)
                            } else if (x + y) % 2 == 0 && r < 10 {
                                shade(base, 15, 16)
                            } else {
                                base
                            }
                        }
                        Ground::Rock | Ground::Scree => rock_px(x, y, s, r),
                        Ground::Snow => {
                            if y < 8 {
                                if (x + y) % 2 == 0 && y > 4 { rgb(176, 200, 232) } else { rgb(216, 232, 248) }
                            } else if y < 10 && (x + y) % 2 == 0 {
                                rgb(176, 200, 232)
                            } else {
                                rock_px(x, y, s, r)
                            }
                        }
                        Ground::Water => {
                            if (x * 3 + y * 7) % 17 == 0 { rgb(120, 184, 240) } else if y > 20 { rgb(32, 72, 152) } else { rgb(48, 104, 192) }
                        }
                        Ground::Road => {
                            let bx = (x + (y / 6) * 5) % 10;
                            if bx == 0 || y % 6 == 0 { rgb(96, 88, 88) } else if r < 5 { rgb(176, 168, 160) } else { rgb(152, 144, 136) }
                        }
                    };
                    front[(y * 32 + x) as usize] = c;
                }
            }
            let mut fr = [[0u32; 32 * 32]; 3];
            for (k, num) in [16u32, 13, 10].iter().enumerate() {
                for i in 0..32 * 32 {
                    fr[k][i] = shade(front[i], *num, 16);
                }
            }
            self.tex.push(Tex { top, front: fr, edge, lip });
        }
    }

    // ------------------------------------------------------------ per frame

    /// Sky, clouds and the three parallax strips for one view.
    pub fn draw_back(&self, f: &mut Frame, cam_x: f32, cam_alt: f32, y0: i32, h: i32, t: u32, ground_y: i32) {
        let y1 = y0 + h;
        // sky rows: scale the gradient to the view height
        for y in y0.max(0)..y1.min(H) {
            let sy = ((y - y0) * self.sky_h / (ground_y - y0 + 20).max(1)).min(self.sky_h - 1);
            let a = self.sky[(sy * 2) as usize];
            let b = self.sky[(sy * 2 + 1) as usize];
            let row = (y * W) as usize;
            for x in 0..W {
                f.px[row + x as usize] = if (x & 1) == 0 { a } else { b };
            }
        }
        // sun (or a high, pale summit sun)
        let (sx, sy) = (200, y0 + 14 + (ground_y - y0) / 10);
        if self.stage != 2 || cam_x < 99999.0 {
            f.fill_ellipse(sx, sy, 9, 9, mix(self.haze, rgb(248, 248, 224), 0.7));
            f.fill_ellipse(sx, sy, 7, 7, rgb(248, 248, 216));
        }
        // clouds drift slowly
        for (k, c) in self.clouds.iter().enumerate() {
            let speed = 0.04 + k as f32 * 0.02;
            let x = ((k as f32 * 160.0 - cam_x * 0.05 - t as f32 * speed) as i32).rem_euclid(W + 120) - 60;
            let y = y0 + 10 + k as i32 * 14 * (ground_y - y0) / 130;
            c.draw_clip(f, x, y, y0, y1);
        }
        // parallax: x and a little y
        let lift = |k: f32| -> i32 { (-(cam_alt * k).clamp(-60.0, 60.0)) as i32 };
        let horizon = ground_y - 6;
        self.far.draw(f, (cam_x * 0.06) as i32 + 40, horizon - self.far.h + 6 + lift(0.03), y0, y1);
        self.hills.draw(f, (cam_x * 0.2) as i32, horizon - self.hills.h + 26 + lift(0.08), y0, y1);
        self.trees.draw(f, (cam_x * 0.45) as i32, horizon - self.trees.h + 34 + lift(0.16), y0, y1);
        // everything below the tree line is the slope behind the trail
        let fill_from = horizon + 34 + lift(0.16);
        let back = match self.stage {
            0 => rgb(24, 88, 64),
            1 => rgb(120, 192, 88),
            _ => rgb(120, 120, 144),
        };
        for y in fill_from.max(y0)..y1.min(H) {
            let row = (y * W) as usize;
            for x in 0..W {
                f.px[row + x as usize] = back;
            }
        }
        // birds
        if self.stage > 0 {
            for k in 0..3u32 {
                let bx = ((k as f32 * 97.0 + t as f32 * (0.3 + k as f32 * 0.1) - cam_x * 0.1) as i32).rem_euclid(W + 40) - 20;
                let by = y0 + 20 + (k as i32 * 9) + ((t as f32 * 0.05 + k as f32).sin() * 3.0) as i32;
                let flap = ((t / 8 + k) % 2) as i32;
                let c = rgb(40, 40, 64);
                f.set(bx - 2, by - flap, c);
                f.set(bx - 1, by - 1 + flap, c);
                f.set(bx, by, c);
                f.set(bx + 1, by - 1 + flap, c);
                f.set(bx + 2, by - flap, c);
            }
        }
    }
}

// ---------------------------------------------------------------- props

pub struct PropArt {
    pub pine: [CSprite; 2],
    pub leafy: [CSprite; 2],
    pub birch: CSprite,
    pub fir: [CSprite; 2],
    pub bush: [CSprite; 2],
    pub fern: CSprite,
    pub flowers: [CSprite; 3],
    pub boulder: [CSprite; 2],
    pub fence: CSprite,
    pub ribbon: CSprite,
    pub signs: Vec<CSprite>,
    pub deer_sign: CSprite,
    pub km: Vec<CSprite>,
    pub start_arch: CSprite,
    pub finish_arch: CSprite,
    pub summit: CSprite,
    pub cairn: CSprite,
    pub snow_pole: CSprite,
    pub house: [CSprite; 2],
    pub church: CSprite,
    pub hut: CSprite,
    pub aid_table: CSprite,
    pub hole: CSprite,
    pub root: [CSprite; 2],
    pub rock: [CSprite; 2],
    pub log: CSprite,
    pub mushroom: [CSprite; 2],
    pub branch: CSprite,
    pub trunk: CSprite,
    pub snow_rock: CSprite,
    pub stone: CSprite,
    pub podium: CSprite,
    /// stone, bone, fossil, lost shoe, roots, ice
    pub buried: [CSprite; 6],
}

const BARK: u32 = rgb(112, 72, 48);
const BARK_D: u32 = rgb(80, 48, 32);
const BARK_L: u32 = rgb(152, 104, 64);

/// Draw at (128, 200) as the base point and cut the box around it.
fn cut(f: &Frame, w: i32, h: i32) -> CSprite {
    let mut s = CSprite::cut(f, 128 - w / 2, 201 - h, w, h, true);
    s.ay = s.h - 2;
    s
}

fn pine(h: i32, dark: bool) -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    f.rect(x - 1, base - 6, 3, 7, BARK);
    let (g, gd, gl) = if dark { (rgb(24, 96, 72), rgb(16, 64, 48), rgb(48, 128, 80)) } else { (rgb(40, 120, 72), rgb(24, 88, 56), rgb(72, 152, 88)) };
    for tier in 0..4 {
        let top = base - h + tier * h / 5;
        let bot = base - 5 - (3 - tier) * h / 7;
        let half = 4 + tier * 3;
        f.fill_poly(&[(x, top), (x + half + 1, bot), (x - half, bot)], g);
        f.fill_poly(&[(x, top), (x - half, bot), (x - half / 2, bot)], gd);
        f.line(x + 1, top + 2, x + half, bot - 1, gl);
        f.hline(x - half, x + half, bot, gd);
    }
    cut(&f, 34, h + 2)
}

fn leafy(big: bool) -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let h = if big { 46 } else { 36 };
    f.thick_line(x, base, x, base - h / 2, 4, BARK);
    f.line(x - 1, base, x - 1, base - h / 2, BARK_D);
    f.thick_line(x, base - h / 2 + 4, x - 7, base - h / 2 - 4, 2, BARK);
    f.thick_line(x, base - h / 2 + 2, x + 7, base - h / 2 - 6, 2, BARK);
    let (d, m, l, hl) = (rgb(32, 104, 48), rgb(56, 144, 56), rgb(96, 176, 64), rgb(152, 208, 96));
    let r = if big { 15 } else { 11 };
    let cy = base - h + r;
    for (dx, dy, rr) in [(-8, 4, r - 4), (8, 3, r - 4), (0, 0, r), (-4, -6, r - 5), (6, -5, r - 6)] {
        f.fill_ellipse(x + dx, cy + dy + 1, rr, rr - 2, d);
    }
    for (dx, dy, rr) in [(-8, 3, r - 6), (8, 2, r - 6), (0, -1, r - 2), (-4, -7, r - 7), (6, -6, r - 8)] {
        f.fill_ellipse(x + dx + 1, cy + dy - 1, rr, rr - 2, m);
    }
    for (dx, dy, rr) in [(2, -4, r - 8), (8, -2, r - 10), (-3, -9, r - 10)] {
        f.fill_ellipse(x + dx + 2, cy + dy - 2, rr.max(2), (rr - 2).max(1), l);
    }
    for k in 0..12 {
        let a = k as f32 * 2.39;
        let px = x + (a.cos() * (r as f32 - 4.0)) as i32;
        let py = cy - 4 + (a.sin() * (r as f32 - 6.0)) as i32;
        if (px + py) % 2 == 0 {
            f.set(px, py, hl);
        }
    }
    cut(&f, 2 * r + 20, h + 2)
}

fn birch() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    f.rect(x - 1, base - 34, 3, 35, rgb(232, 232, 224));
    for y in (base - 34..base).step_by(4) {
        f.hline(x - 1, x, y + (y % 3), rgb(48, 48, 56));
    }
    f.line(x + 1, base - 34, x + 1, base, rgb(184, 184, 176));
    let (m, l) = (rgb(120, 184, 72), rgb(176, 216, 96));
    f.fill_ellipse(x, base - 36, 9, 10, m);
    f.fill_ellipse(x + 2, base - 39, 5, 6, l);
    f.fill_ellipse(x - 5, base - 31, 5, 4, m);
    cut(&f, 26, 50)
}

fn fir(h: i32) -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    f.rect(x - 1, base - 4, 3, 5, BARK_D);
    let (g, gd, sn) = (rgb(32, 96, 80), rgb(16, 64, 56), rgb(232, 240, 248));
    for k in 0..h - 4 {
        let y = base - 4 - k;
        let half = ((h - 4 - k) * 6 / h).max(1) + if k % 5 == 0 { 1 } else { 0 };
        f.hline(x - half, x + half, y, g);
        f.hline(x - half, x - half / 2, y, gd);
        if k % 7 == 3 {
            f.hline(x + 1, x + half, y, sn);
        }
    }
    cut(&f, 22, h + 2)
}

fn bush(berries: bool) -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let (d, m, l) = (rgb(32, 104, 48), rgb(56, 144, 56), rgb(104, 184, 72));
    f.fill_ellipse(x - 5, base - 5, 7, 5, d);
    f.fill_ellipse(x + 5, base - 5, 7, 5, d);
    f.fill_ellipse(x, base - 8, 7, 6, m);
    f.fill_ellipse(x + 2, base - 10, 3, 2, l);
    f.fill_ellipse(x - 6, base - 6, 2, 2, l);
    if berries {
        for (dx, dy) in [(-6, -4), (3, -6), (6, -3), (-1, -10)] {
            f.set(x + dx, base + dy, rgb(216, 40, 72));
        }
    }
    f.hline(x - 11, x + 11, base, d);
    cut(&f, 26, 16)
}

fn fern() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let g = rgb(72, 160, 64);
    for (ex, ey) in [(-10, -6), (-6, -11), (0, -13), (6, -11), (10, -6)] {
        f.line(x, base, x + ex, base + ey, g);
        for k in 1..5 {
            let px = x + ex * k / 5;
            let py = base + ey * k / 5;
            f.set(px, py - 1, rgb(120, 200, 80));
            f.set(px + 1, py, rgb(48, 128, 56));
        }
    }
    cut(&f, 26, 16)
}

fn flowers(k: u32) -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let g = rgb(72, 160, 64);
    let cols = [rgb(64, 96, 232), rgb(248, 216, 48), rgb(232, 64, 96), rgb(248, 248, 248)];
    for i in 0..7 {
        let dx = (i * 5) % 17 - 8;
        let hh = 3 + (i * 7 + k as i32) % 5;
        f.line(x + dx, base, x + dx, base - hh, g);
        let c = cols[((i as u32 + k) % 4) as usize];
        f.set(x + dx, base - hh - 1, c);
        f.set(x + dx - 1, base - hh, c);
        f.set(x + dx + 1, base - hh, c);
    }
    f.hline(x - 9, x + 9, base, g);
    cut(&f, 22, 12)
}

fn boulder(big: bool) -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let (rx, ry) = if big { (12, 9) } else { (7, 5) };
    f.fill_ellipse(x, base - ry, rx, ry, rgb(120, 120, 136));
    f.fill_ellipse(x + 2, base - ry - 2, rx - 3, ry - 2, rgb(152, 152, 168));
    f.fill_ellipse(x + 3, base - ry - 3, rx / 3, ry / 3, rgb(200, 200, 208));
    f.hline(x - rx + 2, x + rx - 2, base - 1, rgb(88, 88, 104));
    if big {
        f.fill_ellipse(x - 6, base - ry - 3, 4, 2, rgb(88, 152, 64));
    }
    cut(&f, 2 * rx + 4, 2 * ry + 3)
}

fn fence() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let w = rgb(168, 120, 72);
    for px in [x - 11, x, x + 11] {
        f.rect(px - 1, base - 12, 3, 13, w);
        f.set(px - 1, base - 12, BARK_L);
    }
    f.rect(x - 13, base - 10, 27, 2, w);
    f.rect(x - 13, base - 5, 27, 2, w);
    f.hline(x - 13, x + 13, base - 10, rgb(200, 152, 96));
    cut(&f, 30, 14)
}

fn ribbon() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    f.line(x, base, x, base - 18, rgb(168, 120, 72));
    f.rect(x + 1, base - 18, 6, 3, rgb(248, 72, 168));
    f.rect(x + 3, base - 15, 4, 2, rgb(248, 72, 168));
    f.set(x + 6, base - 13, rgb(248, 72, 168));
    cut(&f, 12, 20)
}

fn board(l1: &str, l2: &str, bg: u32, fg: u32) -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let w = text_width(l1, false).max(text_width(l2, false)) + 8;
    let h = if l2.is_empty() { 11 } else { 19 };
    let top = base - 12 - h;
    f.rect(x - w / 2 + 3, top + h - 1, 3, 13, BARK);
    f.rect(x + w / 2 - 6, top + h - 1, 3, 13, BARK);
    f.rect(x - w / 2, top, w, h, bg);
    f.hline(x - w / 2, x + w / 2 - 1, top, mix(bg, 0xFFFFFF, 0.3));
    f.hline(x - w / 2, x + w / 2 - 1, top + h - 1, shade(bg, 2, 3));
    f.text(x - text_width(l1, false) / 2, top + 2, l1, fg);
    if !l2.is_empty() {
        f.text(x - text_width(l2, false) / 2, top + 10, l2, fg);
    }
    cut(&f, w + 2, h + 14)
}

fn deer_sign() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    f.rect(x - 1, base - 16, 3, 17, rgb(160, 160, 176));
    let cy = base - 24;
    for d in 0..10 {
        f.hline(x - d, x + d, cy - 10 + d, rgb(248, 200, 32));
        f.hline(x - d, x + d, cy + 10 - d, rgb(248, 200, 32));
    }
    let k = OUTLINE;
    // a leaping deer, very small
    f.hline(x - 4, x + 2, cy, k);
    f.hline(x - 3, x + 2, cy + 1, k);
    f.line(x + 2, cy, x + 4, cy - 3, k);
    f.set(x + 5, cy - 3, k);
    f.set(x + 4, cy - 5, k);
    f.set(x + 3, cy - 5, k);
    f.line(x - 4, cy + 1, x - 6, cy + 4, k);
    f.line(x + 2, cy + 1, x + 4, cy + 4, k);
    f.set(x - 5, cy - 1, k);
    cut(&f, 24, 38)
}

fn km(n: u8) -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    f.rect(x - 1, base - 10, 3, 11, rgb(232, 232, 232));
    let s = format!("{} KM", n);
    let w = text_width(&s, false) + 4;
    f.rect(x - w / 2, base - 19, w, 9, rgb(232, 48, 48));
    f.text(x - w / 2 + 2, base - 18, &s, rgb(248, 248, 248));
    cut(&f, w + 2, 21)
}

fn arch(text: &str, c: u32) -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let half = 34;
    let top = base - 54;
    for px in [x - half, x + half - 6] {
        f.rect(px, top, 7, 55, c);
        f.rect(px + 4, top, 3, 55, shade(c, 3, 4));
        f.rect(px + 1, top, 1, 55, mix(c, 0xFFFFFF, 0.4));
    }
    f.rect(x - half, top, 2 * half + 1, 14, c);
    f.hline(x - half, x + half, top, mix(c, 0xFFFFFF, 0.4));
    f.rect(x - half, top + 12, 2 * half + 1, 2, shade(c, 3, 4));
    // checkered trim
    for k in 0..(2 * half / 3) {
        let cc = if k % 2 == 0 { rgb(248, 248, 248) } else { OUTLINE };
        f.rect(x - half + k * 3, top + 14, 3, 2, cc);
    }
    let tw = text_width(text, true);
    f.text_sh(x - tw / 2, top + 3, text, rgb(248, 248, 248), shade(c, 1, 2), true);
    cut(&f, 2 * half + 4, 57)
}

fn summit_cross() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let w = rgb(160, 112, 64);
    // stones at the foot
    for (dx, r) in [(-6, 5), (5, 5), (0, 6)] {
        f.fill_ellipse(x + dx, base - 3, r, 3, rgb(136, 136, 152));
    }
    f.rect(x - 2, base - 50, 5, 46, w);
    f.rect(x - 12, base - 40, 25, 5, w);
    f.line(x + 2, base - 50, x + 2, base - 5, rgb(120, 80, 48));
    f.hline(x - 12, x + 12, base - 40, rgb(200, 152, 96));
    // summit book box and prayer flags
    f.rect(x + 3, base - 26, 6, 5, rgb(200, 48, 48));
    let cols = [rgb(64, 96, 232), rgb(248, 248, 248), rgb(232, 64, 64), rgb(64, 184, 96), rgb(248, 216, 48)];
    for k in 0..7 {
        let px = x + 14 + k * 4;
        let py = base - 38 + k * 4;
        f.rect(px, py, 3, 4, cols[k as usize % 5]);
    }
    f.line(x + 12, base - 38, x + 42, base - 8, rgb(200, 200, 200));
    cut(&f, 62, 54)
}

fn cairn() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let c = [rgb(136, 136, 152), rgb(168, 168, 184), rgb(120, 120, 136)];
    for (k, (w, h)) in [(7, 3), (6, 3), (5, 2), (4, 2), (2, 2)].iter().enumerate() {
        let y = base - 1 - k as i32 * 3 - h / 2;
        f.fill_ellipse(x + (k as i32 % 2), y, *w, *h, c[k % 3]);
        f.hline(x - w / 2, x + w / 2, y - h / 2, rgb(200, 200, 216));
    }
    cut(&f, 18, 18)
}

fn snow_pole() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    for y in base - 24..=base {
        let c = if ((base - y) / 4) % 2 == 0 { rgb(248, 128, 32) } else { OUTLINE };
        f.rect(x, y, 2, 1, c);
    }
    cut(&f, 6, 26)
}

fn house(k: u32) -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let wall = if k == 0 { rgb(184, 112, 64) } else { rgb(232, 224, 200) };
    f.rect(x - 18, base - 14, 36, 15, rgb(232, 232, 224));
    f.rect(x - 18, base - 30, 36, 17, wall);
    for wx in [x - 12, x + 4] {
        f.rect(wx, base - 25, 8, 7, rgb(248, 232, 160));
        f.rect(wx - 3, base - 25, 2, 7, rgb(200, 48, 48));
        f.rect(wx + 9, base - 25, 2, 7, rgb(200, 48, 48));
        f.rect(wx, base - 18, 8, 2, rgb(232, 64, 96));
        f.hline(wx, wx + 7, base - 22, BARK_D);
    }
    f.rect(x - 4, base - 12, 8, 13, BARK);
    let roof = rgb(112, 64, 48);
    f.fill_poly(&[(x - 24, base - 30), (x, base - 46), (x + 24, base - 30)], roof);
    f.hline(x - 24, x + 24, base - 30, shade(roof, 2, 3));
    f.line(x, base - 46, x + 24, base - 30, rgb(160, 96, 64));
    cut(&f, 52, 48)
}

fn church() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let wall = rgb(240, 240, 232);
    f.rect(x - 16, base - 26, 24, 27, wall);
    f.fill_poly(&[(x - 18, base - 26), (x - 4, base - 38), (x + 10, base - 26)], rgb(152, 72, 56));
    f.rect(x + 8, base - 50, 10, 51, wall);
    f.rect(x + 10, base - 44, 6, 6, rgb(64, 48, 40));
    f.fill_ellipse(x + 13, base - 41, 2, 2, rgb(248, 216, 48));
    f.fill_ellipse(x + 13, base - 54, 6, 5, rgb(64, 136, 88));
    f.rect(x + 12, base - 64, 2, 6, rgb(64, 136, 88));
    f.fill_ellipse(x + 13, base - 59, 3, 3, rgb(64, 136, 88));
    f.rect(x + 12, base - 68, 2, 5, rgb(248, 216, 48));
    f.rect(x - 9, base - 18, 6, 10, rgb(96, 128, 200));
    f.rect(x - 4, base - 9, 6, 10, BARK);
    cut(&f, 44, 70)
}

fn hut() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    f.rect(x - 14, base - 16, 28, 17, rgb(136, 88, 56));
    for y in (base - 16..base).step_by(3) {
        f.hline(x - 14, x + 13, y, rgb(104, 64, 40));
    }
    f.rect(x - 4, base - 10, 7, 11, rgb(64, 40, 32));
    f.rect(x + 6, base - 12, 5, 4, rgb(248, 232, 160));
    f.fill_poly(&[(x - 18, base - 16), (x, base - 28), (x + 18, base - 16)], rgb(96, 96, 112));
    f.hline(x - 18, x + 18, base - 16, rgb(72, 72, 88));
    f.text(x - 11, base - 25, "HUT", rgb(248, 248, 248));
    cut(&f, 40, 30)
}

fn aid_table() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    // banner on two poles
    f.rect(x - 16, base - 34, 2, 35, rgb(160, 160, 176));
    f.rect(x + 15, base - 34, 2, 35, rgb(160, 160, 176));
    f.rect(x - 16, base - 34, 33, 9, rgb(48, 120, 216));
    f.text(x - 8, base - 33, "AID", rgb(248, 248, 248));
    // table with a white cloth, cups and a jug
    f.rect(x - 14, base - 12, 28, 3, rgb(248, 248, 248));
    f.rect(x - 14, base - 9, 28, 3, rgb(216, 216, 232));
    for k in 0..5 {
        f.rect(x - 12 + k * 5, base - 15, 3, 3, if k % 2 == 0 { rgb(232, 64, 64) } else { rgb(64, 160, 232) });
    }
    f.rect(x + 8, base - 19, 4, 7, rgb(200, 232, 248));
    f.rect(x - 13, base - 6, 2, 7, BARK);
    f.rect(x + 11, base - 6, 2, 7, BARK);
    cut(&f, 36, 36)
}

fn hole() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    f.fill_ellipse(x, base - 2, 7, 3, rgb(152, 104, 64));
    f.fill_ellipse(x, base - 2, 4, 2, rgb(40, 24, 16));
    cut(&f, 18, 7)
}

fn root(k: u32) -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let (c, d) = (rgb(136, 88, 56), rgb(96, 56, 32));
    // across the trail, arching out of the dirt
    let dx = if k == 0 { 3 } else { -3 };
    f.thick_line(x - 6, base - 10, x - 2 + dx, base - 14, 3, c);
    f.thick_line(x - 2 + dx, base - 14, x + 3, base - 7, 3, c);
    f.thick_line(x + 3, base - 7, x + 6, base - 1, 3, c);
    f.line(x - 6, base - 9, x - 2 + dx, base - 13, d);
    f.line(x - 2 + dx, base - 13, x + 3, base - 6, d);
    f.set(x - 1 + dx, base - 16, rgb(176, 120, 72));
    cut(&f, 18, 18)
}

fn rock(k: u32, snow: bool) -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let (rx, ry) = if k == 0 { (7, 6) } else { (6, 7) };
    f.fill_ellipse(x, base - ry - 1, rx, ry, rgb(112, 112, 128));
    f.fill_ellipse(x + 1, base - ry - 2, rx - 2, ry - 2, rgb(152, 152, 168));
    f.fill_ellipse(x + 2, base - ry - 3, 2, 2, rgb(208, 208, 216));
    if snow {
        f.fill_ellipse(x, base - 2 * ry, rx - 1, 2, rgb(248, 248, 248));
        f.hline(x - rx + 2, x + rx - 3, base - 2 * ry + 2, rgb(200, 224, 248));
    }
    cut(&f, 2 * rx + 4, 2 * ry + 4)
}

fn log() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    // lying across the trail: the body recedes to the back, the round end faces us
    f.fill_poly(&[(x - 6, base - 18), (x + 2, base - 18), (x + 7, base - 6), (x - 3, base - 6)], BARK);
    f.line(x - 6, base - 18, x - 3, base - 6, BARK_D);
    f.line(x + 2, base - 18, x + 7, base - 7, BARK_L);
    f.fill_ellipse(x + 2, base - 5, 6, 5, rgb(200, 152, 96));
    f.circle_outline(x + 2, base - 5, 6, BARK_D);
    f.circle_outline(x + 2, base - 5, 3, rgb(168, 120, 72));
    f.set(x + 2, base - 5, rgb(144, 96, 56));
    f.fill_ellipse(x - 1, base - 17, 2, 1, rgb(88, 160, 64));
    cut(&f, 20, 22)
}

fn mushroom(k: u32) -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let (cap, dot) = if k == 0 { (rgb(232, 48, 48), rgb(248, 248, 248)) } else { (rgb(216, 136, 48), rgb(248, 216, 160)) };
    f.rect(x - 1, base - 5, 3, 6, rgb(240, 232, 216));
    f.fill_ellipse(x, base - 6, 5, 3, cap);
    f.hline(x - 4, x + 4, base - 4, shade(cap, 2, 3));
    f.set(x - 2, base - 8, dot);
    f.set(x + 2, base - 7, dot);
    f.set(x, base - 6, dot);
    // a tiny friend
    f.rect(x + 6, base - 2, 1, 3, rgb(240, 232, 216));
    f.fill_ellipse(x + 6, base - 3, 2, 1, cap);
    cut(&f, 18, 12)
}

fn branch() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    // a crooked tree leaning over the trail; the low branch is at x (base point)
    let tx = x - 16;
    f.thick_line(tx, base - 6, tx + 2, base - 46, 4, BARK);
    f.line(tx - 1, base - 6, tx + 1, base - 46, BARK_D);
    f.thick_line(tx + 1, base - 30, x - 2, base - 22, 3, BARK);
    f.thick_line(x - 2, base - 22, x + 10, base - 20, 2, BARK);
    f.line(x - 2, base - 21, x + 10, base - 19, BARK_D);
    let (m, l) = (rgb(56, 144, 56), rgb(104, 184, 72));
    for (dx, dy) in [(0, -24), (5, -23), (10, -22), (3, -19), (8, -18), (12, -19)] {
        f.fill_ellipse(x + dx, base + dy, 3, 2, m);
        f.set(x + dx, base + dy - 1, l);
    }
    f.fill_ellipse(tx + 2, base - 48, 10, 7, rgb(32, 104, 48));
    f.fill_ellipse(tx + 4, base - 50, 6, 4, m);
    // base point is the trail; the trunk stands behind it
    let mut s = cut(&f, 60, 58);
    s.ax += 0;
    s
}

fn trunk() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    // fallen tree propped on two stumps at duck height
    for sx in [x - 13, x + 12] {
        f.rect(sx - 2, base - 15, 5, 16, BARK_D);
        f.hline(sx - 2, sx + 2, base - 15, BARK_L);
    }
    f.rect(x - 18, base - 23, 36, 7, BARK);
    f.hline(x - 18, x + 17, base - 23, BARK_L);
    f.hline(x - 18, x + 17, base - 17, BARK_D);
    for k in 0..5 {
        f.set(x - 14 + k * 7, base - 20, BARK_D);
    }
    f.fill_ellipse(x + 18, base - 20, 3, 4, rgb(200, 152, 96));
    f.set(x + 18, base - 20, BARK_D);
    f.fill_ellipse(x - 8, base - 24, 3, 1, rgb(88, 160, 64));
    cut(&f, 44, 26)
}

fn stone() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    f.fill_ellipse(x, base - 2, 5, 2, rgb(136, 136, 152));
    f.hline(x - 3, x + 2, base - 3, rgb(200, 200, 216));
    cut(&f, 14, 6)
}

fn podium() -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let c = [rgb(248, 200, 48), rgb(200, 200, 216), rgb(216, 136, 72)];
    for (k, (dx, h)) in [(0, 30), (-30, 20), (30, 12)].iter().enumerate() {
        f.rect(x + dx - 14, base - h, 29, *h + 1, c[k]);
        f.hline(x + dx - 14, x + dx + 14, base - h, mix(c[k], 0xFFFFFF, 0.5));
        f.rect(x + dx + 10, base - h, 4, *h + 1, shade(c[k], 3, 4));
        let s = format!("{}", k + 1);
        f.text_sh(x + dx - 3, base - h + 4, &s, rgb(248, 248, 248), shade(c[k], 1, 2), true);
    }
    cut(&f, 92, 32)
}

fn buried(k: u32) -> CSprite {
    let mut f = scratch();
    let (x, base) = (128, 200);
    let dark = rgb(72, 48, 32);
    match k {
        0 => {
            f.fill_ellipse(x, base - 3, 5, 3, rgb(152, 144, 136));
            f.hline(x - 3, x + 1, base - 5, rgb(200, 192, 184));
            f.hline(x - 4, x + 4, base, dark);
        }
        1 => {
            // a bone
            f.hline(x - 5, x + 5, base - 2, rgb(240, 232, 208));
            for ex in [x - 6, x + 6] {
                f.rect(ex - 1, base - 4, 2, 2, rgb(240, 232, 208));
                f.rect(ex - 1, base - 1, 2, 2, rgb(240, 232, 208));
            }
            f.hline(x - 5, x + 5, base - 1, rgb(200, 192, 168));
        }
        2 => {
            // an ammonite
            f.fill_ellipse(x, base - 4, 5, 4, rgb(176, 160, 136));
            f.circle_outline(x, base - 4, 3, rgb(120, 104, 88));
            f.set(x, base - 4, rgb(120, 104, 88));
            f.set(x + 1, base - 3, rgb(120, 104, 88));
        }
        3 => {
            // a lost shoe (Pete's?)
            f.rect(x - 5, base - 3, 10, 3, rgb(232, 48, 120));
            f.rect(x - 5, base - 6, 4, 3, rgb(232, 48, 120));
            f.hline(x - 5, x + 4, base, rgb(248, 248, 248));
            f.set(x - 3, base - 5, rgb(248, 248, 248));
        }
        4 => {
            // roots
            f.line(x - 6, base - 8, x - 1, base - 3, rgb(104, 64, 40));
            f.line(x - 1, base - 3, x + 4, base - 6, rgb(104, 64, 40));
            f.line(x - 1, base - 3, x, base + 1, rgb(104, 64, 40));
            f.line(x + 4, base - 6, x + 7, base - 2, rgb(104, 64, 40));
        }
        _ => {
            // an ice crystal
            f.line(x, base - 7, x, base, rgb(200, 232, 248));
            f.line(x - 3, base - 5, x + 3, base - 2, rgb(200, 232, 248));
            f.line(x - 3, base - 2, x + 3, base - 5, rgb(200, 232, 248));
            f.set(x, base - 4, WHITE_ICE);
        }
    }
    let mut s = CSprite::cut(&f, x - 8, base - 9, 17, 11, false);
    s.ay = s.h - 2;
    s
}

const WHITE_ICE: u32 = rgb(248, 248, 248);

impl PropArt {
    pub fn new() -> PropArt {
        PropArt {
            pine: [pine(44, false), pine(58, true)],
            leafy: [leafy(true), leafy(false)],
            birch: birch(),
            fir: [fir(40), fir(54)],
            bush: [bush(false), bush(true)],
            fern: fern(),
            flowers: [flowers(0), flowers(1), flowers(2)],
            boulder: [boulder(true), boulder(false)],
            fence: fence(),
            ribbon: ribbon(),
            signs: SIGNS.iter().map(|(a, b)| board(a, b, rgb(176, 120, 72), rgb(64, 32, 16))).collect(),
            deer_sign: deer_sign(),
            km: (1..=6).map(|k| km(k * 2)).collect(),
            start_arch: arch("START", rgb(48, 120, 216)),
            finish_arch: arch("FINISH", rgb(216, 48, 64)),
            summit: summit_cross(),
            cairn: cairn(),
            snow_pole: snow_pole(),
            house: [house(0), house(1)],
            church: church(),
            hut: hut(),
            aid_table: aid_table(),
            hole: hole(),
            root: [root(0), root(1)],
            rock: [rock(0, false), rock(1, false)],
            log: log(),
            mushroom: [mushroom(0), mushroom(1)],
            branch: branch(),
            trunk: trunk(),
            snow_rock: rock(0, true),
            stone: stone(),
            podium: podium(),
            buried: [buried(0), buried(1), buried(2), buried(3), buried(4), buried(5)],
        }
    }
}

impl Default for PropArt {
    fn default() -> Self {
        Self::new()
    }
}

/// The course profile for the HUD minimap: a silhouette `w` x `h`.
pub fn profile(course: &super::course::Course, w: i32, h: i32) -> Vec<u32> {
    let mut px = vec![KEY; (w * h) as usize];
    let x0 = super::course::START_X;
    let span = course.finish - x0;
    let (lo, hi) = (course.min_h, course.max_h.max(course.min_h + 1.0));
    for x in 0..w {
        let wx = x0 + span * x as f32 / (w - 1) as f32;
        let a = course.height(wx);
        let top = h - 1 - ((a - lo) / (hi - lo) * (h - 2) as f32) as i32;
        for y in top..h {
            let c = if y == top { rgb(160, 224, 120) } else if (x + y) % 2 == 0 { rgb(56, 104, 72) } else { rgb(48, 88, 64) };
            px[(y * w + x) as usize] = c;
        }
    }
    px
}
