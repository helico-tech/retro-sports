//! The look of a venue: banded sky by altitude, two parallax layers drawn
//! once per stage, clouds, terrain columns from the height map, props and
//! the animated life on top of them.

use super::art::*;
use super::course::*;
use crate::engine::gfx::*;
use crate::engine::rng::Rng;
use crate::engine::ui::OUTLINE;

/// A pre-rendered parallax layer. Its bottom row sits at altitude `base`
/// when the camera is there; it moves `kx`, `ky` times as fast as the world.
pub struct Layer {
    pub pic: Pic,
    pub kx: f32,
    pub ky: f32,
    /// skyline foot, in rows below the view centre, at camera height `REL0`
    pub foot0: f32,
    /// ground plane rows under the skyline
    pub plane: i32,
}

/// Camera height above the general terrain where the layers sit at `foot0`.
pub const REL0: f32 = 40.0;

/// Rows at the bottom of a layer that repeat below it (the ground plane).
const TILE: i32 = 12;

impl Layer {
    /// `rel`: camera height above the smoothed terrain under it. The layers
    /// sink slowly as you climb, so close to the ground the hills sit just
    /// above it and high up the far ground plane opens below you.
    pub fn draw(&self, f: &mut Frame, cam_x: f32, rel: f32, cy: i32, vh: i32) {
        let p = &self.pic;
        let off = ((cam_x - 128.0) * self.kx) as i32;
        let foot = cy + (self.foot0 + (rel - REL0) * self.ky) as i32;
        let top = foot - (p.h - self.plane - 1);
        for sy in top.max(0)..vh.min(H) {
            let mut r = sy - top;
            if r >= p.h {
                r = p.h - TILE + (r - p.h) % TILE;
            }
            let row = (r * p.w) as usize;
            let out = (sy * W) as usize;
            for sx in 0..W {
                let px = (off + sx).clamp(0, p.w - 1);
                let c = p.px[row + px as usize];
                if c != CLEAR {
                    f.px[out + sx as usize] = c;
                }
            }
        }
    }
}

pub struct Scenery {
    pub venue: Venue,
    pub sky: Vec<u32>,
    pub far: Layer,
    pub mid: Layer,
    /// cumulus clouds by size (small, medium, big)
    pub cumulus: Vec<Pic>,
    pub cow_cloud: Pic,
    pub storm: Option<Pic>,
    /// small background clouds: (x, y, pic index) in the far layer's space
    pub deco: Vec<(f32, f32, usize)>,
    /// one picture per course prop (None: drawn live)
    pub props: Vec<Option<Pic>>,
    pub envelopes: Vec<Pic>,
    pub family: Pic,
    pub sun: (i32, i32),
    /// smoothed terrain height every 8 px (what "high" means for the layers)
    pub ref_g: Vec<f32>,
}


fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

impl Scenery {
    pub fn new(course: &Course, seed: u64) -> Self {
        let venue = course.venue;
        let mut rng = Rng(seed | 1);
        let sky = match venue {
            Venue::Dunes => vec![
                rgb(32, 72, 184),
                rgb(40, 88, 200),
                rgb(56, 112, 216),
                rgb(72, 136, 232),
                rgb(96, 160, 240),
                rgb(128, 184, 240),
                rgb(168, 208, 248),
                rgb(208, 232, 248),
            ],
            Venue::Alps => vec![
                rgb(16, 40, 152),
                rgb(24, 56, 176),
                rgb(40, 80, 200),
                rgb(56, 112, 224),
                rgb(88, 144, 232),
                rgb(128, 176, 240),
                rgb(176, 208, 248),
                rgb(216, 232, 248),
            ],
            Venue::Canyon => vec![
                rgb(40, 40, 112),
                rgb(56, 56, 136),
                rgb(88, 72, 160),
                rgb(136, 88, 168),
                rgb(184, 104, 160),
                rgb(224, 128, 136),
                rgb(240, 160, 112),
                rgb(248, 200, 120),
            ],
        };
        let (far, mid) = match venue {
            Venue::Dunes => (dunes_far(&mut rng, &sky), dunes_mid(&mut rng, &sky)),
            Venue::Alps => (alps_far(&mut rng, &sky), alps_mid(&mut rng, &sky)),
            Venue::Canyon => (canyon_far(&mut rng, &sky), canyon_mid(&mut rng, &sky)),
        };
        let tint = match venue {
            Venue::Canyon => rgb(248, 216, 200),
            _ => rgb(248, 248, 248),
        };
        let cumulus = vec![cumulus(&mut rng, 70, 30, tint), cumulus(&mut rng, 100, 40, tint), cumulus(&mut rng, 130, 50, tint)];
        let mut deco = Vec::new();
        for k in 0..14 {
            deco.push((k as f32 * 90.0 + rng.range(0.0, 60.0), rng.range(30.0, 150.0), rng.int(3) as usize));
        }
        let props = course.props.iter().map(|p| prop_pic(p, venue)).collect();
        let envelopes = (0..course.gates.len()).map(envelope).collect();
        Scenery {
            venue,
            sky,
            far,
            mid,
            cumulus,
            cow_cloud: cow_cloud(),
            storm: course.storm.map(|_| storm_pic()),
            deco,
            props,
            envelopes,
            family: envelope(7),
            sun: match venue {
                Venue::Dunes => (206, 46),
                Venue::Alps => (60, 40),
                Venue::Canyon => (196, 120),
            },
            ref_g: ref_ground(course),
        }
    }

    /// Camera height above the general terrain.
    pub fn rel(&self, cam: (f32, f32)) -> f32 {
        let i = (cam.0 / 8.0).clamp(0.0, (self.ref_g.len() - 1) as f32);
        let (a, b) = (i.floor() as usize, (i.floor() as usize + 1).min(self.ref_g.len() - 1));
        let g = self.ref_g[a] + (self.ref_g[b] - self.ref_g[a]) * i.fract();
        cam.1 - g
    }

    /// Sky color for screen row `sy` (centre row `cy`) at camera altitude `cam_y`:
    /// a gradient across the screen that darkens slowly as you climb.
    #[inline]
    pub fn sky_at(&self, cam_y: f32, cy: i32, x: i32, sy: i32) -> u32 {
        let n = self.sky.len() as i32;
        // u: 0 at the top of the sky, 1 at the horizon
        let u = ((sy - cy) as f32 + 150.0 - (cam_y - 200.0) * 0.18) / 260.0;
        let pos = (u.clamp(0.0, 0.999) * (n * 4) as f32) as i32;
        let band = (pos / 4).min(n - 1);
        let sub = pos % 4;
        let next = (band + 1).min(n - 1);
        if sub == 3 && (x + sy) & 1 == 0 { self.sky[next as usize] } else { self.sky[band as usize] }
    }

    pub fn draw_sky(&self, f: &mut Frame, cam: (f32, f32), cy: i32, vh: i32, t: u32) {
        for sy in 0..vh.min(H) {
            let out = (sy * W) as usize;
            let a = self.sky_at(cam.1, cy, 0, sy);
            let b = self.sky_at(cam.1, cy, 1, sy);
            for sx in 0..W {
                f.px[out + sx as usize] = if (sx & 1) == 0 { a } else { b };
            }
        }
        // the sun hangs low in the canyon and high elsewhere
        let (sx, sy0) = self.sun;
        let sy = sy0 + ((cam.1 - 300.0) * 0.08) as i32;
        if sy < vh + 20 {
            let (core, halo) = match self.venue {
                Venue::Canyon => (rgb(248, 216, 120), rgb(248, 168, 96)),
                _ => (rgb(248, 248, 216), rgb(232, 240, 248)),
            };
            for r in (13..=19).rev().step_by(3) {
                for dy in -r..=r {
                    let yy = sy + dy;
                    if yy < 0 || yy >= vh {
                        continue;
                    }
                    let w = (((r * r - dy * dy) as f32).sqrt()) as i32;
                    for xx in sx - w..=sx + w {
                        if (xx + yy + r) % (if r > 16 { 4 } else { 2 }) == 0 {
                            f.set(xx, yy, halo);
                        }
                    }
                }
            }
            for dy in -11..=11 {
                let yy = sy + dy;
                if yy < 0 || yy >= vh {
                    continue;
                }
                let w = ((121 - dy * dy) as f32).sqrt() as i32;
                f.hline(sx - w, sx + w, yy, core);
            }
            f.hline(sx - 6, sx + 2, sy - 7, rgb(248, 248, 248));
        }
        let _ = t;
    }

    /// Far layer, its decorative clouds and the cow cloud, then the mid layer.
    pub fn draw_back(&self, f: &mut Frame, cam: (f32, f32), cy: i32, vh: i32, t: u32) {
        let rel = self.rel(cam);
        self.far.draw(f, cam.0, rel, cy, vh);
        // small clouds drift slowly in the far sky
        let kx = 0.22;
        for &(x, y, k) in &self.deco {
            let span = 1400.0;
            let wx = (x + t as f32 * 0.02).rem_euclid(span);
            let sx = (wx - (cam.0 * kx).rem_euclid(span)).rem_euclid(span) as i32 - 100;
            let sy = cy - (y + 40.0 - (rel - REL0) * 0.3) as i32;
            if sy > -40 && sy < vh + 40 {
                let p = &self.cumulus[k];
                draw_pic_clip(f, p, sx, sy, vh, true);
            }
        }
        // the cow-shaped cloud
        {
            let span = 2000.0;
            let wx = (700.0 + t as f32 * 0.03).rem_euclid(span);
            let sx = (wx - (cam.0 * 0.25).rem_euclid(span)).rem_euclid(span) as i32 - 200;
            let sy = cy - (120.0 - (rel - REL0) * 0.3) as i32;
            draw_pic_clip(f, &self.cow_cloud, sx, sy, vh, false);
        }
        self.mid.draw(f, cam.0, rel, cy, vh);
    }
}

fn ref_ground(c: &Course) -> Vec<f32> {
    let n = (LEN / 8 + 1) as usize;
    (0..n)
        .map(|k| {
            let x = k as f32 * 8.0;
            let (mut sum, mut cnt) = (0.0, 0.0);
            let mut d = -320.0;
            while d <= 320.0 {
                sum += c.floor_at((x + d).clamp(0.0, LEN as f32));
                cnt += 1.0;
                d += 20.0;
            }
            sum / cnt
        })
        .collect()
}

/// Draw a picture clipped to rows < vh; `small` halves it (far away).
pub fn draw_pic_clip(f: &mut Frame, p: &Pic, x: i32, y: i32, vh: i32, small: bool) {
    let step = if small { 2 } else { 1 };
    let w = p.w / step;
    let h = p.h / step;
    let x0 = x - p.ox / step;
    let y0 = y - p.oy / step;
    if x0 > W || x0 + w < 0 || y0 > vh || y0 + h < 0 {
        return;
    }
    for sy in 0..h {
        let py = y0 + sy;
        if py < 0 || py >= vh.min(H) {
            continue;
        }
        for sx in 0..w {
            let c = p.px[((sy * step) * p.w + sx * step) as usize];
            if c != CLEAR {
                let px = x0 + sx;
                if (0..W).contains(&px) {
                    f.px[(py * W + px) as usize] = c;
                }
            }
        }
    }
}

// ------------------------------------------------------------ layer art

fn new_layer(w: i32, h: i32) -> Pic {
    Pic::new(w, h, 0, 0)
}

fn layer_w(kx: f32) -> i32 {
    (LEN as f32 * kx) as i32 + W + 8
}

/// Append a ground plane below a layer: bands of fields that get bigger
/// toward the viewer, colors from `pal(x, band, field)`.
fn with_plane(p: Pic, rows: i32, haze: u32, pal: impl Fn(i32, i32, u32) -> u32) -> Pic {
    let mut q = Pic::new(p.w, p.h + rows, 0, 0);
    q.px[..p.px.len()].copy_from_slice(&p.px);
    let mut band = 0;
    let mut start = 0;
    let mut r = 0;
    while r < rows {
        let bh = 2 + band;
        for y in r..(r + bh).min(rows) {
            let first = y == start;
            for x in 0..p.w {
                let fw = 14 + band * 9;
                let fid = ((x + (hash(band, 3) % 97) as i32) / fw) as u32 ^ hash(band, 5);
                let mut c = pal(x, band, hash(fid as i32, band));
                if first {
                    c = shade(c, 3, 4);
                }
                let hz = (0.55 - band as f32 * 0.05).max(0.18);
                q.set(x, p.h + y, mix(c, haze, hz));
            }
        }
        r += bh;
        start = r;
        band += 1;
    }
    q
}

const PLANE: i32 = 60;

fn dunes_far(rng: &mut Rng, sky: &[u32]) -> Layer {
    let (kx, ky) = (0.12, 0.1);
    let w = layer_w(kx);
    let h = 70;
    let mut p = new_layer(w, h);
    let hz = h - 24;
    let haze = sky[sky.len() - 1];
    // the sea, out to the horizon, on the west side
    let sea_end = 140;
    for x in 0..w {
        let land_h = if x < sea_end { 0 } else { ((x - sea_end) as f32 / 30.0).min(1.0) as i32 * (6 + (((x as f32) * 0.05).sin() * 3.0 + ((x as f32) * 0.13).sin() * 2.0) as i32) };
        for y in hz - land_h..h {
            let c = if y < hz {
                mix(rgb(168, 184, 144), haze, 0.45)
            } else if x < sea_end + 20 && y < hz + 8 {
                let k = y - hz;
                let base = [rgb(104, 144, 200), rgb(88, 128, 192), rgb(72, 112, 184), rgb(64, 104, 176)][(k / 2).min(3) as usize];
                if hash(x, y).is_multiple_of(11) { rgb(200, 224, 248) } else { base }
            } else {
                let k = y - hz;
                mix([rgb(152, 176, 120), rgb(136, 168, 104), rgb(120, 160, 96)][(k / 8).min(2) as usize], haze, 0.35)
            };
            p.set(x, y, c);
        }
    }
    // distant villages: church spires, windmills, water towers
    let mut x = sea_end + 40;
    let sil = mix(rgb(120, 136, 128), haze, 0.4);
    while x < w - 20 {
        match rng.int(4) {
            0 => {
                p.rect(x, hz - 14, 4, 14, sil);
                p.poly(&[(x - 1, hz - 14), (x + 2, hz - 26), (x + 5, hz - 14)], sil);
                p.rect(x - 8, hz - 6, 8, 6, sil);
                p.rect(x + 4, hz - 5, 9, 5, sil);
            }
            1 => {
                p.poly(&[(x, hz), (x + 2, hz - 11), (x + 5, hz - 11), (x + 7, hz)], sil);
                p.line(x - 2, hz - 16, x + 9, hz - 6, sil);
                p.line(x - 2, hz - 6, x + 9, hz - 16, sil);
            }
            2 => {
                for k in 0..3 {
                    p.ellipse(x + k * 6, hz - 4, 4, 4, mix(rgb(96, 128, 96), haze, 0.35));
                }
            }
            _ => {
                p.rect(x + 2, hz - 12, 2, 12, sil);
                p.ellipse(x + 3, hz - 14, 4, 3, sil);
            }
        }
        x += 40 + rng.int(70);
    }
    let p = with_plane(p, 24, haze, |_, _, h| [rgb(136, 168, 104), rgb(152, 176, 112), rgb(120, 160, 96)][(h % 3) as usize]);
    Layer { pic: p, kx, ky, foot0: -6.0, plane: 24 }
}

fn dunes_mid(rng: &mut Rng, sky: &[u32]) -> Layer {
    let (kx, ky) = (0.4, 0.22);
    let w = layer_w(kx);
    let h = 70;
    let mut p = new_layer(w, h);
    let haze = sky[sky.len() - 2];
    let (p1, p2) = (rng.range(0.0, 6.0), rng.range(0.0, 6.0));
    for x in 0..w {
        let xf = x as f32;
        let wx = xf / kx;
        // dunes near the coast, flat polder further inland
        let duney = if wx < 1500.0 { 1.0 } else if wx < 2200.0 { 1.0 - (wx - 1500.0) / 700.0 } else { 0.2 };
        let top = h - 12 - (duney * (14.0 + 10.0 * (xf * 0.021 + p1).sin() + 5.0 * (xf * 0.057 + p2).sin())) as i32;
        for y in top..h {
            let d = y - top;
            let c = if duney > 0.5 {
                if d < 1 {
                    rgb(248, 232, 184)
                } else if d < 4 {
                    rgb(232, 208, 152)
                } else if (d < 7 && (x + y) & 1 == 0) || d >= 7 {
                    rgb(200, 184, 128)
                } else {
                    rgb(232, 208, 152)
                }
            } else if d < 1 {
                rgb(160, 208, 112)
            } else if d < 5 {
                rgb(120, 184, 88)
            } else if d < 8 && (x + y) & 1 == 0 {
                rgb(96, 160, 72)
            } else {
                rgb(104, 168, 80)
            };
            p.set(x, y, mix(c, haze, 0.15));
        }
        // marram grass on the dunes
        if duney > 0.5 && hash(x, 7).is_multiple_of(3) {
            let gh = 2 + (hash(x, 9) % 4) as i32;
            p.line(x, top, x + (hash(x, 3) % 3) as i32 - 1, top - gh, mix(rgb(120, 160, 72), haze, 0.3));
        }
    }
    // rows of trees and farms inland
    let mut x = (1500.0 * kx) as i32;
    while x < w - 30 {
        let top = (0..h).find(|&y| p.get(x, y) != CLEAR).unwrap_or(h - 12);
        if rng.f() < 0.5 {
            for k in 0..(2 + rng.int(4)) {
                let tx = x + k * 7;
                p.ellipse(tx, top - 6, 4, 6, mix(rgb(56, 120, 72), haze, 0.3));
                p.ellipse(tx - 1, top - 8, 2, 3, mix(rgb(88, 152, 88), haze, 0.3));
                p.line(tx, top - 1, tx, top, mix(rgb(96, 64, 48), haze, 0.3));
            }
        } else {
            let c = mix(rgb(176, 72, 56), haze, 0.3);
            p.rect(x, top - 8, 14, 8, c);
            p.poly(&[(x - 1, top - 8), (x + 7, top - 15), (x + 15, top - 8)], mix(rgb(96, 56, 48), haze, 0.3));
            p.rect(x + 3, top - 5, 2, 2, mix(rgb(248, 232, 160), haze, 0.3));
            p.rect(x + 9, top - 5, 2, 2, mix(rgb(248, 232, 160), haze, 0.3));
        }
        x += 30 + rng.int(60);
    }
    let split = (1500.0 * kx) as i32;
    let p = with_plane(p, PLANE, haze, |x, band, h| {
        // dunes fade into the polder over a dithered stretch
        let blend = ((x - split + 60) as f32 / 120.0).clamp(0.0, 1.0);
        let sandy = (hash(x / 3, band) % 100) as f32 / 100.0 >= blend;
        if sandy {
            [rgb(232, 208, 152), rgb(216, 192, 136), rgb(176, 184, 112), rgb(240, 224, 168)][(h % 4) as usize]
        } else {
            [
                rgb(112, 184, 80),
                rgb(136, 192, 88),
                rgb(216, 200, 88),
                rgb(200, 104, 104),
                rgb(96, 160, 80),
                rgb(160, 128, 88),
                rgb(200, 136, 176),
                rgb(120, 176, 80),
            ][(h % 8) as usize]
        }
    });
    Layer { pic: p, kx, ky, foot0: 12.0, plane: PLANE }
}

#[allow(clippy::too_many_arguments)]
fn mountains(p: &mut Pic, pts: &[(i32, i32)], body: u32, side: u32, snow: u32, snow_d: u32, base: i32, snowline: i32) {
    let mut poly: Vec<(i32, i32)> = pts.to_vec();
    poly.push((pts[pts.len() - 1].0, base));
    poly.push((pts[0].0, base));
    p.poly(&poly, body);
    for w in pts.windows(3) {
        let (a, pk, b) = (w[0], w[1], w[2]);
        if pk.1 < a.1 && pk.1 < b.1 {
            // shaded east face
            p.poly(&[pk, b, (pk.0 + (b.0 - pk.0) / 4, b.1 + (base - b.1) / 3)], side);
            if pk.1 < snowline {
                let cap = (snowline - pk.1).min(22);
                let l = (pk.0 + (a.0 - pk.0) * cap / (a.1 - pk.1).max(1), pk.1 + cap);
                let r = (pk.0 + (b.0 - pk.0) * cap / (b.1 - pk.1).max(1), pk.1 + cap);
                p.poly(&[pk, r, (r.0 - 3, r.1 + 3), ((pk.0 + r.0) / 2, r.1 - 2), (pk.0, r.1 + 4), ((pk.0 + l.0) / 2, l.1 - 1), (l.0 + 2, l.1 + 2), l], snow);
                p.poly(&[pk, r, (r.0 - 3, r.1 + 3), ((pk.0 + r.0) / 2, r.1 - 2), (pk.0 + 1, pk.1 + 3)], snow_d);
            }
        }
    }
}

fn alps_far(rng: &mut Rng, sky: &[u32]) -> Layer {
    let (kx, ky) = (0.12, 0.1);
    let w = layer_w(kx);
    let h = 120;
    let mut p = new_layer(w, h);
    let haze = sky[sky.len() - 2];
    let mut pts = vec![(-10, h - 40)];
    let mut x = 0;
    while x < w + 40 {
        x += 18 + rng.int(30);
        let peak = rng.f() < 0.55;
        let y = if peak { 8 + rng.int(40) } else { 50 + rng.int(30) };
        pts.push((x, y));
    }
    pts.push((w + 60, h - 40));
    mountains(&mut p, &pts, mix(rgb(112, 128, 192), haze, 0.35), mix(rgb(88, 104, 168), haze, 0.35), mix(rgb(248, 248, 248), haze, 0.15), mix(rgb(200, 216, 240), haze, 0.2), h, 46);
    // a second, nearer and darker ridge
    let mut pts = vec![(-10, h - 10)];
    let mut x = 0;
    while x < w + 40 {
        x += 30 + rng.int(40);
        pts.push((x, 50 + rng.int(40)));
    }
    pts.push((w + 60, h - 10));
    mountains(&mut p, &pts, mix(rgb(88, 104, 168), haze, 0.25), mix(rgb(72, 88, 152), haze, 0.25), mix(rgb(240, 244, 248), haze, 0.1), mix(rgb(184, 200, 232), haze, 0.15), h, 62);
    let p = with_plane(p, 24, haze, |_, _, h| [rgb(96, 136, 120), rgb(104, 144, 120), rgb(88, 128, 112)][(h % 3) as usize]);
    Layer { pic: p, kx, ky, foot0: -4.0, plane: 24 }
}

fn alps_mid(rng: &mut Rng, sky: &[u32]) -> Layer {
    let (kx, ky) = (0.4, 0.22);
    let w = layer_w(kx);
    let h = 90;
    let mut p = new_layer(w, h);
    let haze = sky[sky.len() - 1];
    let (p1, p2) = (rng.range(0.0, 6.0), rng.range(0.0, 6.0));
    let mut tops = vec![0; w as usize];
    for x in 0..w {
        let xf = x as f32;
        let top = h - 30 - (18.0 * (xf * 0.013 + p1).sin() + 9.0 * (xf * 0.041 + p2).sin()) as i32;
        tops[x as usize] = top;
        for y in top..h {
            let d = y - top;
            let c = if d < 2 {
                rgb(128, 200, 104)
            } else if d < 10 {
                if (x + y) & 1 == 0 && d > 6 { rgb(80, 152, 80) } else { rgb(104, 176, 88) }
            } else {
                rgb(80, 152, 80)
            };
            p.set(x, y, mix(c, haze, 0.3));
        }
    }
    // pine forests on the hills
    let mut x = 0;
    while x < w {
        if hash(x, 1) % 5 < 3 {
            let n = 3 + rng.int(8);
            for k in 0..n {
                let tx = x + k * 4 + rng.int(3);
                if tx >= w {
                    break;
                }
                let base = tops[tx as usize] + 3 + rng.int(6);
                let th = 8 + rng.int(6);
                p.poly(&[(tx, base - th), (tx + 4, base), (tx - 4, base)], mix(rgb(32, 96, 72), haze, 0.3));
                p.line(tx, base - th, tx + 3, base - 1, mix(rgb(24, 72, 56), haze, 0.3));
            }
            x += n * 4 + 10;
        } else {
            x += 20 + rng.int(30);
        }
    }
    // a village with a church
    for vx in [300, 900] {
        if vx >= w - 40 {
            continue;
        }
        let top = tops[vx as usize];
        for k in 0..4 {
            let hx = vx + k * 9;
            let ht = tops[hx as usize];
            p.rect(hx, ht - 6, 7, 7, mix(rgb(240, 232, 216), haze, 0.25));
            p.poly(&[(hx - 1, ht - 6), (hx + 3, ht - 10), (hx + 8, ht - 6)], mix(rgb(176, 72, 56), haze, 0.25));
        }
        p.rect(vx + 40, top - 18, 5, 18, mix(rgb(240, 232, 216), haze, 0.25));
        p.ellipse(vx + 42, top - 20, 3, 3, mix(rgb(96, 128, 80), haze, 0.25));
        p.line(vx + 42, top - 26, vx + 42, top - 23, mix(rgb(96, 128, 80), haze, 0.25));
    }
    let p = with_plane(p, PLANE, haze, |_, _, h| {
        [rgb(104, 176, 88), rgb(128, 192, 96), rgb(48, 112, 72), rgb(96, 168, 80), rgb(144, 200, 104), rgb(40, 96, 64), rgb(112, 184, 88)][(h % 7) as usize]
    });
    Layer { pic: p, kx, ky, foot0: 12.0, plane: PLANE }
}

#[allow(clippy::too_many_arguments)]
fn mesa(p: &mut Pic, x: i32, w: i32, top: i32, base: i32, body: u32, side: u32, bands: &[u32]) {
    p.poly(&[(x, base), (x + 3, top + 4), (x + 5, top), (x + w - 5, top), (x + w - 3, top + 4), (x + w, base)], body);
    for (k, &c) in bands.iter().enumerate() {
        let y = top + 4 + k as i32 * ((base - top) / (bands.len() as i32 + 1)).max(3);
        for yy in y..(y + 2).min(base) {
            for xx in x + 1..x + w {
                if p.get(xx, yy) == body && (xx + yy) % 3 != 0 {
                    p.set(xx, yy, c);
                }
            }
        }
    }
    // shadowed east side
    for yy in top..base {
        for xx in (x + w - 6).max(x)..x + w {
            if p.get(xx, yy) != CLEAR {
                p.set(xx, yy, side);
            }
        }
    }
    p.hline(x + 5, x + w - 5, top, mix(body, 0xFFFFFF, 0.25));
}

fn canyon_far(rng: &mut Rng, sky: &[u32]) -> Layer {
    let (kx, ky) = (0.12, 0.1);
    let w = layer_w(kx);
    let h = 90;
    let mut p = new_layer(w, h);
    let haze = sky[sky.len() - 2];
    let body = mix(rgb(152, 88, 120), haze, 0.4);
    let side = mix(rgb(120, 72, 112), haze, 0.4);
    let bands = [mix(rgb(176, 104, 120), haze, 0.4)];
    let mut x = -20;
    while x < w {
        let mw = 30 + rng.int(90);
        let top = 20 + rng.int(40);
        mesa(&mut p, x, mw, top, h, body, side, &bands);
        if rng.f() < 0.3 {
            // a lone spire
            let sx = x + mw + 10;
            p.poly(&[(sx, h), (sx + 3, top - 10), (sx + 7, top - 12), (sx + 9, h)], body);
        }
        x += mw + rng.int(40);
    }
    p.rect(0, h - 12, w, 12, mix(rgb(176, 112, 120), haze, 0.4));
    let p = with_plane(p, 24, haze, |_, _, h| [rgb(184, 120, 120), rgb(192, 128, 120), rgb(176, 112, 112)][(h % 3) as usize]);
    Layer { pic: p, kx, ky, foot0: -4.0, plane: 24 }
}

fn canyon_mid(rng: &mut Rng, sky: &[u32]) -> Layer {
    let (kx, ky) = (0.4, 0.22);
    let w = layer_w(kx);
    let h = 100;
    let mut p = new_layer(w, h);
    let haze = sky[sky.len() - 1];
    let body = mix(rgb(200, 104, 64), haze, 0.25);
    let side = mix(rgb(152, 72, 56), haze, 0.25);
    let bands = [mix(rgb(232, 160, 104), haze, 0.25), mix(rgb(176, 88, 56), haze, 0.25), mix(rgb(232, 176, 128), haze, 0.25)];
    let mut x = -10;
    while x < w {
        let mw = 40 + rng.int(110);
        let top = 20 + rng.int(46);
        mesa(&mut p, x, mw, top, h, body, side, &bands);
        x += mw + 20 + rng.int(60);
    }
    // desert floor
    for x in 0..w {
        for y in h - 16..h {
            let c = if y == h - 16 { rgb(232, 176, 112) } else if (x + y) & 1 == 0 && y < h - 12 { rgb(216, 152, 96) } else { rgb(200, 136, 88) };
            p.set(x, y, mix(c, haze, 0.25));
        }
    }
    let p = with_plane(p, PLANE, haze, |_, _, h| {
        [rgb(216, 152, 96), rgb(232, 176, 112), rgb(200, 128, 80), rgb(224, 160, 104), rgb(184, 112, 72), rgb(240, 192, 128)][(h % 6) as usize]
    });
    Layer { pic: p, kx, ky, foot0: 14.0, plane: PLANE }
}

/// Puffy cumulus with a flat base. Origin: middle of the base.
fn cumulus(rng: &mut Rng, w: i32, h: i32, tint: u32) -> Pic {
    let mut p = Pic::new(w + 4, h + 4, (w + 4) / 2, h + 1);
    let mut puffs = vec![];
    let n = w / 14 + 3;
    for k in 0..n {
        let cx = 8 + (w - 16) * k / (n - 1).max(1) + rng.int(5) - 2;
        let mid = 1.0 - ((cx - w / 2) as f32 / (w as f32 / 2.0)).abs();
        let r = 7 + (mid * h as f32 * 0.45) as i32 + rng.int(4);
        puffs.push((cx + 2, h + 1 - r / 2 - 2, r));
    }
    let top_c = mix(tint, 0xFFFFFF, 0.5);
    let body = tint;
    let shade1 = mix(tint, rgb(168, 184, 224), 0.35);
    let shade2 = mix(tint, rgb(136, 152, 208), 0.55);
    for &(cx, cy, r) in &puffs {
        p.ellipse(cx, cy, r, r * 4 / 5, body);
    }
    // cut the base flat
    for y in h + 1..p.h {
        for x in 0..p.w {
            p.set(x, y, CLEAR);
        }
    }
    // shading: lower and right parts darker, the tops brighter
    let src = p.px.clone();
    let (pw, ph) = (p.w, p.h);
    let at = |x: i32, y: i32| if x >= 0 && y >= 0 && x < pw && y < ph { src[(y * pw + x) as usize] } else { CLEAR };
    for y in 0..ph {
        for x in 0..pw {
            if at(x, y) == CLEAR {
                continue;
            }
            let base_d = h + 1 - y;
            let c = if at(x, y - 2) == CLEAR && at(x - 1, y - 2) == CLEAR {
                top_c
            } else if base_d < 4 {
                if (x + y) & 1 == 0 { shade2 } else { shade1 }
            } else if base_d < 8 || at(x + 3, y + 1) == CLEAR {
                shade1
            } else {
                body
            };
            p.set(x, y, c);
        }
    }
    p.outline(mix(tint, rgb(120, 136, 192), 0.5));
    p
}

fn cow_cloud() -> Pic {
    let mut p = Pic::new(64, 36, 32, 34);
    let w = rgb(248, 248, 248);
    p.ellipse(28, 18, 18, 9, w);
    p.ellipse(18, 15, 8, 7, w);
    p.ellipse(36, 14, 9, 7, w);
    // head with ears and snout
    p.ellipse(50, 12, 7, 6, w);
    p.ellipse(55, 15, 4, 3, w);
    p.ellipse(46, 6, 3, 2, w);
    p.ellipse(54, 6, 2, 2, w);
    // legs and tail
    for lx in [16, 22, 34, 40] {
        p.rect(lx, 25, 4, 7, w);
        p.ellipse(lx + 2, 32, 2, 1, w);
    }
    p.line(10, 16, 4, 22, w);
    p.ellipse(4, 23, 2, 2, w);
    // faint spots
    let spot = rgb(208, 216, 240);
    p.ellipse(24, 17, 4, 3, spot);
    p.ellipse(36, 20, 3, 2, spot);
    p.ellipse(16, 14, 2, 2, spot);
    p.set(52, 11, rgb(120, 136, 184));
    p.set(56, 15, rgb(232, 176, 200));
    p.map(|x, y, c| if c == w && (y > 22 || (x + y) % 7 == 0 && y > 18) { rgb(224, 232, 248) } else { c });
    p.outline(rgb(176, 192, 232));
    p
}

/// The hungry storm cloud: a cumulonimbus with an anvil, slanted rain and
/// a face whose open mouth is where it sucks pilots in.
fn storm_pic() -> Pic {
    let (w, h) = (220, 184);
    let base = 150;
    let mut p = Pic::new(w, h, w / 2, base);
    let dark = rgb(48, 40, 80);
    let body = rgb(80, 72, 120);
    let mid = rgb(112, 104, 152);
    let lit = rgb(160, 152, 200);
    let puffs: [(i32, i32, i32, bool); 22] = [
        (110, 128, 42, false),
        (74, 134, 30, false),
        (146, 134, 30, false),
        (110, 98, 38, false),
        (84, 104, 26, false),
        (136, 100, 28, false),
        (110, 70, 34, false),
        (92, 74, 24, false),
        (128, 72, 24, false),
        (110, 50, 26, false),
        (110, 34, 40, true),
        (72, 30, 34, true),
        (148, 30, 34, true),
        (40, 28, 24, true),
        (180, 28, 24, true),
        (18, 30, 14, true),
        (202, 30, 14, true),
        (60, 140, 18, false),
        (160, 140, 18, false),
        (96, 120, 20, false),
        (126, 116, 20, false),
        (110, 18, 26, true),
    ];
    for &(cx, cy, r, flat) in &puffs {
        let (rx, ry) = if flat { (r * 8 / 5, r * 11 / 20) } else { (r, r * 4 / 5) };
        p.ellipse(cx, cy, rx, ry, dark);
        p.ellipse(cx - 2, cy - 2, rx - 3, ry - 3, body);
        p.ellipse(cx - rx / 4, cy - ry / 3, rx / 2, ry / 3, mid);
        p.ellipse(cx - rx / 3, cy - ry / 2, rx / 4, (ry / 6).max(1), lit);
    }
    // flat dark base
    for y in base..h {
        for x in 0..w {
            p.set(x, y, CLEAR);
        }
    }
    for x in 0..w {
        if p.get(x, base - 1) != CLEAR {
            for y in base - 4..base {
                p.set(x, y, if y == base - 1 { rgb(32, 24, 56) } else { dark });
            }
        }
    }
    // rain, slanted with the wind
    for x in (40..180).step_by(4) {
        for y in base..h {
            let k = y - base;
            if (k + x) % 9 < 5 {
                p.set(x + k / 3, y, rgb(136, 144, 192));
            }
        }
    }
    // the face: angry eyes and a hungry mouth over the suction column
    for (ex, dir) in [(92, 1), (128, -1)] {
        p.ellipse(ex, 112, 9, 7, rgb(24, 16, 40));
        p.ellipse(ex, 112, 7, 5, rgb(248, 248, 232));
        p.rect(ex + dir * 2 - 1, 113, 4, 4, rgb(24, 16, 40));
        p.set(ex + dir * 2, 113, rgb(248, 248, 248));
        p.thick(ex - 9, 101 + dir * 4, ex + 9, 101 - dir * 4, 3, rgb(24, 16, 40));
    }
    p.ellipse(110, 136, 22, 11, rgb(24, 16, 40));
    p.ellipse(110, 138, 18, 8, rgb(88, 24, 56));
    p.ellipse(110, 141, 10, 4, rgb(184, 72, 104));
    for k in 0..6 {
        let tx = 94 + k * 6;
        p.poly(&[(tx, 127), (tx + 5, 127), (tx + 2, 132)], rgb(248, 248, 232));
    }
    p.outline(rgb(24, 16, 48));
    p
}

/// A balloon envelope with vertical gores; `k` picks the colors.
fn envelope(k: usize) -> Pic {
    let cols = [
        (rgb(232, 48, 48), rgb(248, 216, 48)),
        (rgb(48, 120, 232), rgb(248, 248, 248)),
        (rgb(56, 192, 88), rgb(248, 232, 96)),
        (rgb(184, 72, 216), rgb(96, 224, 248)),
        (rgb(248, 136, 32), rgb(248, 248, 248)),
        (rgb(232, 72, 168), rgb(96, 224, 248)),
        (rgb(248, 216, 48), rgb(232, 48, 48)),
        (rgb(96, 200, 248), rgb(248, 104, 184)),
    ];
    let (a, b) = cols[k % cols.len()];
    let (w, h) = (36, 44);
    let mut p = Pic::new(w, h, w / 2, h - 1);
    let cx = w / 2;
    for y in 0..h - 2 {
        let t = y as f32 / (h - 2) as f32;
        // round top, narrowing to the mouth
        let half = if t < 0.55 { (1.0 - ((0.55 - t) / 0.55).powi(2)).sqrt() * 16.0 } else { 16.0 - (t - 0.55) / 0.45 * 11.0 };
        let half = half.max(4.0) as i32;
        for x in cx - half..=cx + half {
            let u = (x - cx) as f32 / half as f32;
            let gore = ((u.asin() + 1.6) * 2.6) as i32;
            let mut c = if gore % 2 == 0 { a } else { b };
            if u > 0.55 {
                c = shade(c, 3, 4);
            } else if u < -0.6 && y > 3 {
                c = mix(c, 0xFFFFFF, 0.25);
            }
            if y == h / 2 || y == h / 2 + 1 {
                c = shade(c, 2, 3);
            }
            p.set(x, y, c);
        }
    }
    p.hline(cx - 4, cx + 4, h - 2, rgb(96, 64, 40));
    p.outline(OUTLINE);
    p
}

// ------------------------------------------------------------ props

fn pic_text(p: &mut Pic, x: i32, y: i32, s: &str, c: u32) {
    let mut cx = x;
    for ch in s.chars() {
        if let Some(g) = glyph(ch) {
            for (gy, row) in g.iter().enumerate() {
                for (gx, b) in row.bytes().enumerate() {
                    if b == b'#' {
                        p.set(cx + gx as i32, y + gy as i32, c);
                    }
                }
            }
        }
        cx += 6;
    }
}

fn prop_pic(pr: &Prop, venue: Venue) -> Option<Pic> {
    let v = pr.var as i32;
    Some(match pr.kind {
        PropKind::Lighthouse => {
            let mut p = Pic::new(20, 64, 10, 63);
            for y in 12..63 {
                let half = 4 + (y - 12) / 10;
                let band = ((y - 12) / 9) % 2 == 0;
                for x in 10 - half..=10 + half {
                    let mut c = if band { rgb(232, 48, 56) } else { rgb(248, 248, 240) };
                    if x > 10 + half / 2 {
                        c = shade(c, 3, 4);
                    }
                    p.set(x, y, c);
                }
            }
            p.rect(3, 10, 15, 2, rgb(40, 40, 56));
            p.rect(6, 3, 9, 7, rgb(56, 56, 72));
            p.rect(7, 4, 7, 5, rgb(248, 232, 120));
            p.poly(&[(5, 3), (10, -2), (16, 3)], rgb(232, 48, 56));
            p.rect(9, 54, 3, 9, rgb(56, 48, 64));
            p.rect(9, 30, 2, 3, rgb(56, 72, 104));
            p.outline(OUTLINE);
            p
        }
        PropKind::Windmill => {
            let mut p = Pic::new(26, 44, 13, 43);
            p.rect(3, 34, 20, 9, rgb(168, 72, 56));
            p.hline(3, 22, 34, rgb(200, 104, 80));
            p.rect(1, 32, 24, 2, rgb(96, 64, 48));
            p.poly(&[(5, 32), (8, 9), (18, 9), (21, 32)], if v == 0 { rgb(48, 104, 72) } else { rgb(72, 72, 88) });
            p.poly(&[(15, 32), (16, 9), (18, 9), (21, 32)], if v == 0 { rgb(32, 80, 56) } else { rgb(56, 56, 72) });
            p.poly(&[(7, 10), (13, 2), (19, 10)], rgb(120, 88, 56));
            p.rect(11, 18, 3, 4, rgb(248, 248, 240));
            p.rect(11, 38, 4, 5, rgb(64, 40, 32));
            p.outline(OUTLINE);
            p
        }
        PropKind::BeachHut => {
            let cols = [rgb(232, 56, 64), rgb(56, 136, 232), rgb(248, 200, 48)];
            let mut p = Pic::new(14, 16, 7, 15);
            for x in 1..13 {
                let c = if (x / 2) % 2 == 0 { cols[v as usize % 3] } else { rgb(248, 248, 240) };
                p.rect(x, 5, 1, 10, c);
            }
            p.poly(&[(0, 5), (7, 0), (14, 5)], rgb(88, 64, 72));
            p.rect(5, 9, 4, 6, rgb(72, 56, 64));
            p.outline(OUTLINE);
            p
        }
        PropKind::Pine => {
            let th = 20 + v * 4;
            let mut p = Pic::new(18, th + 4, 9, th + 3);
            p.rect(8, th - 3, 3, 6, rgb(96, 56, 32));
            let green = rgb(32, 120, 80);
            let dark = rgb(24, 88, 64);
            for tier in 0..3 {
                let top = tier * th / 4;
                let bot = th - 3 - (2 - tier) * th / 6;
                let half = 4 + tier * 2;
                p.poly(&[(9, top), (9 + half + 1, bot), (9 - half, bot)], green);
                p.poly(&[(9, top), (9 + half + 1, bot), (9, bot)], dark);
                p.hline(9 - half, 9 - 2, bot - 1, rgb(72, 160, 96));
            }
            if venue == Venue::Alps && v == 3 {
                p.set(9, 0, rgb(248, 248, 248));
                p.hline(7, 10, 2, rgb(248, 248, 248));
            }
            p.outline(OUTLINE);
            p
        }
        PropKind::RoundTree => {
            let mut p = Pic::new(20, 24, 10, 23);
            p.rect(9, 14, 3, 9, rgb(104, 72, 48));
            p.ellipse(10, 9, 8, 8, rgb(48, 128, 64));
            p.ellipse(7, 7, 4, 4, rgb(80, 160, 80));
            p.ellipse(13, 12, 4, 3, rgb(32, 104, 56));
            p.outline(OUTLINE);
            p
        }
        PropKind::Poplar => {
            let h = 26 + v * 3;
            let mut p = Pic::new(12, h + 2, 6, h + 1);
            p.rect(5, h - 4, 2, 5, rgb(104, 72, 48));
            for y in 0..h - 3 {
                let t = y as f32 / (h - 3) as f32;
                let half = ((t * std::f32::consts::PI).sin() * 4.0) as i32 + 1;
                p.hline(6 - half, 6 + half - 1, y, if y % 4 == 0 { rgb(80, 160, 72) } else { rgb(56, 136, 64) });
                p.set(6 + half - 1, y, rgb(40, 104, 56));
            }
            p.outline(OUTLINE);
            p
        }
        PropKind::Cactus => {
            let h = 14 + v * 3;
            let mut p = Pic::new(16, h + 2, 8, h + 1);
            let g = rgb(72, 152, 72);
            let gd = rgb(48, 112, 56);
            p.rect(6, 0, 4, h + 1, g);
            p.rect(9, 1, 1, h, gd);
            if v > 0 {
                p.rect(1, h / 2, 5, 3, g);
                p.rect(1, h / 2 - 5, 3, 6, g);
                p.rect(10, h / 3 + 2, 4, 3, g);
                p.rect(12, h / 3 - 3, 3, 6, g);
            }
            for y in (2..h).step_by(3) {
                p.set(7, y, rgb(120, 200, 104));
            }
            if v == 3 {
                p.ellipse(8, 0, 2, 1, rgb(248, 104, 168));
            }
            p.outline(OUTLINE);
            p
        }
        PropKind::Rock => {
            let mut p = Pic::new(16, 10, 8, 9);
            let c = if venue == Venue::Canyon { rgb(184, 96, 64) } else { rgb(144, 144, 160) };
            p.ellipse(8, 6, 7, 4, c);
            p.ellipse(6, 4, 3, 2, mix(c, 0xFFFFFF, 0.3));
            p.ellipse(11, 7, 3, 2, shade(c, 3, 4));
            for x in 0..16 {
                p.set(x, 9, CLEAR);
            }
            p.outline(OUTLINE);
            p
        }
        PropKind::Chapel => {
            let mut p = Pic::new(32, 44, 14, 43);
            let wall = rgb(248, 240, 224);
            let wall_d = rgb(208, 200, 192);
            p.rect(2, 26, 20, 17, wall);
            p.rect(16, 26, 6, 17, wall_d);
            p.poly(&[(0, 27), (12, 18), (24, 27)], rgb(184, 72, 56));
            p.rect(22, 14, 8, 29, wall);
            p.rect(27, 14, 3, 29, wall_d);
            p.ellipse(26, 10, 4, 4, rgb(88, 136, 96));
            p.line(26, 2, 26, 6, rgb(248, 216, 48));
            p.hline(25, 27, 3, rgb(248, 216, 48));
            p.rect(24, 18, 3, 4, rgb(56, 48, 64));
            p.rect(9, 34, 5, 9, rgb(120, 72, 40));
            p.rect(5, 30, 2, 4, rgb(96, 144, 216));
            p.rect(17, 30, 2, 4, rgb(96, 144, 216));
            p.outline(OUTLINE);
            p
        }
        PropKind::House => {
            let mut p = Pic::new(22, 24, 11, 23);
            if venue == Venue::Dunes {
                // brick with a stepped gable
                let brick = [rgb(176, 72, 56), rgb(152, 64, 48), rgb(200, 96, 64), rgb(160, 88, 72)][v as usize % 4];
                p.rect(2, 8, 18, 15, brick);
                for (k, wdt) in [(0, 18), (1, 14), (2, 10), (3, 6)] {
                    p.rect(2 + (18 - wdt) / 2, 8 - k * 2 - 2, wdt, 2, brick);
                }
                for y in (9..22).step_by(3) {
                    p.hline(2, 19, y, shade(brick, 7, 8));
                }
                p.rect(5, 12, 4, 4, rgb(248, 248, 240));
                p.rect(6, 13, 2, 2, rgb(56, 88, 136));
                p.rect(13, 12, 4, 4, rgb(248, 248, 240));
                p.rect(14, 13, 2, 2, rgb(56, 88, 136));
                p.rect(9, 17, 4, 6, rgb(40, 96, 64));
            } else {
                // a chalet with flower boxes
                p.rect(2, 10, 18, 13, rgb(248, 240, 224));
                p.rect(2, 10, 18, 5, rgb(168, 104, 56));
                p.poly(&[(0, 11), (11, 2), (22, 11)], rgb(120, 72, 40));
                p.rect(5, 16, 4, 3, rgb(96, 144, 216));
                p.rect(13, 16, 4, 3, rgb(96, 144, 216));
                p.hline(5, 8, 19, rgb(232, 56, 64));
                p.hline(13, 16, 19, rgb(232, 56, 64));
                p.rect(9, 18, 4, 5, rgb(120, 72, 40));
            }
            p.outline(OUTLINE);
            p
        }
        PropKind::Greenhouse => {
            let mut p = Pic::new(30, 13, 15, 12);
            let glass = rgb(176, 216, 232);
            p.rect(1, 4, 28, 8, glass);
            for k in 0..4 {
                p.poly(&[(1 + k * 7, 4), (4 + k * 7, 0), (8 + k * 7, 4)], mix(glass, 0xFFFFFF, 0.3));
            }
            for x in (1..29).step_by(4) {
                p.line(x, 4, x, 11, rgb(232, 240, 248));
            }
            p.hline(1, 28, 7, rgb(232, 240, 248));
            p.line(4, 5, 7, 9, rgb(248, 248, 248));
            p.outline(rgb(96, 120, 136));
            p
        }
        PropKind::Cabin => {
            let mut p = Pic::new(26, 20, 13, 19);
            p.rect(2, 8, 22, 11, rgb(152, 96, 56));
            for y in (9..19).step_by(2) {
                p.hline(2, 23, y, rgb(120, 72, 40));
            }
            p.poly(&[(0, 9), (13, 1), (26, 9)], rgb(104, 96, 104));
            p.rect(6, 11, 4, 4, rgb(248, 216, 120));
            p.rect(15, 11, 5, 8, rgb(88, 56, 32));
            p.rect(18, 2, 3, 5, rgb(120, 112, 120));
            p.outline(OUTLINE);
            p
        }
        PropKind::Sign => {
            let mut p = Pic::new(40, 22, 20, 21);
            p.rect(6, 12, 2, 10, rgb(96, 64, 40));
            p.rect(32, 12, 2, 10, rgb(96, 64, 40));
            p.rect(1, 1, 38, 12, rgb(248, 216, 48));
            p.rect(2, 2, 36, 10, rgb(232, 72, 48));
            pic_text(&mut p, 2, 3, "LAUNCH", rgb(248, 248, 240));
            p.outline(OUTLINE);
            p
        }
        PropKind::Boat => {
            let mut p = Pic::new(18, 20, 9, 18);
            p.poly(&[(1, 14), (17, 14), (14, 18), (4, 18)], rgb(232, 72, 56));
            p.line(9, 1, 9, 14, rgb(96, 64, 40));
            p.poly(&[(10, 2), (16, 13), (10, 13)], rgb(248, 248, 240));
            p.poly(&[(8, 4), (8, 13), (3, 13)], rgb(248, 216, 48));
            p.outline(OUTLINE);
            p
        }
        PropKind::Mesa => return None,
        _ => return None,
    })
}

// ------------------------------------------------------------ terrain

/// Terrain colors from the surface down: (top, surface, under, deep).
fn ground_cols(venue: Venue, s: Surf) -> [u32; 4] {
    match s {
        Surf::Sand => [rgb(248, 240, 184), rgb(240, 216, 144), rgb(216, 184, 112), rgb(184, 152, 96)],
        Surf::Grass | Surf::Tulips => match venue {
            Venue::Alps => [rgb(160, 224, 104), rgb(104, 192, 72), rgb(128, 96, 64), rgb(96, 72, 56)],
            _ => [rgb(152, 224, 96), rgb(96, 184, 64), rgb(136, 104, 64), rgb(104, 80, 56)],
        },
        Surf::Field => [rgb(168, 232, 112), rgb(112, 200, 80), rgb(136, 104, 64), rgb(104, 80, 56)],
        Surf::Snow => [rgb(248, 248, 248), rgb(224, 232, 248), rgb(168, 176, 200), rgb(128, 136, 168)],
        Surf::Rock => match venue {
            Venue::Canyon => [rgb(248, 184, 120), rgb(224, 144, 88), rgb(200, 112, 72), rgb(160, 80, 56)],
            _ => [rgb(184, 184, 192), rgb(152, 152, 168), rgb(120, 120, 136), rgb(96, 96, 112)],
        },
    }
}

const STRATA: [u32; 6] = [rgb(216, 120, 72), rgb(232, 160, 104), rgb(192, 96, 64), rgb(240, 192, 136), rgb(176, 80, 56), rgb(224, 136, 88)];

impl Scenery {
    /// Terrain and water for every screen column.
    pub fn draw_ground(&self, f: &mut Frame, c: &Course, cam: (f32, f32), cy: i32, vh: i32, t: u32) {
        let vh = vh.min(H);
        let x0 = cam.0 as i32 - 128;
        for sx in 0..W {
            let wx = x0 + sx;
            let g = c.ground_at(wx as f32);
            let top = cy - (g - cam.1).round() as i32;
            let surf = c.surf_at(wx as f32);
            let cols = ground_cols(self.venue, surf);
            let water = c.water_at(wx as f32);
            for sy in top.max(0)..vh {
                let d = sy - top;
                let alt = cam.1 + (cy - sy) as f32;
                let mut col = if d == 0 {
                    cols[0]
                } else if d < 4 {
                    cols[1]
                } else if d < 6 {
                    if (sx + sy) & 1 == 0 { cols[1] } else { cols[2] }
                } else if d < 14 {
                    cols[2]
                } else if d < 16 {
                    if (sx + sy) & 1 == 0 { cols[2] } else { cols[3] }
                } else {
                    cols[3]
                };
                if self.venue == Venue::Canyon && surf == Surf::Rock && d > 3 {
                    // layered rock: bands follow altitude, not the surface
                    let b = ((alt + 400.0) / 11.0) as i32;
                    let k = (b.rem_euclid(STRATA.len() as i32)) as usize;
                    col = STRATA[k];
                    if ((alt + 400.0) as i32).rem_euclid(11) == 0 && (sx + sy) & 1 == 0 {
                        col = STRATA[(k + 1) % STRATA.len()];
                    }
                    if d < 6 {
                        col = mix(col, cols[1], 0.5);
                    }
                }
                if surf == Surf::Tulips && (1..4).contains(&d) {
                    let row = (wx / 10).rem_euclid(4);
                    let tc = [rgb(248, 56, 72), rgb(248, 216, 48), rgb(248, 120, 184), rgb(176, 72, 216)][row as usize];
                    col = if (wx + d) % 3 == 0 { cols[1] } else { tc };
                }
                if surf == Surf::Field && d < 4 && (wx / 12) % 2 == 0 {
                    col = mix(col, rgb(208, 248, 152), 0.3);
                }
                // speckles, pebbles and a darker band of deep soil
                let hh = hash(wx, sy - top);
                if d > 2 && hh.is_multiple_of(23) {
                    col = shade(col, 7, 8);
                }
                if d > 18 && surf != Surf::Rock && hh % 97 < 2 {
                    col = rgb(168, 152, 136);
                } else if d > 18 && surf != Surf::Rock && hh % 97 == 2 {
                    col = rgb(208, 192, 176);
                }
                if d > 30 && ((alt as i32).rem_euclid(9) == 0) && (sx + sy) & 1 == 0 && surf != Surf::Rock {
                    col = shade(col, 6, 7);
                }
                f.px[(sy * W + sx) as usize] = col;
            }
            // grass blades and marram tufts sticking up
            if top > 0 && top < vh {
                let hsh = hash(wx, 77);
                match surf {
                    Surf::Sand if hsh.is_multiple_of(7) => {
                        let gh = 2 + (hsh >> 8) as i32 % 4;
                        let lean = ((hsh >> 4) % 3) as i32 - 1;
                        f.line(sx, top, sx + lean, top - gh, rgb(112, 168, 72));
                    }
                    Surf::Grass | Surf::Field | Surf::Tulips if hsh.is_multiple_of(4) => f.set(sx, top - 1, cols[1]),
                    _ => {}
                }
            }
            if let Some(level) = water
                && level > g {
                    let wtop = cy - (level - cam.1).round() as i32;
                    for sy in wtop.max(0)..top.min(vh) {
                        let d = sy - wtop;
                        let mut col = match d {
                            0 => rgb(200, 232, 248),
                            1..=2 => rgb(96, 168, 224),
                            3..=6 => rgb(72, 136, 208),
                            _ => rgb(56, 112, 184),
                        };
                        if self.venue == Venue::Alps {
                            col = mix(col, rgb(48, 160, 160), 0.35);
                        }
                        if d > 0 && d < 5 && hash(wx + (t / 12) as i32 * 3, sy).is_multiple_of(17) {
                            col = rgb(232, 248, 248);
                        }
                        f.px[(sy * W + sx) as usize] = col;
                    }
                }
        }
    }
}
