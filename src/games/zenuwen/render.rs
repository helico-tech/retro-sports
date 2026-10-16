//! Drawing a `World`: the street seen from the canal side, everything on it
//! sorted by depth, the BOA's view cones on the bricks, the canal with its
//! boats, the HUD, the lock panel, banners and popups.

use super::art::*;
use super::data::*;
use super::scenery::*;
use super::world::*;
use crate::engine::gfx::*;
use crate::engine::sprites::{self as sp, Palette};
use crate::engine::ui::{grad6, grad7, LOGO_SHADOW, OUTLINE, SUNSET};

/// World depth to the screen row of someone's feet.
pub fn sy(y: f32) -> i32 {
    STREET + y.round() as i32
}

fn sx(x: f32, cam: f32) -> i32 {
    (x - cam).round() as i32
}

pub fn white() -> u32 {
    rgb(248, 248, 248)
}

/// Speech bubble with a tail pointing down at (x, y).
pub fn bubble(f: &mut Frame, x: i32, y: i32, text: &str) {
    let w = text_width(text, false) + 6;
    let bx = (x - w / 2).clamp(2, W - w - 2);
    let by = y - 13;
    f.rect(bx - 1, by - 1, w + 2, 12, OUTLINE);
    f.rect(bx, by, w, 10, white());
    for k in 0..3 {
        f.hline(x - 2 + k, x + 2 - k, by + 10 + k, if k == 2 { OUTLINE } else { white() });
        f.set(x - 3 + k, by + 10 + k, OUTLINE);
        f.set(x + 3 - k, by + 10 + k, OUTLINE);
    }
    f.text(bx + 3, by + 2, text, rgb(16, 16, 48));
}

// ------------------------------------------------------------------ bikes

/// A bike on its own (parked, loose, flying), anchor (ground under the crank) at (x, y).
#[allow(clippy::too_many_arguments)]
pub fn draw_bike(f: &mut Frame, art: &Art, kind: Kind, color: usize, x: i32, y: i32, flip: bool, wheel: usize, lock: Lock, hot: bool, t: u32) {
    let g = geo(kind);
    let bp = bike_pal(kind, color);
    let m = |dx: i32| if flip { -dx } else { dx };
    let thin = kind == Kind::Race;
    for (c, r, front) in [(g.rear, g.rear_r, false), (g.front, g.front_r, true)] {
        let bent = kind == Kind::Wreck && front;
        let wb = art.wheel(r, thin, bent, wheel);
        wb.draw(f, x + m(c.0) - r - 1, y + c.1 - r - 1, &bp, false);
    }
    let b = art.bike(kind);
    let bx = if flip { x + BIKE_AX - b.w + 1 } else { x - BIKE_AX };
    b.draw(f, bx, y - BIKE_AY, &bp, flip);
    draw_lock(f, kind, lock, x, y, flip, t);
    if hot {
        // the cut lock dangles from the frame
        let swing = ((t / 6) % 3) as i32 - 1;
        let (hx, hy) = (x + m(-2), y - 9);
        f.line(hx, hy, hx + swing, hy + 4, rgb(136, 136, 160));
        f.rect(hx + swing - 1, hy + 4, 3, 2, rgb(200, 200, 216));
        if (t / 8).is_multiple_of(2) {
            f.set(hx + swing + 2, hy + 3, rgb(248, 248, 168));
        }
    }
    if kind == Kind::Gold && (t / 6).is_multiple_of(2) {
        let k = ((t / 40) % 5) as i32;
        let (px, py) = (x + m(-8 + k * 4), y - 12 + (k % 3) * 2);
        f.set(px, py, white());
        f.set(px - 1, py, rgb(248, 248, 168));
        f.set(px + 1, py, rgb(248, 248, 168));
        f.set(px, py - 1, rgb(248, 248, 168));
        f.set(px, py + 1, rgb(248, 248, 168));
    }
}

fn draw_lock(f: &mut Frame, kind: Kind, lock: Lock, x: i32, y: i32, flip: bool, _t: u32) {
    let g = geo(kind);
    let m = |dx: i32| if flip { -dx } else { dx };
    let (rx, ry) = (x + m(g.rear.0), y + g.rear.1);
    let (fx, fy) = (x + m(g.front.0), y + g.front.1);
    let chain = |f: &mut Frame| {
        // a chain from the front wheel round the frame
        let (ax, ay) = (fx, fy);
        let (bx, by) = (x + m(4), y - 9);
        let mut k = 0;
        line_points(ax, ay, bx, by, |px, py| {
            f.set(px, py, if k % 2 == 0 { rgb(248, 216, 48) } else { rgb(48, 48, 64) });
            k += 1;
        });
        f.rect(ax - 1, ay - 1, 3, 3, OUTLINE);
        f.set(ax, ay, rgb(248, 216, 48));
    };
    let u = |f: &mut Frame| {
        // a red U-lock through the rear wheel
        let c = rgb(216, 48, 48);
        f.line(rx - 2, ry - 5, rx - 2, ry + 1, c);
        f.line(rx + 2, ry - 5, rx + 2, ry + 1, c);
        f.hline(rx - 1, rx + 1, ry + 2, c);
        f.hline(rx - 3, rx + 3, ry - 6, rgb(48, 48, 64));
    };
    match lock {
        Lock::Ring => {
            // the wheel lock on the rear fork
            f.rect(rx - 2, ry - 6, 5, 3, OUTLINE);
            f.hline(rx - 1, rx + 1, ry - 5, rgb(200, 200, 216));
        }
        Lock::Chain => chain(f),
        Lock::U => u(f),
        Lock::Double => {
            chain(f);
            u(f);
        }
        Lock::None => {}
    }
}

/// Someone on a bike: wheels, the far crank, the frame, the near crank, the rider.
#[allow(clippy::too_many_arguments)]
pub fn draw_rider(f: &mut Frame, art: &Art, style: usize, pal: &Palette, kind: Kind, bp: Option<Pal>, x: i32, y: i32, flip: bool, crank: usize, wheel: usize) {
    let g = geo(kind);
    let bp = bp.unwrap_or_else(|| bike_pal(kind, 0));
    let m = |dx: i32| if flip { -dx } else { dx };
    let thin = kind == Kind::Race;
    for (c, r, front) in [(g.rear, g.rear_r, false), (g.front, g.front_r, true)] {
        let bent = kind == Kind::Wreck && front;
        art.wheel(r, thin, bent, wheel).draw(f, x + m(c.0) - r - 1, y + c.1 - r - 1, &bp, false);
    }
    let a = std::f32::consts::TAU * crank as f32 / 8.0;
    let (cx, cy) = (x + m(g.crank.0), y + g.crank.1);
    let back = (cx - m((a.cos() * 3.0).round() as i32), cy - (a.sin() * 3.0).round() as i32);
    f.line(cx, cy, back.0, back.1, rgb(88, 88, 104));
    let b = art.bike(kind);
    let bx = if flip { x + BIKE_AX - b.w + 1 } else { x - BIKE_AX };
    b.draw(f, bx, y - BIKE_AY, &bp, flip);
    let front = (cx + m((a.cos() * 3.0).round() as i32), cy + (a.sin() * 3.0).round() as i32);
    f.line(cx, cy, front.0, front.1, rgb(200, 200, 216));
    let race = kind == Kind::Race;
    blit_rider(f, &art.style(style).ride[race as usize][crank % 8], x, y, flip, pal);
}

fn shadow(f: &mut Frame, x: i32, y: i32, w: i32) {
    f.shade_ellipse(x, y, w, 2, 5, 8);
}

// ------------------------------------------------------------------ props

fn draw_tree_trunk(f: &mut Frame, x: i32, y: i32) {
    let top = 92;
    f.rect(x - 4, top, 9, y - top, OUTLINE);
    for yy in top..y {
        for xx in x - 3..x + 4 {
            let bark = (xx * 3 + yy / 3) % 7 == 0;
            let c = if xx >= x + 2 { rgb(72, 48, 32) } else if bark { rgb(88, 56, 40) } else { rgb(120, 80, 56) };
            f.set(xx, yy, c);
        }
    }
    // roots and a grate
    f.rect(x - 7, y - 1, 15, 2, OUTLINE);
    f.hline(x - 6, x + 6, y - 1, rgb(104, 104, 112));
}

fn draw_canopy(f: &mut Frame, x: i32, t: u32, phase: i32) {
    let cy = 82;
    let sway = (((t / 30) as i32 + phase).rem_euclid(4) - 1).clamp(-1, 1);
    let blobs = [(-12, 4, 11, 8), (10, 3, 12, 9), (-1, -6, 14, 10), (0, 8, 13, 7), (-18, 10, 7, 5), (18, 9, 8, 6)];
    for &(dx, dy, rx, ry) in &blobs {
        f.fill_ellipse(x + dx + sway, cy + dy, rx + 1, ry + 1, OUTLINE);
    }
    for &(dx, dy, rx, ry) in &blobs {
        f.fill_ellipse(x + dx + sway, cy + dy, rx, ry, rgb(56, 120, 56));
    }
    for &(dx, dy, rx, ry) in &blobs {
        f.fill_ellipse(x + dx + sway - 2, cy + dy - 2, rx - 3, ry - 3, rgb(88, 160, 72));
    }
    for k in 0..14 {
        let px = x - 18 + (k * 7) % 36 + sway;
        let py = cy - 6 + (k * 5) % 16;
        f.set(px, py, rgb(144, 208, 104));
    }
}

fn draw_kiosk(f: &mut Frame, x: i32, y: i32, t: u32) {
    let (w, h) = (42, 26);
    let x0 = x - w / 2;
    f.rect(x0 - 1, y - h - 1, w + 2, h + 1, OUTLINE);
    f.rect(x0, y - h + 6, w, h - 6, rgb(64, 96, 64));
    f.rect(x0 + 3, y - h + 9, w - 6, 9, rgb(40, 40, 56));
    f.hline(x0, x0 + w - 1, y - 9, rgb(200, 168, 120));
    for k in 0..w {
        let c = if (k / 4) % 2 == 0 { rgb(232, 56, 48) } else { white() };
        f.line(x0 + k, y - h, x0 + k, y - h + 5, c);
    }
    f.text_center(x, y - h + 10, "KOFFIE", rgb(248, 232, 168), OUTLINE, false);
    // steam
    if (t / 20).is_multiple_of(2) {
        f.set(x + 8, y - h - 3, rgb(232, 232, 240));
        f.set(x + 9, y - h - 5, rgb(232, 232, 240));
    }
}

fn draw_organ(f: &mut Frame, x: i32, y: i32, t: u32) {
    let (w, h) = (30, 28);
    let x0 = x - w / 2;
    f.rect(x0 - 1, y - h - 1, w + 2, h - 3, OUTLINE);
    f.rect(x0, y - h, w, h - 5, rgb(176, 32, 48));
    // golden pipes and figures
    for k in 0..6 {
        let px = x0 + 3 + k * 4;
        let ph = 10 + (k % 3) * 3;
        f.rect(px, y - h + 3, 3, ph, rgb(248, 216, 64));
        f.set(px + 1, y - h + 3, white());
    }
    f.rect(x0 + 2, y - 11, w - 4, 3, rgb(248, 136, 24));
    // wheels
    for wx in [x0 + 5, x0 + w - 6] {
        f.fill_ellipse(wx, y - 4, 4, 4, OUTLINE);
        f.fill_ellipse(wx, y - 4, 3, 3, rgb(120, 72, 40));
        f.set(wx, y - 4, rgb(248, 216, 64));
    }
    // notes
    if (t / 15).is_multiple_of(2) {
        let k = ((t / 30) % 3) as i32;
        f.text(x + 10 + k * 3, y - h - 8 - k * 2, "*", rgb(248, 248, 168));
    }
}

fn draw_spot_prop(f: &mut Frame, art: &Art, sp_: Spot, x: i32, y: i32, t: u32) {
    match sp_ {
        Spot::Tree => draw_tree_trunk(f, x, y),
        Spot::Krul => art.krul.draw_foot(f, x, y, &PROPS, false),
        Spot::Glasbak => art.glasbak.draw_foot(f, x, y, &PROPS, false),
        Spot::Organ => draw_organ(f, x, y, t),
        Spot::Kiosk => draw_kiosk(f, x, y, t),
    }
}

// ------------------------------------------------------------------ people

fn person_frame(ppl: &People, anim: f32, running: bool, moving: bool) -> &sp::Sprite {
    if !moving {
        return &ppl.stand;
    }
    let k = (anim as usize) % 4;
    if running {
        &ppl.run[k]
    } else {
        &ppl.walk[k]
    }
}

fn draw_thief(f: &mut Frame, w: &World, art: &Art, cam: f32, t: u32) {
    let th = &w.thief;
    let who = th.who;
    let style = who.def().style;
    let ppl = art.style(style);
    let mut pal = thief_palette(who);
    if th.wet > 0 {
        pal = wet_palette(&pal);
    }
    let x = sx(th.x, cam);
    let y = sy(th.y);
    let flip = th.face_left;
    match th.state {
        TState::Ride => {
            let Some(bi) = th.bike else { return };
            let b = &w.bikes[bi];
            shadow(f, x, y, 14);
            let crank = if th.moving { ((th.anim * 2.0) as usize) % 8 } else { 0 };
            let wheel = ((th.anim * 2.5) as usize) % 4;
            let wob = if th.edge_t > 0 { ((t / 2) % 3) as i32 - 1 } else { 0 };
            draw_rider(f, art, style, &pal, b.kind, Some(bike_pal(b.kind, b.color)), x + wob, y, flip, crank, wheel);
            if b.hot > 0 || b.flagged {
                draw_bike_hot(f, b.kind, x, y, flip, t);
            }
        }
        TState::Pick => {
            shadow(f, x, y, 8);
            let k = ((th.st_t / 5) % 2) as usize;
            blit_person(f, &ppl.pick[k], x, y, flip, &pal);
        }
        TState::Casual => {
            shadow(f, x, y, 7);
            if th.casual_kind == 0 {
                blit_person(f, &ppl.casual, x, y, flip, &pal);
            } else {
                blit_person(f, &ppl.selfie, x, y, flip, &pal);
                let (hx, hy) = sp::canvas_to_screen(20, 4, x, y, flip);
                art.phone.draw(f, hx - 1, hy - 1, &PROPS, flip);
                if (th.st_t / 30).is_multiple_of(4) && th.st_t % 30 < 3 {
                    f.fill_ellipse(hx, hy, 4, 4, white());
                }
            }
        }
        TState::Hide => {
            let spot = th.spot.map(|i| w.spots[i].0).unwrap_or(Spot::Tree);
            match spot {
                Spot::Krul => {
                    // only the feet show under the krul
                    let c = pal[sp::ACC as usize];
                    f.rect(x - 4, y - 2, 3, 2, OUTLINE);
                    f.rect(x + 1, y - 2, 3, 2, OUTLINE);
                    f.hline(x - 4, x - 2, y - 2, c);
                    f.hline(x + 1, x + 3, y - 2, c);
                }
                _ => {
                    // peeking out from behind it
                    let dx = if spot == Spot::Tree { 6 } else { 9 };
                    sp::blit_ghost(f, &ppl.stand, x + if flip { -dx } else { dx }, y - 1, !flip, &pal, t);
                }
            }
        }
        TState::Fall => {
            shadow(f, x, y, 12);
            blit_person(f, &ppl.fall, x, y, flip, &pal);
            stars(f, x, y - 14, t);
        }
        TState::Busted => {
            shadow(f, x, y, 9);
            blit_person(f, &ppl.sit, x, y, flip, &pal);
        }
        TState::Swim => {
            let wl = WATER + 3;
            let bob = ((t / 12) % 2) as i32;
            f.fill_ellipse(x, wl + 3, 11, 4, rgb(40, 88, 96));
            f.fill_ellipse(x, wl + 3, 9, 2, rgb(56, 112, 120));
            // head and hands above the water, the rest under it
            blit_person_clip(f, &ppl.swim, x, wl + 14 + bob, flip, &pal, wl + 2);
            f.hline(x - 8, x + 8, wl + 1, rgb(200, 232, 240));
            if (t / 10).is_multiple_of(3) {
                f.set(x - 10, wl, rgb(200, 232, 240));
                f.set(x + 10, wl, rgb(200, 232, 240));
            }
        }
        TState::Climb => {
            let k = th.st_t as i32;
            let yy = WATER + 14 - k * 20 / 36;
            blit_person_clip(f, &ppl.climb, x, yy, flip, &pal, WATER);
        }
        TState::Toss => {
            shadow(f, x, y, 7);
            blit_person(f, &ppl.toss, x, y, flip, &pal);
        }
        TState::Out => {
            shadow(f, x, y, 7);
            let s = match w.result {
                Some(Outcome::Cleared) => {
                    if (t / 15).is_multiple_of(2) {
                        &ppl.victory
                    } else {
                        &ppl.cheer
                    }
                }
                Some(Outcome::Busted) => &ppl.sit,
                _ => &ppl.stand,
            };
            blit_person(f, s, x, y, flip, &pal);
        }
        TState::Walk => {
            shadow(f, x, y, 7);
            blit_person(f, person_frame(ppl, th.anim, th.running, th.moving), x, y, flip, &pal);
        }
    }
}

fn draw_bike_hot(f: &mut Frame, _kind: Kind, x: i32, y: i32, flip: bool, t: u32) {
    let m = |dx: i32| if flip { -dx } else { dx };
    let swing = ((t / 5) % 3) as i32 - 1;
    let (hx, hy) = (x + m(-3), y - 9);
    f.line(hx, hy, hx + swing, hy + 4, rgb(136, 136, 160));
    f.rect(hx + swing - 1, hy + 4, 3, 2, rgb(200, 200, 216));
}

fn stars(f: &mut Frame, x: i32, y: i32, t: u32) {
    for k in 0..3 {
        let a = t as f32 * 0.15 + k as f32 * 2.1;
        let (px, py) = (x + (a.cos() * 7.0) as i32, y + (a.sin() * 2.5) as i32);
        f.set(px, py, rgb(248, 232, 64));
        f.set(px + 1, py, rgb(248, 232, 64));
        f.set(px, py - 1, white());
    }
}

fn draw_boa(f: &mut Frame, w: &World, art: &Art, i: usize, cam: f32, t: u32) {
    let b = &w.boas[i];
    let x = sx(b.x, cam);
    let y = sy(b.y);
    if !(-40..W + 40).contains(&x) {
        return;
    }
    let pal = match b.ctrl {
        Ctrl::Human(p) => boa_player_palette(p),
        Ctrl::Cpu => boa_palette(i + w.stage),
    };
    let ppl = art.style(2);
    let flip = b.face_left;
    if b.kind == BoaKind::Bike {
        shadow(f, x, y, 14);
        let crank = if b.moving { ((b.anim * 2.0) as usize) % 8 } else { 0 };
        draw_rider(f, art, 2, &pal, Kind::Oma, Some(boa_bike_pal()), x, y, flip, crank, ((b.anim * 2.5) as usize) % 4);
        if b.whistle_t > 0 {
            whistle_lines(f, x + if flip { -8 } else { 8 }, y - 27, t);
        }
        return;
    }
    shadow(f, x, y, 7);
    let s = match b.state {
        BoaState::Ticket => &ppl.write,
        BoaState::Coffee => &ppl.sip,
        BoaState::Wait => &ppl.whistle,
        _ if b.whistle_t > 0 => &ppl.whistle,
        _ => person_frame(ppl, b.anim, b.running, b.moving),
    };
    blit_person(f, s, x, y, flip, &pal);
    if b.kind == BoaKind::Chief {
        // the chief's peaked cap with gold braid
        let (hx, hy) = sp::canvas_to_screen(12, 1, x, y, flip);
        f.hline(hx - 3, hx + 3, hy, rgb(248, 216, 64));
    }
    match b.state {
        BoaState::Ticket => {
            let (hx, hy) = sp::canvas_to_screen(17, 13, x, y, flip);
            art.ticket.draw(f, hx - 4, hy - 4, &PROPS, flip);
        }
        BoaState::Coffee => {
            let (hx, hy) = sp::canvas_to_screen(16, 8, x, y, flip);
            art.coffee.draw(f, hx - 3, hy - 3, &PROPS, flip);
        }
        _ => {}
    }
    if b.whistle_t > 0 || b.state == BoaState::Wait && (t / 20).is_multiple_of(2) {
        whistle_lines(f, x + if flip { -6 } else { 6 }, y - 25, t);
    }
}

fn whistle_lines(f: &mut Frame, x: i32, y: i32, t: u32) {
    let c = rgb(248, 248, 168);
    let k = ((t / 4) % 2) as i32;
    f.line(x, y - 3 - k, x + 3, y - 6 - k, c);
    f.line(x + 1, y, x + 5, y, c);
    f.line(x, y + 3 + k, x + 3, y + 6 + k, c);
}

fn draw_dog(f: &mut Frame, art: &Art, d: &Dog, cam: f32, t: u32) {
    let x = sx(d.x, cam);
    let y = sy(d.y);
    shadow(f, x, y, 7);
    let k = (d.anim as usize) % 2;
    art.dog[k].draw_foot(f, x, y + 1, &PROPS, d.face_left);
    if d.bark > 0 && (t / 6).is_multiple_of(2) {
        bubble(f, x, y - 12, "WAF!");
    }
}

fn draw_ped(f: &mut Frame, w: &World, art: &Art, p: &Ped, cam: f32, t: u32) {
    let x = sx(p.x, cam);
    let y = sy(p.y);
    if !(-30..W + 30).contains(&x) {
        return;
    }
    let pal = ped_palette(p.pal);
    let ppl = art.style(p.style);
    shadow(f, x, y, 7);
    let s = match p.state {
        PState::Walk => person_frame(ppl, p.anim, false, true),
        PState::Photo => &ppl.photo,
        PState::Fall => &ppl.fall,
        PState::Dance => {
            if ((p.anim * 2.0) as u32 + t / 20).is_multiple_of(2) {
                &ppl.cheer
            } else {
                &ppl.victory
            }
        }
    };
    let hop = if p.state == PState::Dance && (t / 10 + p.home as u32).is_multiple_of(3) { 1 } else { 0 };
    if p.suitcase && p.state == PState::Walk {
        // a rolling suitcase behind them
        let bx = x + if p.face_left { 8 } else { -12 };
        f.rect(bx - 1, y - 11, 6, 11, OUTLINE);
        f.rect(bx, y - 10, 4, 9, [rgb(64, 120, 216), rgb(216, 64, 120), rgb(96, 96, 112)][p.pal % 3]);
        f.line(bx + 2, y - 11, x + if p.face_left { 3 } else { -3 }, y - 16, OUTLINE);
    }
    blit_person(f, s, x, y - hop, p.face_left, &pal);
    if p.state == PState::Fall && p.st_t < 60 {
        stars(f, x, y - 12, t);
    }
    if p.state == PState::Photo && p.st_t % 40 < 3 {
        let (hx, hy) = sp::canvas_to_screen(18, 8, x, y, p.face_left);
        f.fill_ellipse(hx, hy, 3, 3, white());
    }
    let _ = w;
}

// ------------------------------------------------------------------ the canal

fn draw_sjaak(f: &mut Frame, w: &World, art: &Art, cam: f32, t: u32) {
    let s = &w.sjaak;
    let x = sx(s.x, cam);
    if !(-60..W + 60).contains(&x) {
        return;
    }
    let bob = ((t / 16) % 2) as i32;
    let wl = WATER + 8 + bob;
    // the pile of bikes on deck
    for (k, &(kind, color)) in s.pile.iter().enumerate().take(6) {
        let px = x - 13 + (k as i32 % 3) * 13 + (k as i32 / 3) * 6;
        let py = wl - 5 - (k as i32 / 3) * 7;
        let kind = if kind == Kind::Bak { Kind::Oma } else { kind };
        draw_bike(f, art, kind, color, px, py, k % 2 == 1, k % 4, Lock::None, false, t);
    }
    // Sjaak at the stern, by the outboard
    let stern = if s.face_left { x + 20 } else { x - 20 };
    let pal = sjaak_palette();
    let ppl = art.style(2);
    let pose = if s.happy > 0 { if (t / 8).is_multiple_of(2) { &ppl.victory } else { &ppl.cheer } } else { &ppl.stand };
    blit_person_clip(f, pose, stern, wl + 2, !s.face_left, &pal, wl - 2);
    // sunglasses
    let (gx, gy) = sp::canvas_to_screen(15, 5, stern, wl + 2, !s.face_left);
    f.hline(gx - 2, gx + 1, gy, rgb(8, 8, 8));
    // a wooden sloop
    let (l, r) = (x - 28, x + 28);
    let pts = [(l + if s.face_left { 0 } else { 2 }, wl - 4), (r - if s.face_left { 2 } else { 0 }, wl - 4), (r - 4, wl + 4), (l + 4, wl + 4)];
    f.fill_poly(&[(pts[0].0 - 1, pts[0].1 - 1), (pts[1].0 + 1, pts[1].1 - 1), (pts[2].0 + 1, pts[2].1 + 1), (pts[3].0 - 1, pts[3].1 + 1)], OUTLINE);
    f.fill_poly(&pts, rgb(136, 80, 48));
    f.hline(l + 2, r - 2, wl - 4, rgb(208, 168, 112));
    f.hline(l + 3, r - 3, wl - 1, rgb(104, 56, 40));
    f.hline(l + 4, r - 4, wl + 2, rgb(104, 56, 40));
    // the sign by the bow, high above his head: we buy bikes
    let sxp = x + if s.face_left { -12 } else { 12 };
    f.rect(sxp - 1, wl - 30, 1, 26, OUTLINE);
    f.rect(sxp - 16, wl - 40, 32, 11, OUTLINE);
    f.rect(sxp - 15, wl - 39, 30, 9, rgb(232, 216, 168));
    f.text_center(sxp, wl - 38, "INKOOP", rgb(176, 32, 32), rgb(232, 216, 168), false);
    // outboard and wake
    let ox = if s.face_left { r - 2 } else { l + 2 };
    f.rect(ox - 2, wl - 7, 4, 9, OUTLINE);
    f.rect(ox - 1, wl - 6, 2, 7, rgb(48, 48, 64));
    if s.moving {
        let back = if s.face_left { 1 } else { -1 };
        for k in 0..4 {
            let wx = ox + back * (4 + k * 5 + (t as i32 / 3) % 5);
            f.hline(wx - 2, wx + 1, wl + 4 + (k % 2), rgb(200, 232, 240));
        }
    }
}

fn draw_boat(f: &mut Frame, b: &Boat, art: &Art, cam: f32, t: u32) {
    let x = sx(b.x, cam);
    if !(-140..W + 140).contains(&x) {
        return;
    }
    let bob = ((t / 20 + b.x as u32) % 2) as i32;
    let wl = WATER + 16 + bob;
    match b.kind {
        BoatKind::Tour => {
            let (l, r) = (x - 56, x + 56);
            f.rect(l - 1, wl - 9, r - l + 2, 13, OUTLINE);
            f.rect(l, wl - 8, r - l, 11, rgb(32, 40, 88));
            f.hline(l, r - 1, wl - 8, rgb(248, 248, 248));
            // the glass roof
            f.rect(l + 7, wl - 20, r - l - 14, 12, OUTLINE);
            f.rect(l + 8, wl - 19, r - l - 16, 11, rgb(160, 208, 232));
            let mut k = l + 10;
            let mut n = 0;
            while k < r - 12 {
                f.line(k, wl - 19, k, wl - 9, white());
                // tourists' heads, all looking at you
                let hc = [rgb(248, 200, 160), rgb(160, 96, 56), rgb(224, 160, 104)][n % 3];
                f.rect(k + 3, wl - 14 - (n % 2) as i32, 3, 3, hc);
                k += 9;
                n += 1;
            }
            f.text_center(x, wl - 6, "RONDVAART", rgb(248, 216, 48), OUTLINE, false);
        }
        BoatKind::Party => {
            let (l, r) = (x - 30, x + 30);
            f.rect(l - 1, wl - 7, r - l + 2, 10, OUTLINE);
            f.rect(l, wl - 6, r - l, 8, rgb(248, 136, 24));
            f.hline(l, r - 1, wl - 6, rgb(248, 200, 120));
            for k in 0..7 {
                let px = l + 5 + k * 8;
                let hop = (t / 8 + k as u32).is_multiple_of(3) as i32;
                f.rect(px - 2, wl - 15 - hop, 5, 9, OUTLINE);
                f.rect(px - 1, wl - 14 - hop, 3, 7, if k % 2 == 0 { rgb(248, 136, 24) } else { rgb(248, 248, 248) });
                f.rect(px - 1, wl - 17 - hop, 3, 3, [rgb(248, 200, 160), rgb(160, 96, 56)][(k % 2) as usize]);
                if (t / 12 + k as u32).is_multiple_of(4) {
                    f.set(px - 2, wl - 19 - hop, rgb(248, 136, 24));
                    f.set(px + 2, wl - 19 - hop, rgb(248, 136, 24));
                }
            }
            if (t / 16).is_multiple_of(2) {
                f.text(x + 12, wl - 28, "*", rgb(248, 248, 168));
                f.text(x - 18, wl - 26, "*", rgb(248, 248, 168));
            }
        }
        BoatKind::Crane => {
            let (l, r) = (x - 34, x + 34);
            f.rect(l - 1, wl - 8, r - l + 2, 12, OUTLINE);
            f.rect(l, wl - 7, r - l, 10, rgb(64, 64, 72));
            f.hline(l, r - 1, wl - 7, rgb(232, 200, 48));
            // the crane
            let (cx, cy) = (x - 16, wl - 8);
            f.rect(cx - 4, cy - 10, 9, 10, OUTLINE);
            f.rect(cx - 3, cy - 9, 7, 9, rgb(232, 200, 48));
            let fishing = !b.done && b.vx == 0.0;
            let reach = if fishing { (b.t as f32 / 60.0).min(1.0) } else { 0.6 };
            let (tx, ty) = (cx + 20 + (12.0 * reach) as i32, cy - 30);
            f.thick_line(cx, cy - 8, tx, ty, 3, OUTLINE);
            f.line(cx, cy - 8, tx, ty, rgb(248, 216, 48));
            // the claw goes down into the water, and comes up with a bike
            let down = if fishing {
                let k = b.t as i32;
                if k < 150 {
                    k.min(80) * 46 / 80
                } else {
                    (46 - (k - 150) * 46 / 100).max(0)
                }
            } else {
                4
            };
            f.line(tx, ty, tx, ty + down, rgb(48, 48, 64));
            let clawy = ty + down;
            f.line(tx - 3, clawy, tx, clawy - 2, rgb(48, 48, 64));
            f.line(tx + 3, clawy, tx, clawy - 2, rgb(48, 48, 64));
            if fishing && b.t > 150 && b.t < 250 {
                draw_bike(f, art, Kind::Wreck, 0, tx, clawy + 16, false, 0, Lock::None, false, t);
                if (t / 4).is_multiple_of(2) {
                    f.set(tx - 6, clawy + 18, rgb(120, 184, 216));
                    f.set(tx + 4, clawy + 20, rgb(120, 184, 216));
                }
            }
            f.text_center(x + 8, wl - 5, "FIETSEN-", rgb(248, 248, 248), OUTLINE, false);
        }
    }
}

// ------------------------------------------------------------------ cones

fn draw_cones(f: &mut Frame, w: &World, cam: f32, t: u32) {
    for b in &w.boas {
        if b.ctrl != Ctrl::Cpu || b.state == BoaState::Ticket {
            continue;
        }
        let col = match b.state {
            BoaState::Chase | BoaState::Wait => rgb(248, 96, 72),
            BoaState::Investigate | BoaState::Search => rgb(248, 176, 64),
            _ => rgb(248, 240, 160),
        };
        let range = if b.state == BoaState::Coffee { b.vision * 0.3 } else { b.vision };
        let dir = if b.face_left { -1.0 } else { 1.0 };
        let x0 = b.x;
        let x1 = b.x + dir * range;
        let (lo, hi) = (x0.min(x1).floor() as i32, x0.max(x1).ceil() as i32);
        for wx in lo..=hi {
            let scx = wx - cam as i32;
            if !(0..W).contains(&scx) {
                continue;
            }
            for wy in 0..=(DEPTH as i32) {
                let y = STREET + wy;
                if (scx + y + (t / 6) as i32) % 3 != 0 {
                    continue;
                }
                if w.sight(b, wx as f32, wy as f32) > 0.0 {
                    let i = (y * W + scx) as usize;
                    f.px[i] = mix(f.px[i], col, 0.45);
                }
            }
        }
    }
}

// ------------------------------------------------------------------ the view

#[derive(Clone, Copy, PartialEq)]
enum Ent {
    Bike(usize),
    Thief,
    Boa(usize),
    Dog(usize),
    Ped(usize),
    Spot(usize),
    Heron(usize),
    Bollard(i32),
}

/// Draw the street around view `v`'s camera. `clip`: the rows that matter.
pub fn draw_view(f: &mut Frame, w: &World, sc: &Scenery, art: &Art, v: usize, t: u32, clip: (i32, i32)) {
    let cam = w.cam[v];
    sc.draw_back(f, cam, t, clip);
    sc.draw_street(f, cam, clip);
    draw_cones(f, w, cam, t);
    sc.draw_water(f, cam, t, clip);
    sc.draw_houseboats(f, cam, t, clip, &|f, x, y| art.cat.draw(f, x, y, &PROPS, false));
    // sunken bikes poke out of the water; coots nest on some
    for (i, b) in w.bikes.iter().enumerate() {
        if b.state == BikeState::Sunk {
            let x = sx(b.x, cam);
            f.line(x - 3, WATER + 2, x, WATER - 3, OUTLINE);
            f.line(x, WATER - 3, x + 3, WATER - 4, OUTLINE);
            if i % 2 == 0 {
                f.rect(x - 4, WATER, 9, 2, rgb(136, 104, 64));
                art.coot.draw(f, x - 4, WATER - 6, &PROPS, false);
            }
        }
    }
    for b in &w.boats {
        draw_boat(f, b, art, cam, t);
    }
    draw_sjaak(f, w, art, cam, t);
    // the swimmer is in the water, in front of the quay wall
    if matches!(w.thief.state, TState::Swim | TState::Climb) {
        draw_thief(f, w, art, cam, t);
    }

    // everything on the street, back to front
    let mut ents: Vec<(f32, Ent)> = Vec::new();
    for (i, b) in w.bikes.iter().enumerate() {
        if matches!(b.state, BikeState::Parked | BikeState::Loose) {
            ents.push((b.y, Ent::Bike(i)));
        }
    }
    if !matches!(w.thief.state, TState::Swim | TState::Climb) {
        let y = if w.thief.state == TState::Hide { w.thief.y - 0.5 } else { w.thief.y };
        ents.push((y, Ent::Thief));
    }
    for (i, b) in w.boas.iter().enumerate() {
        ents.push((b.y, Ent::Boa(i)));
        if let Some(d) = &b.dog {
            ents.push((d.y, Ent::Dog(i)));
        }
    }
    for (i, p) in w.peds.iter().enumerate() {
        ents.push((p.y, Ent::Ped(i)));
    }
    for (i, s) in w.spots.iter().enumerate() {
        ents.push((s.2, Ent::Spot(i)));
    }
    for (i, h) in w.herons.iter().enumerate() {
        if !h.gone && !h.flying {
            ents.push((h.y, Ent::Heron(i)));
        }
    }
    let first = (cam as i32 / 80) * 80;
    for k in 0..5 {
        let bx = first + k * 80 + 40;
        if !w.spots.iter().any(|s| (s.1 as i32 - bx).abs() < 14) {
            ents.push((DEPTH + 0.5, Ent::Bollard(bx)));
        }
    }
    ents.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for &(_, e) in &ents {
        match e {
            Ent::Bike(i) => {
                let b = &w.bikes[i];
                let x = sx(b.x, cam);
                if (-30..W + 30).contains(&x) {
                    shadow(f, x, sy(b.y), 12);
                    draw_bike(f, art, b.kind, b.color, x, sy(b.y), b.face_left, 0, b.lock, b.hot > 0 || b.state == BikeState::Loose && b.flagged, t);
                }
            }
            Ent::Thief => draw_thief(f, w, art, cam, t),
            Ent::Boa(i) => draw_boa(f, w, art, i, cam, t),
            Ent::Dog(i) => {
                if let Some(d) = &w.boas[i].dog {
                    draw_dog(f, art, d, cam, t);
                }
            }
            Ent::Ped(i) => draw_ped(f, w, art, &w.peds[i], cam, t),
            Ent::Spot(i) => {
                let (s, x, y) = w.spots[i];
                let px = sx(x, cam);
                if (-40..W + 40).contains(&px) {
                    draw_spot_prop(f, art, s, px, sy(y), t);
                }
            }
            Ent::Heron(i) => {
                let h = &w.herons[i];
                let x = sx(h.x, cam);
                art.heron.draw_foot(f, x, sy(h.y), &PROPS, (h.t / 300).is_multiple_of(2));
            }
            Ent::Bollard(bx) => {
                let x = bx - cam as i32;
                art.bollard.draw_foot(f, x, sy(DEPTH) + 1, &PROPS, false);
            }
        }
    }
    // tree tops over everything
    for &(s, x, _) in &w.spots {
        if s == Spot::Tree {
            let px = sx(x, cam);
            if (-40..W + 40).contains(&px) {
                draw_canopy(f, px, t, x as i32);
                if w.tree_bike.map(|k| w.spots[k].1 == x).unwrap_or(false) {
                    draw_bike(f, art, Kind::Oma, 6, px + 4, 86, true, 0, Lock::None, false, t);
                }
            }
        }
    }
    // flying herons
    for h in &w.herons {
        if h.flying {
            let x = sx(h.x, cam);
            let y = sy(h.y) - 20 - h.z as i32;
            art.heron_fly[((h.t / 8) % 2) as usize].draw(f, x - 14, y, &PROPS, h.vx < 0.0);
        }
    }
    // the bike flying onto Sjaak's boat
    if let (Some((kind, color, x0, y0, x1)), TState::Toss) = (w.thief.toss, w.thief.state) {
        let k = (w.thief.st_t as f32 / 26.0).min(1.0);
        let x = x0 + (x1 - x0) * k;
        let ys = sy(y0) as f32 + (WATER as f32 + 2.0 - sy(y0) as f32) * k - (k * std::f32::consts::PI).sin() * 34.0;
        draw_bike(f, art, kind, color, sx(x, cam), ys as i32, (w.t / 6).is_multiple_of(2), (w.t as usize / 3) % 4, Lock::None, false, t);
    }
    // fx
    for fx in &w.fx {
        let x = sx(fx.x, cam);
        let y = sy(fx.y);
        match fx.kind {
            FxKind::Splash => {
                let k = fx.t as i32;
                let wl = WATER + 2;
                for d in 0..6 {
                    let a = d as f32 / 5.0 * std::f32::consts::PI;
                    let r = k as f32 * 0.9;
                    let (px, py) = (x + (a.cos() * r) as i32, wl - (a.sin() * (20.0 - (k as f32 - 10.0).powi(2) * 0.2).max(0.0)) as i32);
                    f.rect(px - 1, py - 1, 3, 3, rgb(200, 232, 248));
                }
                f.hline(x - k / 2 - 4, x + k / 2 + 4, wl + 1, white());
            }
            FxKind::Krak => {
                if fx.t < 12 {
                    let r = 6 + fx.t as i32;
                    for d in 0..8 {
                        let a = d as f32 * std::f32::consts::TAU / 8.0;
                        f.line(x + (a.cos() * 4.0) as i32, y - 8 + (a.sin() * 4.0) as i32, x + (a.cos() * r as f32) as i32, y - 8 + (a.sin() * r as f32) as i32, rgb(248, 248, 168));
                    }
                }
            }
            FxKind::Dust => {
                let r = 3 + fx.t as i32 / 4;
                f.shade_ellipse(x - 6, y - 2, r, r / 2 + 1, 5, 4);
                f.shade_ellipse(x + 6, y - 3, r, r / 2 + 1, 5, 4);
            }
            FxKind::Drip => {
                f.set(x, fx.y as i32 + STREET, rgb(120, 184, 232));
            }
            FxKind::Note => {
                let c = rgb(248, 248, 200);
                let ny = fx.y as i32 + STREET;
                f.rect(x, ny, 2, 2, c);
                f.line(x + 1, ny, x + 1, ny - 4, c);
                f.set(x + 2, ny - 4, c);
            }
            FxKind::Flash => {}
            FxKind::Ring => {
                // how far the KRAK carries: every BOA inside this hears it
                if fx.t < 24 {
                    let r = (170.0 * (fx.t as f32 / 20.0).min(1.0)) as i32;
                    let n = (r * 6).max(16);
                    for k in 0..n {
                        if k % 3 == (fx.t % 3) as i32 {
                            let a = k as f32 / n as f32 * std::f32::consts::TAU;
                            f.set(x + (a.cos() * r as f32) as i32, y - 8 + (a.sin() * r as f32 * 0.35) as i32, rgb(248, 248, 168));
                        }
                    }
                }
            }
        }
    }
    // prompts for the human thief, under anything a BOA says
    if w.views[v] == View::Thief && matches!(w.thief.ctrl, Ctrl::Human(_)) && w.phase == Phase::Play {
        draw_prompts(f, w, cam, t);
    }
    // what the BOA's say and think (one bubble per spot, never stacked)
    let mut taken: Vec<(i32, i32, i32)> = Vec::new();
    for b in &w.boas {
        let x = sx(b.x, cam);
        if !(-20..W + 20).contains(&x) {
            continue;
        }
        let head = sy(b.y) - if b.kind == BoaKind::Bike { 34 } else { 32 };
        if let Some((s, _)) = b.say {
            let bw = text_width(s, false) + 8;
            let free = !taken.iter().any(|&(tx, ty, tw)| (tx - x).abs() * 2 < tw + bw && (ty - head).abs() < 13);
            if free {
                bubble(f, x, head, s);
                taken.push((x, head, bw));
                continue;
            }
        }
        if b.ctrl == Ctrl::Cpu {
            match b.state {
                BoaState::Chase | BoaState::Wait => mark(f, x, head - 2, "!", rgb(232, 64, 48), 1.0),
                _ if b.sus > 0.12 => mark(f, x, head - 2, "?", rgb(248, 216, 48), b.sus.min(1.0)),
                BoaState::Patrol if b.next_look < 24 && (t / 4).is_multiple_of(2) => {
                    // about to glance back over the shoulder
                    let back = if b.face_left { ">" } else { "<" };
                    f.text_center(x + if b.face_left { 5 } else { -5 }, head - 9, back, rgb(248, 248, 168), OUTLINE, true);
                }
                _ => {}
            }
        }
    }
    // who you are, when you might get lost among the others
    draw_tags(f, w, cam, t);
    for p in &w.popups {
        let x = sx(p.x, cam);
        let y = sy(p.y);
        let tw = text_width(&p.text, true);
        f.shade_rect(x - tw / 2 - 2, y - 2, tw + 5, 11, 1, 2);
        f.text_center(x, y, &p.text, p.color, OUTLINE, true);
    }
    // where Sjaak is, when he is off screen
    let sjx = sx(w.sjaak.x, cam);
    let busy = w.phase == Phase::Countdown || w.thief.state == TState::Pick;
    if !(-10..=W + 10).contains(&sjx) && w.views[v] == View::Thief && !busy {
        let left = sjx < 0;
        let ax = if left { 4 } else { W - 5 };
        let ay = if w.views.len() > 1 { WATER } else { WATER + 6 };
        let d = ((w.sjaak.x - w.thief.x).abs() / 10.0) as i32;
        let lab = format!("SJAAK {}M", d);
        let tw = text_width(&lab, false);
        let tx = if left { ax + 6 } else { ax - 6 - tw };
        f.rect(tx - 2, ay - 5, tw + 4, 11, rgb(16, 16, 48));
        f.text_sh(tx, ay - 3, &lab, rgb(248, 216, 48), OUTLINE, false);
        let dir = if left { -1 } else { 1 };
        for k in 0..4 {
            f.line(ax - dir * k, ay - 3 + k, ax - dir * k, ay + 3 - k, rgb(248, 216, 48));
        }
    }
}

/// A "1P"/"2P" pointer over the player: always in 2P; in 1P at the start
/// and whenever someone else stands close.
fn draw_tags(f: &mut Frame, w: &World, cam: f32, t: u32) {
    let two = w.views.len() > 1;
    let tag = |f: &mut Frame, x: i32, head: i32, p: usize| {
        let col = if p == 0 { rgb(248, 232, 64) } else { rgb(96, 224, 248) };
        let y = head - 12;
        f.rect(x - 7, y - 1, 15, 9, OUTLINE);
        f.rect(x - 6, y, 13, 7, col);
        f.text(x - 5, y, if p == 0 { "1P" } else { "2P" }, rgb(16, 16, 48));
        f.set(x, y + 8, OUTLINE);
        f.hline(x - 1, x + 1, y + 7, col);
    };
    let th = &w.thief;
    if let Ctrl::Human(p) = th.ctrl {
        let crowded = w.peds.iter().any(|q| (q.x - th.x).abs() < 14.0 && (q.y - th.y).abs() < 10.0)
            || w.boas.iter().any(|b| (b.x - th.x).abs() < 14.0 && (b.y - th.y).abs() < 10.0);
        let start = w.phase == Phase::Countdown || (w.phase == Phase::Play && w.phase_t < 180);
        let show = two || start || crowded;
        let visible = !matches!(th.state, TState::Hide | TState::Swim | TState::Climb | TState::Out);
        if show && visible && (start || (t / 20) % 4 != 3) {
            let head = sy(th.y) - if th.state == TState::Ride { 34 } else { 32 } + if th.state == TState::Pick { 8 } else { 0 };
            tag(f, sx(th.x, cam), head - 9, p);
        }
    }
    for b in &w.boas {
        if let Ctrl::Human(p) = b.ctrl {
            tag(f, sx(b.x, cam), sy(b.y) - 41, p);
        }
    }
}

/// "?" filling up, or a red "!".
fn mark(f: &mut Frame, x: i32, y: i32, s: &str, c: u32, level: f32) {
    let h = 9;
    f.rect(x - 5, y - h - 2, 11, h + 3, OUTLINE);
    f.rect(x - 4, y - h - 1, 9, h + 1, rgb(16, 16, 48));
    let fill = (level * (h + 1) as f32) as i32;
    f.rect(x - 4, y - fill, 9, fill, shade(c, 1, 2));
    f.text_center(x + 1, y - h + 1, s, c, OUTLINE, false);
}

fn draw_prompts(f: &mut Frame, w: &World, cam: f32, t: u32) {
    let th = &w.thief;
    let x = sx(th.x, cam);
    // under the feet: the head is for tags and bubbles
    let y = (sy(th.y) + 13).min(WATER + 14);
    let line = match th.state {
        TState::Walk => {
            if let Some(i) = w.bike_at_hand() {
                let b = &w.bikes[i];
                let needed = w.needed().contains(&b.kind);
                let what = if b.state == BikeState::Loose || b.lock == Lock::None {
                    "Z: OPSTAPPEN".to_string()
                } else {
                    format!("Z: {}", b.lock.name())
                };
                let kind = if needed { format!("{} *", b.kind.def().name) } else { b.kind.def().name.to_string() };
                Some((what, kind))
            } else if w.spot_at_hand().is_some() {
                Some(("C: VERSTOPPEN".to_string(), String::new()))
            } else {
                None
            }
        }
        TState::Ride if w.sjaak_near() => Some(("Z: GOOIEN!".to_string(), String::new())),
        TState::Hide => Some(("C: TEVOORSCHIJN".to_string(), String::new())),
        TState::Swim => Some(("UP: ERUIT KLIMMEN".to_string(), String::new())),
        _ => None,
    };
    if w.banners[0].is_some() {
        return;
    }
    if let Some((a, b)) = line {
        let tw = text_width(&a, false).max(text_width(&b, false));
        let bottom = if b.is_empty() { y + 9 } else { y + 18 };
        f.shade_rect(x - tw / 2 - 3, y - 2, tw + 6, bottom - y + 2, 1, 3);
        if (t / 12) % 4 != 3 {
            f.text_center(x, y, &a, rgb(248, 232, 64), OUTLINE, false);
        }
        if !b.is_empty() {
            f.text_center(x, y + 9, &b, rgb(216, 216, 232), OUTLINE, false);
        }
    }
}

// ------------------------------------------------------------------ HUD

/// A tiny bike for the order list.
pub fn bike_icon(f: &mut Frame, kind: Kind, x: i32, y: i32, done: bool) {
    let (c, cd) = match kind {
        Kind::Oma => (rgb(64, 64, 88), rgb(32, 32, 48)),
        Kind::Race => (rgb(216, 48, 48), rgb(144, 24, 40)),
        Kind::Bak => (rgb(176, 112, 56), rgb(112, 64, 32)),
        Kind::Gold => (rgb(248, 216, 64), rgb(200, 144, 24)),
        Kind::Wreck => (rgb(168, 96, 56), rgb(112, 56, 32)),
    };
    let (c, cd) = if done { (rgb(72, 88, 72), rgb(48, 56, 48)) } else { (c, cd) };
    let wc = if done { rgb(96, 104, 96) } else { rgb(200, 200, 216) };
    let (rw, fw) = if kind == Kind::Bak { ((x + 3, y + 6), (x + 14, y + 6)) } else { ((x + 3, y + 6), (x + 12, y + 6)) };
    for (wx, wy) in [rw, fw] {
        f.circle_outline(wx, wy, 3, OUTLINE);
        f.circle_outline(wx, wy, 2, wc);
    }
    f.line(rw.0, rw.1, x + 7, y + 6, c);
    f.line(x + 7, y + 6, x + 5, y + 1, c);
    f.line(x + 5, y + 1, x + 10, y + 2, cd);
    f.line(x + 10, y + 2, fw.0, fw.1, c);
    f.hline(x + 3, x + 6, y, rgb(48, 32, 32));
    if kind == Kind::Bak {
        f.rect(x + 10, y + 1, 6, 4, rgb(176, 112, 56));
    }
    if kind == Kind::Race {
        f.line(x + 10, y + 2, x + 12, y + 1, OUTLINE);
    }
    if done {
        let g = rgb(120, 232, 88);
        f.line(x + 5, y + 4, x + 8, y + 8, g);
        f.line(x + 8, y + 8, x + 14, y - 1, g);
        f.line(x + 6, y + 4, x + 9, y + 8, g);
    }
}

pub fn fmt_clock(frames: u32) -> String {
    let s = frames.div_ceil(60);
    format!("{}:{:02}", s / 60, s % 60)
}

pub fn draw_hud(f: &mut Frame, w: &World, art: &Art, t: u32) {
    f.rect(0, 0, W, 32, rgb(16, 16, 48));
    f.hline(0, W - 1, 32, rgb(248, 136, 24));
    f.hline(0, W - 1, 33, OUTLINE);
    // the order
    f.text_sh(4, 2, "BESTELLING", rgb(184, 184, 216), OUTLINE, false);
    for (k, &(kind, done)) in w.order.iter().enumerate() {
        bike_icon(f, kind, 4 + k as i32 * 19, 12, done);
    }
    // the clock
    let secs = w.time_left / 60;
    let col = if secs < 20 && (t / 15).is_multiple_of(2) { rgb(232, 64, 48) } else { rgb(248, 248, 248) };
    let clock = fmt_clock(w.time_left);
    f.text_grad_center(128, 4, &clock, &grad7(col), OUTLINE, true);
    // cash, fines, cutters
    let cash = format!("EUR {}", w.cash());
    f.text_sh(W - 4 - text_width(&cash, false), 3, &cash, rgb(248, 216, 48), OUTLINE, false);
    for k in 0..MAX_FINES {
        let x = W - 44 + k as i32 * 10;
        if k < w.thief.fines {
            art.ticket.draw(f, x, 13, &PROPS, false);
        } else {
            f.rect_outline(x, 13, 9, 11, rgb(56, 56, 88));
        }
    }
    for k in 0..w.thief.cutters {
        art.cutter.draw(f, W - 66 - k as i32 * 9, 15 + k as i32 * 2, &PROPS, false);
    }
    // what makes you a target right now
    let th = &w.thief;
    let riding_hot = th.bike.map(|i| w.bikes[i].hot > 0 || w.bikes[i].flagged).unwrap_or(false);
    let badge = if th.wanted > 0 {
        Some(("GEZOCHT!", rgb(232, 64, 48), th.wanted as f32 / 420.0))
    } else if riding_hot || th.hot > 0 {
        let flagged = th.bike.map(|i| w.bikes[i].flagged).unwrap_or(false);
        let k = th.bike.map(|i| w.bikes[i].hot as f32 / 300.0).unwrap_or(th.hot as f32 / 120.0);
        Some(("HEET!", rgb(248, 136, 24), if flagged { 1.0 } else { k }))
    } else {
        None
    };
    if let Some((txt, col, k)) = badge {
        let bw = 50;
        f.rect(128 - bw / 2 - 1, 12, bw + 2, 9, OUTLINE);
        f.rect(128 - bw / 2, 13, (bw as f32 * k.clamp(0.0, 1.0)) as i32, 7, shade(col, 1, 2));
        if (t / 10) % 3 != 2 {
            f.text_center(128, 13, txt, col, OUTLINE, false);
        }
    }
    // the street map: bikes on the order, BOA's, Sjaak, you
    let (mx, my, mw) = (84, 22, 88);
    f.rect(mx - 1, my - 2, mw + 3, 9, OUTLINE);
    f.rect(mx, my - 1, mw + 1, 7, rgb(40, 40, 72));
    f.hline(mx, mx + mw, my + 5, rgb(48, 96, 120));
    let px = |x: f32| mx + (x / w.len * mw as f32) as i32;
    let needed = w.needed();
    for b in &w.bikes {
        if matches!(b.state, BikeState::Parked | BikeState::Loose) && needed.contains(&b.kind) {
            f.set(px(b.x), my + if b.y < 30.0 { 0 } else { 3 }, rgb(248, 216, 48));
        }
    }
    let sj = px(w.sjaak.x);
    f.hline(sj - 1, sj + 1, my + 5, rgb(160, 216, 248));
    for b in &w.boas {
        let c = if matches!(b.state, BoaState::Chase | BoaState::Wait) { rgb(248, 64, 48) } else { rgb(120, 168, 248) };
        let x = px(b.x);
        f.rect(x, my, 1, 4, c);
        f.set(x + if b.face_left { -1 } else { 1 }, my + 1, c);
    }
    if (t / 8).is_multiple_of(2) {
        let x = px(w.thief.x);
        f.rect(x - 1, my + 1, 3, 3, white());
    }
}

/// 2P: a thin strip per view.
pub fn draw_mini_hud(f: &mut Frame, w: &World, v: usize, y: i32, p: usize, t: u32) {
    f.rect(0, y, W, 9, rgb(16, 16, 48));
    let col = if p == 0 { rgb(248, 232, 64) } else { rgb(96, 224, 248) };
    let tag = if p == 0 { "1P" } else { "2P" };
    f.rect(2, y + 1, 13, 7, col);
    f.text(3, y + 1, tag, rgb(16, 16, 48));
    let clock = fmt_clock(w.time_left);
    f.text_sh(128 - text_width(&clock, false) / 2, y + 1, &clock, white(), OUTLINE, false);
    match w.views[v] {
        View::Thief => {
            f.text_sh(18, y + 1, "DIEF", white(), OUTLINE, false);
            for (k, &(kind, done)) in w.order.iter().enumerate() {
                let x = 46 + k as i32 * 9;
                f.rect(x, y + 2, 7, 5, if done { rgb(120, 216, 88) } else { kind_color(kind) });
            }
            let cash = format!("EUR {}", w.cash());
            f.text_sh(W - 4 - text_width(&cash, false), y + 1, &cash, rgb(248, 216, 48), OUTLINE, false);
        }
        View::Boa(i) => {
            let b = &w.boas[i];
            f.text_sh(18, y + 1, "BOA", white(), OUTLINE, false);
            if b.cool == 0 && (t / 30).is_multiple_of(2) {
                f.text_sh(42, y + 1, "C: FLUIT", rgb(248, 216, 48), OUTLINE, false);
            }
            let s = format!("BOETE {} KLACHT {}", b.fines, b.complaints);
            f.text_sh(W - 4 - text_width(&s, false), y + 1, &s, rgb(216, 216, 232), OUTLINE, false);
        }
    }
}

pub fn kind_color(k: Kind) -> u32 {
    match k {
        Kind::Oma => rgb(96, 96, 128),
        Kind::Race => rgb(216, 48, 48),
        Kind::Bak => rgb(176, 112, 56),
        Kind::Gold => rgb(248, 216, 64),
        Kind::Wreck => rgb(168, 96, 56),
    }
}

/// The lock you are working on, big, over the canal.
pub fn draw_lock_panel(f: &mut Frame, w: &World, art: &Art, y: i32, t: u32) {
    let th = &w.thief;
    let Some(i) = th.pick else { return };
    let b = &w.bikes[i];
    let (x0, w_) = (28, 200);
    f.rect(x0 - 1, y - 1, w_ + 2, 28, OUTLINE);
    f.rect(x0, y, w_, 26, rgb(16, 16, 48));
    f.rect_outline(x0, y, w_, 26, rgb(248, 136, 24));
    f.text_sh(x0 + 4, y + 3, b.lock.name(), rgb(248, 216, 48), OUTLINE, false);
    let hint = match b.lock {
        Lock::Ring => "TAP Z!",
        Lock::Chain | Lock::Double => "ZAGEN: < > < >",
        Lock::U => "Z IN HET GROEN!",
        Lock::None => "",
    };
    f.text_sh(x0 + w_ - 4 - text_width(hint, false), y + 3, hint, white(), OUTLINE, false);
    // progress
    let (bx, by, bw) = (x0 + 4, y + 14, 90);
    f.rect(bx - 1, by - 1, bw + 2, 6, OUTLINE);
    f.rect(bx, by, bw, 4, rgb(48, 48, 80));
    f.rect(bx, by, (b.progress.min(1.0) * bw as f32) as i32, 4, rgb(120, 216, 88));
    if b.lock == Lock::U {
        // the swinging needle and the green zone
        let (dx, dw) = (x0 + 100, 66);
        f.rect(dx - 1, y + 11, dw + 2, 10, OUTLINE);
        f.rect(dx, y + 12, dw, 8, rgb(96, 40, 48));
        let half = th.who.u_zone() / 2.0;
        let z0 = dx + ((th.zone - half) * dw as f32) as i32;
        let z1 = dx + ((th.zone + half) * dw as f32) as i32;
        f.rect(z0, y + 12, (z1 - z0).max(1), 8, rgb(96, 200, 88));
        let n = 0.5 + 0.5 * th.needle.sin();
        let nx = dx + (n * dw as f32) as i32;
        f.rect(nx - 1, y + 10, 3, 12, OUTLINE);
        f.line(nx, y + 11, nx, y + 20, white());
    } else if b.lock != Lock::None {
        // the button to press, bouncing
        let k = ((t / 6) % 2) as i32;
        let lab = if matches!(b.lock, Lock::Chain | Lock::Double) { if th.saw_left { ">" } else { "<" } } else { "Z" };
        f.rect(x0 + 120, y + 11 - k, 13, 11, OUTLINE);
        f.rect(x0 + 121, y + 12 - k, 11, 9, rgb(248, 216, 48));
        f.text(x0 + 124, y + 13 - k, lab, rgb(16, 16, 48));
    }
    if th.cutters > 0 {
        art.cutter.draw(f, x0 + w_ - 26, y + 14, &PROPS, false);
        f.text_sh(x0 + w_ - 10, y + 14, "X", rgb(248, 216, 48), OUTLINE, false);
    }
}

// ------------------------------------------------------------------ banners

pub fn draw_banner(f: &mut Frame, b: &Banner, cy: i32, small: bool) {
    let n = b.text.chars().count() as i32;
    let mut scale = if b.scale > 0 { b.scale } else if n * 18 < 236 { 3 } else { 2 };
    if small {
        scale = scale.min(2);
    }
    let h = 7 * scale;
    let (x, y) = if b.pop {
        (128, cy - h / 2 - if b.t < 4 { 4 - b.t } else { 0 })
    } else {
        let x = if b.t < 10 { 128 + (10 - b.t) * 30 } else { 128 };
        (x, cy - h / 2 + if (10..16).contains(&b.t) { (16 - b.t) / 2 } else { 0 })
    };
    let x = if b.t > b.dur - 8 { x - (b.t - (b.dur - 8)) * 34 } else { x };
    let grad: Vec<u32> = if b.color == white() { SUNSET.to_vec() } else { grad6(b.color).to_vec() };
    f.logo(x, y, &b.text, scale, &grad, OUTLINE, LOGO_SHADOW);
}
