//! The three stages. Each is a static sky frame plus parallax strips that
//! are painted once (far 0.3, mid 0.6, ground 1.0) and copied every frame,
//! and an animated layer on top (clouds, water, gulls, crowd, lights).

use super::data::Stage;
use super::sim::{CAM_MAX, CAM_MIN, FIELD_END, YD};
use crate::engine::gfx::*;
use crate::engine::rng::Rng;
use crate::engine::ui::OUTLINE;

/// Screen row of the dog's line (z = 0).
pub const GROUND_Y: i32 = 190;
/// Top of the field.
pub const FIELD_TOP: i32 = 164;
/// Transparent in a layer.
const KEY: u32 = 0x010203;
const WHITE_C: u32 = rgb(248, 248, 248);

pub struct Layer {
    pub w: i32,
    pub y0: i32,
    pub h: i32,
    pub factor: f32,
    pub opaque: bool,
    pub px: Vec<u32>,
}

impl Layer {
    /// Paint a strip wide enough for the camera range, 256 columns at a time.
    fn paint(y0: i32, y1: i32, factor: f32, opaque: bool, paint: impl Fn(&mut Frame, i32)) -> Layer {
        let w = W + ((CAM_MAX - CAM_MIN) * factor).ceil() as i32 + 2;
        let h = y1 - y0;
        let mut px = vec![KEY; (w * h) as usize];
        let mut f = Frame::new();
        let mut ox = 0;
        while ox < w {
            f.clear(KEY);
            paint(&mut f, ox);
            for y in 0..h {
                for x in 0..W.min(w - ox) {
                    px[(y * w + ox + x) as usize] = f.get(x, y0 + y);
                }
            }
            ox += W;
        }
        Layer { w, y0, h, factor, opaque, px }
    }

    pub fn offset(&self, cam: f32) -> i32 {
        (((cam - CAM_MIN) * self.factor) as i32).clamp(0, self.w - W)
    }

    pub fn draw(&self, f: &mut Frame, cam: f32) {
        let off = self.offset(cam) as usize;
        let w = self.w as usize;
        for y in 0..self.h {
            let sy = self.y0 + y;
            if !(0..H).contains(&sy) {
                continue;
            }
            let row = &self.px[y as usize * w + off..y as usize * w + off + W as usize];
            let dst = &mut f.px[(sy * W) as usize..(sy * W + W) as usize];
            if self.opaque {
                dst.copy_from_slice(row);
            } else {
                for (d, &c) in dst.iter_mut().zip(row.iter()) {
                    if c != KEY {
                        *d = c;
                    }
                }
            }
        }
    }
}

/// World x -> screen x.
pub fn sx(x: f32, cam: f32) -> i32 {
    (x - cam).round() as i32
}

/// World x -> column in a factor-1 layer.
fn col(x: f32) -> i32 {
    (x - CAM_MIN) as i32
}

struct Cloud {
    x: f32,
    y: i32,
    w: i32,
    speed: f32,
    cow: bool,
}

pub struct Scenery {
    pub stage: Stage,
    pub sky: Frame,
    pub far: Layer,
    pub mid: Layer,
    pub ground: Layer,
    pub props: Layer,
    clouds: Vec<Cloud>,
    stars: Vec<(i32, i32, u32)>,
    boats: Vec<(f32, i32, u32)>,
    pub ufo_at: Option<u32>,
    pub pond: Option<(f32, f32)>,
}

impl Scenery {
    pub fn new(stage: Stage, pond: Option<(f32, f32)>, blankets: &[(f32, f32, u32)], seed: u64) -> Self {
        let mut rng = Rng(seed | 1);
        let mut sky = Frame::new();
        let (far, mid, ground, props) = match stage {
            Stage::Park => {
                park_sky(&mut sky);
                (park_far(), park_mid(seed), park_ground(pond, blankets, seed), park_props())
            }
            Stage::Pier => {
                pier_sky(&mut sky, &mut rng);
                (pier_far(), pier_mid(), pier_ground(seed), pier_props())
            }
            Stage::Bowl => {
                bowl_sky(&mut sky, &mut rng);
                (bowl_far(seed), bowl_mid(), bowl_ground(seed), bowl_props())
            }
        };
        let mut clouds = Vec::new();
        let n = match stage {
            Stage::Park => 6,
            Stage::Pier => 4,
            Stage::Bowl => 0,
        };
        let cow = rng.int(4) == 0;
        for k in 0..n {
            clouds.push(Cloud {
                x: rng.range(0.0, 420.0),
                y: 44 + rng.int(50),
                w: 14 + rng.int(16),
                speed: rng.range(0.02, 0.06),
                cow: cow && k == 0,
            });
        }
        let mut stars = Vec::new();
        if stage == Stage::Bowl {
            for _ in 0..70 {
                stars.push((rng.int(W), 34 + rng.int(70), rng.next() % 64));
            }
        }
        let mut boats = Vec::new();
        if stage == Stage::Pier {
            for k in 0..3 {
                boats.push((rng.range(0.0, 380.0), 128 + k * 3, [rgb(248, 248, 240), rgb(248, 200, 96), rgb(232, 96, 88)][k as usize]));
            }
        }
        let ufo_at = if rng.int(6) == 0 { Some(600 + rng.next() % 3000) } else { None };
        Scenery { stage, sky, far, mid, ground, props, clouds, stars, boats, ufo_at, pond }
    }

    /// Everything behind the play: sky, far and mid layers with their life,
    /// the ground. `t` is the frame counter, `excite` 0..1 for crowds.
    pub fn draw_back(&self, f: &mut Frame, cam: f32, t: u32, excite: f32) {
        f.copy_from(&self.sky);
        self.sky_life(f, cam, t);
        self.far.draw(f, cam);
        self.far_life(f, cam, t, excite);
        self.mid.draw(f, cam);
        self.mid_life(f, cam, t, excite);
        self.ground.draw(f, cam);
        self.props.draw(f, cam);
        self.ground_life(f, cam, t);
    }

    fn sky_life(&self, f: &mut Frame, cam: f32, t: u32) {
        for c in &self.clouds {
            let x = (c.x - cam * 0.08 + t as f32 * c.speed).rem_euclid(420.0) as i32 - 80;
            if c.cow {
                cow_cloud(f, x, c.y);
            } else {
                cloud(f, x, c.y, c.w, self.stage == Stage::Pier);
            }
        }
        for &(x, y, ph) in &self.stars {
            let tw = (t + ph * 7) / 20 % 6;
            let c = if tw == 0 { rgb(120, 120, 176) } else { rgb(232, 232, 248) };
            f.set(x, y, c);
            if tw == 3 && ph % 3 == 0 {
                f.set(x - 1, y, rgb(144, 144, 200));
                f.set(x + 1, y, rgb(144, 144, 200));
                f.set(x, y - 1, rgb(144, 144, 200));
                f.set(x, y + 1, rgb(144, 144, 200));
            }
        }
        if let Some(at) = self.ufo_at {
            let k = t as i32 - at as i32;
            if (0..420).contains(&k) {
                ufo(f, 280 - k, 52 + ((k as f32 * 0.08).sin() * 6.0) as i32, t);
            }
        }
    }

    fn far_life(&self, f: &mut Frame, cam: f32, t: u32, excite: f32) {
        let off = self.far.offset(cam);
        match self.stage {
            Stage::Park => {
                // the windmill's sails turn
                let (x, y) = (110 - off, 116);
                windmill_sails(f, x, y, t);
            }
            Stage::Pier => {
                // glints on the sea
                for k in 0..14 {
                    let y = 132 + (k * 5 % 22);
                    let x = ((k * 61 + t as i32 / (3 + k % 3)) % 300) - 20;
                    let len = 2 + k % 4;
                    f.hline(x, x + len, y, rgb(248, 200, 136));
                }
                for &(bx, by, c) in &self.boats {
                    let x = (bx + t as f32 * 0.03).rem_euclid(400.0) as i32 - off;
                    sailboat(f, x, by, c);
                }
                // the lighthouse beam
                if (t / 30) % 2 == 0 {
                    let (lx, ly) = (34 - off, 106);
                    f.fill_ellipse(lx, ly, 3, 2, rgb(248, 248, 200));
                    for k in 1..18 {
                        if k % 2 == 0 {
                            f.set(lx + 3 + k * 2, ly - 1 + k / 6, rgb(248, 232, 160));
                        }
                    }
                }
            }
            Stage::Bowl => {
                crowd(f, off, t, excite);
                // floodlights flicker a little
                for k in 0..3 {
                    let x = 40 + k * 150 - off;
                    let on = (t / 3 + k as u32 * 7) % 97 != 0;
                    if on {
                        for dx in -7..=7 {
                            f.set(x + dx, 64, rgb(248, 248, 224));
                        }
                    }
                }
            }
        }
    }

    fn mid_life(&self, f: &mut Frame, cam: f32, t: u32, excite: f32) {
        let off = self.mid.offset(cam);
        match self.stage {
            Stage::Park => {
                // a balloon drifting up behind the trees, now and then
                let k = (t % 1400) as i32;
                if k < 500 {
                    let x = 300 - off + (k as f32 * 0.02).sin() as i32 * 3;
                    let y = 160 - k / 4;
                    f.line(x, y + 6, x + 1, y + 14, rgb(232, 232, 232));
                    f.fill_ellipse(x, y, 4, 5, OUTLINE);
                    f.fill_ellipse(x, y, 3, 4, rgb(232, 48, 64));
                    f.set(x - 1, y - 2, rgb(248, 200, 200));
                }
            }
            Stage::Pier => {
                // gulls perched on the posts, bobbing
                for k in 0..5 {
                    let x = 40 + k * 96 - off;
                    let bob = ((t / 20 + k as u32) % 6 == 0) as i32;
                    perched_gull(f, x, 150 - bob, (t / 40 + k as u32) % 9 == 0);
                }
            }
            Stage::Bowl => {
                // camera flashes in the crowd
                let mut r = Rng((t / 4) as u64 * 31 + 7);
                let n = 1 + (excite * 6.0) as i32;
                for _ in 0..n {
                    if r.int(3) == 0 {
                        let (x, y) = (r.int(W), 100 + r.int(48));
                        f.set(x, y, WHITE_C);
                        f.set(x - 1, y, rgb(200, 200, 232));
                        f.set(x + 1, y, rgb(200, 200, 232));
                        f.set(x, y - 1, rgb(200, 200, 232));
                        f.set(x, y + 1, rgb(200, 200, 232));
                    }
                }
            }
        }
    }

    fn ground_life(&self, f: &mut Frame, cam: f32, t: u32) {
        match self.stage {
            Stage::Park => {
                if let Some((a, b)) = self.pond {
                    let (x0, x1) = (sx(a, cam), sx(b, cam));
                    for k in 0..6 {
                        let x = x0 + 4 + ((k * 13 + t as i32 / 9) % (x1 - x0 - 8).max(1));
                        let y = 184 + (k * 5) % 14;
                        if (t / 10 + k as u32) % 3 != 0 {
                            f.hline(x, x + 2, y, rgb(200, 232, 248));
                        }
                    }
                    // a duck paddles back and forth
                    let span = (x1 - x0 - 16).max(1);
                    let p = (t / 3) as i32 % (span * 2);
                    let (dx, flip) = if p < span { (p, false) } else { (span * 2 - p, true) };
                    duck(f, x0 + 8 + dx, 196, flip, t);
                }
            }
            Stage::Pier => {
                // waves under the pier
                for x in (0..W).step_by(2) {
                    let wx = x as f32 + cam;
                    let y = 219 + ((wx * 0.15 + t as f32 * 0.08).sin() * 1.5) as i32;
                    f.set(x, y, rgb(120, 176, 232));
                }
                // the sea beyond the end of the pier
                let ex = sx(FIELD_END, cam);
                if ex < W {
                    for k in 0..10 {
                        let x = ex + 10 + ((k * 37 + t as i32 / 4) % 200);
                        let y = 196 + (k * 7) % 22;
                        f.hline(x, x + 3, y, rgb(144, 192, 240));
                    }
                }
            }
            Stage::Bowl => {
                // the mascot dances by the end zone
                let x = sx(FIELD_END + 40.0, cam);
                if (-20..W + 20).contains(&x) {
                    hotdog(f, x, 176, t);
                }
            }
        }
    }
}

// ------------------------------------------------------------------ park

fn park_sky(f: &mut Frame) {
    f.band_gradient(
        0,
        168,
        &[rgb(56, 112, 216), rgb(72, 136, 232), rgb(96, 160, 240), rgb(128, 184, 248), rgb(168, 208, 248), rgb(208, 232, 248), rgb(232, 240, 232)],
    );
    // sun with a soft ring
    f.fill_ellipse(214, 56, 14, 14, rgb(248, 248, 200));
    f.fill_ellipse(214, 56, 11, 11, rgb(248, 240, 160));
    for k in 0..16 {
        let a = k as f32 / 16.0 * std::f32::consts::TAU;
        let (x, y) = (214 + (a.cos() * 18.0) as i32, 56 + (a.sin() * 18.0) as i32);
        f.set(x, y, rgb(248, 248, 216));
    }
}

fn park_far() -> Layer {
    Layer::paint(96, 168, 0.3, false, |f, ox| {
        // two rows of rolling hills
        let hill = |f: &mut Frame, base: i32, amp: f32, per: f32, ph: f32, c: u32, c2: u32| {
            for x in 0..W {
                let wx = (x + ox) as f32;
                let h = ((wx / per + ph).sin() * amp + (wx / (per * 0.43) + ph * 2.0).sin() * amp * 0.35) as i32;
                let top = base - h;
                for y in top..168 {
                    let c = if y < top + 2 && (x + y) % 2 == 0 { c2 } else { c };
                    f.set(x, y, c);
                }
            }
        };
        hill(f, 138, 10.0, 46.0, 0.5, rgb(120, 168, 200), rgb(160, 200, 224));
        hill(f, 150, 8.0, 31.0, 2.1, rgb(88, 152, 136), rgb(128, 184, 160));
        // the windmill on the far hill (sails animate)
        let (x, y) = (110 - ox, 116);
        f.fill_poly(&[(x - 5, y + 22), (x + 5, y + 22), (x + 3, y), (x - 3, y)], rgb(96, 104, 136));
        f.fill_poly(&[(x - 4, y + 1), (x + 4, y + 1), (x, y - 5)], rgb(136, 72, 72));
        f.rect(x - 1, y + 14, 2, 4, rgb(56, 56, 88));
        // a church spire
        let (cx, cy) = (250 - ox, 130);
        f.rect(cx - 4, cy, 8, 10, rgb(104, 128, 160));
        f.fill_poly(&[(cx - 3, cy), (cx + 3, cy), (cx, cy - 12)], rgb(104, 128, 160));
        f.set(cx, cy - 13, rgb(248, 216, 96));
    })
}

fn tree(f: &mut Frame, x: i32, base: i32, r: i32, rng: &mut Rng) {
    let (d, m, l) = (rgb(24, 96, 56), rgb(40, 136, 64), rgb(96, 184, 80));
    f.rect(x - 2, base - r, 4, r, rgb(104, 64, 40));
    f.rect(x, base - r, 2, r, rgb(136, 88, 56));
    let cy = base - r - r / 2;
    f.fill_ellipse(x, cy, r + 1, r, OUTLINE);
    f.fill_ellipse(x, cy, r, r - 1, d);
    f.fill_ellipse(x - 1, cy - 1, r - 2, r - 3, m);
    for _ in 0..(r * 2) {
        let (dx, dy) = (rng.int(r * 2) - r, rng.int(r) - r + 1);
        if dx * dx + dy * dy * 3 < r * r {
            f.set(x + dx - 2, cy + dy, l);
            f.set(x + dx - 1, cy + dy, l);
        }
    }
    f.dither_rect(x - r + 2, cy + r / 2, r * 2 - 4, 2, d, m);
}

fn park_mid(seed: u64) -> Layer {
    Layer::paint(100, 168, 0.6, false, |f, ox| {
        let mut rng = Rng(seed ^ 0x77 | 1);
        // hedge along the back of the field
        for x in 0..W {
            let wx = x + ox;
            let top = 158 + ((wx as f32 * 0.31).sin() * 1.5) as i32;
            for y in top..168 {
                let c = if y == top { rgb(80, 168, 72) } else if (wx + y) % 3 == 0 { rgb(32, 112, 56) } else { rgb(48, 136, 64) };
                f.set(x, y, c);
            }
        }
        // a row of trees
        let mut x = 6;
        while x < 520 {
            let r = 9 + rng.int(6);
            tree(f, x - ox, 162, r, &mut rng);
            x += 26 + rng.int(30);
        }
        // a lamp post and a bench now and then
        for k in 0..4 {
            let lx = 60 + k * 130 - ox;
            f.rect(lx, 138, 2, 24, rgb(56, 56, 72));
            f.fill_ellipse(lx + 1, 136, 3, 3, OUTLINE);
            f.fill_ellipse(lx + 1, 136, 2, 2, rgb(248, 240, 200));
            let bx = lx + 26;
            f.rect(bx, 156, 18, 2, rgb(152, 96, 56));
            f.rect(bx, 152, 18, 2, rgb(176, 112, 64));
            f.rect(bx + 1, 158, 2, 4, rgb(56, 56, 72));
            f.rect(bx + 15, 158, 2, 4, rgb(56, 56, 72));
        }
    })
}

fn yard_signs(f: &mut Frame, ox: i32, y: i32, board: u32, ink: u32) {
    for yd in (10..=90).step_by(10) {
        let x = col(yd as f32 * YD) - ox;
        let s = format!("{}", yd);
        let w = text_width(&s, false);
        f.rect(x - w / 2 - 2, y, w + 4, 9, OUTLINE);
        f.rect(x - w / 2 - 1, y + 1, w + 2, 7, board);
        f.text(x - w / 2, y + 1, &s, ink);
        f.rect(x - 1, y + 9, 2, 3, OUTLINE);
    }
}

fn park_ground(pond: Option<(f32, f32)>, blankets: &[(f32, f32, u32)], seed: u64) -> Layer {
    let blankets = blankets.to_vec();
    Layer::paint(FIELD_TOP - 2, H, 1.0, true, move |f, ox| {
        let mut rng = Rng(seed ^ 0x1234 | 1);
        let (g1, g2, g3) = (rgb(88, 176, 72), rgb(72, 160, 64), rgb(56, 136, 56));
        for x in 0..W {
            let wx = x + ox;
            // mowed stripes, 5 yards wide
            let stripe = ((wx as f32 + CAM_MIN) / (5.0 * YD)).floor() as i32 % 2 == 0;
            for y in FIELD_TOP - 2..H {
                let mut c = if stripe { g1 } else { g2 };
                if y < FIELD_TOP + 2 {
                    c = g3;
                } else if y > 212 {
                    c = if (x + y) % 2 == 0 { g3 } else { c };
                }
                f.set(x, y, c);
            }
        }
        for _ in 0..900 {
            let (x, y) = (rng.int(W), FIELD_TOP + rng.int(H - FIELD_TOP));
            f.set(x, y, if rng.int(2) == 0 { rgb(104, 192, 88) } else { rgb(56, 128, 48) });
        }
        // yard lines
        for yd in (10..=90).step_by(10) {
            let x = col(yd as f32 * YD) - ox;
            for y in FIELD_TOP + 2..212 {
                if y % 3 != 0 {
                    f.set(x, y, rgb(224, 240, 216));
                }
            }
        }
        // the throw line
        let tx = col(0.0) - ox;
        f.rect(tx - 1, FIELD_TOP + 2, 3, 212 - FIELD_TOP - 2, WHITE_C);
        // flowers along the front
        for _ in 0..90 {
            let (x, y) = (rng.int(W), 206 + rng.int(16));
            let c = [rgb(248, 248, 248), rgb(248, 216, 64), rgb(248, 120, 168)][rng.int(3) as usize];
            f.set(x, y, c);
            f.set(x, y + 1, rgb(48, 120, 48));
        }
        // the pond
        if let Some((a, b)) = pond {
            let (x0, x1) = (col(a) - ox, col(b) - ox);
            let cx = (x0 + x1) / 2;
            let rx = (x1 - x0) / 2 + 2;
            f.fill_ellipse(cx, 190, rx + 2, 13, rgb(112, 96, 56));
            f.fill_ellipse(cx, 190, rx, 11, rgb(48, 104, 184));
            f.fill_ellipse(cx, 191, rx - 3, 8, rgb(64, 128, 208));
            f.dither_rect(cx - rx + 4, 181, rx * 2 - 8, 2, rgb(48, 104, 184), rgb(96, 160, 224));
            // reeds and lily pads
            for (k, dx) in [-rx - 1, -rx + 2, rx - 2, rx + 1].iter().enumerate() {
                let x = cx + dx;
                f.line(x, 182, x - 1 + k as i32 % 2, 172, rgb(56, 120, 48));
                f.rect(x - 1 + k as i32 % 2, 172, 2, 4, rgb(136, 88, 48));
            }
            f.fill_ellipse(cx - 6, 194, 3, 1, rgb(72, 168, 72));
            f.fill_ellipse(cx + 8, 186, 2, 1, rgb(72, 168, 72));
        }
        // picnic blankets with a basket
        for &(bx, bw, c) in &blankets {
            let x0 = col(bx) - ox;
            let w = bw as i32;
            let (y0, y1) = (184, 198);
            for y in y0..y1 {
                let skew = (y1 - y) / 2;
                for x in 0..w {
                    let check = ((x / 4) + (y - y0) / 3) % 2 == 0;
                    let cc = if check { c } else { rgb(248, 240, 224) };
                    f.set(x0 + x + skew, y, cc);
                }
                f.set(x0 + skew - 1, y, OUTLINE);
                f.set(x0 + w + skew, y, OUTLINE);
            }
            f.hline(x0 + 6, x0 + w + 7, y0 - 1, OUTLINE);
            f.hline(x0 - 1, x0 + w, y1, OUTLINE);
            // basket
            let (kx, ky) = (x0 + w - 6, y0 - 1);
            f.rect(kx - 1, ky - 6, 10, 7, OUTLINE);
            f.rect(kx, ky - 5, 8, 5, rgb(200, 144, 72));
            f.dither_rect(kx, ky - 5, 8, 5, rgb(200, 144, 72), rgb(168, 112, 48));
            f.line(kx + 1, ky - 6, kx + 4, ky - 10, OUTLINE);
            f.line(kx + 4, ky - 10, kx + 7, ky - 6, OUTLINE);
        }
    })
}

fn park_props() -> Layer {
    Layer::paint(112, FIELD_TOP + 8, 1.0, false, |f, ox| {
        // the hedge at the end of the field
        let x0 = col(FIELD_END) - ox;
        for x in x0..x0 + 160 {
            let top = 128 + ((x as f32 * 0.4).sin() * 3.0) as i32 + ((x - x0) < 3) as i32 * 2;
            for y in top..FIELD_TOP + 8 {
                let c = if y < top + 2 {
                    rgb(96, 184, 80)
                } else if (x * 3 + y * 5) % 7 == 0 {
                    rgb(24, 88, 48)
                } else {
                    rgb(40, 128, 56)
                };
                f.set(x, y, c);
            }
            f.set(x, top - 1, OUTLINE);
        }
        for y in 128..FIELD_TOP + 8 {
            f.set(x0 - 1, y, OUTLINE);
        }
        // "PLEASE DO NOT FEED THE HEDGE"
        let (sx0, sy0) = (x0 + 8, 134);
        f.rect(sx0 - 1, sy0 - 1, 70, 19, OUTLINE);
        f.rect(sx0, sy0, 68, 17, rgb(248, 240, 216));
        f.text(sx0 + 2, sy0 + 1, "PLEASE DO", rgb(160, 48, 48));
        f.text(sx0 + 2, sy0 + 9, "NOT FEED HEDGE", rgb(160, 48, 48));
        yard_signs(f, ox, FIELD_TOP - 4, rgb(248, 248, 240), rgb(48, 96, 48));
        // the throw-line flag
        let tx = col(0.0) - ox;
        f.rect(tx - 1, 140, 2, 30, OUTLINE);
        f.rect(tx, 140, 1, 30, rgb(232, 232, 240));
        f.fill_poly(&[(tx + 1, 140), (tx + 13, 144), (tx + 1, 148)], rgb(232, 48, 64));
    })
}

// ------------------------------------------------------------------ pier

fn pier_sky(f: &mut Frame, rng: &mut Rng) {
    f.band_gradient(
        0,
        134,
        &[rgb(48, 32, 104), rgb(80, 40, 128), rgb(128, 56, 144), rgb(184, 72, 136), rgb(224, 96, 104), rgb(248, 136, 88), rgb(248, 184, 104)],
    );
    // the striped sun sitting on the sea
    let (cx, cy, r) = (150, 132, 26);
    for y in cy - r..cy {
        let t = (y - (cy - r)) as f32 / r as f32;
        let c = mix(rgb(248, 248, 168), rgb(248, 128, 64), t);
        let gap = y > cy - 14 && (y - (cy - 14)) % 5 < 1 + (y - (cy - 14)) / 6;
        if gap {
            continue;
        }
        let w = (((r * r - (y - cy) * (y - cy)) as f32).sqrt()) as i32;
        f.hline(cx - w, cx + w, y, c);
    }
    for (x, y, w) in [(16, 60, 64), (170, 48, 72), (90, 80, 44), (200, 96, 50), (30, 104, 60)] {
        f.hline(x, x + w, y, rgb(200, 88, 136));
        f.hline(x + 6, x + w - 8, y + 1, rgb(160, 64, 128));
    }
    for _ in 0..20 {
        let (x, y) = (rng.int(W), 34 + rng.int(20));
        f.set(x, y, rgb(200, 184, 232));
    }
    // the sea
    f.band_gradient(134, H, &[rgb(56, 48, 128), rgb(48, 72, 152), rgb(40, 96, 176), rgb(40, 112, 192)]);
    for y in 134..150 {
        let w = 4 + (y - 134) * 2;
        let mut x = 150 - w;
        while x < 150 + w {
            let len = 2 + rng.int(5);
            if rng.f() < 0.5 {
                f.hline(x, x + len, y, if y % 2 == 0 { rgb(248, 200, 96) } else { rgb(248, 152, 80) });
            }
            x += len + 2;
        }
    }
}

fn pier_far() -> Layer {
    Layer::paint(84, 140, 0.3, false, |f, ox| {
        // a rocky point with a lighthouse on the left
        let rock = rgb(64, 40, 88);
        f.fill_poly(&[(-10 - ox, 136), (60 - ox, 136), (46 - ox, 120), (20 - ox, 114), (4 - ox, 122)], rock);
        let (lx, ly) = (34 - ox, 106);
        f.fill_poly(&[(lx - 4, 118), (lx + 4, 118), (lx + 2, ly), (lx - 2, ly)], rgb(232, 224, 232));
        for k in 0..3 {
            f.hline(lx - 3 + k / 2, lx + 3 - k / 2, 116 - k * 4, rgb(216, 56, 72));
            f.hline(lx - 3 + k / 2, lx + 3 - k / 2, 115 - k * 4, rgb(216, 56, 72));
        }
        f.rect(lx - 3, ly - 4, 6, 4, rgb(88, 80, 112));
        f.fill_poly(&[(lx - 3, ly - 4), (lx + 3, ly - 4), (lx, ly - 8)], rgb(216, 56, 72));
        // distant city on the right
        let mut x = 220;
        let mut k = 0;
        while x < 380 {
            let h = 6 + (k * 7 % 13);
            f.rect(x - ox, 134 - h, 9, h, rgb(88, 56, 112));
            if k % 2 == 0 {
                f.set(x + 3 - ox, 134 - h + 3, rgb(248, 200, 120));
            }
            x += 10;
            k += 1;
        }
    })
}

fn pier_mid() -> Layer {
    Layer::paint(112, 170, 0.6, false, |f, ox| {
        // the far railing of the pier, with lamps and bunting
        let rail = rgb(232, 224, 216);
        let post = rgb(176, 160, 160);
        f.hline(0, W - 1, 152, OUTLINE);
        f.hline(0, W - 1, 153, rail);
        f.hline(0, W - 1, 154, OUTLINE);
        f.hline(0, W - 1, 159, rail);
        for x in 0..W {
            let wx = x + ox;
            if wx % 24 == 0 {
                f.rect(x - 1, 150, 4, 16, OUTLINE);
                f.rect(x, 151, 2, 14, post);
            }
            if wx % 6 == 0 {
                f.line(x, 155, x, 164, post);
            }
        }
        for k in 0..6 {
            let lx = 20 + k * 96 - ox;
            f.rect(lx - 1, 118, 3, 34, OUTLINE);
            f.rect(lx, 118, 1, 34, rgb(72, 64, 96));
            f.fill_ellipse(lx, 116, 4, 3, OUTLINE);
            f.fill_ellipse(lx, 116, 3, 2, rgb(248, 232, 160));
            // bunting to the next lamp
            let nx = lx + 96;
            let cols = [rgb(232, 56, 64), rgb(248, 216, 48), rgb(64, 160, 232), rgb(248, 248, 248)];
            for j in 0..12 {
                let x = lx + 4 + j * 8;
                let sag = ((j as f32 / 11.0 * std::f32::consts::PI).sin() * 8.0) as i32;
                let y = 120 + sag;
                if x < nx {
                    f.fill_poly(&[(x, y), (x + 5, y), (x + 2, y + 5)], cols[j as usize % 4]);
                    f.hline(x, x + 7, y - 1, rgb(96, 80, 104));
                }
            }
        }
    })
}

fn pier_ground(seed: u64) -> Layer {
    Layer::paint(FIELD_TOP - 4, H, 1.0, true, move |f, ox| {
        let mut rng = Rng(seed ^ 0x99 | 1);
        // the sea under and beyond the pier
        f.band_gradient(FIELD_TOP - 4, H, &[rgb(40, 96, 176), rgb(40, 104, 184), rgb(32, 88, 168)]);
        let end = col(FIELD_END) - ox;
        let (wood, wood_d, gap) = (rgb(184, 128, 80), rgb(160, 104, 64), rgb(104, 64, 40));
        for x in 0..W.min(end + 1) {
            for y in FIELD_TOP - 4..208 {
                // boards run across, a dark gap every 6 rows
                let r = (y - FIELD_TOP + 4) % 7;
                let mut c = if r == 0 { gap } else if r == 6 { wood_d } else { wood };
                let wx = x + ox;
                if r != 0 && (wx * 7 + (y / 7) * 53) % 97 == 0 {
                    c = gap;
                }
                f.set(x, y, c);
            }
        }
        for _ in 0..500 {
            let (x, y) = (rng.int(W.min(end.max(0))), FIELD_TOP + rng.int(44));
            f.set(x, y, rgb(200, 144, 96));
        }
        // yard numbers painted on the deck
        for yd in (10..=90).step_by(10) {
            let x = col(yd as f32 * YD) - ox;
            for y in (FIELD_TOP..206).step_by(2) {
                f.set(x, y, rgb(232, 224, 208));
            }
            let s = format!("{}", yd);
            f.text(x - text_width(&s, false) / 2, 197, &s, rgb(232, 224, 208));
        }
        let tx = col(0.0) - ox;
        f.rect(tx - 1, FIELD_TOP - 2, 3, 208 - FIELD_TOP + 2, WHITE_C);
        // the front beam and the pilings
        if end > 0 {
            let e = end.min(W);
            f.rect(0, 208, e, 5, rgb(120, 80, 48));
            f.hline(0, e - 1, 208, rgb(152, 104, 64));
            f.hline(0, e - 1, 213, OUTLINE);
            for x in 0..e {
                if (x + ox) % 40 < 6 {
                    for y in 213..H {
                        f.set(x, y, if (x + ox) % 40 < 2 { rgb(104, 72, 56) } else { rgb(80, 56, 48) });
                    }
                }
            }
        }
        // the end of the pier
        if (0..W + 10).contains(&end) {
            f.rect(end, FIELD_TOP - 4, 2, 213 - FIELD_TOP + 4, OUTLINE);
        }
    })
}

fn pier_props() -> Layer {
    Layer::paint(120, FIELD_TOP + 2, 1.0, false, |f, ox| {
        let end = col(FIELD_END) - ox;
        // end railing and a sign
        f.rect(end - 3, 136, 4, 30, OUTLINE);
        f.rect(end - 2, 137, 2, 28, rgb(232, 224, 216));
        let (sx0, sy0) = (end + 6, 138);
        f.rect(sx0 - 1, sy0 - 1, 60, 19, OUTLINE);
        f.rect(sx0, sy0, 58, 17, rgb(248, 232, 120));
        f.text(sx0 + 2, sy0 + 1, "NO DIVING", rgb(176, 32, 32));
        f.text(sx0 + 2, sy0 + 9, "NO GULLS", rgb(176, 32, 32));
        f.rect(sx0 + 4, sy0 + 18, 2, 10, OUTLINE);
        yard_signs(f, ox, FIELD_TOP - 6, rgb(64, 48, 88), rgb(248, 216, 96));
        // a life ring on a post
        let rx = col(-40.0) - ox;
        f.rect(rx, 140, 2, 24, OUTLINE);
        f.fill_ellipse(rx + 1, 146, 6, 6, OUTLINE);
        f.fill_ellipse(rx + 1, 146, 5, 5, rgb(232, 56, 56));
        f.fill_ellipse(rx + 1, 146, 2, 2, OUTLINE);
        f.rect(rx - 4, 145, 3, 3, WHITE_C);
        f.rect(rx + 4, 145, 3, 3, WHITE_C);
    })
}

// ------------------------------------------------------------------ bowl

fn bowl_sky(f: &mut Frame, rng: &mut Rng) {
    f.band_gradient(0, 120, &[rgb(8, 8, 32), rgb(16, 16, 48), rgb(24, 24, 72), rgb(40, 32, 96), rgb(64, 40, 112)]);
    f.fill_ellipse(46, 48, 9, 9, rgb(248, 240, 208));
    f.fill_ellipse(43, 46, 2, 2, rgb(216, 208, 176));
    f.fill_ellipse(49, 52, 1, 1, rgb(216, 208, 176));
    for _ in 0..30 {
        f.set(rng.int(W), 34 + rng.int(60), rgb(120, 120, 176));
    }
}

fn bowl_far(seed: u64) -> Layer {
    Layer::paint(56, 152, 0.3, false, move |f, ox| {
        let mut rng = Rng(seed ^ 0x51 | 1);
        // the stands: tiers rising away
        for y in 96..152 {
            let tier = (y - 96) / 7;
            let c = if (y - 96) % 7 == 0 { rgb(24, 24, 56) } else if tier % 2 == 0 { rgb(56, 48, 96) } else { rgb(48, 40, 88) };
            f.hline(0, W - 1, y, c);
        }
        // the crowd's heads (static part; the wave is drawn live)
        for y in (99..150).step_by(7) {
            let mut x = (y * 3) % 5;
            while x < W {
                let c = [rgb(232, 72, 72), rgb(72, 136, 232), rgb(248, 216, 72), rgb(232, 232, 232), rgb(120, 200, 104)][rng.int(5) as usize];
                f.set(x, y, shade(c, 3, 5));
                f.set(x, y + 1, shade(c, 1, 2));
                f.set(x, y - 1, rgb(176, 136, 112));
                x += 3 + rng.int(2);
            }
        }
        // roof and floodlight towers
        f.rect(0, 90, W, 6, rgb(32, 32, 64));
        f.hline(0, W - 1, 90, rgb(80, 80, 128));
        for k in 0..3 {
            let x = 40 + k * 150 - ox;
            f.rect(x - 1, 66, 3, 30, rgb(72, 72, 104));
            f.rect(x - 9, 58, 19, 9, OUTLINE);
            for j in 0..4 {
                f.fill_ellipse(x - 6 + j * 4, 62, 1, 2, rgb(248, 248, 216));
            }
        }
        // the scoreboard
        let (bx, by) = (220 - ox, 62);
        f.rect(bx - 1, by - 1, 62, 28, OUTLINE);
        f.rect(bx, by, 60, 26, rgb(16, 16, 32));
        f.text(bx + 3, by + 2, "DISC DOGS", rgb(248, 200, 48));
        f.text(bx + 3, by + 10, "GRAND BOWL", rgb(96, 224, 248));
        f.text(bx + 3, by + 18, "92", rgb(248, 96, 96));
        f.rect(bx + 28, by + 26, 4, 8, rgb(72, 72, 104));
    })
}

fn crowd(f: &mut Frame, off: i32, t: u32, excite: f32) {
    // the wave rolls along the stands
    let wave = ((t * 2) as i32 % 700) - 100 - off / 3;
    for y in (99..150).step_by(7) {
        for x in (wave - 10).max(0)..(wave + 10).min(W) {
            if (x + y) % 4 == 0 {
                f.set(x, y - 2, rgb(200, 160, 136));
                f.set(x, y - 3, rgb(248, 248, 248));
            }
        }
    }
    if excite > 0.3 {
        for y in (99..150).step_by(7) {
            for x in (0..W).step_by(5) {
                if ((x as u32 + y as u32 + t / 4) % 3) == 0 {
                    f.set(x, y - 2, rgb(200, 160, 136));
                }
            }
        }
    }
}

fn bowl_mid() -> Layer {
    Layer::paint(146, 168, 0.6, false, |f, ox| {
        let words = ["BARK COLA", "HELICO", "FLEA-B-GONE", "RUFF FM", "BONE APPETIT", "HELICO"];
        let cols = [(rgb(176, 24, 40), rgb(248, 248, 248)), (rgb(24, 24, 96), rgb(248, 216, 48)), (rgb(16, 104, 64), rgb(248, 248, 248)), (rgb(88, 24, 120), rgb(96, 224, 248))];
        let mut x = 0;
        let mut k = 0;
        while x < 520 {
            let (bg, fg) = cols[k % 4];
            let w = words[k % words.len()];
            let bw = 76;
            f.rect(x - ox, 152, bw, 12, bg);
            f.hline(x - ox, x - ox + bw - 1, 152, mix(bg, 0xFFFFFF, 0.4));
            f.hline(x - ox, x - ox + bw - 1, 163, OUTLINE);
            f.set(x - ox, 153, OUTLINE);
            f.text(x - ox + bw / 2 - text_width(w, false) / 2, 154, w, fg);
            x += bw;
            k += 1;
        }
    })
}

fn bowl_ground(seed: u64) -> Layer {
    Layer::paint(FIELD_TOP - 2, H, 1.0, true, move |f, ox| {
        let mut rng = Rng(seed ^ 0x3 | 1);
        let (g1, g2) = (rgb(56, 160, 72), rgb(48, 144, 64));
        for x in 0..W {
            let wx = x + ox;
            let stripe = ((wx as f32 + CAM_MIN) / (5.0 * YD)).floor() as i32 % 2 == 0;
            for y in FIELD_TOP - 2..H {
                let mut c = if stripe { g1 } else { g2 };
                // the floodlit middle is brighter
                if (176..204).contains(&y) {
                    c = mix(c, rgb(120, 216, 120), 0.18);
                }
                if y < FIELD_TOP + 2 {
                    c = rgb(32, 96, 48);
                }
                f.set(x, y, c);
            }
        }
        for _ in 0..600 {
            f.set(rng.int(W), FIELD_TOP + rng.int(H - FIELD_TOP), rgb(40, 128, 56));
        }
        for yd in (10..=90).step_by(10) {
            let x = col(yd as f32 * YD) - ox;
            f.rect(x, FIELD_TOP + 2, 2, 212 - FIELD_TOP - 2, rgb(232, 240, 232));
            let s = format!("{}", if yd > 50 { 100 - yd } else { yd });
            f.text_bold(x - text_width(&s, true) / 2 + 1, 202, &s, rgb(232, 240, 232));
        }
        let tx = col(0.0) - ox;
        f.rect(tx - 1, FIELD_TOP + 2, 4, 212 - FIELD_TOP - 2, rgb(248, 216, 48));
        // the end zone
        let e = col(FIELD_END) - ox;
        for x in e.max(0)..(e + 150).min(W) {
            for y in FIELD_TOP + 2..212 {
                if ((x - e) + y) / 6 % 2 == 0 {
                    f.set(x, y, rgb(176, 40, 72));
                } else {
                    f.set(x, y, rgb(200, 56, 88));
                }
            }
        }
        if e + 20 < W && e + 120 > 0 {
            f.text_grad(e + 20, 184, "DOGS", &crate::engine::ui::grad7(rgb(248, 216, 64)), OUTLINE, true);
        }
        f.rect(e - 1, FIELD_TOP + 2, 3, 212 - FIELD_TOP - 2, WHITE_C);
        // track around the field
        f.rect(0, 212, W, 12, rgb(152, 72, 56));
        f.hline(0, W - 1, 212, WHITE_C);
        for x in (0..W).step_by(2) {
            f.set(x, 218, rgb(184, 104, 88));
        }
    })
}

fn bowl_props() -> Layer {
    Layer::paint(118, FIELD_TOP + 2, 1.0, false, |f, ox| {
        // the padded wall behind the end zone
        let e = col(FIELD_END) - ox;
        f.rect(e, 138, 160, 28, OUTLINE);
        f.rect(e + 1, 139, 158, 26, rgb(32, 48, 120));
        f.hline(e + 1, e + 158, 139, rgb(72, 96, 176));
        f.text(e + 8, 143, "INTO THE CROWD", rgb(248, 216, 48));
        f.text(e + 8, 153, "= NO POINTS", rgb(248, 248, 248));
        yard_signs(f, ox, FIELD_TOP - 6, rgb(24, 24, 56), rgb(248, 248, 248));
        // goal post-ish disc target at the throw line
        let tx = col(0.0) - ox;
        f.rect(tx, 128, 2, 38, OUTLINE);
        f.rect(tx + 0, 129, 1, 36, rgb(248, 216, 48));
    })
}

// ------------------------------------------------------------------ little things

fn cloud(f: &mut Frame, x: i32, y: i32, w: i32, sunset: bool) {
    let (c, d) = if sunset { (rgb(248, 176, 160), rgb(216, 120, 144)) } else { (rgb(248, 248, 248), rgb(208, 224, 248)) };
    f.fill_ellipse(x, y, w, 4, d);
    f.fill_ellipse(x + w / 3, y - 3, w / 2, 4, c);
    f.fill_ellipse(x - w / 3, y - 1, w / 2, 3, c);
    f.fill_ellipse(x, y - 1, w - 2, 3, c);
    f.dither_rect(x - w + 3, y + 2, w * 2 - 6, 1, d, c);
}

/// A rare cloud that looks a lot like a cow.
fn cow_cloud(f: &mut Frame, x: i32, y: i32) {
    let c = rgb(248, 248, 248);
    let d = rgb(208, 224, 248);
    f.fill_ellipse(x, y, 13, 6, d);
    f.fill_ellipse(x, y - 1, 12, 5, c);
    f.fill_ellipse(x + 14, y - 4, 5, 4, c);
    f.rect(x + 12, y - 10, 2, 3, c);
    f.rect(x + 17, y - 10, 2, 3, c);
    for lx in [x - 9, x - 4, x + 4, x + 9] {
        f.rect(lx, y + 4, 2, 5, c);
    }
    f.fill_ellipse(x - 3, y - 2, 3, 2, d);
    f.fill_ellipse(x + 6, y + 1, 2, 1, d);
    f.line(x - 13, y - 2, x - 16, y + 3, c);
}

fn ufo(f: &mut Frame, x: i32, y: i32, t: u32) {
    f.fill_ellipse(x, y, 10, 3, OUTLINE);
    f.fill_ellipse(x, y, 9, 2, rgb(176, 184, 200));
    f.fill_ellipse(x, y - 3, 4, 3, OUTLINE);
    f.fill_ellipse(x, y - 3, 3, 2, rgb(120, 232, 184));
    for k in 0..4 {
        let on = (t / 4 + k) % 4 == 0;
        f.set(x - 6 + k as i32 * 4, y + 1, if on { rgb(248, 248, 96) } else { rgb(248, 96, 96) });
    }
}

fn windmill_sails(f: &mut Frame, x: i32, y: i32, t: u32) {
    let a0 = t as f32 * 0.02;
    for k in 0..4 {
        let a = a0 + k as f32 * std::f32::consts::FRAC_PI_2;
        let (ex, ey) = (x + (a.cos() * 13.0) as i32, y + (a.sin() * 13.0) as i32);
        f.line(x, y, ex, ey, rgb(232, 224, 216));
        let (mx, my) = (x + (a.cos() * 8.0) as i32, y + (a.sin() * 8.0) as i32);
        let (px, py) = ((-a.sin() * 2.0) as i32, (a.cos() * 2.0) as i32);
        f.line(mx + px, my + py, ex + px, ey + py, rgb(184, 176, 176));
    }
    f.fill_ellipse(x, y, 1, 1, rgb(80, 64, 64));
}

fn sailboat(f: &mut Frame, x: i32, y: i32, sail: u32) {
    f.fill_poly(&[(x - 5, y), (x + 5, y), (x + 3, y + 2), (x - 3, y + 2)], rgb(88, 56, 72));
    f.line(x, y - 9, x, y, rgb(88, 56, 72));
    f.fill_poly(&[(x + 1, y - 9), (x + 1, y - 1), (x + 6, y - 1)], sail);
    f.fill_poly(&[(x - 1, y - 7), (x - 1, y - 1), (x - 4, y - 1)], shade(sail, 3, 4));
}

fn perched_gull(f: &mut Frame, x: i32, y: i32, squawk: bool) {
    f.fill_ellipse(x, y - 3, 4, 2, OUTLINE);
    f.fill_ellipse(x, y - 3, 3, 1, rgb(248, 248, 248));
    f.hline(x - 3, x, y - 3, rgb(144, 152, 176));
    f.fill_ellipse(x + 3, y - 6, 2, 2, OUTLINE);
    f.fill_ellipse(x + 3, y - 6, 1, 1, rgb(248, 248, 248));
    f.set(x + 3, y - 6, OUTLINE);
    f.hline(x + 5, x + 6, y - 6 + squawk as i32, rgb(248, 184, 48));
    if squawk {
        f.set(x + 6, y - 7, rgb(248, 184, 48));
    }
    f.set(x - 1, y - 1, rgb(248, 152, 48));
    f.set(x + 1, y - 1, rgb(248, 152, 48));
}

fn duck(f: &mut Frame, x: i32, y: i32, flip: bool, t: u32) {
    let d = if flip { -1 } else { 1 };
    let bob = ((t / 12) % 2) as i32;
    let y = y + bob;
    f.fill_ellipse(x, y, 5, 3, OUTLINE);
    f.fill_ellipse(x, y, 4, 2, rgb(248, 248, 240));
    f.fill_ellipse(x + 3 * d, y - 4, 3, 3, OUTLINE);
    f.fill_ellipse(x + 3 * d, y - 4, 2, 2, rgb(56, 152, 88));
    f.set(x + 4 * d, y - 5, OUTLINE);
    f.hline(x + 5 * d, x + 7 * d, y - 3, rgb(248, 184, 48));
    f.hline(x - 4, x + 4, y + 3, rgb(96, 160, 224));
}

/// BARKY, the stadium's hot dog mascot, dancing.
fn hotdog(f: &mut Frame, x: i32, y: i32, t: u32) {
    let k = (t / 10) % 4;
    let lean = [0, 2, 0, -2][k as usize];
    let jump = if k == 1 || k == 3 { 1 } else { 0 };
    let y = y - jump;
    // legs
    f.rect(x - 4, y - 6, 3, 6, OUTLINE);
    f.rect(x + 2, y - 6, 3, 6, OUTLINE);
    f.rect(x - 3, y - 6, 1, 5, rgb(248, 248, 248));
    f.rect(x + 3, y - 6, 1, 5, rgb(248, 248, 248));
    // bun and sausage
    f.fill_ellipse(x + lean / 2, y - 16, 7, 11, OUTLINE);
    f.fill_ellipse(x + lean / 2, y - 16, 6, 10, rgb(232, 176, 96));
    f.fill_ellipse(x + lean, y - 18, 3, 11, OUTLINE);
    f.fill_ellipse(x + lean, y - 18, 2, 10, rgb(200, 72, 56));
    // mustard zigzag
    for j in 0..6 {
        f.set(x + lean - 1 + (j % 2) * 2, y - 25 + j * 3, rgb(248, 216, 48));
    }
    // face and waving arms
    f.set(x + lean - 1, y - 22, OUTLINE);
    f.set(x + lean + 1, y - 22, OUTLINE);
    f.hline(x + lean - 1, x + lean + 1, y - 19, OUTLINE);
    let wave = if k < 2 { -4 } else { 4 };
    f.line(x - 6, y - 16, x - 9, y - 20 + wave / 2, OUTLINE);
    f.line(x + 6, y - 16, x + 9, y - 20 - wave / 2, OUTLINE);
}
