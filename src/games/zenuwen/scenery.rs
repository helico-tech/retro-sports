//! Stage backdrops. The canal houses (or Centraal Station) are drawn once
//! per stage into a strip as wide as the street; the sky, a far skyline,
//! the brick street, the quay and the canal (with the houses mirrored in
//! it) are drawn per frame from the camera.

use super::data::*;
use crate::engine::gfx::*;
use crate::engine::rng::Rng;
use crate::engine::ui::OUTLINE;

/// Screen rows: the house strip starts at `HY0`; the street (world y 0) at
/// `STREET`; the quay edge (world y `DEPTH`) at `QUAY`; the water at `WATER`.
pub const HY0: i32 = 28;
pub const HH: i32 = 98;
pub const STREET: i32 = 126;
pub const QUAY: i32 = 186;
pub const WATER: i32 = 194;
/// Row inside the strip where the house fronts meet the sidewalk.
const GROUND: i32 = 90;
const FAR_W: i32 = 512;
const FAR_H: i32 = 46;

pub struct Scenery {
    pub venue: Venue,
    pub len: i32,
    /// len x HH, 0 = transparent (sky)
    pub houses: Vec<u32>,
    /// FAR_W x FAR_H skyline silhouettes, wrapped
    pub far: Vec<u32>,
    pub sky: Vec<u32>,
    /// houseboats along the quay: x, width, color
    pub houseboats: Vec<(i32, i32, u32)>,
    pub water: u32,
}

const BRICKS: [u32; 6] = [rgb(136, 56, 48), rgb(104, 48, 40), rgb(160, 80, 56), rgb(88, 40, 40), rgb(120, 72, 56), rgb(72, 40, 40)];
const PLASTER: [u32; 5] = [rgb(232, 224, 200), rgb(200, 208, 200), rgb(56, 64, 56), rgb(64, 72, 96), rgb(216, 192, 152)];

fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

struct Strip<'a> {
    px: &'a mut Vec<u32>,
    w: i32,
}

impl Strip<'_> {
    fn set(&mut self, x: i32, y: i32, c: u32) {
        if x >= 0 && y >= 0 && x < self.w && y < HH {
            self.px[(y * self.w + x) as usize] = c;
        }
    }
    fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: u32) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }
    fn get(&self, x: i32, y: i32) -> u32 {
        if x >= 0 && y >= 0 && x < self.w && y < HH {
            self.px[(y * self.w + x) as usize]
        } else {
            0
        }
    }
}

impl Scenery {
    pub fn new(venue: Venue, len: f32, seed: u64) -> Self {
        let len = len as i32;
        let mut rng = Rng(seed | 1);
        let mut houses = vec![0u32; (len * HH) as usize];
        {
            let mut s = Strip { px: &mut houses, w: len };
            let (sx0, sx1) = if venue == Venue::Station { (len / 2 - 230, len / 2 + 230) } else { (i32::MAX, i32::MAX) };
            let mut x = -rng.int(20);
            while x < len {
                if x >= sx0 && x < sx1 {
                    station(&mut s, sx0, sx1);
                    x = sx1;
                    continue;
                }
                let w = 26 + rng.int(14);
                let w = if x + w > sx0 && x < sx0 { sx0 - x } else { w };
                house(&mut s, x, w, &mut rng, venue);
                x += w;
            }
            sidewalk(&mut s, len);
            if venue == Venue::Koningsdag {
                bunting(&mut s, len, &mut rng);
            }
        }
        let far = skyline(venue, &mut rng);
        let sky = match venue {
            Venue::Gracht => vec![rgb(104, 152, 224), rgb(136, 176, 232), rgb(168, 200, 240), rgb(208, 216, 232), rgb(240, 216, 192)],
            Venue::Station => vec![rgb(72, 128, 208), rgb(96, 152, 224), rgb(128, 176, 232), rgb(168, 200, 240), rgb(200, 216, 240)],
            Venue::Koningsdag => vec![rgb(56, 136, 232), rgb(88, 160, 240), rgb(120, 184, 248), rgb(160, 208, 248), rgb(200, 224, 248)],
        };
        let mut houseboats = Vec::new();
        let n = if venue == Venue::Station { 1 } else { 2 };
        for k in 0..n {
            let x = len * (k * 2 + 1) / (n * 2) + rng.int(120) - 60;
            houseboats.push((x, 56 + rng.int(16), [rgb(64, 120, 88), rgb(152, 72, 56), rgb(64, 88, 152)][rng.int(3) as usize]));
        }
        Scenery { venue, len, houses, far, sky, houseboats, water: rgb(40, 88, 96) }
    }

    /// Sky, clouds, far skyline and the house strip.
    pub fn draw_back(&self, f: &mut Frame, cam: f32, t: u32, clip: (i32, i32)) {
        let (y0, y1) = (clip.0.max(0), clip.1.min(STREET));
        // sky bands
        let top = 18;
        for y in y0..y1 {
            let k = ((y - top).max(0) * self.sky.len() as i32 / (STREET - 40 - top)).clamp(0, self.sky.len() as i32 - 1) as usize;
            let next = (k + 1).min(self.sky.len() - 1);
            let sub = ((y - top).max(0) * self.sky.len() as i32 * 4 / (STREET - 40 - top)) % 4;
            let row = &mut f.px[(y * W) as usize..((y + 1) * W) as usize];
            for (x, p) in row.iter_mut().enumerate() {
                *p = if sub == 3 && (x as i32 + y) & 1 == 0 { self.sky[next] } else { self.sky[k] };
            }
        }
        // clouds drift slowly
        for k in 0..4 {
            let speed = 0.08 + 0.03 * k as f32;
            let w = 380.0;
            let cx = ((k as f32 * 97.0 + t as f32 * speed * 0.2 - cam * 0.15) % w + w) % w - 60.0;
            let cy = 40 + (k * 11) % 26;
            cloud(f, cx as i32, cy, 9 + k % 2 * 3, clip);
        }
        // far skyline, parallax
        let fo = (cam * 0.35) as i32;
        for y in 0..FAR_H {
            let sy = STREET - 40 - FAR_H + y + 10;
            if sy < y0 || sy >= y1 {
                continue;
            }
            for x in 0..W {
                let c = self.far[(y * FAR_W + (x + fo).rem_euclid(FAR_W)) as usize];
                if c != 0 {
                    f.px[(sy * W + x) as usize] = c;
                }
            }
        }
        // the houses
        let ci = cam.floor() as i32;
        for y in 0..HH {
            let sy = HY0 + y;
            if sy < y0 || sy >= y1 {
                continue;
            }
            for x in 0..W {
                let wx = x + ci;
                if wx < 0 || wx >= self.len {
                    continue;
                }
                let c = self.houses[(y * self.len + wx) as usize];
                if c != 0 {
                    f.px[(sy * W + x) as usize] = c;
                }
            }
        }
    }

    /// The brick street from the house fronts to the quay edge.
    pub fn draw_street(&self, f: &mut Frame, cam: f32, clip: (i32, i32)) {
        let ci = cam.floor() as i32;
        let (base, mortar, dark) = match self.venue {
            Venue::Station => (rgb(128, 120, 120), rgb(96, 88, 96), rgb(112, 104, 112)),
            _ => (rgb(144, 88, 72), rgb(104, 64, 56), rgb(128, 72, 64)),
        };
        for y in STREET.max(clip.0)..QUAY.min(clip.1) {
            let r = y - STREET;
            let row = r / 3;
            let off = if row % 2 == 0 { 0 } else { 4 };
            for x in 0..W {
                let wx = x + ci;
                let c = if r % 3 == 2 || (wx + off).rem_euclid(8) == 0 {
                    mortar
                } else if hash((wx + off).div_euclid(8), row).is_multiple_of(5) {
                    dark
                } else {
                    base
                };
                f.px[(y * W + x) as usize] = c;
            }
        }
        // a gutter along the sidewalk, and the curb at the quay
        if (clip.0..clip.1).contains(&(STREET + 1)) {
            f.hline(0, W - 1, STREET, rgb(72, 64, 72));
            f.hline(0, W - 1, STREET + 1, rgb(96, 88, 96));
        }
    }

    /// The quay wall, the canal with reflections, and the houseboats.
    pub fn draw_water(&self, f: &mut Frame, cam: f32, t: u32, clip: (i32, i32)) {
        let ci = cam.floor() as i32;
        // coping stones
        for y in QUAY.max(clip.0)..(QUAY + 3).min(clip.1) {
            for x in 0..W {
                let wx = x + ci;
                let c = if wx.rem_euclid(18) == 0 { rgb(136, 136, 144) } else if y == QUAY { rgb(216, 216, 216) } else { rgb(184, 184, 192) };
                f.px[(y * W + x) as usize] = c;
            }
        }
        // the quay wall: dark brick with mooring rings
        for y in (QUAY + 3).max(clip.0)..WATER.min(clip.1) {
            let r = y - QUAY - 3;
            for x in 0..W {
                let wx = x + ci;
                let off = if r % 2 == 0 { 0 } else { 3 };
                let c = if (wx + off).rem_euclid(6) == 0 { rgb(56, 32, 32) } else { rgb(88, 48, 40) };
                f.px[(y * W + x) as usize] = c;
            }
        }
        for k in 0..6 {
            let wx = k * 96 + 40;
            let sx = (wx - ci).rem_euclid(576) - 40;
            if (clip.0..clip.1).contains(&(QUAY + 5)) {
                f.circle_outline(sx, QUAY + 6, 2, rgb(64, 64, 72));
            }
        }
        // water, with the houses mirrored and wobbling
        for y in WATER.max(clip.0)..H.min(clip.1) {
            let r = y - WATER;
            let src = GROUND - 2 - r * 2;
            let wob = ((y as f32 * 0.9 + t as f32 * 0.06).sin() * 1.6) as i32;
            for x in 0..W {
                let wx = x + ci + wob;
                let mut c = self.water;
                if src >= 0 && wx >= 0 && wx < self.len {
                    let h = self.houses[(src * self.len + wx) as usize];
                    if h != 0 {
                        c = mix(h, self.water, 0.6);
                    } else {
                        c = mix(self.sky[self.sky.len() - 2], self.water, 0.55);
                    }
                }
                if r < 2 {
                    c = shade(c, 3, 4);
                }
                f.px[(y * W + x) as usize] = c;
            }
        }
        // ripples
        for k in 0..18 {
            let wx = k * 61 + (t as i32 / 6) % 40;
            let sx = (wx - ci).rem_euclid(W + 40) - 20;
            let sy = WATER + 4 + (k * 7) % 26;
            if (clip.0..clip.1).contains(&sy) {
                let wl = 3 + k % 4;
                f.hline(sx, sx + wl, sy, rgb(120, 184, 192));
            }
        }
    }

    /// Houseboats moored along the quay (drawn behind the boats that move).
    pub fn draw_houseboats(&self, f: &mut Frame, cam: f32, t: u32, clip: (i32, i32), cat: &dyn Fn(&mut Frame, i32, i32)) {
        for &(wx, w, col) in &self.houseboats {
            let sx = wx - cam as i32;
            if sx + w < -10 || sx - 10 > W {
                continue;
            }
            let bob = ((t / 40 + wx as u32) % 2) as i32;
            let wl = WATER + 12 + bob;
            if wl - 26 > clip.1 || wl + 6 < clip.0 {
                continue;
            }
            // hull
            f.rect(sx - 1, wl - 5, w + 2, 9, OUTLINE);
            f.rect(sx, wl - 4, w, 7, rgb(40, 40, 48));
            f.hline(sx, sx + w - 1, wl - 4, rgb(88, 88, 96));
            // cabin with windows and plants on the roof
            let cx = sx + 4;
            let cw = w - 8;
            f.rect(cx - 1, wl - 20, cw + 2, 17, OUTLINE);
            f.rect(cx, wl - 19, cw, 15, col);
            f.rect(cx, wl - 19, cw, 2, shade(col, 3, 4));
            let mut wx2 = cx + 3;
            while wx2 + 6 < cx + cw {
                f.rect(wx2, wl - 15, 6, 6, rgb(248, 232, 168));
                f.rect(wx2 + 1, wl - 14, 4, 4, rgb(48, 64, 96));
                wx2 += 10;
            }
            for p in 0..cw / 7 {
                let px = cx + 2 + p * 7;
                f.rect(px, wl - 23, 4, 3, rgb(200, 104, 72));
                f.rect(px, wl - 26, 4, 3, if p % 2 == 0 { rgb(72, 168, 72) } else { rgb(232, 72, 120) });
            }
            if wx % 2 == 0 {
                cat(f, cx + cw - 14, wl - 27);
            }
            // reflection of the hull
            f.dither_rect(sx, wl + 4, w, 2, rgb(24, 56, 64), self.water);
        }
    }
}

fn cloud(f: &mut Frame, x: i32, y: i32, r: i32, clip: (i32, i32)) {
    let parts = [(0, 0, r), (r, -r / 2, r * 3 / 4), (-r, 1, r * 2 / 3), (r * 2, 2, r / 2)];
    let inside = |yy: i32| yy >= clip.0 && yy < clip.1.min(STREET);
    for &(dx, dy, rr) in &parts {
        for yy in -rr..=rr / 2 {
            let sy = y + dy + yy;
            if !inside(sy) {
                continue;
            }
            let w = ((rr * rr - yy * yy).max(0) as f32).sqrt() as i32 + rr / 3;
            for xx in -w..=w {
                let c = if yy > rr / 4 { rgb(208, 216, 232) } else { rgb(248, 248, 248) };
                f.set(x + dx + xx, sy, c);
            }
        }
    }
}

/// One canal house: body, windows, door with stoop, and a gable on top.
fn house(s: &mut Strip, x: i32, w: i32, rng: &mut Rng, venue: Venue) {
    if w < 10 {
        return;
    }
    let h = 48 + rng.int(22);
    let top = GROUND - h;
    let brick = rng.int(3) > 0;
    let base = if brick { BRICKS[rng.int(BRICKS.len() as i32) as usize] } else { PLASTER[rng.int(PLASTER.len() as i32) as usize] };
    let dark = shade(base, 3, 4);
    let trim = if brick && rng.int(3) == 0 { rgb(232, 224, 200) } else { rgb(248, 248, 240) };
    for yy in top..GROUND {
        for xx in x..x + w {
            let c = if brick && (yy % 3 == 0) && hash(xx / 5, yy).is_multiple_of(3) { dark } else { base };
            s.set(xx, yy, c);
        }
    }
    // gable
    let gable = rng.int(5);
    let cx = x + w / 2;
    let gh = 12 + rng.int(6);
    match gable {
        0 => {
            // step gable
            let steps = 3;
            for k in 0..steps {
                let inset = (k + 1) * w / 8;
                s.rect(x + inset, top - (k + 1) * gh / steps, w - 2 * inset, gh / steps + 1, base);
                s.rect(x + inset, top - (k + 1) * gh / steps, w - 2 * inset, 1, trim);
            }
        }
        1 => {
            // neck gable with curly shoulders
            let nw = w * 9 / 20;
            s.rect(cx - nw / 2, top - gh, nw, gh, base);
            for k in 0..gh / 2 {
                s.set(cx - nw / 2 - k / 2 - 1, top - k, trim);
                s.set(cx + nw / 2 + k / 2, top - k, trim);
            }
            s.rect(cx - nw / 2 - 1, top - gh - 2, nw + 2, 2, trim);
            s.rect(cx - 1, top - gh - 4, 3, 2, trim);
        }
        2 => {
            // bell gable
            let bw = w * 3 / 5;
            for yy in 0..gh {
                let k = 1.0 - (yy as f32 / gh as f32).powi(2);
                let half = (bw as f32 / 2.0 * k.sqrt()) as i32;
                for xx in cx - half..=cx + half {
                    s.set(xx, top - yy, base);
                }
                s.set(cx - half - 1, top - yy, trim);
                s.set(cx + half + 1, top - yy, trim);
            }
            s.rect(cx - 2, top - gh - 1, 5, 2, trim);
        }
        3 => {
            // spout gable: a point
            for yy in 0..gh {
                let half = w / 2 - yy * w / 2 / gh;
                for xx in cx - half..=cx + half {
                    s.set(xx, top - yy, base);
                }
                s.set(cx - half, top - yy, trim);
                s.set(cx + half, top - yy, trim);
            }
        }
        _ => {
            // flat with a cornice
            s.rect(x, top - 3, w, 3, trim);
            s.rect(x + 1, top - 4, w - 2, 1, shade(trim, 3, 4));
        }
    }
    // the hoist beam
    if gable < 4 {
        s.rect(cx - 1, top - gh + 4, 3, 2, rgb(40, 32, 32));
    } else {
        s.rect(cx - 1, top + 1, 3, 2, rgb(40, 32, 32));
    }
    // windows, floor by floor
    let n = if w > 34 { 3 } else { 2 };
    let ww = 5;
    let gap = (w - n * ww) / (n + 1);
    let mut fy = top + 6;
    let glass = if venue == Venue::Gracht && rng.int(4) == 0 { rgb(248, 224, 136) } else { rgb(40, 56, 88) };
    while fy + 10 < GROUND - 16 {
        for k in 0..n {
            let wx = x + gap + k * (ww + gap);
            s.rect(wx - 1, fy - 1, ww + 2, 11, trim);
            s.rect(wx, fy, ww, 9, glass);
            s.rect(wx, fy + 4, ww, 1, trim);
            s.set(wx + 2, fy + 1, rgb(120, 152, 200));
            if venue == Venue::Koningsdag && hash(wx, fy).is_multiple_of(9) {
                // an orange flag out of the window
                for d in 0..6 {
                    s.set(wx + ww + d, fy - d / 2, rgb(64, 48, 40));
                }
                s.rect(wx + ww + 3, fy - 2, 7, 5, rgb(248, 136, 24));
                s.rect(wx + ww + 3, fy + 1, 7, 1, rgb(192, 80, 16));
            }
        }
        fy += 15;
    }
    // ground floor: door and a window, stoop stairs
    let door_left = rng.int(2) == 0;
    let dx = if door_left { x + 3 } else { x + w - 10 };
    s.rect(dx - 1, GROUND - 15, 8, 15, trim);
    let door = [rgb(32, 72, 48), rgb(24, 24, 32), rgb(120, 32, 32), rgb(32, 48, 96)][rng.int(4) as usize];
    s.rect(dx, GROUND - 14, 6, 14, door);
    s.set(dx + 4, GROUND - 7, rgb(232, 200, 72));
    s.rect(dx, GROUND - 13, 6, 3, rgb(232, 224, 168));
    let wx = if door_left { x + w - 10 } else { x + 3 };
    s.rect(wx - 1, GROUND - 14, 8, 10, trim);
    s.rect(wx, GROUND - 13, 6, 8, glass);
    // a dark seam between houses
    for yy in top - 2..GROUND {
        if s.get(x, yy) != 0 {
            s.set(x, yy, shade(s.get(x, yy), 2, 3));
        }
    }
}

/// Centraal Station: red brick, two towers, a clock, big arched windows.
fn station(s: &mut Strip, x0: i32, x1: i32) {
    let brick = rgb(152, 64, 48);
    let dark = rgb(112, 48, 40);
    let stone = rgb(232, 216, 176);
    let top = GROUND - 56;
    for yy in top..GROUND {
        for xx in x0..x1 {
            let c = if yy % 4 == 0 { dark } else { brick };
            s.set(xx, yy, c);
        }
    }
    // stone bands
    for yy in [top, top + 20, top + 38] {
        s.rect(x0, yy, x1 - x0, 2, stone);
    }
    // roof
    for yy in 0..10 {
        s.rect(x0 + yy, top - yy, x1 - x0 - 2 * yy, 1, rgb(64, 72, 88));
    }
    // arched windows along both floors
    let mut wx = x0 + 8;
    while wx + 10 < x1 - 8 {
        for fy in [top + 5, top + 24] {
            s.rect(wx, fy + 2, 8, 11, stone);
            s.rect(wx + 1, fy + 3, 6, 10, rgb(48, 64, 96));
            s.rect(wx + 2, fy + 1, 4, 1, stone);
            s.set(wx + 3, fy + 4, rgb(120, 152, 200));
        }
        wx += 16;
    }
    // two towers, the left one with the clock
    for (k, tx) in [x0 + 70, x1 - 90].iter().enumerate() {
        let tw = 24;
        let tt = top - 34;
        for yy in tt..top + 2 {
            for xx in *tx..*tx + tw {
                s.set(xx, yy, if yy % 4 == 0 { dark } else { brick });
            }
        }
        s.rect(*tx, tt, tw, 2, stone);
        // spire
        for yy in 0..12 {
            s.rect(*tx + 2 + yy, tt - yy, tw - 4 - 2 * yy, 1, rgb(64, 96, 88));
        }
        s.rect(*tx + tw / 2, tt - 16, 1, 5, rgb(232, 200, 72));
        let (ccx, ccy) = (*tx + tw / 2, tt + 12);
        for dy in -6i32..=6 {
            for dx in -6i32..=6 {
                let d = dx * dx + dy * dy;
                if d <= 36 {
                    s.set(ccx + dx, ccy + dy, if d >= 26 { rgb(232, 200, 72) } else if k == 0 { rgb(248, 248, 240) } else { rgb(40, 48, 72) });
                }
            }
        }
        if k == 0 {
            for d in 0..4 {
                s.set(ccx, ccy - d, rgb(24, 24, 32));
            }
            for d in 0..3 {
                s.set(ccx + d, ccy, rgb(24, 24, 32));
            }
        } else {
            // the wind vane dial
            for d in -4..=4 {
                s.set(ccx + d, ccy, rgb(232, 200, 72));
            }
        }
    }
    // the big entrance arch in the middle
    let mx = (x0 + x1) / 2;
    s.rect(mx - 14, GROUND - 26, 28, 26, stone);
    s.rect(mx - 11, GROUND - 23, 22, 23, rgb(32, 32, 48));
    for k in 0..3 {
        s.rect(mx - 10 + k * 8, GROUND - 22, 4, 22, rgb(56, 64, 96));
    }
    // a sign
    s.rect(mx - 20, GROUND - 33, 40, 6, rgb(32, 48, 128));
    for k in 0..12 {
        s.rect(mx - 17 + k * 3, GROUND - 31, 2, 2, rgb(248, 248, 248));
    }
}

/// The sidewalk in front of the houses.
fn sidewalk(s: &mut Strip, len: i32) {
    for yy in GROUND..HH {
        for xx in 0..len {
            let c = if (xx + (yy - GROUND) * 2).rem_euclid(9) == 0 || yy == GROUND + 4 { rgb(136, 136, 144) } else { rgb(168, 168, 176) };
            s.set(xx, yy, c);
        }
    }
    for xx in 0..len {
        s.set(xx, GROUND, rgb(104, 104, 112));
        s.set(xx, HH - 1, rgb(120, 120, 128));
    }
}

/// Orange and red-white-blue bunting strung between the houses.
fn bunting(s: &mut Strip, len: i32, rng: &mut Rng) {
    let mut x = 10;
    while x < len - 60 {
        let span = 50 + rng.int(40);
        let y = 24 + rng.int(10);
        for k in 0..span {
            let sag = ((k as f32 / span as f32 * std::f32::consts::PI).sin() * 8.0) as i32;
            s.set(x + k, y + sag, rgb(64, 48, 40));
            if k % 6 == 0 {
                let col = [rgb(248, 136, 24), rgb(232, 48, 48), rgb(248, 248, 248), rgb(48, 72, 176)][((k / 6) % 4) as usize];
                for d in 0..4 {
                    for e in 0..(4 - d) {
                        s.set(x + k + e + d / 2, y + sag + 1 + d, col);
                    }
                }
            }
        }
        x += span + 30 + rng.int(60);
    }
}

/// Distant towers and roofs, in haze.
fn skyline(venue: Venue, rng: &mut Rng) -> Vec<u32> {
    let mut px = vec![0u32; (FAR_W * FAR_H) as usize];
    let c = match venue {
        Venue::Station => rgb(136, 160, 200),
        _ => rgb(144, 168, 208),
    };
    let c2 = shade(c, 7, 8);
    let mut set = |x: i32, y: i32, col: u32| {
        if x >= 0 && y >= 0 && x < FAR_W && y < FAR_H {
            px[(y * FAR_W + x) as usize] = col;
        }
    };
    // low roofs
    let mut x = 0;
    while x < FAR_W {
        let w = 14 + rng.int(16);
        let h = 6 + rng.int(8);
        for yy in FAR_H - h..FAR_H {
            for xx in x..x + w {
                set(xx, yy, if (xx / 7) % 2 == 0 { c } else { c2 });
            }
        }
        for k in 0..w / 2 {
            for xx in x + k..x + w - k {
                set(xx, FAR_H - h - k / 2, c);
            }
        }
        x += w;
    }
    // towers: the Westertoren (a spire with the crown), a dome, cranes by the IJ
    let towers: &[(i32, i32, u8)] = match venue {
        Venue::Station => &[(60, 34, 2), (200, 30, 2), (330, 26, 0), (450, 20, 1)],
        _ => &[(90, 40, 0), (300, 22, 1), (420, 28, 0)],
    };
    for &(tx, th, kind) in towers {
        match kind {
            0 => {
                for yy in FAR_H - th..FAR_H {
                    let w = if yy < FAR_H - th + 10 { 2 } else if yy < FAR_H - th + 16 { 4 } else { 6 };
                    for xx in tx - w / 2..tx + w / 2 + 1 {
                        set(xx, yy, c2);
                    }
                }
                set(tx, FAR_H - th - 1, c2);
                set(tx, FAR_H - th - 2, c2);
            }
            1 => {
                for yy in 0..8i32 {
                    let w = ((64 - (8 - yy) * (8 - yy)) as f32).sqrt() as i32;
                    for xx in tx - w..tx + w {
                        set(xx, FAR_H - th + yy, c2);
                    }
                }
                for yy in FAR_H - th + 8..FAR_H {
                    for xx in tx - 8..tx + 8 {
                        set(xx, yy, c2);
                    }
                }
            }
            _ => {
                // a crane
                for yy in FAR_H - th..FAR_H {
                    set(tx, yy, c2);
                    set(tx + 1, yy, c2);
                }
                for xx in tx - 8..tx + 20 {
                    set(xx, FAR_H - th, c2);
                }
                for yy in FAR_H - th..FAR_H - th + 8 {
                    set(tx + 16, yy, c2);
                }
            }
        }
    }
    px
}
