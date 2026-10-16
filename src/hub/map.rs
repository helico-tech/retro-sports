//! The static island: sea, beaches, fields, mountain, village and the
//! venues' fixed props. Drawn once into a Frame; the hub animates on top.

use crate::engine::gfx::*;
use crate::engine::rng::Rng;
use crate::engine::ui::OUTLINE;

pub const MAP_TOP: i32 = 44;
pub const MAP_BOTTOM: i32 = 180;

const CX: f32 = 128.0;
const CY: f32 = 114.0;
const RX: f32 = 114.0;
const RY: f32 = 54.0;

/// Distance from the island centre, scaled so the shoreline is 1.0.
pub fn island_d(x: f32, y: f32) -> f32 {
    let dx = (x - CX) / RX;
    let dy = (y - CY) / RY;
    let a = dy.atan2(dx);
    let r = 1.0 + 0.10 * (3.0 * a + 0.7).sin() + 0.06 * (5.0 * a + 2.1).sin() + 0.035 * (9.0 * a + 0.3).sin();
    (dx * dx + dy * dy).sqrt() / r
}

pub fn is_sea(x: i32, y: i32) -> bool {
    island_d(x as f32, y as f32) >= 1.0 && !in_beach(x, y)
}

/// The big beach in the south-west, where the dodgeball court is.
fn in_beach(x: i32, y: i32) -> bool {
    let (dx, dy) = ((x - 40) as f32 / 30.0, (y - 156) as f32 / 14.0);
    dx * dx + dy * dy < 1.0
}

// palette
pub const SEA_DEEP: u32 = rgb(24, 80, 168);
pub const SEA: u32 = rgb(32, 104, 192);
pub const SHALLOW: u32 = rgb(56, 152, 208);
pub const SHALLOW_2: u32 = rgb(72, 176, 216);
pub const FOAM: u32 = rgb(216, 240, 248);
pub const SAND: u32 = rgb(240, 216, 144);
pub const SAND_D: u32 = rgb(216, 184, 112);
const BANK: u32 = rgb(184, 136, 80);
const BANK_D: u32 = rgb(136, 96, 64);
pub const GRASS: u32 = rgb(96, 184, 72);
pub const GRASS_2: u32 = rgb(80, 160, 64);
const GRASS_D: u32 = rgb(56, 128, 56);
pub const TREE: u32 = rgb(40, 112, 56);
pub const TREE_L: u32 = rgb(88, 168, 72);
pub const ROCK: u32 = rgb(168, 160, 152);
pub const ROCK_D: u32 = rgb(112, 104, 120);
pub const SNOW: u32 = rgb(240, 248, 248);
pub const SNOW_D: u32 = rgb(176, 200, 224);
pub const DITCH: u32 = rgb(64, 136, 200);
const PATH: u32 = rgb(248, 240, 200);
const PATH_D: u32 = rgb(176, 144, 96);
pub const WOOD: u32 = rgb(144, 88, 48);
pub const WOOD_D: u32 = rgb(96, 56, 40);
pub const ROOF: u32 = rgb(208, 72, 56);
pub const ROOF_D: u32 = rgb(152, 48, 48);
pub const WALL: u32 = rgb(248, 232, 200);

/// Smooth value noise in 0..1, for patches of color.
fn noise(x: f32, y: f32) -> f32 {
    let h = |i: i32, j: i32| -> f32 {
        let mut n = (i as u32).wrapping_mul(374761393) ^ (j as u32).wrapping_mul(668265263);
        n = (n ^ (n >> 13)).wrapping_mul(1274126177);
        (n >> 8) as f32 / (1u32 << 24) as f32
    };
    let (i, j) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - i as f32, y - j as f32);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let a = h(i, j) + (h(i + 1, j) - h(i, j)) * sx;
    let b = h(i, j + 1) + (h(i + 1, j + 1) - h(i, j + 1)) * sx;
    a + (b - a) * sy
}

/// Fixed places on the map.
pub struct Spots {
    /// sea pixels where glints sparkle
    pub glints: Vec<(i32, i32)>,
}

pub fn build(f: &mut Frame) -> Spots {
    // sea, shoreline and land, pixel by pixel
    for y in MAP_TOP - 8..MAP_BOTTOM + 8 {
        for x in 0..W {
            let d = island_d(x as f32, y as f32);
            let n = noise(x as f32 / 9.0, y as f32 / 6.0);
            let chk = (x + y) & 1 == 0;
            let c = if in_beach(x, y) && d >= 0.9 {
                if (x * 7 + y * 13) % 23 == 0 { SAND_D } else { SAND }
            } else if d < 0.9 {
                let g = if n > 0.62 { if chk { GRASS_2 } else { GRASS } } else if n > 0.5 && chk { GRASS_2 } else { GRASS };
                if d > 0.86 && chk { GRASS_D } else { g }
            } else if d < 1.0 {
                if (x * 7 + y * 13) % 23 == 0 { SAND_D } else { SAND }
            } else if d < 1.05 {
                FOAM
            } else if d < 1.22 {
                if d > 1.16 && chk { SEA } else if n > 0.55 && chk { SHALLOW_2 } else { SHALLOW }
            } else if n > 0.6 && chk {
                SEA_DEEP
            } else {
                SEA
            };
            f.set(x, y, c);
        }
    }
    // the island has thickness: a sand bank under its south shore
    for x in 0..W {
        for y in (MAP_TOP..MAP_BOTTOM + 8).rev() {
            let land = |yy: i32| !is_sea(x, yy);
            if !land(y) && (land(y - 1) || land(y - 2) || land(y - 3)) {
                let k = (1..=3).find(|k| land(y - k)).unwrap_or(3);
                f.set(x, y, if k == 3 { BANK_D } else { BANK });
            }
        }
    }

    village(f);
    trapveldje(f);
    mountain(f);
    polder(f);
    park(f);
    beach(f);
    cliff(f);
    paths(f);

    let mut rng = Rng(77);
    let mut glints = Vec::new();
    while glints.len() < 48 {
        let (x, y) = (rng.int(W - 8) + 4, rng.int(MAP_BOTTOM - MAP_TOP) + MAP_TOP);
        if island_d(x as f32, y as f32) > 1.2 && is_sea(x + 3, y) {
            glints.push((x, y));
        }
    }
    Spots { glints }
}

fn tree(f: &mut Frame, x: i32, y: i32, r: i32) {
    f.shade_ellipse(x + 2, y + r - 1, r, r / 2 + 1, 5, 8);
    f.fill_ellipse(x, y, r + 1, r + 1, OUTLINE);
    f.fill_ellipse(x, y, r, r, TREE);
    f.fill_ellipse(x - r / 3, y - r / 3, r / 2 + 1, r / 2 + 1, TREE_L);
    f.set(x - r / 2, y - r / 2, rgb(160, 216, 112));
}

fn pine(f: &mut Frame, x: i32, y: i32, h: i32) {
    f.fill_poly(&[(x, y - h - 1), (x - h / 2 - 1, y + 1), (x + h / 2 + 1, y + 1)], OUTLINE);
    f.fill_poly(&[(x, y - h), (x - h / 2, y), (x + h / 2, y)], TREE);
    f.fill_poly(&[(x, y - h), (x - h / 2, y), (x, y)], rgb(56, 136, 72));
}

fn house(f: &mut Frame, x: i32, y: i32, tower: bool) {
    let (w, h) = (7, 5);
    f.rect(x - 1, y - 1, w + 2, h + 2, OUTLINE);
    f.rect(x, y, w, h, WALL);
    f.rect(x + w - 2, y, 2, h, rgb(208, 184, 152));
    f.set(x + 2, y + 2, rgb(64, 96, 160));
    f.rect(x + 4, y + 3, 1, 2, WOOD_D);
    f.fill_poly(&[(x - 2, y), (x + w / 2, y - 5), (x + w + 1, y)], OUTLINE);
    f.fill_poly(&[(x - 1, y - 1), (x + w / 2, y - 4), (x + w, y - 1)], ROOF);
    f.hline(x + w / 2, x + w - 1, y - 1, ROOF_D);
    if tower {
        f.rect(x + 1, y - 13, 5, 10, OUTLINE);
        f.rect(x + 2, y - 12, 3, 9, WALL);
        f.set(x + 3, y - 10, rgb(64, 64, 96));
        f.fill_poly(&[(x + 1, y - 13), (x + 3, y - 19), (x + 5, y - 13)], OUTLINE);
        f.fill_poly(&[(x + 2, y - 13), (x + 3, y - 17), (x + 4, y - 13)], rgb(72, 160, 136));
    }
}

fn village(f: &mut Frame) {
    for (x, y, t) in [(104, 74, false), (114, 70, true), (126, 75, false), (96, 82, false), (134, 82, false), (118, 84, false)] {
        house(f, x, y, t);
    }
    // a canal along the front of the houses, with a quay and a bridge
    let (cx, cy, cw) = CANAL;
    f.rect(cx - 1, cy - 2, cw + 2, 7, OUTLINE);
    f.rect(cx, cy - 1, cw, 5, rgb(184, 176, 168));
    f.rect(cx, cy, cw, 3, DITCH);
    f.hline(cx + 2, cx + 8, cy + 1, rgb(120, 192, 232));
    f.hline(cx + 26, cx + 31, cy + 2, rgb(120, 192, 232));
    let bx = cx + cw - 12;
    f.rect(bx - 1, cy - 2, 7, 6, OUTLINE);
    f.rect(bx, cy - 1, 5, 4, rgb(160, 72, 56));
    f.set(bx + 2, cy + 2, DITCH);
    // bikes parked along the quay
    for k in 0..5 {
        let x = cx + 3 + k * 6;
        f.set(x, cy - 3, OUTLINE);
        f.set(x + 2, cy - 3, OUTLINE);
        f.set(x + 1, cy - 4, [ROOF, rgb(48, 48, 64), rgb(64, 120, 216)][(k % 3) as usize]);
    }
}

/// The canal through the village: left x, water top y, width.
pub const CANAL: (i32, i32, i32) = (90, 91, 50);

pub const PEAK: (i32, i32) = (196, 46);

fn mountain(f: &mut Frame) {
    // a second, farther peak
    f.fill_poly(&[(218, 56), (196, 92), (242, 92)], OUTLINE);
    f.fill_poly(&[(218, 58), (199, 91), (240, 91)], ROCK_D);
    f.fill_poly(&[(218, 58), (212, 68), (220, 66), (224, 67)], SNOW_D);
    // the main peak, lit from the left
    let (px, py) = PEAK;
    f.fill_poly(&[(px, py - 1), (158, 103), (234, 103)], OUTLINE);
    f.fill_poly(&[(px, py), (160, 102), (232, 102)], ROCK_D);
    f.fill_poly(&[(px, py), (160, 102), (px + 4, 102)], ROCK);
    for (k, (x0, y0)) in [(176, 80), (188, 88), (206, 76), (214, 90)].iter().enumerate() {
        f.line(*x0, *y0, x0 + 6 - k as i32 % 2 * 3, y0 + 4, if k < 2 { rgb(136, 128, 136) } else { rgb(88, 80, 104) });
    }
    f.fill_poly(&[(px, py), (184, 64), (188, 61), (191, 66), (196, 60), (201, 65), (205, 61), (209, 66)], SNOW);
    f.fill_poly(&[(px, py), (px + 1, 63), (201, 65), (205, 61), (209, 66)], SNOW_D);
    // pines along the foot
    for (x, y, h) in [(162, 104, 7), (170, 106, 9), (178, 104, 6), (222, 104, 8), (230, 102, 6), (214, 106, 7), (156, 98, 6)] {
        pine(f, x, y, h);
    }
    // the trail zigzags up
    let pts = [(186, 100), (200, 94), (184, 86), (202, 78), (190, 70), (196, 62)];
    for w in pts.windows(2) {
        let mut k = 0;
        line_points(w[0].0, w[0].1, w[1].0, w[1].1, |x, y| {
            if k % 3 != 2 {
                f.set(x, y, rgb(208, 176, 120));
            }
            k += 1;
        });
    }
}

/// The trapveldje behind the village: left x, top y, width, height.
pub const PITCH: (i32, i32, i32, i32) = (96, 105, 20, 12);

fn trapveldje(f: &mut Frame) {
    let (x, y, w, h) = PITCH;
    f.rect(x - 1, y - 1, w + 2, h + 2, OUTLINE);
    f.rect(x, y, w, h, rgb(168, 160, 152));
    for yy in (y + 1..y + h).step_by(3) {
        for xx in (x + (yy / 3) % 2..x + w).step_by(4) {
            f.set(xx, yy, rgb(152, 144, 136));
        }
    }
    // chalk: the outline and the keeper's box
    f.rect_outline(x + 1, y + 1, w - 2, h - 2, rgb(240, 240, 232));
    f.rect_outline(x + 1, y + 3, 5, h - 6, rgb(240, 240, 232));
    // the little goal on the left
    f.rect(x - 1, y + 3, 2, h - 6, rgb(248, 248, 248));
    f.rect(x - 2, y + 3, 1, h - 6, OUTLINE);
}

pub const MILL: (i32, i32) = (146, 120);

fn polder(f: &mut Frame) {
    // fields with ditches
    for y in (124..152).step_by(6) {
        for x in 96..166 {
            if island_d(x as f32, y as f32) < 0.86 {
                f.set(x, y, DITCH);
            }
        }
    }
    // tulip rows
    let colors = [rgb(232, 56, 72), rgb(248, 216, 64), rgb(248, 128, 168), rgb(240, 112, 48)];
    for (k, y) in (127..149).step_by(6).enumerate() {
        for x in (100..124).step_by(2) {
            f.rect(x, y, 1, 3, colors[k % 4]);
        }
    }
    // the road the cyclist uses
    f.rect(92, 154, 76, 3, rgb(184, 176, 168));
    f.hline(92, 167, 155, rgb(208, 200, 192));
    // windmill body (sails are animated)
    let (mx, my) = MILL;
    f.fill_poly(&[(mx - 5, my + 13), (mx - 3, my - 1), (mx + 3, my - 1), (mx + 5, my + 13)], OUTLINE);
    f.fill_poly(&[(mx - 4, my + 12), (mx - 2, my), (mx + 2, my), (mx + 4, my + 12)], rgb(120, 96, 80));
    f.fill_poly(&[(mx, my), (mx + 2, my), (mx + 4, my + 12), (mx, my + 12)], rgb(88, 72, 64));
    f.rect(mx - 1, my + 8, 2, 4, WOOD_D);
    f.fill_poly(&[(mx - 4, my), (mx, my - 4), (mx + 4, my)], OUTLINE);
    f.fill_poly(&[(mx - 3, my - 1), (mx, my - 3), (mx + 3, my - 1)], rgb(64, 128, 96));
    // a few cows dotted in the field
    for (x, y) in [(132, 140), (150, 134), (158, 146)] {
        f.rect(x - 1, y - 1, 6, 4, OUTLINE);
        f.rect(x, y, 4, 2, WALL);
        f.set(x + 1, y, OUTLINE);
        f.set(x + 4, y, rgb(248, 184, 184));
    }
}

pub const POND: (i32, i32) = (66, 108);

fn park(f: &mut Frame) {
    f.fill_ellipse(POND.0, POND.1, 10, 5, OUTLINE);
    f.fill_ellipse(POND.0, POND.1, 9, 4, DITCH);
    f.fill_ellipse(POND.0 - 2, POND.1 - 1, 4, 1, rgb(120, 192, 232));
    for (x, y, r) in [(44, 92, 5), (54, 86, 6), (86, 90, 5), (42, 108, 4), (92, 104, 4), (66, 84, 4), (78, 120, 4)] {
        tree(f, x, y, r);
    }
    // picnic blanket
    for yy in 0..4 {
        for xx in 0..6 {
            f.set(84 + xx, 110 + yy, if (xx / 2 + yy / 2) % 2 == 0 { rgb(232, 64, 64) } else { WALL });
        }
    }
}

pub const COURT: (i32, i32, i32, i32) = (28, 150, 30, 14);

fn palm(f: &mut Frame, x: i32, y: i32, lean: i32) {
    f.shade_ellipse(x + 4, y, 6, 2, 5, 8);
    for k in 0..12 {
        let xx = x + lean * k / 12;
        f.rect(xx - 1, y - k, 3, 1, OUTLINE);
    }
    for k in 0..12 {
        let xx = x + lean * k / 12;
        f.set(xx, y - k, if k % 3 == 0 { WOOD_D } else { WOOD });
    }
    let (tx, ty) = (x + lean, y - 12);
    for (dx, dy) in [(-6, 2), (6, 2), (-4, -3), (5, -3), (0, -4)] {
        f.thick_line(tx, ty, tx + dx, ty + dy, 3, OUTLINE);
    }
    for (dx, dy) in [(-6, 2), (6, 2), (-4, -3), (5, -3), (0, -4)] {
        f.line(tx, ty, tx + dx, ty + dy, TREE_L);
        f.line(tx, ty + 1, tx + dx, ty + dy + 1, TREE);
    }
}

fn beach(f: &mut Frame) {
    let (x, y, w, h) = COURT;
    f.rect(x, y, w, h, rgb(248, 232, 176));
    f.rect_outline(x, y, w, h, WALL);
    f.rect(x + w / 2, y, 1, h, WALL);
    // bunting poles
    for (px, c) in [(x - 2, rgb(232, 64, 64)), (x + w + 1, rgb(64, 160, 232))] {
        f.rect(px, y - 6, 1, 7, WOOD_D);
        f.set(px, y - 7, c);
    }
    palm(f, 18, 148, 3);
    palm(f, 66, 168, -3);
    // a beach umbrella
    f.fill_poly(&[(72, 146), (78, 142), (84, 146)], OUTLINE);
    f.fill_poly(&[(73, 145), (78, 143), (83, 145)], rgb(248, 96, 96));
    f.rect(78, 146, 1, 5, WALL);
}

pub const LIGHTHOUSE: (i32, i32) = (238, 118);
pub const RAMP: (i32, i32) = (224, 128);

fn cliff(f: &mut Frame) {
    // rocky headland on the east coast
    let rock = [(206, 124), (214, 116), (232, 112), (246, 118), (248, 140), (240, 158), (222, 160), (210, 146)];
    f.fill_poly(&rock, ROCK_D);
    let top = [(208, 124), (215, 117), (231, 113), (244, 119), (245, 136), (236, 148), (222, 150), (212, 142)];
    f.fill_poly(&top, rgb(120, 168, 80));
    f.fill_poly(&[(222, 150), (236, 148), (245, 136), (248, 140), (240, 158), (222, 160)], rgb(96, 88, 104));
    for (x, y) in [(226, 154), (236, 152), (242, 146)] {
        f.line(x, y, x + 2, y + 4, rgb(72, 64, 88));
    }
    // the launch ramp
    let (rx, ry) = RAMP;
    f.thick_line(rx - 8, ry - 3, rx + 2, ry + 1, 3, OUTLINE);
    f.line(rx - 8, ry - 3, rx + 2, ry + 1, WOOD);
    f.rect(rx - 6, ry - 2, 1, 4, WOOD_D);
    f.rect(rx - 1, ry, 1, 3, WOOD_D);
    // lighthouse, red and white
    let (lx, ly) = LIGHTHOUSE;
    f.rect(lx - 3, ly - 15, 7, 17, OUTLINE);
    for k in 0..15 {
        f.rect(lx - 2, ly - 14 + k, 5, 1, if (k / 3) % 2 == 0 { rgb(232, 56, 56) } else { WALL });
    }
    f.rect(lx - 2, ly - 18, 5, 4, OUTLINE);
    f.rect(lx - 1, ly - 17, 3, 2, rgb(248, 232, 120));
}

/// Waypoints of the dotted path, venue to venue.
pub const PATH_PTS: [(i32, i32); 17] = [
    (44, 146),
    (52, 132),
    (60, 122),
    (76, 116),
    (84, 110),
    (96, 102),
    (110, 100),
    (122, 106),
    (120, 116),
    (118, 126),
    (124, 146),
    (150, 140),
    (166, 120),
    (182, 108),
    (198, 116),
    (210, 128),
    (220, 136),
];

fn paths(f: &mut Frame) {
    let mut k = 0;
    for w in PATH_PTS.windows(2) {
        line_points(w[0].0, w[0].1, w[1].0, w[1].1, |x, y| {
            if k % 4 == 0 {
                f.rect(x, y + 1, 2, 1, PATH_D);
                f.rect(x, y, 2, 1, PATH);
            }
            k += 1;
        });
    }
}
