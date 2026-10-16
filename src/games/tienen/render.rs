//! TIENEN! drawing: the pitch with the goal, kids, ball, shadow and
//! landing marker in depth order, the life around the pitch, the HUD with
//! everybody's points, banners, popups and bubbles, and the panels for the
//! latjetrap, the counting-out rhyme and the billenbrand.

use super::art::*;
use super::data::*;
use super::scenery::*;
use super::world::*;
use crate::engine::gfx::*;
use crate::engine::sprites::Sprite;
use crate::engine::ui::{grad6, grad7, LOGO_SHADOW, OUTLINE, SUNSET};

pub fn white() -> u32 {
    rgb(248, 248, 248)
}

pub const P1_COL: u32 = rgb(248, 232, 64);
pub const P2_COL: u32 = rgb(96, 224, 248);

pub fn bubble(f: &mut Frame, x: i32, y: i32, text: &str) {
    let w = text_width(text, false) + 6;
    let bx = (x - w / 2).clamp(2, W - w - 2);
    let by = y - 13;
    f.rect(bx - 1, by - 1, w + 2, 12, OUTLINE);
    f.rect(bx, by, w, 10, white());
    let x = x.clamp(bx + 4, bx + w - 4);
    for k in 0..3 {
        f.hline(x - 2 + k, x + 2 - k, by + 10 + k, if k == 2 { OUTLINE } else { white() });
        f.set(x - 3 + k, by + 10 + k, OUTLINE);
        f.set(x + 3 - k, by + 10 + k, OUTLINE);
    }
    f.text(bx + 3, by + 2, text, rgb(16, 16, 48));
}

/// The sprite for a kid right now, and whether it is flipped.
pub fn kid_sprite<'a>(art: &'a Art, k: &Kid, keeper: bool, t: u32) -> (&'a Sprite, bool) {
    let ppl = art.kid(k.id);
    let flip = k.face == -1;
    let moving = k.vx.abs() + k.vy.abs() > 0.25;
    let s = match k.act {
        Act::Swing { kind, .. } => match kind {
            Kind::Volley => &ppl.kick[if k.at < 5 { 0 } else { 1 }],
            Kind::Chest => &ppl.chest,
            Kind::Heel => &ppl.heel,
            Kind::Header => {
                if k.air() {
                    &ppl.header_jump
                } else {
                    &ppl.header
                }
            }
            Kind::Bicycle => &ppl.bike[if k.at < 5 { 0 } else { 1 }],
        },
        Act::Down => &ppl.down,
        Act::Dive(_) => &ppl.dive,
        Act::Catch => {
            if k.air() {
                &ppl.high
            } else {
                &ppl.ready
            }
        }
        Act::Punch => &ppl.punch,
        Act::Hold => &ppl.hold,
        Act::Roll => &ppl.roll,
        Act::Cheer => {
            if (t / 8).is_multiple_of(2) {
                &ppl.victory
            } else {
                &ppl.cheer
            }
        }
        Act::Sulk => &ppl.sulk,
        Act::Bent => &ppl.bent,
        Act::Ouch => &ppl.ouch,
        Act::Idle => {
            if k.air() {
                if keeper {
                    &ppl.high
                } else {
                    &ppl.jump
                }
            } else if moving {
                &ppl.run[(k.anim as usize) % 4]
            } else if keeper {
                &ppl.ready
            } else {
                &ppl.stand
            }
        }
    };
    (s, flip)
}

/// A dive flies sideways in depth: draw the diving keeper turned that way.
fn draw_kid(f: &mut Frame, art: &Art, w: &World, i: usize, t: u32) {
    let k = &w.kids[i];
    let keeper = i == w.keeper && matches!(w.phase, Phase::Play | Phase::Goal | Phase::Wissel | Phase::Car | Phase::Over);
    let (gx, gy) = proj(k.x, k.y, 0.0);
    f.shade_ellipse(gx, gy, 7, 2, 5, 8);
    let (x, y) = proj(k.x, k.y, k.z);
    let (s, flip) = kid_sprite(art, k, keeper, t);
    let pal = kid_palette(k.id);
    blit_person(f, s, x, y, flip, &pal);
}

fn draw_ball(f: &mut Frame, art: &Art, b: &Ball, t: u32) {
    let (gx, gy) = proj(b.x, b.y, 0.0);
    let r = (4.0 - b.z / 20.0).clamp(2.0, 4.0) as i32;
    f.shade_ellipse(gx, gy, r, 1, 1, 2);
    let (x, y) = proj(b.x, b.y, b.z);
    let spin = b.spin.abs() > 0.05 || b.vx.abs() + b.vy.abs() > 0.3;
    let fr = if spin { ((t / 3) % 2) as usize } else { 0 };
    art.ball[fr].draw(f, x - 4, y - 7, &PROPS, false);
}

/// The goal: the back net, the far side, the bar (behind everything) and
/// the near post with its side net (in depth order).
pub fn goal_back(f: &mut Frame) {
    let net = rgb(216, 216, 232);
    let net_d = rgb(112, 112, 136);
    let bz = BAR * 0.7;
    // the inside of the goal: a dark stipple, so the mouth reads
    let quad = [proj(NET_X, G_Y0, bz), proj(NET_X, G_Y1, bz), proj(NET_X, G_Y1, 0.0), proj(NET_X, G_Y0, 0.0)];
    let (minx, maxx) = (quad.iter().map(|p| p.0).min().unwrap_or(0), quad.iter().map(|p| p.0).max().unwrap_or(0));
    let (miny, maxy) = (quad.iter().map(|p| p.1).min().unwrap_or(0), quad.iter().map(|p| p.1).max().unwrap_or(0));
    let mut tmp = Vec::new();
    for y in miny..=maxy {
        for x in minx..=maxx {
            if (x + y) % 2 == 0 && inside(&quad, x, y) {
                tmp.push((x, y));
            }
        }
    }
    for (x, y) in tmp {
        let c = f.get(x, y);
        f.set(x, y, shade(c, 2, 5));
    }
    // the back of the net: a grid
    for k in 0..=6 {
        let y = G_Y0 + (G_Y1 - G_Y0) * k as f32 / 6.0;
        let (x0, y0) = proj(NET_X, y, 0.0);
        let (x1, y1) = proj(NET_X, y, bz);
        f.line(x0, y0, x1, y1, net_d);
        let (x2, y2) = proj(GOAL_X, y, BAR);
        f.line(x1, y1, x2, y2, net);
    }
    for k in 0..=4 {
        let z = bz * k as f32 / 4.0;
        let (x0, y0) = proj(NET_X, G_Y0, z);
        let (x1, y1) = proj(NET_X, G_Y1, z);
        f.line(x0, y0, x1, y1, net_d);
    }
    // the far side net
    for k in 0..=3 {
        let x = NET_X + (GOAL_X - NET_X) * k as f32 / 3.0;
        let zt = bz + (BAR - bz) * k as f32 / 3.0;
        let (a, b) = proj(x, G_Y0, 0.0);
        let (c, d) = proj(x, G_Y0, zt);
        f.line(a, b, c, d, net_d);
    }
    // the far post and the bar
    post(f, G_Y0);
    let (x0, y0) = proj(GOAL_X, G_Y0, BAR);
    let (x1, y1) = proj(GOAL_X, G_Y1, BAR);
    f.thick_line(x0, y0 - 1, x1, y1 - 1, 4, OUTLINE);
    f.line(x0, y0 - 1, x1, y1 - 1, white());
    f.line(x0, y0, x1, y1, rgb(200, 200, 216));
}

fn inside(q: &[(i32, i32); 4], x: i32, y: i32) -> bool {
    let mut sign = 0;
    for i in 0..4 {
        let (a, b) = (q[i], q[(i + 1) % 4]);
        let c = (b.0 - a.0) * (y - a.1) - (b.1 - a.1) * (x - a.0);
        let s = c.signum();
        if s != 0 {
            if sign != 0 && s != sign {
                return false;
            }
            sign = s;
        }
    }
    true
}

fn post(f: &mut Frame, y: f32) {
    let (x0, y0) = proj(GOAL_X, y, 0.0);
    let (_, y1) = proj(GOAL_X, y, BAR);
    f.rect(x0 - 2, y1 - 1, 5, y0 - y1 + 2, OUTLINE);
    f.rect(x0 - 1, y1, 3, y0 - y1, white());
    f.rect(x0 + 1, y1, 1, y0 - y1, rgb(184, 184, 208));
}

pub fn goal_near(f: &mut Frame) {
    let net_d = rgb(136, 136, 160);
    let bz = BAR * 0.7;
    for k in 0..=3 {
        let x = NET_X + (GOAL_X - NET_X) * k as f32 / 3.0;
        let zt = bz + (BAR - bz) * k as f32 / 3.0;
        let (a, b) = proj(x, G_Y1, 0.0);
        let (c, d) = proj(x, G_Y1, zt);
        f.line(a, b, c, d, net_d);
    }
    let (a, b) = proj(NET_X, G_Y1, bz);
    let (c, d) = proj(GOAL_X, G_Y1, BAR);
    f.line(a, b, c, d, net_d);
    post(f, G_Y1);
}

enum Ent {
    Kid(usize),
    Ball,
    NearPost,
    Dog,
    Pigeon(usize),
    Car,
}

/// Everything on the pitch.
pub fn draw_world(f: &mut Frame, w: &World, sc: &Scenery, art: &Art, t: u32) {
    sc.draw(f);
    draw_life(f, w, art, t);
    goal_back(f);
    // the keeper's area glows a little for a 1-point keeper (catch anywhere)
    let mut ents: Vec<(f32, Ent)> = Vec::new();
    for (i, k) in w.kids.iter().enumerate() {
        ents.push((k.y, Ent::Kid(i)));
    }
    let show_ball = !(w.phase == Phase::CountOut);
    if show_ball {
        ents.push((w.ball.y + if w.ball.held { 0.6 } else { 0.0 }, Ent::Ball));
    }
    ents.push((G_Y1, Ent::NearPost));
    if w.dog.is_some() {
        ents.push((w.dog.as_ref().map(|d| d.y).unwrap_or(0.0), Ent::Dog));
    }
    for (i, p) in w.pigeons.iter().enumerate() {
        ents.push((p.y, Ent::Pigeon(i)));
    }
    if w.car.is_some() {
        ents.push((36.0, Ent::Car));
    }
    ents.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    // the landing marker goes under everybody
    if let (Some((lx, ly)), Some(to)) = (w.ball.land, w.ball.cross_to)
        && w.ball.air && w.phase == Phase::Play {
            let (x, y) = proj(lx, ly, 0.0);
            let human = w.kids[to].human();
            let col = match human {
                Some(0) => P1_COL,
                Some(_) => P2_COL,
                None => rgb(200, 200, 216),
            };
            if human.is_some() || (t / 4).is_multiple_of(2) {
                f.line(x - 3, y - 1, x + 3, y + 1, col);
                f.line(x - 3, y + 1, x + 3, y - 1, col);
                if human.is_some() {
                    f.circle_outline(x, y, 5, col);
                }
            }
        }
    for (_, e) in &ents {
        match *e {
            Ent::Kid(i) => draw_kid(f, art, w, i, t),
            Ent::Ball => draw_ball(f, art, &w.ball, t),
            Ent::NearPost => goal_near(f),
            Ent::Dog => {
                if let Some(d) = &w.dog {
                    let (x, y) = proj(d.x, d.y, 0.0);
                    f.shade_ellipse(x, y, 7, 2, 5, 8);
                    let flip = d.tx < d.x;
                    art.dog[((t / 5) % 2) as usize].draw_foot(f, x, y, &PROPS, flip);
                    if d.has {
                        let bx = if flip { x - 11 } else { x + 11 };
                        art.ball[0].draw(f, bx - 4, y - 14, &PROPS, false);
                    }
                }
            }
            Ent::Pigeon(i) => {
                let p = &w.pigeons[i];
                let (x, y) = proj(p.x, p.y, p.z);
                let fr = if p.fly {
                    2 + ((t / 4) % 2) as usize
                } else if ((t + p.peck * 7) / 30).is_multiple_of(3) {
                    1
                } else {
                    0
                };
                if !p.fly {
                    f.set(x, y, shade(f.get(x, y), 2, 3));
                }
                art.pigeon[fr].draw_foot(f, x, y, &PROPS, p.vx < 0.0 && p.fly || (!p.fly && p.peck.is_multiple_of(2)));
            }
            Ent::Car => {
                if let Some(c) = &w.car {
                    let (x, y) = proj(c.x, 36.0, 0.0);
                    f.shade_ellipse(x, y + 1, 20, 3, 1, 2);
                    art.car.draw_foot(f, x, y + 2, &car_pal(c.color), true);
                }
            }
        }
    }
    // keeper on 1: the whole pitch is his area
    if w.phase == Phase::Play && w.kids[w.keeper].pts == 1 && (t / 20).is_multiple_of(2) {
        let (x, y) = proj(w.kids[w.keeper].x, w.kids[w.keeper].y, 0.0);
        f.text_center(x, y + 3, "VANGT OVERAL", rgb(96, 224, 248), OUTLINE, false);
    }
    draw_tags(f, w, t);
    draw_popups(f, w);
    draw_bubbles(f, w);
}

/// 1P / 2P pointers over the humans.
fn draw_tags(f: &mut Frame, w: &World, t: u32) {
    for k in &w.kids {
        let Some(p) = k.human() else { continue };
        if matches!(k.act, Act::Bent) {
            continue;
        }
        let (x, y) = proj(k.x, k.y, k.z);
        let x = if k.x < GOAL_X + 20.0 { x + 14 } else { x };
        let col = if p == 0 { P1_COL } else { P2_COL };
        let ty = y - 40 - ((t / 10) % 2) as i32;
        let tag = if p == 0 { "1P" } else { "2P" };
        f.rect(x - 7, ty, 14, 9, OUTLINE);
        f.rect(x - 6, ty + 1, 12, 7, col);
        f.text(x - 5, ty + 1, tag, rgb(16, 16, 48));
        f.set(x, ty + 9, col);
        f.hline(x - 1, x + 1, ty + 9, OUTLINE);
    }
}

fn draw_popups(f: &mut Frame, w: &World) {
    if w.banner.is_some() {
        return;
    }
    for p in &w.popups {
        let (x, y) = proj(p.x, p.y, 0.0);
        let tw = text_width(&p.text, true);
        f.shade_rect(x - tw / 2 - 2, y - 1, tw + 4, 10, 1, 3);
        f.text_center(x, y, &p.text, p.color, OUTLINE, true);
    }
}

fn draw_bubbles(f: &mut Frame, w: &World) {
    let banner = w.banner.is_some();
    let mut used: Vec<(i32, i32)> = Vec::new();
    for k in &w.kids {
        let Some((s, _)) = &k.bubble else { continue };
        let (x, y) = proj(k.x, k.y, k.z);
        let mut by = y - 36;
        // never two bubbles on top of each other
        while used.iter().any(|&(ux, uy)| (ux - x).abs() < 50 && (uy - by).abs() < 13) {
            by -= 13;
        }
        if banner && (by < 104 || !used.is_empty()) {
            continue;
        }
        used.push((x, by));
        bubble(f, x, by.max(40), s);
    }
}

/// Life around the pitch: the balcony mum, Mevrouw Knorr, the ice cream
/// van, lamps, clouds.
fn draw_life(f: &mut Frame, w: &World, art: &Art, t: u32) {
    let def = w.def();
    match def.place {
        Place::Flat => {
            // Timmy's mum on the gallery, now and then
            let cycle = t % 1800;
            if (600..900).contains(&cycle) && w.banner.is_none() {
                let ppl = art.style(1);
                let s = if (t / 20).is_multiple_of(2) { &ppl.cheer } else { &ppl.stand };
                blit_person_clip(f, s, 96, 72, false, &adult_palette(0), 70);
                if (640..760).contains(&cycle) {
                    bubble(f, 96, 44, "TIMMY! ETEN!");
                }
            }
        }
        Place::School => {
            // Mevrouw Knorr peeks over her fence
            let ppl = art.style(2);
            let peek = w.why == Some(Why::Fence) && w.phase == Phase::Wissel;
            let s = if peek { &ppl.cheer } else { &ppl.stand };
            let x = 40 + ((t / 240) % 3) as i32 * 10;
            blit_person_clip(f, s, x, 104, false, &adult_palette(2), 90);
            art.gnome.draw_foot(f, 84, 100, &PROPS, false);
            art.sunflower.draw_foot(f, 6, 96, &PROPS, false);
            art.sunflower.draw_foot(f, 14, 98, &PROPS, false);
            if peek && w.banner.is_none() {
                bubble(f, x, 76, "DIE IS VAN MIJ!");
            }
        }
        Place::Kooi => {
            // the lamps hum; the ice cream van drives past now and then
            if (t / 3).is_multiple_of(40) {
                f.rect(15, 5, 17, 4, rgb(255, 255, 255));
            }
            let cycle = t % 2400;
            if cycle < 500 {
                let x = -40 + cycle as i32 * 300 / 500;
                art.van.draw_foot(f, x, 103, &PROPS, false);
            }
        }
    }
}

// ------------------------------------------------------------ HUD

pub fn draw_hud(f: &mut Frame, w: &World, art: &Art, t: u32) {
    f.rect(0, 0, W, 26, rgb(16, 16, 48));
    f.hline(0, W - 1, 26, rgb(248, 136, 24));
    f.hline(0, W - 1, 27, OUTLINE);
    for (i, k) in w.kids.iter().enumerate() {
        let x0 = i as i32 * 64;
        let d = kid(k.id);
        let keeper = i == w.keeper && !matches!(w.phase, Phase::Latje | Phase::CountOut);
        let bg = if keeper { rgb(40, 40, 96) } else { rgb(24, 24, 64) };
        f.rect(x0 + 1, 1, 62, 24, bg);
        f.rect(x0 + 1, 1, 3, 24, d.color);
        if keeper {
            f.rect_outline(x0 + 1, 1, 62, 24, if (t / 10).is_multiple_of(2) { rgb(248, 136, 24) } else { rgb(200, 96, 24) });
            art.glove.draw(f, x0 + 23, 14, &PROPS, false);
        }
        let name = if d.name.len() > 7 { &d.name[..7] } else { d.name };
        f.text_sh(x0 + 6, 3, name, rgb(216, 216, 240), OUTLINE, false);
        if let Some(p) = k.human() {
            let col = if p == 0 { P1_COL } else { P2_COL };
            let tx = x0 + 53;
            f.rect(tx, 2, 9, 7, col);
            f.text(tx + 2, 2, if p == 0 { "1" } else { "2" }, rgb(16, 16, 48));
        }
        let pts = format!("{}", k.pts);
        let col = if k.pts <= 2 { rgb(248, 96, 80) } else if k.pts <= 5 { rgb(248, 200, 64) } else { white() };
        f.text_grad(x0 + 6, 13, &pts, &grad7(col), OUTLINE, true);
        if keeper {
            let c = if k.pts <= 5 { rgb(248, 200, 64) } else { rgb(160, 168, 216) };
            f.text(x0 + 33, 15, "KEEPT", c);
        } else if let (Some(q), Some(_)) = (w.quota, k.human()) {
            // the respect target
            let s = k.stats.goal_count();
            let col = if s >= q { rgb(96, 224, 248) } else { rgb(200, 200, 232) };
            art.ball[0].draw(f, x0 + 30, 14, &PROPS, false);
            f.text(x0 + 40, 15, &format!("{}/{}", s.min(q), q), col);
        }
    }
}

/// The shot that finishes a keeper who is nearly out, on the kerb.
pub fn draw_finish_hint(f: &mut Frame, w: &World) {
    let k = &w.kids[w.keeper];
    if w.phase != Phase::Play || k.pts > 5 {
        return;
    }
    let what = match k.pts {
        1 => "A VOLLEY FINISHES",
        2 => "A HEADER OR HEEL FINISHES",
        5 => "A BICYCLE KICK FINISHES",
        3 => "EXACTLY 3 FINISHES",
        _ => "EXACTLY 4 FINISHES",
    };
    let s = format!("{} ON {}: {} HIM", kid(k.id).name, k.pts, what);
    let s = if text_width(&s, false) > 244 { format!("{} ON {}: {}", kid(k.id).name, k.pts, what) } else { s };
    let tw = text_width(&s, false);
    f.rect(128 - tw / 2 - 4, 211, tw + 8, 11, rgb(16, 16, 48));
    f.text_center(128, 213, &s, rgb(248, 200, 64), OUTLINE, false);
}

pub fn draw_banner(f: &mut Frame, b: &Banner, cy: i32) {
    let n = b.text.chars().count() as i32;
    let scale = if n * 18 < 236 { 3 } else { 2 };
    let h = 7 * scale;
    let x = if b.t < 10 { 128 + (10 - b.t as i32) * 30 } else { 128 };
    let x = if b.t + 8 > b.len { x - (b.t as i32 + 8 - b.len as i32) * 34 } else { x };
    let y = cy - h / 2 + if (10..16).contains(&b.t) { (16 - b.t as i32) / 2 } else { 0 };
    let grad: Vec<u32> = if b.color == 0xF8F8F8 { SUNSET.to_vec() } else { grad6(b.color).to_vec() };
    f.logo(x, y, &b.text, scale, &grad, OUTLINE, LOGO_SHADOW);
    if !b.sub.is_empty() && b.t > 6 {
        let sw = text_width(&b.sub, false) + 8;
        let sx = (x - sw / 2).clamp(2, W - sw - 2);
        f.rect(sx, y + h + 10, sw, 11, rgb(16, 16, 48));
        f.rect_outline(sx, y + h + 10, sw, 11, b.color);
        f.text_sh(sx + 4, y + h + 12, &b.sub, white(), OUTLINE, false);
    }
}

// ------------------------------------------------------------ panels

/// The latjetrap: whose turn, the needle, who is safe.
pub fn draw_latje(f: &mut Frame, w: &World, t: u32) {
    let l = &w.latje;
    let Some(&k) = l.left.get(l.turn) else { return };
    let kid_ = &w.kids[k];
    let name = kid(kid_.id).name;
    f.rect(4, 30, 248, 13, rgb(16, 16, 48));
    f.rect_outline(4, 30, 248, 13, rgb(248, 136, 24));
    f.text_sh(8, 33, &format!("LATJETRAP RONDE {}: {}", l.round + 1, name), white(), OUTLINE, false);
    // the gauge: a vertical bar next to the shooter, the bar zone in the middle
    if l.stage == 0 {
        let (sx, sy) = proj(kid_.x, kid_.y, 0.0);
        let (gx, gy, gh) = (sx + 14, sy - 50, 44);
        f.rect(gx - 1, gy - 1, 8, gh + 2, OUTLINE);
        f.rect(gx, gy, 6, gh, rgb(40, 48, 96));
        let zone = (0.064 * gh as f32) as i32;
        f.rect(gx, gy + gh / 2 - zone, 6, zone * 2, rgb(96, 224, 96));
        let ny = gy + gh - (l.needle * gh as f32) as i32;
        f.rect(gx - 3, ny - 1, 12, 3, OUTLINE);
        f.rect(gx - 2, ny, 10, 1, rgb(248, 232, 64));
        if kid_.human().is_some() {
            if (t / 15).is_multiple_of(2) {
                f.text_center(sx, sy + 6, "Z: SHOOT AT THE BAR", P1_COL, OUTLINE, false);
            }
        } else {
            f.text_center(sx, sy + 6, "AIMING...", rgb(200, 200, 232), OUTLINE, false);
        }
    }
}

pub fn draw_count_out(f: &mut Frame, w: &World, t: u32) {
    let l = &w.latje;
    f.rect(4, 30, 248, 13, rgb(16, 16, 48));
    f.rect_outline(4, 30, 248, 13, rgb(248, 136, 24));
    let words: Vec<&str> = COUNT_OUT.iter().take(l.word).copied().collect();
    let line = words.join(" ");
    let tail = if line.len() > 40 { &line[line.len() - 40..] } else { &line[..] };
    f.text_sh(8, 33, tail, P1_COL, OUTLINE, false);
    if let Some(&k) = l.left.get(l.point)
        && l.word > 0 {
            let kd = &w.kids[k];
            let (x, y) = proj(kd.x, kd.y, kd.z);
            let bob = ((t / 6) % 2) as i32;
            f.fill_poly(&[(x - 5, y - 46 - bob), (x + 5, y - 46 - bob), (x, y - 39 - bob)], rgb(248, 232, 64));
            if let Some(word) = COUNT_OUT.get(l.word.saturating_sub(1)) {
                bubble(f, x, y - 50, word);
            }
        }
}

pub fn draw_brand(f: &mut Frame, w: &World, t: u32) {
    let br = &w.brand;
    let Some(lo) = w.loser else { return };
    f.rect(4, 30, 248, 13, rgb(16, 16, 48));
    f.rect_outline(4, 30, 248, 13, rgb(248, 104, 144));
    let shooter = br.shooters.get(br.turn).copied().unwrap_or(0);
    f.text_sh(8, 33, &format!("BILLENBRAND: {} SHOOTS AT {}", kid(w.kids[shooter].id).name, kid(w.kids[lo].id).name), white(), OUTLINE, false);
    if br.stage == 0 {
        // the aim sweeps over the loser's bottom
        let ty = goal_mid() + br.aim * 14.0;
        let (x, y) = proj(GOAL_X + 13.0, ty, 9.0);
        let col = if (t / 4).is_multiple_of(2) { rgb(248, 104, 144) } else { white() };
        f.circle_outline(x, y, 4, col);
        f.hline(x - 6, x + 6, y, col);
        f.rect(x, y - 6, 1, 13, col);
        if w.kids[shooter].human().is_some() && (t / 15).is_multiple_of(2) {
            f.text_center(128, 204, "Z: SHOOT!", P1_COL, OUTLINE, true);
        }
        if w.kids[lo].human().is_some() && (t / 15).is_multiple_of(2) {
            f.text_center(128, 204, "UP/DOWN: WIGGLE!", P1_COL, OUTLINE, true);
        }
    }
}
