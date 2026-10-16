//! Backdrops for the three stages. The sky is drawn once into a cached
//! frame; the far, mid and near layers are pre-rendered wrapping strips
//! that scroll at their own parallax speed; windmill sails, cows, clouds,
//! water, rain and spray are animated on top every frame.

use super::art::{Art, PROPS};
use super::data::Venue;
use crate::engine::gfx::*;
use crate::engine::rng::Rng;
use crate::engine::ui::OUTLINE;

/// Marks a see-through pixel in a strip (never produced by `rgb`).
pub const KEY: u32 = 0x010203;

pub const FAR_Y: i32 = 74;
pub const FAR_H: i32 = 46;
pub const MID_Y: i32 = 96;
pub const MID_H: i32 = 46;
pub const NEAR_Y: i32 = 126;
pub const NEAR_H: i32 = 22;
pub const ROAD_TOP: i32 = 146;
pub const ROAD_BOT: i32 = 190;
pub const WATER_Y: i32 = 195;

pub const FAR_P: f32 = 0.12;
pub const MID_P: f32 = 0.35;
pub const NEAR_P: f32 = 0.75;

pub struct Strip {
    pub w: i32,
    pub h: i32,
    pub y0: i32,
    pub px: Vec<u32>,
    pub opaque: bool,
}

impl Strip {
    /// Paint a wrapping strip of `period` pixels. `paint(frame, ox)` draws
    /// the whole strip shifted left by `ox`; it is called once per 256 px
    /// tile, so the strip is built from ordinary frame drawing.
    fn build(period: i32, y0: i32, h: i32, opaque: bool, mut paint: impl FnMut(&mut Frame, i32)) -> Strip {
        let mut px = vec![KEY; (period * h) as usize];
        let mut f = Frame::new();
        let tiles = (period + W - 1) / W;
        for tile in 0..tiles {
            f.clear(KEY);
            paint(&mut f, tile * W);
            // wrap: things near the end also show at the start
            paint(&mut f, tile * W - period);
            paint(&mut f, tile * W + period);
            for y in 0..h {
                for x in 0..W {
                    let sx = tile * W + x;
                    if sx < period {
                        let c = f.get(x, y0 + y);
                        if c != KEY {
                            px[(y * period + sx) as usize] = c;
                        }
                    }
                }
            }
        }
        Strip { w: period, h, y0, px, opaque }
    }

    pub fn draw(&self, f: &mut Frame, scroll: f32, clip: (i32, i32)) {
        let off = (scroll.floor() as i32).rem_euclid(self.w);
        for y in 0..self.h {
            let sy = self.y0 + y;
            if sy < clip.0 || sy >= clip.1 {
                continue;
            }
            let row = &self.px[(y * self.w) as usize..((y + 1) * self.w) as usize];
            let dst = &mut f.px[(sy * W) as usize..((sy + 1) * W) as usize];
            let mut sx = off as usize;
            let w = self.w as usize;
            if self.opaque {
                for d in dst.iter_mut() {
                    *d = row[sx];
                    sx += 1;
                    if sx == w {
                        sx = 0;
                    }
                }
            } else {
                for d in dst.iter_mut() {
                    let c = row[sx];
                    if c != KEY {
                        *d = c;
                    }
                    sx += 1;
                    if sx == w {
                        sx = 0;
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Mill {
    x: i32,
    y: i32,
}

pub struct Scenery {
    pub venue: Venue,
    pub sky: Frame,
    clouds: Strip,
    far: Strip,
    mid: Strip,
    near: Strip,
    far_mills: Vec<Mill>,
    mid_mills: Vec<Mill>,
    cows: Vec<(i32, i32, u8)>,
    birds: Vec<(f32, f32, f32)>,
    rain: Vec<(f32, f32, f32)>,
    pub asphalt: Strip,
    pub cobbles: Strip,
    pub road: u32,
    pub road_d: u32,
    pub road_l: u32,
    pub line: u32,
    pub water: [u32; 3],
}

const MID_W: i32 = 768;
const FAR_W: i32 = 512;
const NEAR_W: i32 = 384;
const CLOUD_W: i32 = 640;

impl Scenery {
    pub fn new(venue: Venue, seed: u64) -> Self {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let mut sky = Frame::new();
        let (road, road_d, road_l, line, water) = match venue {
            Venue::Polder => (rgb(176, 72, 64), rgb(144, 56, 56), rgb(200, 96, 80), rgb(240, 232, 216), [rgb(40, 88, 96), rgb(56, 112, 112), rgb(88, 144, 136)]),
            Venue::Amsterdam => (rgb(168, 64, 64), rgb(128, 48, 56), rgb(192, 88, 80), rgb(240, 232, 216), [rgb(40, 64, 88), rgb(56, 88, 112), rgb(96, 128, 152)]),
            Venue::Dijk => (rgb(120, 120, 128), rgb(96, 96, 104), rgb(144, 144, 152), rgb(232, 232, 224), [rgb(64, 88, 96), rgb(88, 112, 120), rgb(128, 152, 152)]),
        };
        match venue {
            Venue::Polder => sky.band_gradient(0, 120, &[rgb(56, 104, 200), rgb(72, 128, 216), rgb(96, 152, 224), rgb(128, 176, 232), rgb(160, 200, 240), rgb(200, 224, 240)]),
            Venue::Amsterdam => sky.band_gradient(0, 120, &[rgb(80, 104, 160), rgb(104, 128, 176), rgb(128, 152, 192), rgb(160, 176, 200), rgb(192, 200, 208), rgb(216, 216, 208)]),
            Venue::Dijk => sky.band_gradient(0, 120, &[rgb(40, 48, 72), rgb(56, 64, 88), rgb(72, 80, 104), rgb(96, 104, 120), rgb(120, 128, 136), rgb(152, 160, 160)]),
        }
        if venue == Venue::Polder {
            // a pale sun high up
            sky.fill_ellipse(206, 44, 10, 10, rgb(248, 248, 216));
            sky.fill_ellipse(206, 44, 8, 8, rgb(248, 248, 240));
        }
        let clouds = Self::make_clouds(venue, &mut rng);
        let (far, far_mills) = Self::make_far(venue, &mut rng);
        let (mid, mid_mills, cows) = Self::make_mid(venue, &mut rng);
        let near = Self::make_near(venue, &mut rng);
        let birds = (0..6).map(|_| (rng.range(0.0, 256.0), rng.range(40.0, 90.0), rng.range(0.3, 0.8))).collect();
        let rain = if venue == Venue::Dijk { (0..70).map(|_| (rng.range(0.0, 256.0), rng.range(0.0, 224.0), rng.range(3.0, 5.0))).collect() } else { Vec::new() };
        let asphalt = Self::make_road(venue, false, road, road_d, road_l, line, &mut rng);
        let cobbles = Self::make_road(venue, true, road, road_d, road_l, line, &mut rng);
        Scenery { venue, sky, clouds, far, mid, near, far_mills, mid_mills, cows, birds, rain, asphalt, cobbles, road, road_d, road_l, line, water }
    }

    fn make_clouds(venue: Venue, rng: &mut Rng) -> Strip {
        let (lit, body, dark) = match venue {
            Venue::Polder => (rgb(248, 248, 248), rgb(224, 232, 248), rgb(168, 184, 216)),
            Venue::Amsterdam => (rgb(232, 232, 240), rgb(200, 208, 224), rgb(152, 160, 184)),
            Venue::Dijk => (rgb(112, 120, 136), rgb(88, 96, 112), rgb(56, 64, 80)),
        };
        let n = if venue == Venue::Dijk { 9 } else { 6 };
        let cl: Vec<(i32, i32, i32)> = (0..n).map(|k| (k * CLOUD_W / n + rng.int(40), 40 + rng.int(36), 14 + rng.int(12))).collect();
        let cow_cloud = venue == Venue::Polder;
        Strip::build(CLOUD_W, 30, 70, false, |f, ox| {
            for (k, &(x, y, r)) in cl.iter().enumerate() {
                let x = x - ox;
                if cow_cloud && k == 2 {
                    // a cow-shaped cloud, of course
                    f.fill_ellipse(x, y, 16, 7, body);
                    f.fill_ellipse(x + 15, y - 4, 6, 5, body);
                    f.fill_ellipse(x, y - 2, 14, 5, lit);
                    f.fill_ellipse(x + 15, y - 6, 4, 3, lit);
                    for lx in [-11, -6, 6, 11] {
                        f.rect(x + lx, y + 5, 3, 7, body);
                    }
                    f.rect(x + 19, y - 11, 2, 3, body);
                    f.rect(x + 12, y - 11, 2, 3, body);
                    f.hline(x - 15, x + 13, y + 6, dark);
                    continue;
                }
                // cumulus: lit puffs on top, flat shaded base
                f.fill_ellipse(x, y, r + 6, r / 2 + 2, dark);
                f.fill_ellipse(x - r / 2, y - 3, r / 2 + 3, r / 2, body);
                f.fill_ellipse(x + r / 2, y - 4, r / 2 + 4, r / 2 + 2, body);
                f.fill_ellipse(x, y - r / 2, r / 2 + 2, r / 2 + 1, body);
                f.fill_ellipse(x + 1, y - r / 2 - 2, r / 2, r / 2 - 1, lit);
                f.fill_ellipse(x + r / 2 + 1, y - 6, r / 3 + 1, r / 4 + 1, lit);
                f.rect(x - r - 6, y + r / 2 - 1, 2 * r + 12, 4, KEY);
                f.hline(x - r - 2, x + r + 2, y + r / 2 - 2, dark);
            }
        })
    }

    fn make_far(venue: Venue, rng: &mut Rng) -> (Strip, Vec<Mill>) {
        let mut mills = Vec::new();
        let strip = match venue {
            Venue::Polder => {
                let trees: Vec<(i32, i32)> = (0..26).map(|_| (rng.int(FAR_W), 3 + rng.int(4))).collect();
                for k in 0..6 {
                    mills.push(Mill { x: 40 + k * 22 + rng.int(6), y: 112 });
                }
                mills.push(Mill { x: 380, y: 112 });
                let church = 300;
                Strip::build(FAR_W, FAR_Y, FAR_H, false, |f, ox| {
                    let haze = rgb(136, 168, 200);
                    let haze2 = rgb(112, 144, 184);
                    for &(x, r) in &trees {
                        f.fill_ellipse(x - ox, 112 - r, r + 2, r, haze);
                    }
                    // church with a tall spire
                    let cx = church - ox;
                    f.rect(cx - 6, 100, 12, 13, haze2);
                    f.fill_poly(&[(cx - 3, 100), (cx + 3, 100), (cx, 80)], haze2);
                    f.rect(cx + 6, 104, 14, 9, haze2);
                    f.fill_poly(&[(cx + 6, 104), (cx + 20, 104), (cx + 13, 99)], haze2);
                    // farm with a big roof
                    f.fill_poly(&[(160 - ox, 112), (190 - ox, 112), (184 - ox, 102), (166 - ox, 102)], haze2);
                    f.rect(166 - ox, 99, 3, 4, haze2);
                    f.rect(0 - ox, 112, FAR_W + 2, 8, rgb(104, 152, 88));
                    f.dither_rect(0 - ox, 112, FAR_W + 2, 1, rgb(104, 152, 88), haze);
                })
            }
            Venue::Amsterdam => {
                let roofs: Vec<(i32, i32, i32)> = (0..30).map(|k| (k * 18 + rng.int(6), 8 + rng.int(14), 10 + rng.int(8))).collect();
                Strip::build(FAR_W, FAR_Y, FAR_H, false, |f, ox| {
                    let c = rgb(136, 144, 176);
                    for &(x, h, w) in &roofs {
                        let x = x - ox;
                        f.rect(x, 118 - h, w, h + 2, c);
                        f.fill_poly(&[(x, 118 - h), (x + w, 118 - h), (x + w / 2, 112 - h)], c);
                    }
                    // Westertoren: tower with a crown
                    let tx = 200 - ox;
                    let t2 = rgb(120, 128, 168);
                    f.rect(tx - 5, 92, 10, 26, t2);
                    f.rect(tx - 4, 84, 8, 8, t2);
                    f.rect(tx - 3, 78, 6, 6, t2);
                    f.fill_poly(&[(tx - 3, 78), (tx + 3, 78), (tx, 70)], t2);
                    f.set(tx, 69, rgb(216, 192, 96));
                    f.set(tx - 1, 70, rgb(216, 192, 96));
                    f.set(tx + 1, 70, rgb(216, 192, 96));
                    // a church dome far off
                    f.rect(420 - ox, 100, 18, 18, t2);
                    f.fill_ellipse(429 - ox, 100, 8, 7, t2);
                })
            }
            Venue::Dijk => {
                Strip::build(FAR_W, FAR_Y, FAR_H, false, |f, ox| {
                    // the Waddenzee: dark grey-green sea up to the horizon
                    f.rect(0, 104, W, 16, rgb(72, 96, 104));
                    f.rect(0, 104, W, 2, rgb(96, 112, 120));
                    f.dither_rect(0, 106, W, 2, rgb(96, 112, 120), rgb(72, 96, 104));
                    f.rect(0, 112, W, 8, rgb(64, 88, 96));
                    // a coaster on the horizon and a far wind farm
                    let sx = 140 - ox;
                    f.rect(sx, 100, 22, 4, rgb(64, 64, 80));
                    f.rect(sx + 14, 96, 6, 4, rgb(64, 64, 80));
                    f.rect(sx + 3, 97, 2, 3, rgb(64, 64, 80));
                    for k in 0..4 {
                        let wx = 340 + k * 24 - ox;
                        f.line(wx, 104, wx, 88, rgb(136, 144, 152));
                        f.line(wx, 88, wx + 5, 85, rgb(136, 144, 152));
                        f.line(wx, 88, wx - 4, 84, rgb(136, 144, 152));
                        f.line(wx, 88, wx, 94, rgb(136, 144, 152));
                    }
                })
            }
        };
        (strip, mills)
    }

    #[allow(clippy::type_complexity)]
    fn make_mid(venue: Venue, rng: &mut Rng) -> (Strip, Vec<Mill>, Vec<(i32, i32, u8)>) {
        let mut mills = Vec::new();
        let mut cows = Vec::new();
        let strip = match venue {
            Venue::Polder => {
                mills.push(Mill { x: 120, y: 132 });
                mills.push(Mill { x: 540, y: 128 });
                for k in 0..7 {
                    cows.push((200 + k * 70 + rng.int(30), 128 + rng.int(9), (k % 3) as u8));
                }
                let willows: Vec<i32> = (0..8).map(|k| k * 96 + rng.int(30)).collect();
                let farm = 360;
                Strip::build(MID_W, MID_Y, MID_H, false, |f, ox| {
                    // meadow in strips, with ditches running into the distance
                    let g1 = rgb(96, 168, 72);
                    let g2 = rgb(112, 184, 80);
                    f.rect(0, 118, W, 24, g1);
                    for (k, y) in [118, 121, 125, 130, 136].iter().enumerate() {
                        let h = [3, 4, 5, 6, 6][k];
                        if k % 2 == 1 {
                            f.dither_rect(0, *y, W, h, g1, g2);
                        } else {
                            f.rect(0, *y, W, h, if k % 4 == 0 { g2 } else { g1 });
                        }
                    }
                    for k in 0..12 {
                        // ditches: blue lines fanning out toward us
                        let x0 = k * 64 - ox;
                        f.line(x0, 118, x0 - 22, 142, rgb(88, 136, 184));
                        f.line(x0 + 1, 118, x0 - 21, 142, rgb(144, 184, 216));
                    }
                    // farmhouse with a red roof and a haystack
                    let fx = farm - ox;
                    f.rect(fx, 106, 34, 14, rgb(232, 224, 208));
                    f.fill_poly(&[(fx - 4, 107), (fx + 38, 107), (fx + 30, 92), (fx + 4, 92)], rgb(184, 64, 48));
                    f.hline(fx + 4, fx + 30, 92, rgb(216, 96, 72));
                    for wx in [fx + 4, fx + 13, fx + 24] {
                        f.rect(wx, 110, 5, 5, rgb(48, 72, 104));
                        f.rect(wx + 2, 110, 1, 5, rgb(232, 224, 208));
                    }
                    f.rect(fx + 30, 111, 4, 9, rgb(56, 104, 64));
                    f.fill_ellipse(fx + 50, 116, 7, 6, rgb(216, 184, 88));
                    f.hline(fx + 44, fx + 56, 113, rgb(184, 152, 64));
                    // pollard willows (knotwilgen) along a ditch
                    for &wx in &willows {
                        let x = wx - ox;
                        f.rect(x - 1, 112, 3, 9, rgb(104, 88, 64));
                        f.fill_ellipse(x, 110, 6, 4, rgb(120, 160, 104));
                        f.fill_ellipse(x - 2, 108, 3, 2, rgb(152, 192, 128));
                        f.set(x - 4, 112, rgb(120, 160, 104));
                        f.set(x + 4, 113, rgb(120, 160, 104));
                    }
                })
            }
            Venue::Amsterdam => {
                let houses: Vec<(i32, i32, i32, u8, u8)> = {
                    let mut v = Vec::new();
                    let mut x = 0;
                    while x < MID_W {
                        let w = 18 + rng.int(4) * 2;
                        let h = 28 + rng.int(14);
                        v.push((x, w, h, rng.int(5) as u8, rng.int(4) as u8));
                        x += w;
                    }
                    v
                };
                let trees: Vec<i32> = (0..10).map(|k| k * 80 + rng.int(20)).collect();
                Strip::build(MID_W, MID_Y, MID_H, false, |f, ox| {
                    let walls = [rgb(136, 64, 48), rgb(88, 56, 48), rgb(72, 96, 80), rgb(216, 208, 192), rgb(160, 88, 64)];
                    for &(x, w, h, col, gable) in &houses {
                        let x = x - ox;
                        let top = 134 - h;
                        let c = walls[col as usize];
                        f.rect(x, top, w, h + 6, c);
                        f.rect(x + w - 1, top, 1, h + 6, shade(c, 3, 4));
                        // gables: step, neck, bell, spout
                        match gable {
                            0 => {
                                f.rect(x + 3, top - 4, w - 6, 4, c);
                                f.rect(x + 6, top - 8, w - 12, 4, c);
                            }
                            1 => {
                                f.rect(x + w / 2 - 4, top - 9, 8, 9, c);
                                f.fill_poly(&[(x + 2, top), (x + w / 2 - 4, top), (x + w / 2 - 4, top - 5)], c);
                                f.fill_poly(&[(x + w - 2, top), (x + w / 2 + 4, top), (x + w / 2 + 4, top - 5)], c);
                            }
                            2 => {
                                f.fill_ellipse(x + w / 2, top, w / 2 - 2, 7, c);
                            }
                            _ => {
                                f.fill_poly(&[(x, top), (x + w, top), (x + w / 2, top - 9)], c);
                            }
                        }
                        // white trim, hoist beam, windows in rows
                        f.hline(x + 1, x + w - 2, top, rgb(232, 232, 224));
                        f.rect(x + w / 2, top - 3, 1, 3, rgb(56, 40, 32));
                        let mut wy = top + 4;
                        while wy < 128 {
                            for k in 0..2 {
                                let wx = x + 3 + k * (w - 10);
                                f.rect(wx, wy, 4, 5, rgb(232, 232, 224));
                                f.rect(wx + 1, wy + 1, 2, 3, rgb(56, 72, 104));
                            }
                            wy += 9;
                        }
                        f.rect(x + w / 2 - 2, 128, 4, 6, rgb(48, 40, 40));
                    }
                    // a quay with elms in front of the houses
                    f.rect(0, 134, W, 8, rgb(120, 112, 112));
                    f.hline(0, W - 1, 134, rgb(160, 152, 152));
                    for &tx in &trees {
                        let x = tx - ox;
                        f.rect(x, 116, 2, 18, rgb(72, 56, 48));
                        f.fill_ellipse(x + 1, 112, 9, 8, rgb(72, 120, 72));
                        f.fill_ellipse(x - 2, 109, 5, 4, rgb(96, 152, 88));
                    }
                })
            }
            Venue::Dijk => {
                // the seaward slope of the dike: basalt blocks, then grass with sheep
                for k in 0..9 {
                    cows.push((40 + k * 84 + rng.int(30), 132 + rng.int(6), 3));
                }
                let monument = 600;
                Strip::build(MID_W, MID_Y, MID_H, false, |f, ox| {
                    f.rect(0, 116, W, 6, rgb(64, 64, 72));
                    for x in (0..W + 8).step_by(6) {
                        let xx = x - (ox % 6);
                        f.rect(xx, 116, 5, 2, rgb(88, 88, 96));
                        f.rect(xx + 3, 119, 5, 2, rgb(80, 80, 88));
                    }
                    f.rect(0, 122, W, 20, rgb(104, 144, 80));
                    f.dither_rect(0, 122, W, 2, rgb(104, 144, 80), rgb(64, 64, 72));
                    f.dither_rect(0, 130, W, 2, rgb(104, 144, 80), rgb(120, 160, 88));
                    // the Vlieter monument tower where the dike was closed
                    let mx = monument - ox;
                    f.rect(mx, 92, 10, 30, rgb(184, 176, 160));
                    f.rect(mx + 2, 88, 6, 4, rgb(184, 176, 160));
                    f.rect(mx + 10, 100, 2, 22, rgb(144, 136, 128));
                    for wy in (96..118).step_by(6) {
                        f.rect(mx + 4, wy, 2, 3, rgb(64, 64, 80));
                    }
                })
            }
        };
        (strip, mills, cows)
    }

    fn make_near(venue: Venue, rng: &mut Rng) -> Strip {
        match venue {
            Venue::Polder => {
                let flowers: Vec<(i32, i32, u8)> = (0..60).map(|_| (rng.int(NEAR_W), 136 + rng.int(9), rng.int(3) as u8)).collect();
                Strip::build(NEAR_W, NEAR_Y, NEAR_H, false, |f, ox| {
                    // grass verge with a wooden fence and flowers
                    f.rect(0, 136, W, 12, rgb(80, 152, 64));
                    f.dither_rect(0, 136, W, 2, rgb(80, 152, 64), rgb(112, 184, 80));
                    f.dither_rect(0, 145, W, 1, rgb(80, 152, 64), rgb(56, 112, 48));
                    for k in 0..(NEAR_W / 24 + 2) {
                        let x = k * 24 - ox;
                        f.rect(x, 128, 2, 12, rgb(104, 72, 48));
                        f.set(x, 128, rgb(144, 104, 64));
                    }
                    f.hline(0, W - 1, 131, rgb(168, 128, 80));
                    f.hline(0, W - 1, 135, rgb(168, 128, 80));
                    for &(x, y, c) in &flowers {
                        let col = [rgb(248, 232, 64), rgb(248, 248, 248), rgb(248, 120, 160)][c as usize];
                        f.set(x - ox, y, col);
                    }
                })
            }
            Venue::Amsterdam => Strip::build(NEAR_W, NEAR_Y, NEAR_H, false, |f, ox| {
                // pavement with a row of parked bikes and lamp posts
                f.rect(0, 140, W, 8, rgb(152, 144, 136));
                f.hline(0, W - 1, 140, rgb(184, 176, 168));
                for x in (0..W + 12).step_by(12) {
                    f.set(x - (ox % 12), 144, rgb(128, 120, 112));
                }
                let cols = [rgb(48, 48, 56), rgb(56, 104, 168), rgb(184, 56, 56), rgb(56, 128, 88), rgb(216, 216, 216)];
                for k in 0..(NEAR_W / 14) {
                    let x = k * 14 + 4 - ox;
                    let c = cols[(k as usize * 7) % 5];
                    f.circle_outline(x, 138, 3, rgb(40, 40, 48));
                    f.circle_outline(x + 9, 138, 3, rgb(40, 40, 48));
                    f.line(x, 138, x + 4, 134, c);
                    f.line(x + 4, 134, x + 9, 138, c);
                    f.line(x + 4, 134, x + 8, 134, c);
                    f.line(x + 8, 134, x + 9, 138, c);
                    f.line(x + 4, 134, x + 3, 132, c);
                    f.hline(x + 2, x + 4, 131, rgb(40, 40, 48));
                    f.line(x + 8, 134, x + 9, 131, rgb(176, 176, 184));
                }
                for k in 0..(NEAR_W / 96) {
                    let x = k * 96 + 50 - ox;
                    f.rect(x, 120, 2, 22, rgb(40, 48, 48));
                    f.rect(x - 2, 118, 6, 3, rgb(40, 48, 48));
                    f.rect(x - 1, 121, 4, 2, rgb(248, 232, 168));
                }
            }),
            Venue::Dijk => Strip::build(NEAR_W, NEAR_Y, NEAR_H, false, |f, ox| {
                // the dike crest: grass with a wire fence and a few posts
                f.rect(0, 138, W, 10, rgb(96, 136, 72));
                f.dither_rect(0, 138, W, 2, rgb(96, 136, 72), rgb(120, 160, 88));
                f.dither_rect(0, 146, W, 2, rgb(96, 136, 72), rgb(72, 104, 56));
                for k in 0..(NEAR_W / 32 + 2) {
                    let x = k * 32 - ox;
                    f.rect(x, 128, 2, 12, rgb(120, 104, 88));
                }
                f.hline(0, W - 1, 131, rgb(160, 160, 160));
                f.hline(0, W - 1, 135, rgb(160, 160, 160));
            }),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn make_road(venue: Venue, cobble: bool, road: u32, road_d: u32, road_l: u32, line: u32, rng: &mut Rng) -> Strip {
        let specks: Vec<(i32, i32, bool)> = (0..260).map(|_| (rng.int(256), ROAD_TOP + 3 + rng.int(40), rng.f() < 0.5)).collect();
        let verge = match venue {
            Venue::Polder => (rgb(80, 152, 64), rgb(56, 112, 48)),
            Venue::Amsterdam => (rgb(152, 144, 136), rgb(112, 104, 104)),
            Venue::Dijk => (rgb(96, 136, 72), rgb(72, 104, 56)),
        };
        Strip::build(256, ROAD_TOP, WATER_Y - ROAD_TOP, true, |f, ox| {
            if ox != 0 {
                return;
            }
            // curb stones along the far edge
            f.rect(0, ROAD_TOP, W, 2, rgb(184, 176, 168));
            for x in (0..W).step_by(8) {
                f.set(x, ROAD_TOP, rgb(136, 128, 120));
                f.set(x, ROAD_TOP + 1, rgb(136, 128, 120));
            }
            if cobble {
                // Amsterdam cobbles: rows of rounded stones
                f.rect(0, ROAD_TOP + 2, W, ROAD_BOT - ROAD_TOP - 2, rgb(104, 96, 96));
                let mut y = ROAD_TOP + 2;
                let mut row = 0;
                while y < ROAD_BOT {
                    let h = 3 + (y - ROAD_TOP) / 16;
                    let off = if row % 2 == 0 { 0 } else { 3 };
                    let mut x = -off;
                    while x < W {
                        f.rect(x + 1, y, 5, h - 1, rgb(152, 144, 136));
                        f.hline(x + 1, x + 4, y, rgb(184, 176, 168));
                        f.set(x + 5, y + h - 2, rgb(120, 112, 112));
                        x += 6;
                    }
                    y += h;
                    row += 1;
                }
            } else {
                f.rect(0, ROAD_TOP + 2, W, ROAD_BOT - ROAD_TOP - 2, road);
                f.dither_rect(0, ROAD_TOP + 2, W, 2, road, road_d);
                for &(x, y, l) in &specks {
                    f.set(x, y, if l { road_l } else { road_d });
                }
                // the dashed line between the lanes
                for x in (0..W).step_by(24) {
                    f.rect(x, 167, 14, 2, line);
                    f.hline(x, x + 13, 169, road_d);
                }
                // a painted bike symbol now and then (it is a fietspad)
                if venue != Venue::Dijk {
                    let (bx, by) = (180, 182);
                    f.circle_outline(bx, by, 3, line);
                    f.circle_outline(bx + 10, by, 3, line);
                    f.line(bx, by, bx + 4, by - 4, line);
                    f.line(bx + 4, by - 4, bx + 10, by, line);
                    f.line(bx + 4, by - 4, bx + 8, by - 4, line);
                }
            }
            // near curb and verge, the canal bank below it
            f.rect(0, ROAD_BOT, W, 2, rgb(200, 192, 184));
            f.rect(0, ROAD_BOT + 2, W, 3, verge.0);
            f.dither_rect(0, ROAD_BOT + 4, W, 1, verge.0, verge.1);
            for x in (0..W).step_by(8) {
                f.set(x + 3, ROAD_BOT, rgb(152, 144, 136));
            }
        })
    }

    /// The road, with cobbles where `cobbled(screen x)` says so.
    pub fn draw_road(&self, f: &mut Frame, cam: f32, clip: (i32, i32), cobbled: impl Fn(f32) -> bool) {
        self.asphalt.draw(f, cam, clip);
        // cobble columns copied over the asphalt
        let off = cam.floor() as i32;
        let mut any = false;
        for x in (0..W).step_by(16) {
            if cobbled((off + x) as f32) {
                any = true;
                break;
            }
        }
        if !any && !cobbled((off + W - 1) as f32) {
            return;
        }
        for x in 0..W {
            if !cobbled((off + x) as f32) {
                continue;
            }
            let sx = (off + x).rem_euclid(256);
            for y in ROAD_TOP + 2..ROAD_BOT {
                if y >= clip.0 && y < clip.1 {
                    f.px[(y * W + x) as usize] = self.cobbles.px[((y - ROAD_TOP) * 256 + sx) as usize];
                }
            }
        }
    }

    /// Wind strength for the mills, headwind (+) for cloud drift.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&self, f: &mut Frame, art: &Art, cam: f32, t: u32, wind: f32, head: f32, clip: (i32, i32)) {
        // sky (cached)
        let (y0, y1) = (clip.0.max(0), clip.1.min(ROAD_TOP));
        if y1 > y0 {
            let (s, e) = ((y0 * W) as usize, (y1.min(120) * W) as usize);
            if e > s {
                f.px[s..e].copy_from_slice(&self.sky.px[s..e]);
            }
        }
        let tt = t as f32;
        let drift = tt * (0.08 + 0.12 * head.max(-0.5));
        self.clouds.draw(f, cam * 0.04 + drift, clip);
        if self.venue == Venue::Polder {
            // geese flying over in a V
            let gx = ((tt * 0.4) as i32 % 600) - 60;
            for k in 0..5 {
                let (x, y) = (256 - gx + k * 6, 46 + (k - 2).abs() * 3);
                let flap = ((t / 8 + k as u32) % 2) as i32;
                f.set(x - 1, y - flap, rgb(64, 72, 96));
                f.set(x, y, rgb(64, 72, 96));
                f.set(x + 1, y - flap, rgb(64, 72, 96));
            }
        }
        self.far.draw(f, cam * FAR_P, clip);
        let spin = tt * (0.02 + wind * 0.06);
        for m in &self.far_mills {
            let x = (m.x as f32 - cam * FAR_P).rem_euclid(FAR_W as f32) as i32;
            for xx in [x, x - FAR_W, x + FAR_W] {
                if (-20..W + 20).contains(&xx) {
                    far_mill(f, art, xx, m.y, spin + m.x as f32, clip);
                }
            }
        }
        if self.venue == Venue::Dijk {
            // whitecaps rolling in
            for k in 0..14 {
                let x = ((k * 37 + (tt * (0.3 + 0.05 * k as f32)) as i32) % 280) - 12;
                let y = 106 + (k * 5) % 13;
                if (clip.0..clip.1).contains(&y) {
                    f.hline(x, x + 2 + k % 3, y, rgb(200, 216, 216));
                }
            }
        }
        self.mid.draw(f, cam * MID_P, clip);
        if self.venue == Venue::Dijk {
            // waves bursting over the basalt in white spray
            for k in 0..7u32 {
                let period = 70 + k * 17;
                let ph = (t + k * 41) % period;
                if ph >= 30 {
                    continue;
                }
                let base = (k as f32 * 131.0 - cam * MID_P).rem_euclid(W as f32 + 60.0) as i32 - 30;
                let rise = (ph as i32 * 2).min(22);
                let fade = ph as i32;
                for b in 0..5 {
                    let bx = base + b * 4 - 8 + (b * fade) / 5;
                    let by = 118 - rise + (b % 3) * 3 + (fade * fade) / 50;
                    let r = 1 + fade / 9;
                    if by - r < clip.0 || by + r >= clip.1 {
                        continue;
                    }
                    if fade > 18 {
                        // falling apart into droplets
                        for d in 0..3 {
                            f.set(bx + d * 2 - 2, by + d, rgb(208, 224, 232));
                        }
                    } else {
                        f.fill_ellipse(bx, by, r + 1, r, rgb(248, 248, 248));
                        f.set(bx - r, by + r, rgb(200, 216, 224));
                        f.set(bx + r + 1, by - 1, rgb(200, 216, 224));
                    }
                }
            }
        }
        for m in &self.mid_mills {
            let x = (m.x as f32 - cam * MID_P).rem_euclid(MID_W as f32) as i32;
            for xx in [x, x - MID_W, x + MID_W] {
                if (-40..W + 40).contains(&xx) {
                    big_mill(f, art, xx, m.y, spin * 0.8 + 1.0, clip);
                }
            }
        }
        for &(cx, cy, var) in &self.cows {
            let x = (cx as f32 - cam * MID_P).rem_euclid(MID_W as f32) as i32;
            if !(-30..W + 30).contains(&x) || cy - 16 < clip.0 || cy > clip.1 {
                continue;
            }
            if var == 3 {
                let s = &art.sheep[((t / 30) as usize + cx as usize) % 2];
                s.draw_foot(f, x, cy, &PROPS, cx % 2 == 0);
            } else {
                let graze = ((t / 90) as i32 + cx) % 3 != 0;
                let b = if graze { &art.cow_graze } else { &art.cow };
                b.draw_foot(f, x, cy, &PROPS, var == 1);
            }
        }
        self.near.draw(f, cam * NEAR_P, clip);
    }

    /// Water in front of the road: bands, ripples moving with the camera.
    pub fn draw_water(&self, f: &mut Frame, cam: f32, t: u32, clip: (i32, i32)) {
        let top = WATER_Y;
        for y in top.max(clip.0)..H.min(clip.1) {
            let k = y - top;
            let c = if k < 2 { self.water[2] } else if k < 9 { self.water[1] } else { self.water[0] };
            let row = &mut f.px[(y * W) as usize..((y + 1) * W) as usize];
            row.fill(c);
            if k == 2 || k == 9 {
                for x in (0..W).step_by(2) {
                    f.px[(y * W + x + (y & 1)) as usize] = self.water[if k == 2 { 2 } else { 1 }];
                }
            }
        }
        // ripples
        let off = cam as i32;
        for k in 0..18 {
            let y = top + 3 + (k * 7) % 26;
            if y < clip.0 || y >= clip.1 {
                continue;
            }
            let x = ((k * 71 - off + (t as i32 / 3) * (1 + k % 2)).rem_euclid(300)) - 20;
            f.hline(x, x + 3 + k % 4, y, self.water[2]);
        }
        if self.venue == Venue::Dijk {
            for k in 0..8 {
                let x = ((k * 43 - off + t as i32).rem_euclid(290)) - 16;
                let y = top + 6 + (k * 9) % 20;
                if y >= clip.0 && y < clip.1 {
                    f.hline(x, x + 4, y, rgb(200, 216, 216));
                }
            }
        }
        // reeds on the near bank
        if self.venue != Venue::Amsterdam {
            for k in 0..20 {
                let x = ((k * 53 - (cam * 1.15) as i32).rem_euclid(300)) - 20;
                let sway = (((t as f32) * 0.05 + k as f32).sin() * 1.5) as i32;
                let h = 8 + k % 5;
                let y = H - 1;
                if y - h >= clip.0 && y < clip.1 + 1 {
                    f.line(x, y, x + sway, y - h, rgb(64, 120, 56));
                    f.line(x + 2, y, x + 2 + sway, y - h + 3, rgb(88, 144, 64));
                    f.rect(x + sway, y - h - 2, 1, 3, rgb(120, 72, 40));
                }
            }
        }
    }

    /// Rain and sea spray over everything (the storm on the dike).
    pub fn draw_weather(&self, f: &mut Frame, t: u32, gust: f32, clip: (i32, i32)) {
        if self.rain.is_empty() {
            return;
        }
        let tt = t as f32;
        for &(x, y, s) in &self.rain {
            let yy = (y + tt * s).rem_euclid(224.0) as i32;
            let xx = (x - tt * s * 0.6 * gust).rem_euclid(256.0) as i32;
            if yy >= clip.0 && yy + 4 < clip.1 {
                f.line(xx, yy, xx - 2, yy + 4, rgb(160, 176, 192));
            }
        }
    }

    pub fn draw_gulls(&self, f: &mut Frame, art: &Art, t: u32, clip: (i32, i32)) {
        if self.venue != Venue::Dijk && self.venue != Venue::Amsterdam {
            return;
        }
        let tt = t as f32;
        for (k, &(x, y, s)) in self.birds.iter().enumerate().take(4) {
            let xx = (x + tt * s * 0.35).rem_euclid(300.0) as i32 - 20;
            let yy = (y + (tt * 0.05 + s * 7.0).sin() * 4.0) as i32;
            if yy >= clip.0 && yy + 6 < clip.1 {
                art.gull[((t / 10) as usize + k) % 2].draw(f, xx, yy, &PROPS, false);
            }
        }
    }
}

fn sail_points(cx: i32, cy: i32, len: f32, a: f32) -> [(i32, i32); 4] {
    let mut p = [(0, 0); 4];
    for (k, q) in p.iter_mut().enumerate() {
        let ang = a + k as f32 * std::f32::consts::FRAC_PI_2;
        *q = (cx + (ang.cos() * len).round() as i32, cy + (ang.sin() * len).round() as i32);
    }
    p
}

fn far_mill(f: &mut Frame, art: &Art, x: i32, base: i32, a: f32, clip: (i32, i32)) {
    let c = rgb(104, 128, 168);
    let b = &art.mill_far;
    let top = base - b.h;
    if top - 12 < clip.0 || base > clip.1 {
        return;
    }
    b.draw_flat(f, x - b.w / 2, top, c, false);
    let (cx, cy) = (x, top + 2);
    for p in sail_points(cx, cy, 10.0, a) {
        f.line(cx, cy, p.0, p.1, c);
    }
}

/// A big detailed windmill with lattice sails turning in the wind.
pub fn big_mill(f: &mut Frame, art: &Art, x: i32, base: i32, a: f32, clip: (i32, i32)) {
    let b = &art.mill;
    let top = base - b.h;
    if top - 30 > clip.1 || base < clip.0 {
        return;
    }
    let pal = {
        let mut p = PROPS;
        p[4] = rgb(72, 80, 72); // body: dark thatch
        p[3] = rgb(104, 112, 96);
        p[10] = rgb(64, 96, 72); // stage: green
        p[11] = rgb(40, 64, 48);
        p[23] = rgb(96, 104, 96);
        p[24] = rgb(56, 64, 56);
        p
    };
    b.draw(f, x - b.w / 2, top, &pal, false);
    let (cx, cy) = (x + 2, top + 5);
    for p in sail_points(cx, cy, 26.0, a) {
        // a stock with a lattice frame on one side
        f.thick_line(cx, cy, p.0, p.1, 2, OUTLINE);
        let (dx, dy) = ((p.0 - cx) as f32 / 26.0, (p.1 - cy) as f32 / 26.0);
        let (nx, ny) = (-dy, dx);
        for k in 6..26 {
            let sx = cx as f32 + dx * k as f32;
            let sy = cy as f32 + dy * k as f32;
            let ex = sx + nx * 5.0;
            let ey = sy + ny * 5.0;
            if k % 3 == 0 {
                f.line(sx as i32, sy as i32, ex as i32, ey as i32, rgb(232, 224, 200));
            }
        }
        let (s0, s1) = ((cx as f32 + dx * 6.0 + nx * 5.0, cy as f32 + dy * 6.0 + ny * 5.0), (p.0 as f32 + nx * 5.0, p.1 as f32 + ny * 5.0));
        f.line(s0.0 as i32, s0.1 as i32, s1.0 as i32, s1.1 as i32, rgb(232, 224, 200));
        f.line(cx, cy, p.0, p.1, rgb(152, 104, 64));
    }
    f.fill_ellipse(cx, cy, 2, 2, OUTLINE);
    f.set(cx, cy, rgb(200, 200, 200));
}

