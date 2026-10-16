//! TIENEN! backdrops, drawn once per venue: the sky, the buildings behind
//! the pitch, the pitch itself with its chalk lines, and the kerb in front.
//! The goal, the people and everything that moves are drawn by `render`.

use super::data::Place;
use super::world::*;
use crate::engine::gfx::{mix, rgb, shade, Frame, W};
use crate::engine::rng::Rng;
use crate::engine::ui::OUTLINE;

pub struct Scenery {
    pub place: Place,
    pub back: Frame,
}

/// The pitch outline on screen: far left, far right, near right, near left.
pub fn pitch_poly() -> [(i32, i32); 4] {
    [proj(GOAL_X, 0.0, 0.0), proj(FIELD_X1, 0.0, 0.0), proj(FIELD_X1, DEPTH, 0.0), proj(GOAL_X, DEPTH, 0.0)]
}

/// Screen y of the far line and the near line.
pub fn far_y() -> i32 {
    proj(0.0, 0.0, 0.0).1
}
pub fn near_y() -> i32 {
    proj(0.0, DEPTH, 0.0).1
}

fn bands(f: &mut Frame, y0: i32, y1: i32, cols: &[u32]) {
    let n = cols.len() as i32;
    for y in y0..y1 {
        let k = ((y - y0) * n / (y1 - y0).max(1)).min(n - 1);
        // dither the seam between two bands
        let seam = (y - y0) * n % (y1 - y0).max(1);
        for x in 0..W {
            let c = if k + 1 < n && seam > (y1 - y0) * 3 / 4 && (x + y) % 2 == 0 { cols[(k + 1) as usize] } else { cols[k as usize] };
            f.set(x, y, c);
        }
    }
}

impl Scenery {
    pub fn new(place: Place, seed: u64) -> Self {
        let mut f = Frame::new();
        let mut rng = Rng(seed | 1);
        match place {
            Place::Flat => draw_flat(&mut f, &mut rng),
            Place::School => draw_school(&mut f, &mut rng),
            Place::Kooi => draw_kooi(&mut f, &mut rng),
        }
        draw_pitch(&mut f, place, &mut rng);
        draw_front(&mut f, place, &mut rng);
        Scenery { place, back: f }
    }

    pub fn draw(&self, f: &mut Frame) {
        f.copy_from(&self.back);
    }
}

// ------------------------------------------------------------ the flats

fn draw_flat(f: &mut Frame, rng: &mut Rng) {
    bands(f, 0, 64, &[rgb(88, 152, 224), rgb(104, 168, 232), rgb(128, 184, 240), rgb(152, 200, 240), rgb(176, 216, 240)]);
    // clouds
    for &(cx, cy, w) in &[(40, 30, 30), (190, 22, 40), (120, 44, 22)] {
        f.fill_ellipse(cx, cy, w / 2, 5, rgb(240, 248, 248));
        f.fill_ellipse(cx - w / 4, cy + 2, w / 3, 4, rgb(216, 232, 240));
        f.fill_ellipse(cx + w / 5, cy - 3, w / 4, 4, rgb(248, 248, 248));
    }
    // the galerijflat: five floors of galleries
    let (x0, x1, top, base) = (-4, 260, 30, 102);
    f.rect(x0, top, x1 - x0, base - top, rgb(184, 176, 160));
    f.rect(x0, top - 3, x1 - x0, 3, rgb(120, 112, 104));
    let floor_h = 14;
    let door_cols = [rgb(216, 88, 48), rgb(64, 120, 168), rgb(232, 184, 48), rgb(96, 144, 80), rgb(168, 72, 104)];
    for fl in 0..5 {
        let y = top + 2 + fl * floor_h;
        // the wall with doors and windows, set back
        f.rect(x0, y, x1 - x0, floor_h - 3, rgb(160, 152, 136));
        let mut x = 6;
        let mut k = fl as usize;
        while x < 256 {
            if (148..176).contains(&x) {
                x += 30;
                continue;
            }
            // door
            let dc = door_cols[k % door_cols.len()];
            f.rect(x, y + 2, 5, floor_h - 5, dc);
            f.rect(x + 1, y + 3, 3, 2, shade(dc, 3, 2).min(0xF8F8F8));
            // kitchen window with a curtain
            f.rect(x + 8, y + 2, 12, 5, rgb(72, 96, 128));
            f.rect(x + 8, y + 2, 12, 1, rgb(232, 232, 216));
            if rng.int(3) == 0 {
                f.rect(x + 8, y + 2, 4, 5, rgb(232, 216, 176));
            }
            if rng.int(4) == 0 {
                // a plant on the sill
                f.rect(x + 15, y + 5, 3, 2, rgb(72, 144, 72));
            }
            x += 26;
            k += 1;
        }
        // the gallery: a concrete edge and a railing
        f.rect(x0, y + floor_h - 3, x1 - x0, 3, rgb(208, 200, 184));
        f.hline(x0, x1, y + floor_h - 1, rgb(128, 120, 112));
        f.hline(x0, x1, y + floor_h - 6, rgb(96, 96, 104));
        let mut px = 0;
        while px < 256 {
            f.rect(px, y + floor_h - 6, 1, 4, rgb(96, 96, 104));
            px += 4;
        }
    }
    // the stairwell of brick
    f.rect(146, top - 8, 30, base - top + 8, rgb(152, 80, 56));
    for y in (top - 8..base).step_by(4) {
        f.hline(146, 175, y, rgb(120, 64, 48));
        let off = if (y / 4) % 2 == 0 { 0 } else { 4 };
        let mut x = 146 + off;
        while x < 176 {
            f.set(x, y + 2, rgb(120, 64, 48));
            x += 8;
        }
    }
    for k in 0..5 {
        f.rect(152, top + 2 + k * floor_h, 18, 8, rgb(104, 136, 168));
        f.rect(152, top + 2 + k * floor_h, 18, 1, rgb(176, 200, 216));
    }
    // the ground floor: storage boxes with garage doors, and a hedge
    f.rect(x0, base - 10, x1 - x0, 10, rgb(144, 136, 128));
    let mut x = 2;
    while x < 256 {
        f.rect(x, base - 9, 16, 9, rgb(104, 112, 120));
        for yy in (base - 8..base).step_by(2) {
            f.hline(x + 1, x + 14, yy, rgb(88, 96, 104));
        }
        x += 20;
    }
    f.rect(0, base, W, 16, rgb(176, 168, 152));
    for x in (0..W).step_by(10) {
        f.rect(x, base, 1, 16, rgb(152, 144, 128));
    }
    f.hline(0, W - 1, base + 8, rgb(152, 144, 128));
    // a hedge and two little trees
    for x in (60..140).step_by(5) {
        f.fill_ellipse(x, base + 2, 4, 3, rgb(56, 120, 64));
        f.set(x - 1, base, rgb(88, 160, 80));
    }
    for &tx in &[200, 236] {
        f.rect(tx, base - 18, 3, 22, rgb(104, 72, 48));
        f.fill_ellipse(tx + 1, base - 22, 10, 9, rgb(48, 112, 56));
        f.fill_ellipse(tx - 2, base - 25, 6, 5, rgb(72, 144, 72));
    }
}

// ------------------------------------------------------------ the school

fn draw_school(f: &mut Frame, rng: &mut Rng) {
    bands(f, 0, 70, &[rgb(72, 136, 216), rgb(96, 160, 232), rgb(120, 176, 240), rgb(160, 200, 240)]);
    for &(cx, cy, w) in &[(70, 24, 40), (200, 34, 30)] {
        f.fill_ellipse(cx, cy, w / 2, 5, rgb(240, 248, 248));
        f.fill_ellipse(cx + w / 5, cy - 3, w / 4, 4, rgb(248, 248, 248));
    }
    let base = 102;
    // the school on the right: red brick, big windows, a little clock tower
    let (sx0, sx1, top) = (110, 262, 44);
    f.rect(sx0, top, sx1 - sx0, base - top, rgb(176, 80, 56));
    for y in (top..base).step_by(3) {
        f.hline(sx0, sx1, y, rgb(152, 64, 48));
    }
    f.fill_poly(&[(sx0 - 4, top), (sx1 + 4, top), (sx1 - 6, top - 12), (sx0 + 6, top - 12)], rgb(96, 64, 72));
    f.hline(sx0 - 4, sx1 + 4, top, rgb(64, 40, 48));
    // tower with the clock
    f.rect(170, top - 30, 22, 30, rgb(176, 80, 56));
    f.fill_poly(&[(168, top - 30), (194, top - 30), (181, top - 44)], rgb(72, 96, 88));
    f.fill_ellipse(181, top - 18, 6, 6, rgb(240, 240, 224));
    f.circle_outline(181, top - 18, 6, OUTLINE);
    f.line(181, top - 18, 181, top - 22, OUTLINE);
    f.line(181, top - 18, 184, top - 17, OUTLINE);
    // windows, two rows
    for row in 0..2 {
        let y = top + 6 + row * 26;
        let mut x = sx0 + 6;
        while x < sx1 - 10 {
            if !(166..196).contains(&x) {
                f.rect(x, y, 16, 18, rgb(240, 240, 232));
                f.rect(x + 1, y + 1, 14, 16, rgb(96, 136, 176));
                f.rect(x + 1, y + 1, 14, 3, rgb(136, 176, 208));
                f.rect(x + 7, y, 2, 18, rgb(240, 240, 232));
                f.rect(x, y + 8, 16, 2, rgb(240, 240, 232));
                // drawings on the window
                if rng.int(2) == 0 {
                    let c = [rgb(248, 216, 48), rgb(232, 64, 48), rgb(96, 176, 88)][rng.int(3) as usize];
                    f.rect(x + 2, y + 11, 4, 4, c);
                }
            }
            x += 22;
        }
    }
    // the door with the name
    f.rect(172, base - 22, 18, 22, rgb(64, 96, 152));
    f.rect(180, base - 22, 1, 22, OUTLINE);
    f.rect(150, top + 2, 62, 9, rgb(240, 232, 200));
    f.text(152, top + 3, "OBS DE", rgb(64, 48, 40));
    f.text(190, top + 3, "BAL", rgb(200, 48, 40));
    // Mevrouw Knorr's garden on the left: her house, flowers, a fence
    f.rect(-2, 50, 100, base - 50, rgb(232, 224, 200));
    f.fill_poly(&[(-6, 50), (102, 50), (80, 30), (14, 30)], rgb(160, 64, 48));
    for y in (32..50).step_by(3) {
        f.hline(4, 94, y, rgb(136, 48, 40));
    }
    f.rect(16, 60, 18, 16, rgb(96, 136, 176));
    f.rect(16, 60, 18, 2, rgb(240, 240, 232));
    f.rect(24, 60, 2, 16, rgb(240, 240, 232));
    // lace curtains
    f.dither_rect(16, 62, 8, 6, rgb(240, 240, 232), rgb(96, 136, 176));
    f.rect(52, 66, 14, 36, rgb(56, 104, 72));
    f.rect(63, 84, 2, 2, rgb(248, 216, 48));
    f.rect(70, 60, 18, 16, rgb(96, 136, 176));
    f.rect(70, 60, 18, 2, rgb(240, 240, 232));
    // her lawn
    f.rect(0, 88, 108, base - 88, rgb(88, 160, 72));
    f.dither_rect(0, 88, 108, 2, rgb(88, 160, 72), rgb(120, 184, 88));
    // the playground fence: green posts and a mesh, the whole width
    let fy0 = 82;
    for x in (0..W).step_by(2) {
        for y in (fy0..base + 12).step_by(2) {
            if (x / 2 + y / 2) % 2 == 0 {
                let c = f.get(x, y);
                f.set(x, y, mix(c, rgb(48, 96, 64), 0.6));
            }
        }
    }
    for x in (0..W).step_by(32) {
        f.rect(x, fy0 - 2, 2, base + 14 - fy0, rgb(40, 80, 56));
    }
    f.hline(0, W - 1, fy0 - 2, rgb(40, 80, 56));
    // the playground in front of the fence
    f.rect(0, base + 12, W, 4, rgb(120, 112, 112));
}

// ------------------------------------------------------------ de kooi

fn draw_kooi(f: &mut Frame, rng: &mut Rng) {
    bands(f, 0, 96, &[rgb(40, 32, 88), rgb(72, 48, 120), rgb(120, 64, 136), rgb(184, 88, 120), rgb(232, 128, 96), rgb(248, 176, 104)]);
    for _ in 0..30 {
        let (x, y) = (rng.int(256), rng.int(36));
        f.set(x, y, rgb(232, 232, 248));
    }
    // the city behind: flats with lit windows
    let blocks = [(0, 52, 40), (36, 40, 30), (62, 60, 50), (108, 34, 26), (130, 56, 44), (170, 46, 36), (202, 62, 30), (228, 38, 30)];
    for &(x, top, w) in &blocks {
        f.rect(x, top, w, 104 - top, rgb(48, 40, 72));
        let mut y = top + 4;
        while y < 98 {
            let mut wx = x + 3;
            while wx < x + w - 3 {
                if rng.int(3) > 0 {
                    let c = if rng.int(5) == 0 { rgb(160, 200, 248) } else { rgb(248, 216, 120) };
                    f.rect(wx, y, 3, 3, c);
                } else {
                    f.rect(wx, y, 3, 3, rgb(32, 28, 56));
                }
                wx += 6;
            }
            y += 7;
        }
    }
    // the road behind the cage
    f.rect(0, 96, W, 8, rgb(56, 56, 72));
    for x in (0..W).step_by(16) {
        f.rect(x, 100, 8, 1, rgb(200, 192, 160));
    }
    f.rect(0, 104, W, 14, rgb(72, 80, 72));
    // the cage: posts and mesh along the far side
    let cy0 = 44;
    for x in (0..W).step_by(2) {
        for y in (cy0..118).step_by(2) {
            if (x / 2 + y / 2) % 2 == 0 {
                let c = f.get(x, y);
                f.set(x, y, mix(c, rgb(152, 160, 176), 0.45));
            }
        }
    }
    for x in (4..W).step_by(36) {
        f.rect(x, cy0 - 4, 3, 118 - cy0 + 4, rgb(96, 104, 120));
        f.rect(x, cy0 - 4, 1, 118 - cy0 + 4, rgb(144, 152, 168));
    }
    f.rect(0, cy0 - 4, W, 2, rgb(96, 104, 120));
    // floodlights on both ends
    for &lx in &[22, 234] {
        f.rect(lx, 6, 3, 112, rgb(72, 80, 96));
        f.rect(lx - 7, 4, 17, 6, rgb(56, 64, 80));
        for k in 0..4 {
            f.rect(lx - 6 + k * 4, 5, 3, 4, rgb(248, 248, 216));
        }
    }
    // graffiti on the boards at the back
    f.rect(0, 110, W, 8, rgb(64, 72, 64));
    let tags = [(30, "HELICO", rgb(248, 104, 160)), (110, "TIENEN", rgb(96, 224, 248)), (190, "R+J", rgb(248, 216, 48))];
    for &(x, s, c) in &tags {
        f.text(x, 110, s, c);
    }
}

// ------------------------------------------------------------ the pitch

fn draw_pitch(f: &mut Frame, place: Place, rng: &mut Rng) {
    let fy = far_y() - 14;
    let ny = near_y() + 6;
    // the ground around the pitch
    let (base_a, base_b) = match place {
        Place::Flat => (rgb(168, 160, 152), rgb(152, 144, 136)),
        Place::School => (rgb(112, 104, 112), rgb(104, 96, 104)),
        Place::Kooi => (rgb(56, 136, 72), rgb(48, 120, 64)),
    };
    for y in fy..ny {
        for x in 0..W {
            let c = match place {
                // tiles: 8x6 with a darker seam
                Place::Flat => {
                    if (x + (y / 6) * 4) % 8 == 0 || y % 6 == 0 {
                        base_b
                    } else {
                        base_a
                    }
                }
                // asphalt: speckles
                Place::School => {
                    if (x * 7 + y * 13) % 11 == 0 {
                        base_b
                    } else {
                        base_a
                    }
                }
                // turf: mowing stripes in depth
                Place::Kooi => {
                    if ((x + (y - fy) / 2) / 16) % 2 == 0 {
                        base_a
                    } else {
                        base_b
                    }
                }
            };
            f.set(x, y, c);
        }
    }
    if place == Place::Flat {
        // a few weeds between the tiles
        for _ in 0..30 {
            let (x, y) = (rng.int(256), fy + rng.int(ny - fy));
            f.set(x, y, rgb(96, 152, 72));
        }
    }
    if place == Place::School {
        // hopscotch near the front
        let (hx, hy) = (196, near_y() - 10);
        for k in 0..4 {
            f.rect_outline(hx + k * 9, hy, 9, 7, rgb(248, 216, 96));
        }
        f.text(hx + 2, hy, "1", rgb(248, 216, 96));
    }
    // the wall behind the goal
    let wall = match place {
        Place::Flat => rgb(136, 128, 120),
        Place::School => rgb(176, 80, 56),
        Place::Kooi => rgb(80, 88, 104),
    };
    let (wx0, wy0) = proj(WALL_X, -8.0, 0.0);
    let (wx1, wy1) = proj(WALL_X, DEPTH + 6.0, 0.0);
    let h = 52;
    f.fill_poly(&[(0, wy0 - h), (wx0, wy0 - h), (wx1, wy1 - h), (wx1, wy1), (0, wy1)], wall);
    f.fill_poly(&[(wx0, wy0 - h), (wx0 + 3, wy0 - h), (wx1 + 3, wy1 - h), (wx1, wy1 - h)], shade(wall, 4, 3).min(0xF8F8F8));
    for k in 1..8 {
        let t = k as f32 / 8.0;
        let (ax, ay) = (wx0 as f32 + (wx1 - wx0) as f32 * t, (wy0 - h) as f32 + (wy1 - wy0) as f32 * t);
        f.line(0, ay as i32 + 8, ax as i32, ay as i32 + 8, shade(wall, 7, 8));
    }
    // chalk lines
    let chalk = if place == Place::Kooi { rgb(240, 248, 240) } else { rgb(240, 240, 232) };
    let line = |f: &mut Frame, a: (f32, f32), b: (f32, f32)| {
        let (x0, y0) = proj(a.0, a.1, 0.0);
        let (x1, y1) = proj(b.0, b.1, 0.0);
        f.line(x0, y0, x1, y1, chalk);
    };
    line(f, (GOAL_X, 0.0), (FIELD_X1, 0.0));
    line(f, (GOAL_X, DEPTH), (FIELD_X1, DEPTH));
    line(f, (FIELD_X1, 0.0), (FIELD_X1, DEPTH));
    line(f, (GOAL_X, 0.0), (GOAL_X, DEPTH));
    // the keeper's area
    line(f, (GOAL_X, AREA_Y0), (AREA_X, AREA_Y0));
    line(f, (AREA_X, AREA_Y0), (AREA_X, AREA_Y1));
    line(f, (GOAL_X, AREA_Y1), (AREA_X, AREA_Y1));
    // the spot for the latjetrap
    let (sx, sy) = proj(SPOT_X, goal_mid(), 0.0);
    f.rect(sx - 1, sy, 3, 1, chalk);
    let _ = mix(chalk, chalk, 0.0);
}

// ------------------------------------------------------------ the front

fn draw_front(f: &mut Frame, place: Place, rng: &mut Rng) {
    let y0 = near_y() + 6;
    match place {
        Place::Flat => {
            // kerb, street, a drain
            f.rect(0, y0, W, 3, rgb(208, 200, 192));
            f.hline(0, W - 1, y0 + 3, rgb(120, 112, 112));
            f.rect(0, y0 + 4, W, 224 - y0 - 4, rgb(88, 88, 96));
            for x in (0..W).step_by(24) {
                f.rect(x, y0 + 18, 12, 2, rgb(232, 232, 224));
            }
            f.rect(8, y0 + 5, 12, 4, rgb(48, 48, 56));
            for k in 0..4 {
                f.rect(9 + k * 3, y0 + 6, 1, 2, rgb(96, 96, 104));
            }
        }
        Place::School => {
            // a low wall with a bike rack
            f.rect(0, y0, W, 224 - y0, rgb(176, 80, 56));
            for y in (y0..224).step_by(4) {
                f.hline(0, W - 1, y, rgb(152, 64, 48));
            }
            f.rect(0, y0, W, 3, rgb(200, 192, 176));
            for x in (20..120).step_by(10) {
                f.thick_line(x, y0 + 4, x, y0 + 14, 2, rgb(160, 168, 176));
                f.hline(x - 3, x + 3, y0 + 4, rgb(160, 168, 176));
            }
        }
        Place::Kooi => {
            // kick boards with tags
            f.rect(0, y0, W, 224 - y0, rgb(104, 72, 48));
            for x in (0..W).step_by(16) {
                f.rect(x, y0, 1, 224 - y0, rgb(72, 48, 32));
            }
            f.rect(0, y0, W, 2, rgb(136, 96, 64));
            let cols = [rgb(248, 104, 160), rgb(96, 224, 248), rgb(248, 216, 48), rgb(120, 232, 120)];
            for k in 0..6 {
                let x = 10 + k * 42 + rng.int(10);
                let c = cols[rng.int(4) as usize];
                f.thick_line(x, y0 + 8, x + 14, y0 + 6, 2, c);
                f.thick_line(x + 3, y0 + 13, x + 18, y0 + 11, 2, c);
            }
        }
    }
}
