//! Draws a race: the world seen from one rider, the riders on their bikes,
//! effects and popups, the HUD (full in 1P, a slim one per half in 2P)
//! and the banners.

use super::art::*;
use super::data::*;
use super::race::*;
use super::scenery::*;
use crate::engine::gfx::*;
use crate::engine::sprites::Palette;
use crate::engine::ui::{grad6, grad7, LOGO_SHADOW, OUTLINE, SUNSET};

/// Screen x of the rider the camera follows.
pub const CAM_X: f32 = 70.0;

pub fn cam_for(race: &Race, i: usize) -> f32 {
    let r = &race.riders[i];
    let x = if r.riding() { r.x } else { r.crash_x };
    x - CAM_X
}

fn bike_pal(who: Who) -> Pal {
    let d = who.def();
    pal_with(d.frame, d.acc)
}

/// The colour the HUD and route map use for a rider.
pub fn rider_color(race: &Race, i: usize) -> u32 {
    match race.riders[i].ctrl {
        Ctrl::Human(0) => rgb(248, 232, 64),
        Ctrl::Human(_) => rgb(96, 224, 248),
        Ctrl::Cpu => race.riders[i].who.def().color,
    }
}

// ------------------------------------------------------------------ riders

/// A bike with nobody on it.
pub fn draw_bike_only(f: &mut Frame, art: &Art, who: Who, bike: Bike, sx: i32, sy: i32, wheel: f32) {
    let g = geo(bike);
    let bp = bike_pal(who);
    let wf = ((wheel / 1.6) as i32).rem_euclid(4) as usize;
    for (c, r) in [(g.rear, g.rear_r), (g.front, g.front_r)] {
        art.wheel(r, bike == Bike::Race, wf).draw(f, sx + c.0 - r - 1, sy + c.1 - r - 1, &bp, false);
    }
    art.bikes[bike.index()].draw(f, sx - BIKE_AX, sy - BIKE_AY, &bp, false);
}

/// A plain rider (no cargo, no extras) in any palette: the extras on the road.
#[allow(clippy::too_many_arguments)]
pub fn draw_bike_rider_pal(f: &mut Frame, art: &Art, who: Who, bike: Bike, sx: i32, sy: i32, crank: usize, wheel: f32, pal: &Palette, bp: Pal, _t: u32) {
    let g = geo(bike);
    let wf = ((wheel / 1.6) as i32).rem_euclid(4) as usize;
    for (c, r) in [(g.rear, g.rear_r), (g.front, g.front_r)] {
        art.wheel(r, false, wf).draw(f, sx + c.0 - r - 1, sy + c.1 - r - 1, &bp, false);
    }
    art.bikes[bike.index()].draw(f, sx - BIKE_AX, sy - BIKE_AY, &bp, false);
    blit_rider(f, &art.set(who, bike).ride[0][crank % 8], sx, sy, pal, H);
}

/// Everything that sits on the bike: rider, bike, wheels, cargo, extras.
#[allow(clippy::too_many_arguments)]
pub fn draw_bike_rider(f: &mut Frame, art: &Art, who: Who, bike: Bike, sx: i32, sy: i32, crank: usize, wheel: f32, tuck: bool, hop: bool, wave: bool, cargo: Option<(Cargo, i32)>, t: u32) {
    let g = geo(bike);
    let bp = bike_pal(who);
    let wf = ((wheel / 1.6) as i32).rem_euclid(4) as usize;
    let thin = bike == Bike::Race;
    // wheels
    for (c, r) in [(g.rear, g.rear_r), (g.front, g.front_r)] {
        let w = art.wheel(r, thin, wf);
        w.draw(f, sx + c.0 - r - 1, sy + c.1 - r - 1, &bp, false);
    }
    // the far crank arm and pedal
    let a = std::f32::consts::TAU * crank as f32 / 8.0;
    let (cx, cy) = (sx + g.crank.0, sy + g.crank.1);
    let back = (cx - (a.cos() * 3.0).round() as i32, cy - (a.sin() * 3.0).round() as i32);
    f.line(cx, cy, back.0, back.1, rgb(88, 88, 104));
    art.bikes[bike.index()].draw(f, sx - BIKE_AX, sy - BIKE_AY, &bp, false);
    // cargo, groceries, kids
    match who {
        Who::Oma => {
            // a shopping bag on the rack with a leek sticking out
            let (rx, ry) = (sx + g.rack.0, sy + g.rack.1);
            f.rect(rx + 1, ry - 6, 7, 7, OUTLINE);
            f.rect(rx + 2, ry - 5, 5, 6, rgb(56, 120, 88));
            f.hline(rx + 2, rx + 6, ry - 3, rgb(40, 88, 64));
            f.line(rx + 3, ry - 6, rx + 1, ry - 12, rgb(232, 240, 216));
            f.line(rx + 4, ry - 6, rx + 2, ry - 12, rgb(232, 240, 216));
            f.line(rx + 1, ry - 12, rx - 1, ry - 15, rgb(72, 168, 72));
            f.line(rx + 2, ry - 12, rx + 2, ry - 16, rgb(96, 192, 80));
        }
        Who::DeVries => {
            let (bx, by) = (sx + g.rack.0, sy + g.rack.1);
            let bob = ((t / 9) % 2) as i32;
            art.kids[0].draw(f, bx + 1, by - 6 - bob, &PROPS, false);
            art.kids[1].draw(f, bx + 6, by - 5 - (1 - bob), &PROPS, false);
            art.dog.draw(f, bx + 11, by - 5, &PROPS, false);
        }
        _ => {}
    }
    if let Some((c, n)) = cargo {
        draw_cargo(f, art, bike, c, n, sx, sy);
    }
    // the near crank arm, under the rider's foot
    let front = (cx + (a.cos() * 3.0).round() as i32, cy + (a.sin() * 3.0).round() as i32);
    f.line(cx, cy, front.0, front.1, rgb(200, 200, 216));
    f.set(cx, cy, rgb(232, 232, 240));
    let set = art.set(who, bike);
    let pal = rider_palette(who);
    let s = if wave {
        &set.wave
    } else if hop {
        &set.hop
    } else {
        &set.ride[tuck as usize][crank % 8]
    };
    blit_rider(f, s, sx, sy, &pal, H);
}

pub fn draw_cargo(f: &mut Frame, art: &Art, bike: Bike, c: Cargo, n: i32, sx: i32, sy: i32) {
    let g = geo(bike);
    let b = art.cargo_bmp(c);
    let (rx, ry) = (sx + g.rack.0, sy + g.rack.1);
    if bike == Bike::Bak {
        // in the box: two rows, peeking over the rim
        for k in 0..n.min(6) {
            let (col, row) = (k % 3, k / 3);
            b.draw(f, rx + col * 5 - 1, ry - b.h + 4 - row * 3, &PROPS, false);
        }
    } else {
        let step = (b.h - 2).max(3);
        for k in 0..n.min(5) {
            b.draw(f, rx + g.rack_w / 2 - b.w / 2 + (k % 2), ry - b.h + 1 - k * step, &PROPS, false);
        }
        if n > 0 {
            // a bungee cord holding it all together
            f.line(rx + 1, ry - 1, rx + g.rack_w - 1, ry - step * n.min(5) + 1, rgb(232, 64, 160));
        }
    }
}

fn draw_shadow(f: &mut Frame, x: i32, y: i32, w: i32) {
    f.shade_ellipse(x, y, w, 2, 5, 8);
}

/// Speech bubble with a tail pointing down at (x, y).
pub fn bubble(f: &mut Frame, x: i32, y: i32, text: &str) {
    let w = text_width(text, false) + 6;
    let bx = (x - w / 2).clamp(2, W - w - 2);
    let by = y - 13;
    f.rect(bx - 1, by - 1, w + 2, 12, OUTLINE);
    f.rect(bx, by, w, 10, rgb(248, 248, 248));
    for k in 0..3 {
        f.hline(x - 2 + k, x + 2 - k, by + 10 + k, if k == 2 { OUTLINE } else { rgb(248, 248, 248) });
        f.set(x - 3 + k, by + 10 + k, OUTLINE);
        f.set(x + 3 - k, by + 10 + k, OUTLINE);
    }
    f.text(bx + 3, by + 2, text, rgb(16, 16, 48));
}

fn draw_rider_in_world(f: &mut Frame, race: &Race, art: &Art, i: usize, cam: f32, t: u32) {
    let r = &race.riders[i];
    let cargo = match r.ctrl {
        Ctrl::Human(_) => Some((race.def().cargo, r.cargo)),
        Ctrl::Cpu => None,
    };
    let gy = ground_y(r.y) as i32;
    match r.state {
        RState::Ride => {
            let sx = (r.x - cam).round() as i32 + if r.wobble > 0 && (r.wobble / 3) % 2 == 0 { 1 } else { 0 };
            if !(-40..W + 40).contains(&sx) {
                return;
            }
            let sy = gy - r.z.round() as i32;
            draw_shadow(f, sx + 2, gy, 13 - (r.z / 3.0) as i32);
            let done = r.finish.is_some();
            let wave = done && r.v < 1.2 && (t / 20) % 2 == 0;
            draw_bike_rider(f, art, r.who, r.bike, sx, sy, r.crank_frame(), r.wheel, r.tuck, r.z > 0.0, wave, cargo, t);
            if r.umbrella {
                // held up in the front hand, above the head
                let sk = ride_skel(posture(r.bike, r.tuck), r.crank_frame());
                let (hx, hy) = (sx + sk[6].0 - RX + 1, sy + sk[6].1 - RY);
                let top = sy + sk[0].1 - RY - 15;
                let pal = pal_with((rgb(232, 64, 48), rgb(152, 32, 40)), (rgb(248, 136, 24), rgb(248, 248, 248)));
                f.line(hx, hy, hx, top + 6, OUTLINE);
                f.line(hx + 1, hy, hx + 1, top + 6, rgb(152, 104, 64));
                let sway = if race.gust_on { ((t / 4) % 2) as i32 } else { 0 };
                art.umbrella.draw(f, hx - 10 + sway, top, &pal, false);
            }
            // the crank glows gold while a PERFECT push is possible
            if matches!(r.ctrl, Ctrl::Human(_)) && !done && r.clean {
                let g = race.gap_for(i);
                if r.since >= g && r.since <= g + 3 {
                    let gc = geo(r.bike).crank;
                    let (cx, cy) = (sx + gc.0, sy + gc.1);
                    for (dx, dy) in [(-4, 0), (4, 0), (0, -4), (0, 4), (-3, -3), (3, 3), (3, -3), (-3, 3)] {
                        f.set(cx + dx, cy + dy, rgb(248, 232, 96));
                    }
                }
            }
            if r.bell_t > 0 {
                let g = geo(r.bike);
                let (bx, by) = (sx + g.bell.0, sy + g.bell.1);
                let ring = (r.bell_t / 3) % 2;
                f.line(bx - 3 - ring, by - 3, bx - 5 - ring, by - 5, rgb(248, 248, 96));
                f.line(bx + 3 + ring, by - 3, bx + 5 + ring, by - 5, rgb(248, 248, 96));
                f.text_sh(bx - 14, by - 13, "TRING!", rgb(248, 248, 96), OUTLINE, false);
            }
            if r.hammer > 0 {
                // the man with the hammer pays a visit: up, BONK, up, BONK
                let down = (r.hammer / 10) % 2 == 0;
                let sk = ride_skel(posture(r.bike, r.tuck), 0);
                let (hx, hy) = (sx + sk[0].0 - RX, sy + sk[0].1 - RY);
                let lift = if down { 0 } else { 9 };
                art.hammer.draw(f, hx - 6, hy - 22 - lift, &PROPS, false);
                if down {
                    for k in 0..4 {
                        let a = t as f32 * 0.2 + k as f32 * 1.57;
                        let (px, py) = (hx + (a.cos() * 9.0) as i32, hy - 6 + (a.sin() * 3.0) as i32);
                        f.set(px, py, rgb(248, 248, 96));
                        f.set(px + 1, py, rgb(248, 248, 96));
                        f.set(px, py - 1, rgb(248, 248, 200));
                    }
                }
            }
            if r.warn > 0 && (t / 4) % 2 == 0 {
                f.text_grad(sx - 2, sy - 44, "!", &grad7(rgb(248, 72, 72)), OUTLINE, true);
            }
            if r.draft > 0.35 && r.finish.is_none() {
                // slipstream: the air parts round the rider in front and
                // closes again behind you; you sit in the calm pocket
                if let Some(front) = race.riders.iter().filter(|o| o.riding() && (o.y - r.y).abs() < 0.4 && o.x - r.x > 8.0 && o.x - r.x < 56.0).map(|o| o.x).reduce(f32::min) {
                    let fx = (front - cam).round() as i32;
                    for (k, (y0, bend)) in [(sy - 36, -3), (sy + 2, 2)].iter().enumerate() {
                        let len = fx + 18 - (sx - 8);
                        for d in 0..len.max(0) {
                            let x = fx + 18 - d;
                            let ph = (d + t as i32 * 2 + k as i32 * 5) % 10;
                            if ph < 6 {
                                let mid = (x - fx).abs() < 14;
                                let y = y0 + if mid { *bend } else { 0 };
                                f.set(x, y, rgb(208, 240, 248));
                            }
                        }
                    }
                }
            }
            if let Some((s, _)) = r.say {
                bubble(f, sx + 4, sy - 34, s);
            }
            if let Ctrl::Human(p) = r.ctrl {
                if !done || (t / 20) % 2 == 0 {
                    player_tag(f, sx + 2, sy - 50 - ((t / 8) % 2) as i32, p);
                }
            }
        }
        RState::Fly => {
            let k = r.st as f32 / 34.0;
            let sx0 = r.crash_x - cam;
            // the abandoned bike, on its side
            fallen_bike(f, art, r.who, r.bike, sx0 as i32, ground_y(r.crash_y) as i32);
            let side = r.crash_y > 1.3 && r.v < 0.01;
            let (dx, top) = if side { (6.0, 6.0) } else { (34.0, 30.0) };
            let x = sx0 + dx * k;
            let y0 = ground_y(r.crash_y) - 14.0;
            let y1 = WATER_Y as f32 + 8.0;
            let y = y0 + (y1 - y0) * k - top * (k * std::f32::consts::PI).sin();
            let ps = &art.people(r.who).fly;
            blit_person(f, ps, x as i32, y as i32 + 14, false, &rider_palette(r.who));
        }
        RState::Swim => {
            let sx0 = r.crash_x - cam;
            fallen_bike(f, art, r.who, r.bike, sx0 as i32, ground_y(r.crash_y) as i32);
        }
        RState::Climb => {
            let sx0 = r.crash_x - cam;
            fallen_bike(f, art, r.who, r.bike, sx0 as i32, ground_y(r.crash_y) as i32);
        }
    }
}

/// Riders in the canal are drawn after the water.
fn draw_swimmer(f: &mut Frame, race: &Race, art: &Art, i: usize, cam: f32, t: u32) {
    let r = &race.riders[i];
    let pal = rider_palette(r.who);
    let x = (r.crash_x + if r.crash_y > 1.3 && r.v < 0.01 { 6.0 } else { 34.0 } - cam) as i32;
    match r.state {
        RState::Swim => {
            let bob = ((t / 10) % 2) as i32;
            let line = WATER_Y + 9;
            let s = &art.people(r.who).swim;
            // only the head and the waving arms stick out of the water
            blit_person_clip(f, s, x, line + 17 + bob, false, &pal, line + 1);
            f.hline(x - 9, x + 9, line + 1, rgb(200, 232, 240));
            f.hline(x - 6, x + 6, line + 2, rgb(136, 184, 192));
            if r.st < 30 {
                let rr = 3 + r.st / 3;
                f.hline(x - rr - 6, x - rr, line + 2, rgb(232, 248, 248));
                f.hline(x + rr, x + rr + 6, line + 2, rgb(232, 248, 248));
            }
            if (t / 12) % 3 == 0 {
                bubble(f, x, line - 26, if r.st < 33 { "PROEST!" } else { "BRRR!" });
            }
            // a duck comes to have a look
            let dx = x + 26 - (r.st / 3).min(12);
            art.duck.draw(f, dx, line - 4, &PROPS, true);
        }
        RState::Climb => {
            let k = r.st as f32 / 26.0;
            let ys = WATER_Y as f32 + 18.0;
            let ye = ground_y(1.0);
            let y = ys + (ye - ys) * k;
            let xx = x as f32 + ((r.crash_x - cam) - x as f32) * k;
            blit_person(f, &art.people(r.who).stand, xx as i32, y as i32, true, &wet_palette(&pal));
        }
        _ => {}
    }
}

fn fallen_bike(f: &mut Frame, art: &Art, who: Who, bike: Bike, x: i32, y: i32) {
    let bp = bike_pal(who);
    let _ = art;
    // two flattened wheels and the frame lying between them
    for dx in [-10, 10] {
        f.fill_ellipse(x + dx, y - 2, 6, 3, OUTLINE);
        f.fill_ellipse(x + dx, y - 2, 5, 2, rgb(48, 48, 64));
        f.fill_ellipse(x + dx, y - 2, 3, 1, rgb(200, 200, 216));
    }
    f.thick_line(x - 10, y - 4, x + 10, y - 5, 3, OUTLINE);
    f.line(x - 10, y - 4, x + 10, y - 5, bp[I_C as usize]);
    if bike == Bike::Bak {
        f.rect(x + 4, y - 10, 16, 7, OUTLINE);
        f.rect(x + 5, y - 9, 14, 5, rgb(160, 104, 56));
    }
}

pub fn player_tag(f: &mut Frame, x: i32, y: i32, p: usize) {
    let col = if p == 0 { rgb(248, 232, 64) } else { rgb(96, 224, 248) };
    let label = if p == 0 { "1P" } else { "2P" };
    f.rect(x - 9, y - 2, 19, 11, OUTLINE);
    f.rect(x - 8, y - 1, 17, 9, col);
    f.text(x - 5, y, label, rgb(16, 16, 48));
    for k in 0..4 {
        f.hline(x - 3 + k, x + 3 - k, y + 9 + k, OUTLINE);
    }
    for k in 0..3 {
        f.hline(x - 2 + k, x + 2 - k, y + 9 + k, col);
    }
}

// ------------------------------------------------------------------ objects

fn draw_obj(f: &mut Frame, art: &Art, o: &Obj, cam: f32, t: u32) {
    let sx = (o.x - cam).round() as i32;
    if !(-50..W + 50).contains(&sx) || o.gone {
        return;
    }
    let gy = ground_y(o.y) as i32;
    match o.kind {
        OKind::Pothole | OKind::Rails => {}
        OKind::Bollard => {
            draw_shadow(f, sx + 1, gy, 4);
            art.bollard.draw_foot(f, sx, gy, &PROPS, false);
        }
        OKind::Herring => {
            art.herring.draw_foot(f, sx, gy, &PROPS, (o.var % 2) == 0);
            if (t / 30 + o.var as u32) % 4 == 0 {
                // it still flaps a bit
                f.set(sx - 6, gy - 3, rgb(200, 200, 216));
            }
        }
        OKind::Cat => {
            if o.state == 0 {
                draw_shadow(f, sx, gy, 6);
                art.cat.draw_foot(f, sx, gy, &PROPS, false);
                // little z's floating up
                let zz = ((t / 20) % 3) as i32;
                let (zx, zy) = (sx + 5 + zz, gy - 10 - zz * 3);
                for (k, c) in [(0, OUTLINE), (1, rgb(232, 232, 248))] {
                    let o = 1 - k;
                    f.hline(zx - o, zx + 3 - o, zy - o, c);
                    f.set(zx + 2 - o, zy + 1 - o, c);
                    f.set(zx + 1 - o, zy + 2 - o, c);
                    f.hline(zx - o, zx + 3 - o, zy + 3 - o, c);
                }
            } else {
                art.cat_run.draw_foot(f, sx, gy - o.z as i32, &PROPS, true);
            }
        }
        OKind::Goose => {
            draw_shadow(f, sx, gy, 5);
            let angry = o.state == 1;
            let b = &art.goose[angry as usize];
            let hop = if angry { ((t / 4) % 2) as i32 } else { 0 };
            b.draw_foot(f, sx, gy - hop, &PROPS, o.face < 0 && !angry);
        }
        OKind::Sheep => {
            draw_shadow(f, sx, gy, 7);
            let walk = if o.state == 1 { ((t / 8) as usize + o.var as usize) % 2 } else { 0 };
            art.sheep[walk].draw_foot(f, sx, gy, &PROPS, false);
        }
        OKind::Cow => {
            draw_shadow(f, sx, gy, 12);
            art.cow.draw_foot(f, sx, gy, &PROPS, o.var % 2 == 0);
            // the tail swishes
            let tx = if o.var % 2 == 0 { sx + 12 } else { sx - 12 };
            let sw = ((t / 15) % 2) as i32;
            f.line(tx, gy - 13, tx + if o.var % 2 == 0 { 2 + sw } else { -2 - sw }, gy - 6, OUTLINE);
        }
        OKind::Tourist => {
            let frames = &art.extras.frames[(2 + o.var as usize * 2) % 6];
            let (s, flip) = match o.state {
                0 => {
                    if (o.t / 120) % 2 == 0 {
                        (&frames[5], o.face < 0)
                    } else {
                        (&frames[((o.t / 8) % 4) as usize], o.face < 0)
                    }
                }
                1 => (&frames[((o.t / 4) % 4) as usize], true),
                _ => (&frames[if (t / 20) % 2 == 0 { 6 } else { 4 }], false),
            };
            draw_shadow(f, sx, gy, 6);
            let pal = extra_palette(o.var as usize + 1);
            blit_person(f, s, sx, gy, flip, &pal);
            if o.state == 0 && (o.t / 120) % 2 == 0 {
                let cx = if o.face < 0 { sx - 9 } else { sx + 5 };
                art.camera.draw(f, cx, gy - 25, &PROPS, false);
                if o.t % 120 == 60 {
                    f.fill_ellipse(cx + 3, gy - 24, 4, 4, rgb(248, 248, 248));
                }
            }
        }
        OKind::Tractor => {
            draw_shadow(f, sx, gy, 16);
            let b = &art.tractor;
            b.draw_foot(f, sx, gy, &PROPS, false);
            // the farmer
            let ps = &art.extras.frames[2][4];
            blit_person(f, ps, sx - 2, gy - 9, false, &extra_palette(3));
            if (t / 6) % 2 == 0 {
                f.fill_ellipse(sx - 2, gy - 25, 2, 2, rgb(152, 152, 160));
            }
        }
        OKind::Sofa => {
            // a lady moving house by bike: an omafiets with a sofa on the rack
            let crank = ((o.x / 5.0) as i32).rem_euclid(8) as usize;
            draw_shadow(f, sx - 6, gy, 20);
            let mut pal = extra_palette(4);
            pal[crate::engine::sprites::SHIRT as usize] = rgb(120, 200, 120);
            pal[crate::engine::sprites::SHIRT_D as usize] = rgb(64, 144, 72);
            draw_bike_rider_pal(f, art, Who::Anouk, Bike::Oma, sx, gy, crank, o.x, &pal, pal_with((rgb(56, 104, 168), rgb(32, 64, 120)), (rgb(248, 248, 248), rgb(200, 200, 216))), t);
            art.sofa.draw(f, sx - 30, gy - 23, &PROPS, false);
            if (o.t / 90) % 3 == 0 {
                bubble(f, sx - 20, gy - 32, "MOVING DAY!");
            }
        }
        OKind::Umbrella => {
            let bob = (((t + o.var as u32 * 13) / 10) % 2) as i32;
            draw_shadow(f, sx, gy, 5);
            let pal = pal_with((rgb(232, 64, 48), rgb(152, 32, 40)), (rgb(248, 136, 24), rgb(232, 232, 240)));
            art.umbrella_fold.draw_foot(f, sx, gy - 8 - bob, &pal, false);
            if (t / 6 + o.var as u32) % 8 == 0 {
                f.set(sx + 4, gy - 16 - bob, rgb(248, 248, 248));
            }
        }
        OKind::Food(food) => {
            let bob = (((t + o.var as u32 * 13) / 10) % 2) as i32;
            draw_shadow(f, sx, gy, 4);
            let b = &art.food[food as usize];
            b.draw_foot(f, sx, gy - 9 - bob, &PROPS, false);
            if (t / 6 + o.var as u32) % 8 == 0 {
                f.set(sx + 4, gy - 16 - bob, rgb(248, 248, 248));
                f.set(sx + 3, gy - 15 - bob, rgb(248, 248, 248));
                f.set(sx + 5, gy - 15 - bob, rgb(248, 248, 248));
            }
        }
    }
}

/// Things painted on the road surface (drawn under everything else).
fn draw_flat(f: &mut Frame, race: &Race, sc: &Scenery, cam: f32, clip: (i32, i32)) {
    for o in &race.objs {
        let sx = (o.x - cam).round() as i32;
        if !(-40..W + 40).contains(&sx) {
            continue;
        }
        match o.kind {
            OKind::Pothole => {
                let gy = ground_y(o.y) as i32;
                f.fill_ellipse(sx, gy - 1, 6, 2, sc.road_d);
                f.fill_ellipse(sx, gy - 1, 5, 1, rgb(40, 32, 40));
                f.set(sx + 7, gy, sc.road_d);
                f.set(sx - 7, gy - 2, sc.road_d);
                f.set(sx + 6, gy - 3, sc.road_d);
            }
            OKind::Rails => {
                // two shiny tram rails running diagonally across the path
                for k in [0, 7] {
                    f.line(sx - 14 + k, ROAD_BOT - 1, sx + 8 + k, ROAD_TOP + 3, rgb(80, 72, 72));
                    f.line(sx - 13 + k, ROAD_BOT - 1, sx + 9 + k, ROAD_TOP + 3, rgb(208, 208, 216));
                }
            }
            _ => {}
        }
    }
    let len = race.length;
    // start and finish lines, chequered
    for (x, big) in [(START_LINE, false), (len, true)] {
        let sx = (x - cam).round() as i32;
        if !(-20..W + 20).contains(&sx) {
            continue;
        }
        for y in ROAD_TOP + 2..ROAD_BOT {
            if y < clip.0 || y >= clip.1 {
                continue;
            }
            for k in 0..(if big { 6 } else { 4 }) {
                let c = if ((y - ROAD_TOP) / 3 + k) % 2 == 0 { rgb(248, 248, 248) } else { rgb(24, 24, 32) };
                f.set(sx + k, y, c);
            }
        }
    }
}

fn draw_wind_sign(f: &mut Frame, x: i32, wind: Wind, t: u32) {
    // a roadside sign on a post with an arrow and the wind's name
    let (bg, fg) = match wind {
        Wind::Head(_) => (rgb(184, 32, 40), rgb(248, 248, 248)),
        Wind::Tail(_) => (rgb(40, 136, 72), rgb(248, 248, 248)),
        Wind::Side(_) => (rgb(232, 184, 32), rgb(24, 24, 32)),
    };
    let base = ROAD_TOP + 1;
    f.rect(x - 1, base - 18, 3, 18, OUTLINE);
    f.rect(x, base - 18, 1, 18, rgb(200, 200, 216));
    let (w, h) = (24, 14);
    let (bx, by) = (x - w / 2, base - 18 - h);
    f.rect(bx - 1, by - 1, w + 2, h + 2, OUTLINE);
    f.rect(bx, by, w, h, bg);
    f.rect_outline(bx + 1, by + 1, w - 2, h - 2, fg);
    let wob = ((t / 6) % 2) as i32;
    let (cx, cy) = (x, by + h / 2);
    match wind {
        Wind::Head(_) => {
            f.hline(cx - 6, cx + 5, cy, fg);
            for k in 0..4 {
                f.line(cx - 6 + k, cy, cx - 2 + k, cy - 4, fg);
                f.line(cx - 6 + k, cy, cx - 2 + k, cy + 4, fg);
            }
            f.set(cx + 6 + wob, cy - 2, fg);
        }
        Wind::Tail(_) => {
            f.hline(cx - 6, cx + 5, cy, fg);
            for k in 0..4 {
                f.line(cx + 6 - k, cy, cx + 2 - k, cy - 4, fg);
                f.line(cx + 6 - k, cy, cx + 2 - k, cy + 4, fg);
            }
            f.set(cx - 7 - wob, cy + 2, fg);
        }
        Wind::Side(_) => {
            f.line(cx, cy - 5, cx, cy + 3, fg);
            for k in 0..3 {
                f.line(cx, cy + 4 - k, cx - 4, cy - k, fg);
                f.line(cx, cy + 4 - k, cx + 4, cy - k, fg);
            }
        }
    }
}

fn draw_finish_arch(f: &mut Frame, x: i32, t: u32) {
    // an orange inflatable arch over the path
    let (l, r) = (x - 6, x + 8);
    let top = ROAD_TOP - 46;
    for (px, base) in [(l, ROAD_TOP + 2), (r, ROAD_BOT + 2)] {
        f.rect(px - 3, top + 6, 7, base - top - 6, OUTLINE);
        f.rect(px - 2, top + 6, 5, base - top - 6, rgb(248, 136, 24));
        f.rect(px - 2, top + 6, 1, base - top - 6, rgb(248, 184, 88));
    }
    f.rect(l - 34, top - 1, 82, 16, OUTLINE);
    f.rect(l - 33, top, 80, 14, rgb(248, 136, 24));
    f.hline(l - 33, l + 46, top, rgb(248, 184, 88));
    let blink = (t / 15) % 2 == 0;
    f.text_center(x + 4, top + 4, "FINISH", if blink { rgb(248, 248, 248) } else { rgb(248, 232, 96) }, OUTLINE, true);
}

fn draw_start_arch(f: &mut Frame, x: i32) {
    let top = ROAD_TOP - 36;
    f.rect(x - 1, top, 3, ROAD_TOP - top, OUTLINE);
    f.rect(x, top, 1, ROAD_TOP - top, rgb(200, 200, 216));
    f.rect(x - 26, top - 2, 54, 17, OUTLINE);
    // the Dutch flag colours, the word on the white band
    f.rect(x - 25, top - 1, 52, 3, rgb(200, 32, 48));
    f.rect(x - 25, top + 2, 52, 9, rgb(248, 248, 248));
    f.rect(x - 25, top + 11, 52, 3, rgb(32, 72, 160));
    f.text_center(x + 1, top + 3, "START", rgb(24, 24, 48), rgb(184, 184, 200), true);
}

fn draw_flag(f: &mut Frame, x: i32, y: i32, t: u32) {
    f.line(x, y, x, y - 18, OUTLINE);
    let wave = ((t / 6) % 2) as i32;
    for (k, c) in [rgb(200, 32, 48), rgb(248, 248, 248), rgb(32, 72, 160)].iter().enumerate() {
        let yy = y - 18 + k as i32 * 2;
        f.hline(x + 1, x + 8, yy, *c);
        f.hline(x + 1, x + 8 + if (k as i32 + wave) % 2 == 0 { 1 } else { 0 }, yy + 1, *c);
    }
}

fn draw_crowd(f: &mut Frame, art: &Art, x0: i32, t: u32, n: usize) {
    // supporters in orange behind the fence near the finish
    for k in 0..n {
        let x = x0 + k as i32 * 13 - 40;
        if !(-20..W + 20).contains(&x) {
            continue;
        }
        let frames = &art.extras.frames[k % 6];
        let jump = ((t / 8 + k as u32 * 3) % 4 == 0) as i32;
        let s = if (t / 16 + k as u32) % 2 == 0 { &frames[6] } else { &frames[7] };
        let mut pal = extra_palette(k * 5 + 1);
        if k % 3 != 2 {
            pal[crate::engine::sprites::SHIRT as usize] = rgb(248, 136, 24);
            pal[crate::engine::sprites::SHIRT_D as usize] = rgb(192, 88, 16);
        }
        blit_person(f, s, x, ROAD_TOP + 2 - jump, k % 2 == 1, &pal);
        if k % 4 == 0 {
            draw_flag(f, x + 6, ROAD_TOP - 18, t + k as u32 * 5);
        }
    }
}

fn draw_bridge(f: &mut Frame, b: &Bridge, cam: f32, t: u32, clip: (i32, i32)) {
    let cx = (b.x - cam).round() as i32;
    if !(-80..W + 80).contains(&cx) {
        return;
    }
    // the canal crossing the path: water in the gap under the deck
    let (l, r) = (cx - 24, cx + 24);
    for y in ROAD_TOP..ROAD_BOT + 5 {
        if y < clip.0 || y >= clip.1 {
            continue;
        }
        for x in l..r {
            let c = if (x + y + (t / 8) as i32) % 9 == 0 { rgb(96, 128, 152) } else { rgb(48, 72, 96) };
            f.set(x, y, c);
        }
    }
    let wood = rgb(232, 232, 224);
    // the deck: two planked leaves that tip up from the middle
    let ang = b.lift * 1.35;
    for side in [-1, 1] {
        let pivot = if side < 0 { l } else { r };
        let len = 24.0;
        let tip = pivot - side * (len * ang.cos()).round() as i32;
        let up = (len * ang.sin()).round() as i32;
        let quad = [(pivot, ROAD_TOP + 2), (tip, ROAD_TOP + 2 - up), (tip, ROAD_BOT - up), (pivot, ROAD_BOT)];
        let mut o = quad;
        for q in o.iter_mut() {
            q.0 += if q.0 == pivot { 0 } else { -side };
        }
        f.fill_poly(&[(o[0].0, o[0].1 - 1), (o[1].0, o[1].1 - 1), (o[2].0, o[2].1 + 1), (o[3].0, o[3].1 + 1)], OUTLINE);
        f.fill_poly(&quad, rgb(160, 112, 64));
        // planks across, and the white rail on the far edge
        for k in 1..6 {
            let y0 = ROAD_TOP + 2 + k * 8;
            f.line(pivot, y0, tip, y0 - up, rgb(120, 80, 48));
        }
        f.line(pivot, ROAD_TOP + 1, tip, ROAD_TOP + 1 - up, rgb(232, 232, 224));
        f.line(pivot, ROAD_TOP - 4, tip, ROAD_TOP - 4 - up, rgb(232, 232, 224));
        f.line(tip, ROAD_TOP - 4 - up, tip, ROAD_TOP + 1 - up, rgb(232, 232, 224));
    }
    // white wooden frames (like the Magere Brug)
    for x in [l - 4, r + 3] {
        f.rect(x - 2, ROAD_TOP - 40, 5, 44, OUTLINE);
        f.rect(x - 1, ROAD_TOP - 40, 3, 44, wood);
    }
    f.rect(l - 6, ROAD_TOP - 42, r - l + 12, 5, OUTLINE);
    f.rect(l - 5, ROAD_TOP - 41, r - l + 10, 3, wood);
    // barriers and lights on our side
    let bx = (b.barrier() - cam).round() as i32 + 8;
    f.rect(bx - 1, ROAD_TOP - 14, 4, 16, OUTLINE);
    f.rect(bx, ROAD_TOP - 13, 2, 15, rgb(200, 200, 216));
    let on = b.lights() && (t / 12) % 2 == 0;
    f.fill_ellipse(bx + 1, ROAD_TOP - 17, 3, 3, OUTLINE);
    f.fill_ellipse(bx + 1, ROAD_TOP - 17, 2, 2, if on { rgb(248, 56, 56) } else { rgb(96, 40, 40) });
    if b.blocking() {
        // boom down across both lanes
        for x in bx..bx + 3 {
            for y in ROAD_TOP + 2..ROAD_BOT {
                let c = if ((y - ROAD_TOP) / 4) % 2 == 0 { rgb(232, 48, 48) } else { rgb(248, 248, 248) };
                if y >= clip.0 && y < clip.1 {
                    f.set(x, y, c);
                }
            }
        }
    }
    if b.state == BState::Open && b.lift > 0.8 {
        // a boat squeezes through
        let k = ((t / 2) % 60) as i32;
        let by = ROAD_TOP - 6 + k;
        if by < ROAD_BOT {
            f.rect(cx - 6, by, 12, 8, OUTLINE);
            f.rect(cx - 5, by + 1, 10, 6, rgb(232, 232, 224));
            f.rect(cx - 3, by + 2, 6, 3, rgb(48, 72, 120));
        }
    }
}

fn draw_snackbar(f: &mut Frame, x: i32, t: u32) {
    let base = ROAD_TOP + 1;
    let (l, w) = (x - 30, 60);
    f.rect(l - 1, base - 34, w + 2, 35, OUTLINE);
    f.rect(l, base - 33, w, 34, rgb(232, 224, 208));
    f.rect(l, base - 33, w, 8, rgb(216, 56, 48));
    f.text_center(x, base - 32, "SNACKBAR", rgb(248, 248, 248), OUTLINE, false);
    for k in 0..6 {
        f.rect(l + k * 10, base - 25, 5, 3, if k % 2 == 0 { rgb(248, 248, 248) } else { rgb(216, 56, 48) });
    }
    f.rect(l + 6, base - 20, 20, 12, rgb(248, 216, 120));
    f.rect(l + 7, base - 19, 18, 10, rgb(96, 120, 160));
    // a kroket wall with little doors
    for k in 0..3 {
        f.rect(l + 32 + k * 8, base - 20, 7, 7, rgb(200, 200, 208));
        f.rect(l + 33 + k * 8, base - 19, 5, 5, rgb(176, 112, 48));
    }
    f.rect(l + 30, base - 11, 26, 11, rgb(120, 88, 64));
    // a little flag that flaps hard in the storm
    let fl = ((t / 3) % 2) as i32;
    f.line(l + 58, base - 34, l + 58, base - 46, OUTLINE);
    f.rect(l + 59, base - 46, 8 + fl, 5, rgb(248, 216, 48));
}

// ------------------------------------------------------------------ view

fn draw_fx(f: &mut Frame, race: &Race, cam: f32, clip: (i32, i32)) {
    for fx in &race.fx {
        let x = (fx.x - cam) as i32;
        let y = fx.y as i32;
        if y < clip.0 || y >= clip.1 || !(0..W).contains(&x) {
            continue;
        }
        match fx.kind {
            FxKind::Drop => {
                f.set(x, y, fx.color);
                f.set(x, y - 1, rgb(232, 248, 248));
            }
            FxKind::Feather | FxKind::Wool => {
                f.set(x, y, fx.color);
                f.set(x + 1, y, fx.color);
                f.set(x, y + 1, rgb(200, 200, 216));
            }
            FxKind::Crumb | FxKind::Dust | FxKind::Spark => {
                f.set(x, y, fx.color);
            }
            FxKind::Sparkle => {
                let r = 1 + (fx.max - fx.life) / 3;
                f.set(x, y, rgb(248, 248, 248));
                f.set(x - r, y, fx.color);
                f.set(x + r, y, fx.color);
                f.set(x, y - r, fx.color);
                f.set(x, y + r, fx.color);
            }
        }
    }
}

fn draw_floaters(f: &mut Frame, race: &Race, art: &Art, cam: f32, t: u32) {
    for fl in &race.floaters {
        let x = (fl.x - cam) as i32;
        if !(-10..W + 10).contains(&x) {
            continue;
        }
        let y = WATER_Y + 6 + (fl.y * 14.0) as i32;
        let bob = ((t / 14 + fl.t as u32 / 20) % 2) as i32;
        let b = art.cargo_bmp(fl.cargo);
        b.draw_clip(f, x - b.w / 2, y - b.h + 2 + bob, &PROPS, false, y + 1 + bob);
        f.hline(x - b.w / 2 - 1, x + b.w / 2, y + 1 + bob, rgb(200, 232, 240));
    }
}

fn draw_wind_streaks(f: &mut Frame, race: &Race, i: usize, t: u32, clip: (i32, i32)) {
    let r = &race.riders[i];
    let (wind, _) = race.wind_for(i);
    let s = wind.strength() * race.gust;
    let n = (s * 9.0) as i32 + if race.gust_on { 4 } else { 0 };
    let head = wind.head();
    let side = wind.side();
    let speed = 1.0 + r.v + head.max(0.0) * 3.0;
    for k in 0..n {
        let h = (k as u32).wrapping_mul(2654435761);
        let y0 = 50 + (h % 140) as i32;
        let len = 6 + (h >> 8) as i32 % 10;
        let ph = (h >> 4) as f32 % 300.0;
        let (x, y) = if side > 0.0 {
            let tt = t as f32 * (1.6 + s);
            (((ph - tt * 0.8).rem_euclid(320.0)) as i32 - 30, y0 + ((tt * 0.5 + ph) as i32 % 60) - 30)
        } else if head > 0.0 {
            (((ph - t as f32 * speed).rem_euclid(320.0)) as i32 - 30, y0)
        } else {
            (((ph + t as f32 * (s * 1.5 - r.v * 0.5)).rem_euclid(320.0)) as i32 - 30, y0)
        };
        if y < clip.0 || y >= clip.1 {
            continue;
        }
        let c = rgb(232, 240, 248);
        for d in 0..len {
            if d % 2 == 0 {
                let yy = if side > 0.0 { y + d / 3 } else { y };
                f.set(x + d, yy, c);
            }
        }
    }
}

/// Draw the race as seen from rider `i` into rows `clip` of the frame.
pub fn draw_view(f: &mut Frame, race: &Race, sc: &Scenery, art: &Art, i: usize, t: u32, clip: (i32, i32)) {
    let cam = cam_for(race, i);
    let st = race.def();
    let (wind, _) = race.wind_for(i);
    sc.draw(f, art, cam, t, wind.strength() * race.gust, wind.head(), clip);
    // roadside stuff behind the road: wind signs, the snack bar, the crowd
    for k in 1..st.sections.len() {
        let x = (st.section_start(k) - 40.0 - cam).round() as i32;
        if (-30..W + 30).contains(&x) {
            draw_wind_sign(f, x, st.sections[k].1, t);
        }
    }
    let x0 = (st.section_start(0) + 150.0 - cam).round() as i32;
    if (-30..W + 30).contains(&x0) {
        draw_wind_sign(f, x0, st.sections[0].1, t);
    }
    if let Some(sx) = st.snackbar {
        let x = (sx - cam).round() as i32;
        if (-60..W + 60).contains(&x) {
            draw_snackbar(f, x, t);
        }
    }
    let fin = (race.length - cam).round() as i32;
    if (-140..W + 140).contains(&fin) {
        draw_crowd(f, art, fin - 70, t, 14);
    }
    let start = (START_LINE - cam).round() as i32;
    if (-60..W + 60).contains(&start) {
        draw_crowd(f, art, start - 120, t, 6);
    }
    for &(x, stand) in &race.stands {
        let sx = (x - cam).round() as i32;
        if (-50..W + 50).contains(&sx) {
            draw_stand(f, art, sx, stand, t);
        }
    }
    sc.draw_road(f, cam, clip, |x| st.on_cobbles(x));
    draw_flat(f, race, sc, cam, clip);
    if let Some(b) = &race.bridge {
        draw_bridge(f, b, cam, t, clip);
    }
    // sort riders and objects by depth
    let mut items: Vec<(f32, i32, usize)> = Vec::with_capacity(48);
    for (k, o) in race.objs.iter().enumerate() {
        if o.gone || matches!(o.kind, OKind::Pothole | OKind::Rails) {
            continue;
        }
        let sx = o.x - cam;
        if (-60.0..W as f32 + 60.0).contains(&sx) {
            items.push((o.y, 0, k));
        }
    }
    for (k, r) in race.riders.iter().enumerate() {
        let y = if r.riding() { r.y } else { r.crash_y };
        items.push((y + 0.01, 1, k));
    }
    items.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for &(_, kind, k) in &items {
        if kind == 0 {
            draw_obj(f, art, &race.objs[k], cam, t);
        } else {
            draw_rider_in_world(f, race, art, k, cam, t);
        }
    }
    if (-40..W + 40).contains(&fin) {
        draw_finish_arch(f, fin, t);
    }
    if (-40..W + 40).contains(&start) {
        draw_start_arch(f, start);
    }
    sc.draw_water(f, cam, t, clip);
    if let Some((Rare::PartyBoat, x)) = race.rare {
        party_boat(f, art, (x - cam * 1.0) as i32, t, clip);
    }
    draw_floaters(f, race, art, cam, t);
    for k in 0..race.riders.len() {
        if matches!(race.riders[k].state, RState::Swim | RState::Climb) {
            draw_swimmer(f, race, art, k, cam, t);
        }
    }
    draw_fx(f, race, cam, clip);
    if race.phase != Phase::Countdown {
        draw_wind_streaks(f, race, i, t, clip);
    }
    sc.draw_weather(f, t, race.gust, clip);
    if let Some((a, b)) = race.shower {
        draw_shower(f, (a - cam) as i32, (b - cam) as i32, t, clip);
    }
    match race.rare {
        Some((Rare::Stork, x)) => {
            let sx = ((x - cam) * 0.5) as i32 + 60;
            if (-20..W + 20).contains(&sx) {
                art.stork[((t / 10) % 2) as usize].draw(f, sx, 52 + ((t / 20) % 3) as i32, &PROPS, false);
            }
        }
        Some((Rare::Kite, x)) => {
            let sx = ((x - cam) * 0.6) as i32 + (t as i32 % 400) / 3;
            kite_surfer(f, sx, t, clip);
        }
        _ => {}
    }
    if race.views.contains(&i) && race.riders[i].bike != Bike::Lig && race.phase == Phase::Race {
        look_ahead(f, race, cam, t, clip);
    }
    sc.draw_gulls(f, art, t, clip);
    if let Some(tx) = race.tram {
        draw_tram(f, tx as i32, t, clip);
    }
    for p in &race.popups {
        let x = (p.x - cam) as i32;
        let y = p.y as i32;
        if y < clip.0 + 2 || y + 8 > clip.1 || !(-40..W + 40).contains(&x) {
            continue;
        }
        if p.life > 8 || (p.life / 2) % 2 == 0 {
            let w = text_width(p.text, true);
            f.text_grad((x - w / 2).clamp(2, W - w - 2), y, p.text, &grad7(p.color), OUTLINE, true);
        }
    }
}

/// Warning signs at the right edge for trouble that is still off screen.
/// (The ligfiets is too low to see over the hedges: no warnings.)
fn look_ahead(f: &mut Frame, race: &Race, cam: f32, t: u32, clip: (i32, i32)) {
    let mut shown = [false; 3];
    for o in &race.objs {
        if o.gone || o.kind.pickup() || !(o.blocking() || o.kind.slow_rider()) {
            continue;
        }
        let sx = o.x - cam;
        if !(W as f32 - 6.0..W as f32 + 150.0).contains(&sx) {
            continue;
        }
        let slot = if o.kind.wide() { 2 } else if o.y < 0.5 { 0 } else { 1 };
        if !(-0.7..1.7).contains(&o.y) && !o.kind.wide() {
            continue;
        }
        if shown[slot] {
            continue;
        }
        shown[slot] = true;
        let y = if slot == 2 { ground_y(0.5) as i32 } else { ground_y(slot as f32) as i32 } - 12;
        if y - 8 < clip.0 || y + 8 > clip.1 {
            continue;
        }
        let x = W - 10 - if (t / 6) % 2 == 0 { 1 } else { 0 };
        // a yellow diamond with a "!"
        for k in 0..7 {
            f.hline(x - k, x + k, y - 7 + k, OUTLINE);
            f.hline(x - k, x + k, y + 7 - k, OUTLINE);
        }
        for k in 0..6 {
            f.hline(x - k, x + k, y - 6 + k, rgb(248, 216, 48));
            f.hline(x - k, x + k, y + 6 - k, rgb(248, 216, 48));
        }
        f.rect(x, y - 3, 1, 4, OUTLINE);
        f.set(x, y + 3, OUTLINE);
    }
    if let Some(b) = &race.bridge {
        let sx = b.barrier() - cam;
        if b.lights() && sx > W as f32 - 6.0 && sx < W as f32 + 400.0 && (t / 10) % 2 == 0 {
            let y = ROAD_TOP - 12;
            if y - 6 >= clip.0 {
                f.fill_ellipse(W - 10, y, 5, 5, OUTLINE);
                f.fill_ellipse(W - 10, y, 4, 4, rgb(248, 56, 56));
            }
        }
    }
}

fn draw_stand(f: &mut Frame, art: &Art, x: i32, stand: Stand, t: u32) {
    let base = ROAD_TOP + 1;
    match stand {
        Stand::Cheese => {
            // a cheese farmer's stand with a striped awning and stacked wheels
            f.rect(x - 20, base - 26, 40, 27, OUTLINE);
            f.rect(x - 19, base - 13, 38, 13, rgb(160, 104, 56));
            f.hline(x - 19, x + 18, base - 13, rgb(200, 144, 88));
            for k in 0..5 {
                f.rect(x - 19 + k * 8, base - 25, 4, 7, rgb(248, 248, 248));
                f.rect(x - 15 + k * 8, base - 25, 4, 7, rgb(232, 136, 24));
            }
            f.line(x - 19, base - 18, x - 19, base - 13, OUTLINE);
            f.line(x + 18, base - 18, x + 18, base - 13, OUTLINE);
            for k in 0..3 {
                art.cargo[0].draw(f, x - 16 + k * 11, base - 17, &PROPS, false);
            }
            art.cargo[0].draw(f, x - 11, base - 21, &PROPS, false);
            f.rect(x - 14, base - 34, 28, 9, OUTLINE);
            f.rect(x - 13, base - 33, 26, 7, rgb(248, 232, 168));
            f.text_center(x, base - 32, "KAAS", rgb(176, 72, 24), rgb(248, 232, 168), false);
            let ppl = &art.extras.frames[1][if (t / 30) % 2 == 0 { 4 } else { 6 }];
            blit_person(f, ppl, x + 26, base - 1, true, &extra_palette(2));
        }
        Stand::HerringCart => {
            // a herring cart: white with blue, a little roof and flags
            f.rect(x - 22, base - 22, 44, 18, OUTLINE);
            f.rect(x - 21, base - 21, 42, 16, rgb(232, 232, 240));
            f.rect(x - 21, base - 11, 42, 6, rgb(48, 88, 168));
            f.rect(x - 24, base - 30, 48, 6, OUTLINE);
            f.rect(x - 23, base - 29, 46, 4, rgb(48, 88, 168));
            for k in 0..4 {
                f.line(x - 20 + k * 13, base - 24, x - 20 + k * 13, base - 21, OUTLINE);
            }
            f.text_center(x, base - 20, "HARING", rgb(48, 88, 168), rgb(200, 216, 232), false);
            for wx in [x - 14, x + 14] {
                f.fill_ellipse(wx, base - 3, 4, 4, OUTLINE);
                f.fill_ellipse(wx, base - 3, 2, 2, rgb(200, 200, 216));
            }
            draw_flag(f, x + 22, base - 30, t);
            let ppl = &art.extras.frames[0][if (t / 40) % 2 == 0 { 4 } else { 6 }];
            blit_person(f, ppl, x - 30, base - 1, false, &extra_palette(5));
            if (t / 120) % 2 == 0 {
                bubble(f, x - 28, base - 36, "HOLLANDSE NIEUWE!");
            }
        }
    }
}

fn draw_shower(f: &mut Frame, a: i32, b: i32, t: u32, clip: (i32, i32)) {
    let (x0, x1) = (a.max(0), b.min(W));
    if x1 <= x0 {
        return;
    }
    // a heavy grey rain cloud over this stretch, then the rain
    let tt = t as i32;
    if clip.0 < 70 {
        let mut x = a + 10;
        let mut k = 0;
        while x < b - 6 {
            if (-30..W + 30).contains(&x) {
                let y = 48 + (k * 5) % 7;
                f.fill_ellipse(x, y + 2, 20, 9, rgb(72, 80, 104));
                f.fill_ellipse(x - 4, y - 2, 13, 7, rgb(104, 112, 136));
                f.fill_ellipse(x + 2, y - 5, 8, 4, rgb(136, 144, 168));
                f.hline(x - 20, x + 20, y + 10, rgb(56, 64, 88));
            }
            x += 30;
            k += 1;
        }
    }
    for k in 0..90 {
        let x = x0 + ((k * 37 + tt * 2) % (x1 - x0).max(1));
        let y = (k * 53 + tt * 5) % 190 + 34;
        if y >= clip.0 && y + 5 < clip.1 {
            f.line(x, y, x - 1, y + 4, rgb(176, 192, 216));
        }
    }
}

fn party_boat(f: &mut Frame, art: &Art, x: i32, t: u32, clip: (i32, i32)) {
    let y = WATER_Y + 18;
    if !(-80..W + 80).contains(&x) || y > clip.1 {
        return;
    }
    // an orange party boat, everyone dancing
    f.rect(x - 34, y - 4, 70, 8, OUTLINE);
    f.rect(x - 33, y - 3, 68, 6, rgb(248, 136, 24));
    f.hline(x - 33, x + 34, y - 3, rgb(248, 184, 88));
    let _ = (art, t);
    for k in 0..5 {
        let px = x - 24 + k * 12;
        let jump = ((t / 7 + k as u32) % 2) as i32;
        let frames = &art.extras.frames[k as usize % 6];
        let s = if jump == 1 { &frames[7] } else { &frames[6] };
        let mut pal = extra_palette(k as usize * 3);
        pal[crate::engine::sprites::SHIRT as usize] = rgb(248, 136, 24);
        pal[crate::engine::sprites::SHIRT_D as usize] = rgb(192, 88, 16);
        blit_person_clip(f, s, px, y - 2 - jump, k % 2 == 1, &pal, y - 3);
    }
    if (t / 30) % 2 == 0 && y - 30 > clip.0 {
        bubble(f, x, y - 18, "HUP HOLLAND HUP!");
    }
}

fn kite_surfer(f: &mut Frame, x: i32, t: u32, clip: (i32, i32)) {
    let y = 90 + ((t / 16) % 3) as i32;
    if !(-40..W + 40).contains(&x) || y - 30 < clip.0 {
        return;
    }
    // a kite surfer, blown along the dike (over the sea)
    let (kx, ky) = (x + 20, y - 26);
    f.fill_ellipse(kx, ky, 12, 3, OUTLINE);
    f.fill_ellipse(kx, ky, 11, 2, rgb(232, 48, 96));
    f.line(kx - 10, ky + 1, x, y - 4, rgb(200, 200, 208));
    f.line(kx + 10, ky + 1, x, y - 4, rgb(200, 200, 208));
    f.rect(x - 1, y - 6, 3, 5, rgb(40, 40, 56));
    f.rect(x - 1, y - 8, 3, 2, rgb(248, 200, 160));
    f.hline(x - 5, x + 5, y, rgb(248, 216, 48));
}

fn draw_tram(f: &mut Frame, x: i32, t: u32, clip: (i32, i32)) {
    // a yellow-and-blue tram gliding along the quay
    let y = 112;
    if y < clip.0 || y + 22 > clip.1 {
        return;
    }
    let w = 96;
    f.rect(x - 1, y - 1, w + 2, 24, OUTLINE);
    f.rect(x, y, w, 22, rgb(248, 216, 48));
    f.rect(x, y + 14, w, 8, rgb(40, 72, 152));
    for k in 0..8 {
        f.rect(x + 4 + k * 11, y + 3, 8, 8, rgb(136, 176, 216));
        f.hline(x + 4 + k * 11, x + 11 + k * 11, y + 3, rgb(200, 224, 240));
    }
    f.line(x + 40, y - 1, x + 48, y - 10, OUTLINE);
    f.line(x + 48, y - 10, x + 56, y - 1, OUTLINE);
    f.hline(x + 30, x + 70, y - 11, OUTLINE);
    if (t / 6) % 2 == 0 {
        f.set(x + 48, y - 12, rgb(248, 248, 200));
    }
}

/// Shift rows horizontally in place (screen shake on cobbles and crashes).
pub fn shake_rows(f: &mut Frame, rows: (i32, i32), dx: i32) {
    if dx == 0 {
        return;
    }
    for y in rows.0.max(0)..rows.1.min(H) {
        let row = &mut f.px[(y * W) as usize..((y + 1) * W) as usize];
        if dx > 0 {
            row.copy_within(0..(W - dx) as usize, dx as usize);
        } else {
            row.copy_within((-dx) as usize..W as usize, 0);
        }
    }
}

// ------------------------------------------------------------------ HUD

fn bar(f: &mut Frame, x: i32, y: i32, w: i32, h: i32, frac: f32, c: u32) {
    f.rect(x - 1, y - 1, w + 2, h + 2, OUTLINE);
    f.rect(x, y, w, h, rgb(40, 40, 72));
    let fw = ((w as f32) * frac.clamp(0.0, 1.0)).round() as i32;
    f.rect(x, y, fw, h, c);
    f.hline(x, x + fw - 1, y, mix(c, 0xFFFFFF, 0.4));
}

/// The rhythm meter: fills after each stroke; gold = PERFECT, green = good.
fn rhythm(f: &mut Frame, race: &Race, i: usize, x: i32, y: i32, w: i32, h: i32, t: u32) {
    let r = &race.riders[i];
    let g = race.gap_for(i);
    let span = g + 16;
    let px = |fr: i32| x + fr * w / span;
    f.rect(x - 1, y - 1, w + 2, h + 2, OUTLINE);
    f.rect(x, y, px(g) - x, h, rgb(96, 32, 48));
    f.rect(px(g), y, px(g + 4) - px(g), h, rgb(200, 160, 32));
    f.rect(px(g + 4), y, x + w - px(g + 4), h, rgb(40, 104, 64));
    let wasted = r.last_stroke == Stroke::Wasted && r.stroke_t < 10;
    if wasted && (t / 2) % 2 == 0 {
        f.rect(x, y, w, h, rgb(232, 48, 48));
    }
    let k = r.since.min(span);
    let mx = px(k);
    let col = if k < g { rgb(248, 120, 120) } else if k < g + 4 && r.clean { rgb(248, 248, 160) } else { rgb(200, 248, 200) };
    f.rect(mx - 1, y - 2, 3, h + 4, OUTLINE);
    f.rect(mx, y - 1, 1, h + 2, col);
    if r.last_stroke == Stroke::Perfect && r.stroke_t < 8 {
        f.rect_outline(x - 2, y - 2, w + 4, h + 4, rgb(248, 232, 96));
    }
}

/// Chevrons: how many, which way. Head = toward you, tail = with you,
/// side = down toward the canal. A gust makes them flicker.
fn wind_icon(f: &mut Frame, x: i32, y: i32, wind: Wind, gust: f32, t: u32) {
    let s = (wind.strength() * gust).min(2.0);
    let n = (1 + (s * 1.5) as i32).min(3);
    let (col, dir) = match wind {
        Wind::Head(_) => (rgb(248, 96, 72), -1),
        Wind::Tail(_) => (rgb(120, 232, 96), 1),
        Wind::Side(_) => (rgb(248, 216, 48), 0),
    };
    let wig = if gust > 1.25 { ((t / 3) % 2) as i32 } else { 0 };
    for k in 0..n {
        if dir == 0 {
            let (cx, cy) = (x + 4, y + k * 3 + wig);
            f.thick_line(cx - 5, cy - 1, cx, cy + 3, 3, OUTLINE);
            f.thick_line(cx + 5, cy - 1, cx, cy + 3, 3, OUTLINE);
            f.line(cx - 4, cy - 1, cx, cy + 2, col);
            f.line(cx + 4, cy - 1, cx, cy + 2, col);
        } else {
            let cx = x + 4 + dir * (k * 5 - 5) + dir * wig;
            let cy = y + 4;
            f.thick_line(cx - dir * 3, cy - 5, cx + dir, cy, 3, OUTLINE);
            f.thick_line(cx - dir * 3, cy + 5, cx + dir, cy, 3, OUTLINE);
            f.line(cx - dir * 3, cy - 4, cx, cy, col);
            f.line(cx - dir * 3, cy + 4, cx, cy, col);
        }
    }
}

/// Route strip: sections coloured by wind, the riders as dots.
fn route_map(f: &mut Frame, race: &Race, x: i32, y: i32, w: i32, me: usize, t: u32) {
    let st = race.def();
    let len = race.length;
    f.rect(x - 1, y - 1, w + 2, 5, OUTLINE);
    let mut acc = 0.0;
    for &(l, wind) in st.sections {
        let x0 = x + (acc / len * w as f32) as i32;
        let x1 = x + ((acc + l) / len * w as f32) as i32;
        let c = match wind {
            Wind::Head(_) => rgb(184, 48, 48),
            Wind::Tail(_) => rgb(48, 152, 72),
            Wind::Side(_) => rgb(200, 160, 32),
        };
        f.rect(x0, y, x1 - x0, 3, c);
        acc += l;
    }
    if let Some(b) = &race.bridge {
        let bx = x + (b.x / len * w as f32) as i32;
        f.rect(bx - 1, y - 2, 3, 7, rgb(248, 248, 248));
    }
    if let Some(s) = st.snackbar {
        let sx = x + (s / len * w as f32) as i32;
        f.rect(sx - 1, y - 2, 3, 7, rgb(248, 176, 48));
    }
    // chequered flag at the end
    for k in 0..3 {
        f.set(x + w + 1 + k, y - 2 + k % 2, rgb(248, 248, 248));
        f.set(x + w + 1 + k, y - 1 - k % 2, rgb(24, 24, 32));
    }
    for (k, r) in race.riders.iter().enumerate() {
        if k == me {
            continue;
        }
        let px = x + (r.x.min(len) / len * w as f32) as i32;
        f.rect(px - 1, y - 1, 3, 5, OUTLINE);
        f.rect(px, y, 1, 3, rider_color(race, k));
    }
    let r = &race.riders[me];
    let px = x + (r.x.min(len) / len * w as f32) as i32;
    let blink = (t / 8) % 2 == 0;
    f.rect(px - 2, y - 2, 5, 7, OUTLINE);
    f.rect(px - 1, y - 1, 3, 5, if blink { rider_color(race, me) } else { rgb(248, 248, 248) });
}

pub fn draw_hud(f: &mut Frame, race: &Race, art: &Art, i: usize, t: u32) {
    let r = &race.riders[i];
    f.rect(0, 0, W, 32, rgb(16, 16, 48));
    f.hline(0, W - 1, 31, rgb(248, 136, 24));
    f.hline(0, W - 1, 32, rgb(176, 32, 88));
    let label = rgb(144, 152, 208);
    // place: big, with the field size
    let place = ordinal(r.place);
    let pc = if r.place == 0 { rgb(248, 216, 48) } else if r.place == 1 { rgb(216, 216, 232) } else { rgb(232, 136, 72) };
    f.logo(17, -2, place, 2, &grad6(pc), OUTLINE, LOGO_SHADOW);
    let of = format!("/{}", race.riders.len());
    f.text(31, 12, &of, label);
    // speed
    f.text(46, 2, "KM/U", label);
    let kmh = format!("{:>2}", kmh(r.v));
    f.text_grad(46, 11, &kmh, &grad7(rgb(96, 224, 248)), OUTLINE, true);
    // energy
    f.text(80, 2, "ENERGIE", label);
    let e = r.energy / 100.0;
    let ec = if r.bonk { rgb(160, 96, 232) } else if e > 0.5 { rgb(120, 216, 88) } else if e > 0.25 { rgb(248, 200, 48) } else { rgb(232, 64, 48) };
    bar(f, 80, 12, 40, 6, e, ec);
    if r.bonk && (t / 10) % 2 == 0 {
        f.text_sh(86, 12, "LEEG!", rgb(248, 248, 248), OUTLINE, false);
    }
    art.food[0].draw(f, 122, 10, &PROPS, false);
    // rhythm
    // the label turns into feedback for a moment after each push
    let (s, c) = match r.last_stroke {
        Stroke::Perfect if r.stroke_t < 24 => ("PERFECT!", rgb(248, 232, 96)),
        Stroke::Wasted if r.stroke_t < 24 => ("TE SNEL!", rgb(248, 96, 96)),
        _ => ("TRAP", label),
    };
    f.text(138, 2, s, c);
    rhythm(f, race, i, 138, 12, 66, 6, t);
    // wind
    let (wind, _) = race.wind_for(i);
    let wn = wind.name();
    let wc = match wind {
        Wind::Head(_) => rgb(248, 120, 96),
        Wind::Tail(_) => rgb(144, 232, 120),
        Wind::Side(_) => rgb(248, 216, 72),
    };
    f.text(253 - text_width(wn, false), 2, wn, wc);
    wind_icon(f, 230, 11, wind, race.gust, t);
    // cargo, then the route
    let cargo = race.def().cargo;
    let b = art.cargo_bmp(cargo);
    if r.cargo_max > 0 {
        b.draw(f, 2, 21, &PROPS, false);
        let s = format!("{}", r.cargo);
        f.text(b.w + 3, 23, &s, rgb(248, 248, 248));
    }
    route_map(f, race, 30, 25, 214, i, t);
}

/// A slim HUD for one half of the split screen.
pub fn draw_mini_hud(f: &mut Frame, race: &Race, i: usize, y: i32, t: u32) {
    let r = &race.riders[i];
    let p = match r.ctrl {
        Ctrl::Human(p) => p,
        _ => 0,
    };
    f.rect(0, y, W, 13, rgb(16, 16, 48));
    f.hline(0, W - 1, y + 13, rgb(248, 136, 24));
    let col = if p == 0 { rgb(248, 232, 64) } else { rgb(96, 224, 248) };
    f.rect(2, y + 2, 15, 9, col);
    f.text(4, y + 3, if p == 0 { "1P" } else { "2P" }, rgb(16, 16, 48));
    f.text_grad(21, y + 3, ordinal(r.place), &grad7(if r.place == 0 { rgb(248, 216, 48) } else { rgb(216, 216, 232) }), OUTLINE, false);
    let s = format!("{:>2} KM/U", kmh(r.v));
    f.text(38, y + 3, &s, rgb(96, 224, 248));
    let e = r.energy / 100.0;
    let ec = if r.bonk { rgb(160, 96, 232) } else if e > 0.5 { rgb(120, 216, 88) } else if e > 0.25 { rgb(248, 200, 48) } else { rgb(232, 64, 48) };
    bar(f, 92, y + 4, 26, 4, e, ec);
    rhythm(f, race, i, 124, y + 4, 40, 4, t);
    let (wind, _) = race.wind_for(i);
    wind_icon(f, 172, y + 2, wind, race.gust, t);
    route_map(f, race, 190, y + 5, 58, i, t);
}

// ------------------------------------------------------------------ banners

pub fn draw_banner(f: &mut Frame, b: &Banner, cy: i32, small: bool) {
    let n = b.text.chars().count() as i32;
    let mut scale = if b.scale > 0 { b.scale } else if n * 18 < 236 { 3 } else { 2 };
    if small {
        scale = scale.min(if b.scale > 0 { 4 } else { 2 });
    }
    let target = 128;
    let h = 7 * scale;
    let (x, y) = if b.pop {
        (target, cy - h / 2 - if b.t < 4 { 4 - b.t } else { 0 })
    } else {
        let x = if b.t < 10 { target + (10 - b.t) * 30 } else { target };
        (x, cy - h / 2 + if (10..16).contains(&b.t) { (16 - b.t) / 2 } else { 0 })
    };
    // slide out at the end
    let x = if b.t > b.dur - 8 { x - (b.t - (b.dur - 8)) * 34 } else { x };
    let grad: Vec<u32> = if b.color == rgb(248, 248, 248) { SUNSET.to_vec() } else { grad6(b.color).to_vec() };
    f.logo(x, y, &b.text, scale, &grad, OUTLINE, LOGO_SHADOW);
}

/// Speed lines and colour for a palette (used by select and intro cards).
pub fn rider_pal(who: Who) -> Palette {
    rider_palette(who)
}
