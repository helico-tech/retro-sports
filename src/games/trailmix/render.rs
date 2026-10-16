//! Draws a race: one view (camera on a runner) into a band of the screen,
//! with parallax, terrain, props, obstacles, animals, runners, effects,
//! speech bubbles, tips and banners; and the HUD.

use super::course::*;
use super::race::*;
use super::runners::{self, GERALD, RUNNERS};
use super::scenery::{ground_index, PropArt, Scenery, TOP};
use super::sprites::*;
use crate::engine::gfx::*;
use crate::engine::sprites::{blit, canvas_to_screen, Palette};
use crate::engine::ui::{grad6, grad7, OUTLINE, SUNSET};

pub struct Ctx<'a> {
    pub race: &'a Race,
    pub sc: &'a Scenery,
    pub art: &'a Art,
    pub props: &'a PropArt,
    pub profile: &'a [u32],
}

/// A view: rows [y0, y0 + h) following runner `runner`.
#[derive(Clone, Copy)]
pub struct View {
    pub y0: i32,
    pub h: i32,
    pub runner: usize,
    /// player index shown in this view (for banners)
    pub slot: usize,
    pub split: bool,
    /// fixed camera x instead of following the runner
    pub focus: Option<f32>,
    pub show_runners: bool,
}

impl View {
    pub fn full(runner: usize) -> View {
        View { y0: 33, h: H - 33, runner, slot: 0, split: false, focus: None, show_runners: true }
    }
    pub fn band(runner: usize, slot: usize) -> View {
        View { y0: 12 + slot as i32 * 112, h: 100, runner, slot, split: true, focus: None, show_runners: true }
    }
    pub fn backdrop(x: f32, y0: i32, h: i32) -> View {
        View { y0, h, runner: 0, slot: 0, split: false, focus: Some(x), show_runners: false }
    }
}

pub struct Cam {
    pub x: f32,
    pub alt: f32,
    pub ground_y: i32,
    pub y0: i32,
    pub y1: i32,
}

impl Cam {
    pub fn sx(&self, wx: f32) -> i32 {
        (wx - self.x).round() as i32
    }
    pub fn sy(&self, alt: f32) -> i32 {
        self.ground_y - (alt - self.alt).round() as i32
    }
}

pub const YELLOW: u32 = rgb(248, 232, 64);
pub const CYAN: u32 = rgb(96, 224, 248);
const WHITE: u32 = rgb(248, 248, 248);
pub const HUD_BG: u32 = rgb(16, 16, 48);
pub const HUD_LINE: u32 = rgb(248, 176, 32);
pub const HUD_LINE2: u32 = rgb(56, 152, 72);

pub fn camera(race: &Race, v: &View) -> Cam {
    let c = &race.course;
    let x = match v.focus {
        Some(fx) => fx + 96.0,
        None => race.runners[v.runner].x.max(0.0),
    };
    let alt = (c.height(x - 60.0) + c.height(x - 25.0) + c.height(x) + c.height(x + 25.0) + c.height(x + 60.0)) / 5.0;
    let lead = if v.split { 100.0 } else { 96.0 };
    Cam { x: x - lead, alt, ground_y: v.y0 + v.h * if v.split { 70 } else { 73 } / 100, y0: v.y0, y1: v.y0 + v.h }
}

fn hash(x: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(2654435761);
    h ^= h >> 15;
    h = h.wrapping_mul(2246822519);
    h ^ (h >> 13)
}

/// The whole view: background, terrain, things on the trail, effects.
pub fn draw_view(f: &mut Frame, cx: &Ctx, v: &View, t: u32) {
    let race = cx.race;
    let cam = camera(race, v);
    cx.sc.draw_back(f, cam.x, cam.alt, v.y0, v.h, t, cam.ground_y);
    draw_fog_back(f, cx, &cam, t);
    draw_back_props(f, cx, &cam);
    draw_terrain(f, cx, &cam, t);
    draw_trail(f, cx, &cam, v, t);
    draw_fx(f, cx, &cam);
    draw_foreground(f, cx, &cam, t);
    draw_fog(f, cx, &cam, t);
    if !v.show_runners {
        return;
    }
    draw_popups(f, cx, &cam);
    draw_bubbles(f, cx, &cam, v);
    draw_tip(f, cx, &cam, v, t);
    if let Some(b) = &race.banners[v.slot.min(1)] {
        draw_banner(f, b, cam.y0, cam.y1);
    }
}

fn prop_sprite(p: &PropArt, k: PropKind, var: u8) -> &CSprite {
    let v = var as usize;
    match k {
        PropKind::Pine => &p.pine[v % 2],
        PropKind::Leafy => &p.leafy[v % 2],
        PropKind::Birch => &p.birch,
        PropKind::Fir => &p.fir[v % 2],
        PropKind::Bush => &p.bush[v % 2],
        PropKind::Fern => &p.fern,
        PropKind::Flowers => &p.flowers[v % 3],
        PropKind::Boulder => &p.boulder[v % 2],
        PropKind::Fence => &p.fence,
        PropKind::Ribbon => &p.ribbon,
        PropKind::Sign(i) => &p.signs[i as usize % p.signs.len()],
        PropKind::DeerSign => &p.deer_sign,
        PropKind::KmPost(n) => &p.km[((n as usize / 2).max(1) - 1).min(p.km.len() - 1)],
        PropKind::StartArch => &p.start_arch,
        PropKind::FinishArch => &p.finish_arch,
        PropKind::SummitCross => &p.summit,
        PropKind::Cairn => &p.cairn,
        PropKind::SnowPole => &p.snow_pole,
        PropKind::House => &p.house[v % 2],
        PropKind::Church => &p.church,
        PropKind::Hut => &p.hut,
    }
}

fn draw_back_props(f: &mut Frame, cx: &Ctx, cam: &Cam) {
    let c = &cx.race.course;
    let mut i = c.first_prop(cam.x - 50.0);
    while i < c.props.len() {
        let p = c.props[i];
        if p.x > cam.x + W as f32 + 50.0 {
            break;
        }
        i += 1;
        if matches!(p.kind, PropKind::StartArch | PropKind::FinishArch) {
            continue;
        }
        let s = prop_sprite(cx.props, p.kind, p.var);
        let sx = cam.sx(p.x);
        let sy = cam.sy(c.height(p.x)) - 1;
        s.draw_clip(f, sx, sy, cam.y0, cam.y1);
    }
    // aid tables behind the trail
    for a in &c.aids {
        let sx = cam.sx(a.x + 16.0);
        if (-40..W + 40).contains(&sx) {
            cx.props.aid_table.draw_clip(f, sx, cam.sy(c.height(a.x + 16.0)) - 1, cam.y0, cam.y1);
        }
    }
}

fn verge_color(cx: &Ctx, g: Ground, k: usize) -> u32 {
    match g {
        Ground::Snow => [rgb(240, 248, 248), rgb(200, 216, 240), WHITE][k],
        Ground::Rock | Ground::Scree => [rgb(136, 136, 152), rgb(104, 104, 128), rgb(176, 176, 192)][k],
        Ground::Road => [rgb(96, 168, 72), rgb(64, 128, 56), rgb(136, 200, 96)][k],
        _ => cx.sc.verge[k],
    }
}

fn draw_terrain(f: &mut Frame, cx: &Ctx, cam: &Cam, t: u32) {
    let c = &cx.race.course;
    for sx in 0..W {
        let wx = cam.x + sx as f32;
        let wi = wx.floor() as i32;
        let top = cam.sy(c.height(wx));
        let g = c.ground(wx);
        let tex = &cx.sc.tex[ground_index(g)];
        let hsh = hash(wi);
        // the bank behind the trail
        let vh = 2 + (hsh % 4) as i32 + if (wi / 3) % 7 == 0 { 2 } else { 0 };
        for y in (top - vh).max(cam.y0)..top.min(cam.y1) {
            let k = if y == top - vh { 2 } else if (y + wi) % 3 == 0 { 1 } else { 0 };
            f.px[(y * W + sx) as usize] = verge_color(cx, g, k);
        }
        let tx = if g == Ground::Water { (wi + (t / 6) as i32).rem_euclid(32) } else { wi.rem_euclid(32) };
        // trail top face
        for y in top.max(cam.y0)..(top + TOP).min(cam.y1) {
            let r = y - top;
            let col = if r == 0 { tex.edge } else { tex.top[(r * 32 + tx) as usize] };
            f.px[(y * W + sx) as usize] = col;
        }
        // lip and front face
        let lip = top + TOP;
        if lip >= cam.y0 && lip < cam.y1 {
            f.px[(lip * W + sx) as usize] = tex.lip;
        }
        let ty = wi.rem_euclid(32);
        for y in (lip + 1).max(cam.y0)..cam.y1 {
            let d = y - lip - 1;
            let band = if d < 26 { 0 } else if d < 56 { 1 } else { 2 };
            let col = tex.front[band][((d & 31) * 32 + ty) as usize];
            f.px[(y * W + sx) as usize] = col;
        }
        // grass hanging over the edge, and a dangling root now and then
        let grassy = matches!(g, Ground::Grass | Ground::Dirt) && cx.sc.stage < 2;
        if grassy && hsh.is_multiple_of(3) {
            let n = 1 + (hsh / 3 % 3) as i32;
            for k in 1..=n {
                let y = lip + k;
                if y >= cam.y0 && y < cam.y1 {
                    f.px[(y * W + sx) as usize] = if k == n { cx.sc.verge[1] } else { cx.sc.verge[0] };
                }
            }
        }
        if matches!(g, Ground::Grass | Ground::Dirt | Ground::Mud) && hsh.is_multiple_of(29) {
            for k in 1..(4 + (hsh / 29 % 6) as i32) {
                let y = lip + k;
                let x = sx + (k / 3) % 2;
                if y >= cam.y0 && y < cam.y1 && x < W {
                    f.px[(y * W + x) as usize] = rgb(88, 56, 32);
                }
            }
        }
    }
    // things buried in the ground: stones, bones, a fossil, a lost shoe
    let cell = 56;
    let first = (cam.x as i32).div_euclid(cell) - 1;
    for k in first..first + W / cell + 3 {
        let h = hash(k * 977 + 13);
        if !h.is_multiple_of(3) {
            continue;
        }
        let wx = (k * cell + (h / 3 % 40) as i32) as f32;
        let g = c.ground(wx);
        if g == Ground::Water || g == Ground::Road {
            continue;
        }
        let lip = cam.sy(c.height(wx)) + TOP;
        let depth = 10 + (h / 7 % 24) as i32;
        let kind = match cx.sc.stage {
            2 if g == Ground::Snow || g == Ground::Rock || g == Ground::Scree => [0, 5, 0, 5, 2, 0][(h / 11 % 6) as usize],
            _ => [0, 0, 1, 2, 3, 4, 0, 4][(h / 11 % 8) as usize],
        };
        let s = &cx.props.buried[kind];
        s.draw_clip(f, cam.sx(wx), lip + depth, (lip + 2).max(cam.y0), cam.y1);
    }
    // stepping stones in the streams
    let mut x = cam.x.floor() as i32 - (cam.x.floor() as i32).rem_euclid(4);
    while (x as f32) < cam.x + W as f32 {
        let g = c.ground(x as f32);
        if g == Ground::Rock && c.ground(x as f32 - 4.0) == Ground::Water {
            let sx = cam.sx(x as f32 + 4.0);
            let sy = cam.sy(c.height(x as f32 + 4.0));
            cx.props.stone.draw_clip(f, sx, sy + 6, cam.y0, cam.y1);
        }
        x += 4;
    }
}

#[derive(Clone, Copy)]
enum Thing {
    Ob(usize),
    Runner(usize),
    Animal(usize),
    Crowd(usize, u32),
    Volunteer(usize),
    Deer(usize),
    Arch(bool),
}

fn draw_trail(f: &mut Frame, cx: &Ctx, cam: &Cam, v: &View, t: u32) {
    let race = cx.race;
    let c = &race.course;
    let lo = cam.x - 60.0;
    let hi = cam.x + W as f32 + 60.0;
    let mut things: Vec<(i32, Thing)> = Vec::with_capacity(48);
    for (i, a) in c.aids.iter().enumerate() {
        if a.x > lo && a.x < hi {
            things.push((0, Thing::Volunteer(i)));
        }
    }
    for (i, cr) in c.crowds.iter().enumerate() {
        if cr.x > lo - 30.0 && cr.x < hi + 30.0 {
            for k in 0..cr.n {
                things.push((-1, Thing::Crowd(i, k)));
            }
        }
    }
    for (i, p) in c.props.iter().enumerate() {
        if matches!(p.kind, PropKind::StartArch | PropKind::FinishArch) && p.x > lo - 40.0 && p.x < hi + 40.0 {
            things.push((-2, Thing::Arch(p.kind == PropKind::FinishArch)));
            let _ = i;
        }
    }
    let mut i = c.first_ob(lo);
    while i < c.obs.len() && c.obs[i].x < hi {
        let o = c.obs[i];
        let depth = if o.kind == ObKind::Branch { 0 } else { 5 };
        things.push((depth, Thing::Ob(i)));
        i += 1;
    }
    for (i, a) in race.animals.iter().enumerate() {
        if a.x > lo && a.x < hi {
            things.push((if a.kind == AnimalKind::Cow { 4 } else { 5 }, Thing::Animal(i)));
        }
    }
    for (i, d) in race.deer.iter().enumerate() {
        if d.active && d.x > lo && d.x < hi {
            things.push((if d.t < DEER_CROSS.1 { 3 } else { 12 }, Thing::Deer(i)));
        }
    }
    for (i, r) in race.runners.iter().enumerate() {
        if r.x > lo && r.x < hi && v.show_runners {
            // the view's own runner is drawn last within its lane
            let d = r.lane * 2 + if i == v.runner { 1 } else { 0 };
            things.push((d.max(1), Thing::Runner(i)));
        }
    }
    things.sort_by_key(|a| a.0);
    // shadows first
    for (_, th) in &things {
        if let Thing::Runner(i) = th {
            let r = &race.runners[*i];
            let gx = cam.sx(r.x);
            let gy = cam.sy(c.height(r.x)) + r.lane;
            let lift = ((r.alt - c.height(r.x)) / 6.0) as i32;
            if gy > cam.y0 && gy < cam.y1 {
                f.shade_ellipse(gx, gy, (6 - lift).max(2), 1, 5, 8);
            }
        }
    }
    for (_, th) in &things {
        match *th {
            Thing::Ob(i) => draw_obstacle(f, cx, cam, i, t),
            Thing::Runner(i) => draw_runner(f, cx, cam, i, t, v),
            Thing::Animal(i) => draw_animal(f, cx, cam, i, t),
            Thing::Crowd(i, k) => draw_fan(f, cx, cam, i, k, t),
            Thing::Volunteer(i) => draw_volunteer(f, cx, cam, i, t, v),
            Thing::Deer(i) => draw_deer(f, cx, cam, i, t),
            Thing::Arch(finish) => {
                let x = if finish { c.finish } else { START_X };
                let s = if finish { &cx.props.finish_arch } else { &cx.props.start_arch };
                s.draw_clip(f, cam.sx(x), cam.sy(c.height(x)) + 4, cam.y0, cam.y1);
                if finish {
                    // the finish line across the trail
                    let sx = cam.sx(x);
                    let sy = cam.sy(c.height(x));
                    for y in sy..sy + TOP {
                        for dx in 0..3 {
                            let cc = if (y + dx) % 2 == 0 { WHITE } else { OUTLINE };
                            if y >= cam.y0 && y < cam.y1 {
                                f.set(sx - 1 + dx, y, cc);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn draw_obstacle(f: &mut Frame, cx: &Ctx, cam: &Cam, i: usize, t: u32) {
    let c = &cx.race.course;
    let o = c.obs[i];
    let sx = cam.sx(o.x);
    let sy = cam.sy(c.height(o.x));
    let p = cx.props;
    let base = sy + TOP - 2;
    let s = match o.kind {
        ObKind::Root => &p.root[o.var as usize % 2],
        ObKind::Rock => &p.rock[o.var as usize % 2],
        ObKind::Log => &p.log,
        ObKind::Mushroom => &p.mushroom[o.var as usize % 2],
        ObKind::SnowRock => &p.snow_rock,
        ObKind::Branch => &p.branch,
        ObKind::Trunk => &p.trunk,
        ObKind::Marmot => {
            cx.art.marmot_sleep.draw(f, sx, base - 1, false, &cx.art.marmot_pal);
            // Zzz
            let k = (t / 20 + i as u32) % 3;
            for z in 0..=k {
                let zx = sx + 4 + z as i32 * 3;
                let zy = base - 10 - z as i32 * 4;
                if zy > cam.y0 {
                    f.text(zx, zy, "Z", rgb(248, 248, 248));
                }
            }
            return;
        }
    };
    let y = if o.kind == ObKind::Branch || o.kind == ObKind::Trunk { sy + TOP - 1 } else { base };
    s.draw_clip(f, sx, y, cam.y0, cam.y1);
}

fn draw_animal(f: &mut Frame, cx: &Ctx, cam: &Cam, i: usize, t: u32) {
    let race = cx.race;
    let c = &race.course;
    let a = &race.animals[i];
    let sx = cam.sx(a.x);
    let sy = cam.sy(c.height(a.x));
    match a.kind {
        AnimalKind::Cow => {
            let chew = a.state == 0 && (a.t / 40) % 3 != 0;
            let pal = &cx.art.cow_pals[(a.home as i32 / 7 % 2) as usize];
            cx.art.cow[if chew { 1 } else { 0 }].draw(f, sx, sy + 5, a.face < 0.0 && a.state == 1, pal);
            if a.rang > 0 {
                // the bell clanks
                let k = (a.rang / 3) % 2;
                f.text(sx + 12 + k, sy - 26, "*", YELLOW);
                if a.rang > 10 {
                    f.text(sx + 16, sy - 32, "!", WHITE);
                }
            }
        }
        AnimalKind::Marmot => {
            cx.props.hole.draw_clip(f, sx, sy + 6, cam.y0, cam.y1);
            match a.state {
                1 => {
                    let rise = (a.t * 2).min(12);
                    // pop up from the hole, clipped at the hole line
                    let mut tmp_y = sy + 6;
                    if rise < 12 {
                        tmp_y += 12 - rise;
                    }
                    let bm = &cx.art.marmot_up;
                    let ox = sx - bm.w / 2;
                    let oy = tmp_y - bm.h + 1;
                    for yy in 0..bm.h {
                        let py = oy + yy;
                        if py > sy + 6 || py < cam.y0 {
                            continue;
                        }
                        for xx in 0..bm.w {
                            let vv = bm.px[(yy * bm.w + xx) as usize];
                            if vv != 0 {
                                f.set(ox + xx, py, cx.art.marmot_pal[vv as usize]);
                            }
                        }
                    }
                    if a.t > 6 && (t / 6).is_multiple_of(2) {
                        // whistle!
                        f.text(sx + 5, sy - 14, "!", WHITE);
                        f.set(sx + 4, sy - 6, WHITE);
                    }
                }
                2 => {
                    cx.art.marmot_up.draw_squash(f, sx, sy + 6, &cx.art.marmot_pal, 5);
                }
                _ => {}
            }
        }
    }
}

fn draw_deer(f: &mut Frame, cx: &Ctx, cam: &Cam, i: usize, t: u32) {
    let race = cx.race;
    let d = &race.deer[i];
    let c = &race.course;
    let base = cam.sy(c.height(d.x));
    let k = d.t;
    let (x, y, show) = if k < 40 {
        (d.x - 34.0, base - 2, false)
    } else if k < DEER_CROSS.0 {
        // waiting on the bank, ears up
        (d.x - 30.0, base - 2, true)
    } else if k < DEER_CROSS.1 + 30 {
        // leaping across, out towards us
        let u = (k - DEER_CROSS.0) as f32 / (DEER_CROSS.1 + 30 - DEER_CROSS.0) as f32;
        let arc = (u * std::f32::consts::PI).sin() * 26.0;
        (d.x - 30.0 + u * 70.0, base - 2 + (u * 40.0) as i32 - arc as i32, true)
    } else {
        (0.0, 0, false)
    };
    if show {
        let sx = cam.sx(x);
        let leaping = k >= DEER_CROSS.0 && (t / 6).is_multiple_of(2);
        let bm = if leaping { &cx.art.deer2 } else { &cx.art.deer };
        bm.draw(f, sx, y, false, &cx.art.deer_pal);
        if k < DEER_CROSS.0 && (t / 6).is_multiple_of(2) {
            f.text_sh(sx + 8, y - 30, "!", rgb(248, 72, 72), OUTLINE, true);
        }
    }
}

fn draw_fan(f: &mut Frame, cx: &Ctx, cam: &Cam, i: usize, k: u32, t: u32) {
    let c = &cx.race.course;
    let cr = c.crowds[i];
    let x = cr.x - 18.0 + k as f32 * 9.0 + (hash(cr.seed as i32 + k as i32) % 4) as f32;
    let sx = cam.sx(x);
    let sy = cam.sy(c.height(x)) + (k % 2) as i32;
    let seed = cr.seed.wrapping_add(k * 7919);
    let style = (seed % 6) as usize;
    let cheer = ((t / 8 + seed) % 2) as usize;
    let s = cx.art.crowd(style, cheer);
    let pal = runners::crowd_palette(seed % 97);
    let flip = seed.is_multiple_of(3);
    let hop = if (t / 8 + seed).is_multiple_of(4) { 1 } else { 0 };
    blit(f, s, sx, sy - hop, flip, &pal);
    // cowbell in the raised hand
    let pose = CROWD_POSES[cheer];
    let (hx, hy) = hand(pose);
    let (bx, by) = canvas_to_screen(hx, hy, sx, sy - hop, flip);
    let swing = if cheer == 0 { 0 } else { 2 };
    cx.art.cowbell.draw(f, bx + swing, by + 6, false, &cx.art.bell_pal);
    // a sign every now and then
    if seed % 5 == 1 {
        let msg = ["GO!", "RUN!", "HOPP!", "YEAH!", "BAAH?"][(seed / 5 % 5) as usize];
        let w = text_width(msg, false) + 4;
        let (px, py) = (sx - w / 2, sy - 44 - hop);
        f.line(sx, py + 9, sx, sy - 30, rgb(120, 80, 48));
        f.rect(px - 1, py - 1, w + 2, 11, OUTLINE);
        f.rect(px, py, w, 9, WHITE);
        f.text(px + 2, py + 1, msg, rgb(216, 40, 56));
    }
}

fn draw_volunteer(f: &mut Frame, cx: &Ctx, cam: &Cam, i: usize, t: u32, v: &View) {
    let race = cx.race;
    let c = &race.course;
    let a = c.aids[i];
    let sx = cam.sx(a.x + 2.0);
    let sy = cam.sy(c.height(a.x)) + 1;
    let s = cx.art.crowd(2 + i, 2);
    let pal = runners::volunteer_palette(i as u32 * 13 + 5);
    blit(f, s, sx, sy, false, &pal);
    // the item, held out over the trail, unless this view's runner just took it
    let (hx, hy) = hand(TPose::Offer);
    let (bx, by) = canvas_to_screen(hx, hy, sx, sy, false);
    let me = &race.runners[v.runner];
    let near = (me.x - a.x).abs() < 30.0;
    let bob = if (t / 16).is_multiple_of(2) { 0 } else { 1 };
    let idx = match a.item {
        Item::TrailMix => 0,
        Item::Banana => 1,
        Item::Stroopwafel => 2,
        Item::Pickle => 3,
    };
    if !(near && me.grab_t > 0) {
        cx.art.items[idx].draw(f, bx + 3, by + 4 + bob, false, &cx.art.item_pal);
    }
}

pub fn runner_pose(race: &Race, i: usize) -> TPose {
    let r = &race.runners[i];
    let c = &race.course;
    let f4 = (r.anim as usize) % 4;
    let pick = |a: [TPose; 4]| a[f4];
    match r.state {
        RState::Ready => TPose::Ready,
        RState::Finished => {
            if r.v < 0.05 {
                if (r.st_t / 20) % 2 == 0 { TPose::Victory } else { TPose::Stand }
            } else {
                pick([TPose::Walk1, TPose::Walk2, TPose::Walk3, TPose::Walk4])
            }
        }
        RState::Tumble => [TPose::Curl, TPose::Tumble2, TPose::Tumble3, TPose::Tumble4][((r.st_t / 4) % 4) as usize],
        RState::Down => TPose::Faceplant,
        RState::GetUp => TPose::GetUp,
        RState::Stun => TPose::Dazed,
        RState::Glissade => {
            if (r.st_t / 10) % 2 == 0 { TPose::Glissade1 } else { TPose::Glissade2 }
        }
        RState::Air => {
            if r.vz > 0.4 { TPose::Jump } else { TPose::Land }
        }
        RState::Run => {
            let s = c.slope(r.x);
            if r.grab_t > 0 {
                TPose::Grab
            } else if r.five_t > 0 {
                TPose::HighFive
            } else if r.bonk {
                if f4.is_multiple_of(2) { TPose::Bonk1 } else { TPose::Bonk2 }
            } else if r.duck {
                if f4.is_multiple_of(2) { TPose::Duck1 } else { TPose::Duck2 }
            } else if r.hiking(s) {
                if f4.is_multiple_of(2) { TPose::Hike1 } else { TPose::Hike2 }
            } else if r.v < 0.15 {
                TPose::Stand
            } else if r.pace == Pace::Walk || r.v < 0.6 {
                pick([TPose::Walk1, TPose::Walk2, TPose::Walk3, TPose::Walk4])
            } else if s > 0.15 {
                pick([TPose::Up1, TPose::Up2, TPose::Up3, TPose::Up4])
            } else if s < -0.15 {
                pick([TPose::Down1, TPose::Down2, TPose::Down3, TPose::Down4])
            } else if r.pace == Pace::Sprint || r.v > 1.8 {
                pick([TPose::Sprint1, TPose::Sprint2, TPose::Sprint3, TPose::Sprint4])
            } else {
                pick([TPose::Run1, TPose::Run2, TPose::Run3, TPose::Run4])
            }
        }
    }
}

pub fn runner_palette(r: &Runner) -> Palette {
    let mut pal = runners::palette(r.def);
    if r.muddy > 0 {
        pal = runners::muddy(&pal);
    }
    if r.pickle > 0 {
        pal = runners::pickled(&pal);
    }
    if r.flash > 0 {
        pal = runners::flash(&pal);
    }
    pal
}

fn goat_frame(r: &Runner, t: u32) -> usize {
    match r.state {
        RState::Air => 2,
        RState::Ready => 3,
        RState::Finished if r.v < 0.05 => {
            if (t / 12).is_multiple_of(2) { 3 } else { 1 }
        }
        _ => {
            if r.v < 0.2 {
                if r.say.is_some() && (t / 8).is_multiple_of(2) { 3 } else { 1 }
            } else {
                (r.anim as usize / 2) % 2
            }
        }
    }
}

fn draw_runner(f: &mut Frame, cx: &Ctx, cam: &Cam, i: usize, t: u32, v: &View) {
    let race = cx.race;
    let r = &race.runners[i];
    let c = &race.course;
    let sx = cam.sx(r.x);
    let mut sy = cam.sy(r.alt) + r.lane;
    if r.goat() {
        let mut fr = goat_frame(r, t);
        if r.shortcut {
            // Gerald climbs the cliff above the switchback
            sy -= 8 + ((t / 6) % 3) as i32;
            fr = 2;
        }
        cx.art.goat[fr].draw(f, sx, sy + 1, false, &cx.art.goat_pal);
        return;
    }
    let pose = runner_pose(race, i);
    let s = cx.art.runner(r.def, pose);
    let pal = runner_palette(r);
    let wob = if r.wobble > 0.3 && (t / 3).is_multiple_of(2) { 1 } else { 0 };
    // power hiking: poles
    if matches!(pose, TPose::Hike1 | TPose::Hike2) {
        let gy = cam.sy(c.height(r.x)) + r.lane;
        for (hx, hy) in [hand(pose), back_hand(pose)] {
            let (px, py) = canvas_to_screen(hx, hy, sx + wob, sy, false);
            f.line(px, py, px + 4, gy, rgb(64, 64, 80));
            f.line(px + 1, py, px + 5, gy, rgb(200, 200, 216));
        }
    }
    if r.hype > 0 {
        // HYPE sparkles
        for k in 0..4u32 {
            let ph = (t + k * 9) % 20;
            let hx = sx - 8 + ((k * 37 + t / 2) % 16) as i32;
            let hy = sy - 4 - ph as i32;
            let col = [rgb(248, 248, 160), rgb(248, 120, 200), rgb(120, 224, 248)][(k % 3) as usize];
            f.set(hx, hy, col);
        }
    }
    blit(f, s, sx + wob, sy, false, &pal);
    if r.pickle > 0 {
        // pickle face: squinting, cheeks puffed
        let (hx, hy) = skel(pose)[0];
        let (px, py) = canvas_to_screen(hx, hy, sx, sy, false);
        f.set(px + 2, py, OUTLINE);
        f.set(px + 3, py, OUTLINE);
        f.hline(px + 1, px + 3, py + 3, OUTLINE);
        if (t / 10).is_multiple_of(2) {
            f.text(px + 6, py - 9, "!", rgb(120, 232, 96));
        }
    }
    if r.state == RState::Stun || r.state == RState::Down {
        for k in 0..3 {
            let a = t as f32 * 0.15 + k as f32 * 2.1;
            let x = sx + (a.cos() * 6.0) as i32 + if r.state == RState::Down { 6 } else { 0 };
            let y = sy - if r.state == RState::Down { 10 } else { 30 } + (a.sin() * 2.0) as i32;
            f.set(x, y, rgb(248, 248, 96));
            f.set(x + 1, y, rgb(248, 248, 96));
        }
    }
    if r.bonk && (t / 20).is_multiple_of(2) {
        f.text_sh(sx - 2, sy - 36, "?", rgb(200, 200, 232), OUTLINE, false);
    }
    // player tag
    if let Some(p) = r.human()
        && (race.phase != Phase::Done || r.state != RState::Finished) {
            let col = if p == 0 { YELLOW } else { CYAN };
            let label = if p == 0 { "1P" } else { "2P" };
            let bob = ((t / 8) % 2) as i32;
            let y = sy - 44 - bob + if matches!(pose, TPose::Duck1 | TPose::Duck2 | TPose::Glissade1 | TPose::Glissade2 | TPose::Faceplant) { 10 } else { 0 };
            if y > cam.y0 + 1 {
                f.rect(sx - 9, y - 2, 19, 11, OUTLINE);
                f.rect(sx - 8, y - 1, 17, 9, col);
                f.text(sx - 5, y, label, rgb(16, 16, 48));
                for k in 0..3 {
                    f.hline(sx - 2 + k, sx + 2 - k, y + 9 + k, OUTLINE);
                }
                for k in 0..2 {
                    f.hline(sx - 1 + k, sx + 1 - k, y + 9 + k, col);
                }
            }
        }
    let _ = v;
}

fn draw_fx(f: &mut Frame, cx: &Ctx, cam: &Cam) {
    for fx in &cx.race.fx {
        let sx = cam.sx(fx.x);
        let sy = cam.sy(fx.alt) + 6;
        if sy < cam.y0 || sy >= cam.y1 {
            continue;
        }
        let age = fx.max - fx.life;
        match fx.kind {
            FxKind::Dust => {
                let r = 1 + (fx.life * 2 / fx.max);
                f.fill_ellipse(sx, sy, r, r, rgb(216, 184, 136));
            }
            FxKind::Splash => {
                f.set(sx, sy, rgb(200, 232, 248));
                f.set(sx, sy + 1, rgb(96, 168, 232));
            }
            FxKind::Mud => {
                f.rect(sx, sy, 2, 2, rgb(96, 64, 32));
            }
            FxKind::Snow => {
                f.set(sx, sy, WHITE);
                f.set(sx + 1, sy, rgb(216, 232, 248));
            }
            FxKind::Spore => {
                let c = if age % 4 < 2 { rgb(248, 232, 160) } else { rgb(232, 200, 120) };
                f.set(sx, sy, c);
            }
            FxKind::Sweat => {
                f.set(sx, sy, rgb(160, 216, 248));
                f.set(sx, sy + 1, rgb(96, 168, 232));
            }
            FxKind::Star => {
                let c = rgb(248, 248, 96);
                f.set(sx, sy, c);
                f.set(sx - 1, sy, c);
                f.set(sx + 1, sy, c);
                f.set(sx, sy - 1, c);
                f.set(sx, sy + 1, c);
            }
            FxKind::Note => {
                f.text(sx, sy, "*", YELLOW);
            }
        }
    }
}

fn draw_foreground(f: &mut Frame, cx: &Ctx, cam: &Cam, _t: u32) {
    // a few plants close to the camera, moving faster than the trail
    let stage = cx.race.course.stage;
    let px = cam.x * 1.35;
    let step = 140;
    let first = (px as i32).div_euclid(step) - 1;
    for k in first..first + 4 {
        let h = hash(k * 31 + stage as i32);
        if !h.is_multiple_of(3) {
            continue;
        }
        let sx = k * step + (h % 60) as i32 - px as i32;
        let y = cam.y1 + 2;
        let s = match stage {
            0 => &cx.props.fern,
            1 => &cx.props.flowers[(h % 3) as usize],
            _ => &cx.props.boulder[1],
        };
        s.draw_clip(f, sx, y, cam.y0, cam.y1);
        if stage == 0 {
            cx.props.bush[(h % 2) as usize].draw_clip(f, sx + 14, y + 1, cam.y0, cam.y1);
        }
    }
}

const BAYER: [u8; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];
const FOG: u32 = rgb(216, 224, 240);

#[inline]
fn half(a: u32, b: u32) -> u32 {
    (((a >> 1) & 0x7F7F7F) + ((b >> 1) & 0x7F7F7F)) & 0xF8F8F8
}

/// The cloud on the summit: the background fades away behind the trail.
fn draw_fog_back(f: &mut Frame, cx: &Ctx, cam: &Cam, t: u32) {
    let d = cx.race.course.fog(cam.x + 100.0);
    if d <= 0.0 {
        return;
    }
    let drift = (t / 3) as i32;
    for y in cam.y0.max(0)..cam.y1.min(H) {
        let row = (y * W) as usize;
        let wave = (((y + drift / 4) as f32) * 0.09).sin() * 2.0;
        let lvl = ((d * 13.0 + wave) as i32).clamp(0, 16) as u8;
        for x in 0..W {
            let i = row + x as usize;
            let b = BAYER[((y & 3) * 4 + ((x + drift) & 3)) as usize];
            if b < lvl {
                f.px[i] = if b + 6 < lvl { FOG } else { half(f.px[i], FOG) };
            }
        }
    }
}

/// Fog in front: thin near the runner, thick further ahead.
fn draw_fog(f: &mut Frame, cx: &Ctx, cam: &Cam, t: u32) {
    let c = &cx.race.course;
    let d = c.fog(cam.x + 100.0);
    if d <= 0.0 {
        return;
    }
    let drift = (t / 3) as i32;
    for y in cam.y0.max(0)..cam.y1.min(H) {
        let row = (y * W) as usize;
        for x in 130..W {
            let a = d * ((x - 130) as f32 / 126.0) * 0.8;
            let lvl = (a * 16.0) as u8;
            let b = BAYER[((y & 3) * 4 + ((x + drift) & 3)) as usize];
            if b < lvl {
                let i = row + x as usize;
                f.px[i] = half(f.px[i], FOG);
            }
        }
    }
    // snowflakes in the cloud
    for k in 0..40u32 {
        let fx = (hash(k as i32 * 7) % 256) as i32;
        let fy = (hash(k as i32 * 13) % 400) as i32;
        let x = (fx - (cam.x * 0.8) as i32 - (t as i32 * (1 + (k % 3) as i32)) / 4).rem_euclid(W);
        let y = cam.y0 + (fy + t as i32 * (1 + (k % 2) as i32) / 2).rem_euclid(cam.y1 - cam.y0);
        if (k as f32) < d * 40.0 {
            f.set(x, y, WHITE);
        }
    }
}

fn draw_popups(f: &mut Frame, cx: &Ctx, cam: &Cam) {
    for p in &cx.race.popups {
        let a = p.life > 8 || (p.life / 2) % 2 == 0;
        if !a {
            continue;
        }
        let sx = cam.sx(p.x);
        let sy = cam.sy(p.alt);
        let w = text_width(p.text, true);
        let x = (sx - w / 2).clamp(2, W - w - 2);
        if sy > cam.y0 + 2 && sy < cam.y1 - 8 {
            f.text_grad(x, sy, p.text, &grad7(p.color), OUTLINE, true);
        }
    }
}

/// Gerald's heckles, and an arrow when he is off screen.
fn draw_bubbles(f: &mut Frame, cx: &Ctx, cam: &Cam, v: &View) {
    let race = cx.race;
    let Some(g) = race.runners.iter().find(|r| r.goat()) else { return };
    let sx = cam.sx(g.x);
    let sy = cam.sy(g.alt) + g.lane;
    if let Some((text, life)) = g.say
        && (-10..W + 10).contains(&sx) && life > 0 {
            let w = text_width(text, false) + 8;
            let bx = (sx - w / 2 + 8).clamp(2, W - w - 2);
            let by = (sy - 44).max(cam.y0 + 3);
            f.rect(bx - 1, by - 1, w + 2, 13, OUTLINE);
            f.rect(bx, by, w, 11, WHITE);
            let tx = sx.clamp(bx + 3, bx + w - 4);
            for k in 0..4 {
                f.hline(tx - 2 + k, tx + 1 - k / 2, by + 11 + k, if k == 3 { OUTLINE } else { WHITE });
                f.set(tx - 3 + k, by + 11 + k, OUTLINE);
            }
            f.text(bx + 4, by + 2, text, rgb(16, 16, 48));
        }
    let me = &race.runners[v.runner];
    if !me.goat() && race.phase == Phase::Race {
        let ahead = sx > W + 6;
        let behind = sx < -10;
        if ahead || behind {
            let d = ((g.x - me.x).abs()) as i32;
            let s = format!("{}M", d);
            let y = cam.ground_y - 36;
            let x = if ahead { W - 4 - text_width(&s, false) - 12 } else { 4 };
            f.rect(x - 2, y - 2, text_width(&s, false) + 17, 11, OUTLINE);
            f.rect(x - 1, y - 1, text_width(&s, false) + 15, 9, rgb(48, 40, 72));
            // goat head
            let hx = if ahead { x } else { x + text_width(&s, false) + 3 };
            f.rect(hx, y + 2, 5, 4, WHITE);
            f.set(hx + if ahead { 4 } else { 0 }, y + 1, rgb(216, 184, 120));
            f.set(hx + 1, y + 3, OUTLINE);
            let tx = if ahead { x + 7 } else { x };
            f.text(tx, y, &s, WHITE);
            let ax = if ahead { W - 5 } else { 1 };
            let dir = if ahead { 1 } else { -1 };
            for k in 0..3 {
                f.set(ax - dir * k, y + 3 - k, YELLOW);
                f.set(ax - dir * k, y + 3 + k, YELLOW);
            }
        }
    }
}

/// What the game would tell the player right now.
pub fn tip(race: &Race, i: usize) -> Option<&'static str> {
    let r = &race.runners[i];
    if race.phase != Phase::Race || r.finished.is_some() {
        return None;
    }
    let c = &race.course;
    let s = c.slope(r.x);
    let g = c.ground(r.x);
    if r.bonk {
        return Some("BONK! WALK IT OFF...");
    }
    if r.wobble > 0.3 {
        return Some("HOLD DOWN TO BRAKE!");
    }
    for a in &race.animals {
        if a.kind == AnimalKind::Cow && a.x - r.x > -10.0 && a.x - r.x < 110.0 && r.v > 0.95 {
            return Some("COW! SLOW DOWN OR GET MOOED");
        }
    }
    for d in &race.deer {
        if d.active && d.x - r.x > -6.0 && d.x - r.x < 90.0 {
            return Some("DEER! HOLD DOWN, IT JUMPS OVER");
        }
    }
    if s > STEEP && r.hike < 0.08 && r.state == RState::Run {
        return Some("STEEP! TAP Z, C, Z, C TO HIKE");
    }
    if g == Ground::Snow && s < -0.12 && r.state != RState::Glissade {
        return Some("HOLD DOWN TO GLISSADE!");
    }
    for a in &c.aids {
        let d = a.x - r.x;
        if d > 0.0 && d < 90.0 {
            return Some(match a.item {
                Item::Pickle => "PICKLE JUICE! PRESS Z AT THE TABLE",
                _ => "AID STATION! PRESS Z AT THE TABLE",
            });
        }
    }
    for cr in &c.crowds {
        let d = cr.x - r.x;
        if d > 0.0 && d < 60.0 {
            return Some("Z: HIGH FIVE THE FANS FOR HYPE!");
        }
    }
    if r.stamina < 20.0 {
        return Some("LOW ON LEGS! EASE OFF THE PACE");
    }
    None
}

fn draw_tip(f: &mut Frame, cx: &Ctx, cam: &Cam, v: &View, t: u32) {
    let race = cx.race;
    if race.cpu_only {
        return;
    }
    let Some(s) = tip(race, v.runner) else { return };
    if (t / 30) % 4 == 3 {
        return;
    }
    let w = text_width(s, false) + 8;
    let x = 128 - w / 2;
    let y = cam.y1 - 12;
    f.shade_rect(x, y, w, 11, 1, 3);
    f.rect_outline(x - 1, y - 1, w + 2, 13, rgb(248, 176, 32));
    f.text_sh(x + 4, y + 2, s, WHITE, OUTLINE, false);
}

pub fn draw_banner(f: &mut Frame, b: &Banner, y0: i32, y1: i32) {
    let n = b.text.chars().count() as i32;
    let scale = if n <= 3 { 5 } else if n * 18 < 236 { 3 } else { 2 };
    let mid = (y0 + y1) / 2 - 24;
    let (x, y) = if b.pop {
        (128, mid - 7 * scale / 2 - if b.t < 4 { 4 - b.t } else { 0 })
    } else {
        let x = if b.t < 10 { 128 + (10 - b.t) * 30 } else { 128 };
        (x, mid - 7 * scale / 2 + if (10..16).contains(&b.t) { (16 - b.t) / 2 } else { 0 })
    };
    let grad: Vec<u32> = if b.color == WHITE { SUNSET.to_vec() } else { grad6(b.color).to_vec() };
    f.logo(x, y.max(y0 + 2), b.text, scale, &grad, OUTLINE, rgb(8, 8, 24));
}

// ---------------------------------------------------------------- HUD

fn ordinal(n: usize) -> &'static str {
    ["1ST", "2ND", "3RD", "4TH", "5TH"][n.clamp(1, 5) - 1]
}

pub fn time_str(frames: u32) -> String {
    let tenths = frames / 6;
    format!("{}:{:02}.{}", tenths / 600, (tenths / 10) % 60, tenths % 10)
}

/// Head icon of a runner (or Gerald) at (x, y), 12x12.
pub fn head_icon(f: &mut Frame, art: &Art, def: usize, x: i32, y: i32) {
    if def == GERALD {
        let g = &art.goat[3];
        // the head part of the bleating goat
        for yy in 0..10 {
            for xx in 0..10 {
                let v = g.px[(yy * g.w + g.w - 11 + xx) as usize];
                if v != 0 {
                    f.set(x + xx, y + yy + 1, art.goat_pal[v as usize]);
                }
            }
        }
        return;
    }
    let s = art.runner(def, TPose::Stand);
    let pal = runners::palette(def);
    for yy in 0..12 {
        for xx in 0..12 {
            let v = s.get(6 + xx, yy);
            if v != 0 {
                f.set(x + xx, y + yy, pal[v as usize]);
            }
        }
    }
}

fn pace_pips(f: &mut Frame, r: &Runner, x: i32, y: i32, t: u32) {
    for k in 0..4 {
        let on = k <= r.pace.idx() as i32 && !r.bonk;
        let col = if !on {
            rgb(48, 48, 80)
        } else {
            [rgb(120, 200, 248), rgb(120, 232, 96), rgb(248, 216, 48), rgb(248, 96, 64)][k as usize]
        };
        let h = 3 + k;
        f.rect(x + k * 6, y + 6 - h, 5, h + 1, OUTLINE);
        f.rect(x + k * 6 + 1, y + 7 - h, 3, h - 1, col);
    }
    let label = if r.bonk {
        if (t / 10).is_multiple_of(2) { "BONK" } else { "" }
    } else {
        PACE_NAMES[r.pace.idx()]
    };
    f.text_sh(x + 26, y, label, WHITE, OUTLINE, false);
}

fn legs_bar(f: &mut Frame, r: &Runner, x: i32, y: i32, w: i32, t: u32) {
    f.rect(x - 1, y - 1, w + 2, 7, OUTLINE);
    f.rect(x, y, w, 5, rgb(40, 32, 64));
    let fill = (r.stamina / 100.0 * w as f32) as i32;
    let col = if r.stamina > 50.0 {
        rgb(96, 216, 88)
    } else if r.stamina > 22.0 {
        rgb(248, 200, 48)
    } else if (t / 6).is_multiple_of(2) {
        rgb(248, 56, 56)
    } else {
        rgb(176, 32, 48)
    };
    f.rect(x, y, fill, 5, col);
    f.hline(x, x + fill - 1, y, mix(col, 0xFFFFFF, 0.45));
    for k in (8..w).step_by(8) {
        f.set(x + k, y + 4, rgb(24, 24, 48));
    }
    if r.hype > 0 && (t / 4).is_multiple_of(2) {
        f.rect_outline(x - 1, y - 1, w + 2, 7, rgb(248, 120, 200));
    }
}

/// Small pole-power meter, shown on steep climbs.
fn pole_meter(f: &mut Frame, r: &Runner, x: i32, y: i32) {
    f.text_sh(x, y, "POLES", rgb(200, 200, 232), OUTLINE, false);
    let w = 30;
    f.rect(x + 31, y, w + 2, 7, OUTLINE);
    let fill = (r.hike.min(1.0) * w as f32) as i32;
    f.rect(x + 32, y + 1, fill, 5, rgb(120, 200, 248));
}

/// Course profile with a dot per runner.
fn minimap(f: &mut Frame, cx: &Ctx, x: i32, y: i32, w: i32, h: i32, t: u32) {
    for yy in 0..h {
        for xx in 0..w {
            let c = cx.profile[(yy * w + xx) as usize];
            if c != KEY {
                f.set(x + xx, y + yy, c);
            }
        }
    }
    // finish flag
    f.line(x + w - 1, y - 3, x + w - 1, y + h - 1, WHITE);
    f.rect(x + w - 1, y - 3, 3, 2, rgb(248, 56, 72));
    let race = cx.race;
    let c = &race.course;
    let (lo, hi) = (c.min_h, c.max_h.max(c.min_h + 1.0));
    let mut order: Vec<usize> = (0..race.runners.len()).collect();
    order.sort_by_key(|&i| race.runners[i].human().is_some());
    for i in order {
        let r = &race.runners[i];
        let px = x + (c.progress(r.x) * (w - 1) as f32) as i32;
        let py = y + h - 1 - ((c.height(r.x) - lo) / (hi - lo) * (h - 2) as f32) as i32;
        let col = match r.human() {
            Some(0) => YELLOW,
            Some(_) => CYAN,
            None if r.def == GERALD => WHITE,
            None => RUNNERS[r.def].shirt.0,
        };
        let blink = r.human().is_some() && (t / 8).is_multiple_of(2);
        f.rect(px - 1, py - 2, 3, 3, OUTLINE);
        f.set(px, py - 1, if blink { WHITE } else { col });
        if r.def == GERALD {
            f.set(px - 1, py - 3, rgb(216, 184, 120));
            f.set(px + 1, py - 3, rgb(216, 184, 120));
        }
    }
}

pub fn draw_hud(f: &mut Frame, cx: &Ctx, runner: usize, t: u32) {
    let race = cx.race;
    let r = &race.runners[runner];
    f.rect(0, 0, W, 31, HUD_BG);
    f.hline(0, W - 1, 31, HUD_LINE);
    f.hline(0, W - 1, 32, HUD_LINE2);
    // left: who, pace, legs
    head_icon(f, cx.art, r.def, 3, 2);
    let name = runners::name(r.def);
    f.text_grad(18, 3, name, &grad7(if r.def < 4 { RUNNERS[r.def].shirt.0 } else { WHITE }), OUTLINE, true);
    pace_pips(f, r, 3, 15, t);
    let steep = race.course.slope(r.x) > STEEP && race.phase == Phase::Race;
    if steep {
        pole_meter(f, r, 3, 23);
    } else {
        f.text_sh(3, 23, "LEGS", rgb(200, 200, 232), OUTLINE, false);
        legs_bar(f, r, 30, 24, 46, t);
    }
    // centre: time and the course
    f.text_center(128, 1, "TIME", rgb(160, 168, 216), OUTLINE, false);
    let tm = match r.finished {
        Some(ft) => ft,
        None => race.t,
    };
    let s = time_str(tm);
    f.text_grad_center(128, 9, &s, &grad7(rgb(248, 224, 72)), OUTLINE, true);
    minimap(f, cx, 84, 19, 88, 10, t);
    // right: place
    let place = r.place.max(1);
    let col = if place == 1 { rgb(248, 216, 48) } else if place == 2 { rgb(200, 216, 232) } else { rgb(232, 136, 72) };
    f.logo(214, 1, ordinal(place), 2, &grad6(col), OUTLINE, rgb(8, 8, 24));
    let of = format!("OF {}", race.runners.len());
    f.text_sh(200, 22, &of, rgb(160, 168, 216), OUTLINE, false);
    if steep {
        legs_bar(f, r, 226, 23, 26, t);
    }
}

/// One line of HUD at the top of a split-screen band.
pub fn draw_mini_hud(f: &mut Frame, cx: &Ctx, runner: usize, y: i32, t: u32) {
    let race = cx.race;
    let r = &race.runners[runner];
    f.rect(0, y, W, 11, HUD_BG);
    f.hline(0, W - 1, y + 11, HUD_LINE);
    let p = r.human().unwrap_or(0);
    let col = if p == 0 { YELLOW } else { CYAN };
    f.rect(2, y + 1, 15, 9, col);
    f.text(4, y + 2, if p == 0 { "1P" } else { "2P" }, rgb(16, 16, 48));
    f.text_sh(20, y + 2, runners::name(r.def), WHITE, OUTLINE, false);
    pace_pips(f, r, 58, y + 2, t);
    if race.course.slope(r.x) > STEEP && race.phase == Phase::Race {
        let w = 40;
        f.rect(128, y + 3, w + 2, 6, OUTLINE);
        f.rect(129, y + 4, (r.hike.min(1.0) * w as f32) as i32, 4, rgb(120, 200, 248));
    } else {
        legs_bar(f, r, 128, y + 3, 40, t);
    }
    let s = time_str(r.finished.unwrap_or(race.t));
    f.text_sh(176, y + 2, &s, rgb(248, 224, 72), OUTLINE, false);
    let pl = ordinal(r.place.max(1));
    f.text_sh(232, y + 2, pl, WHITE, OUTLINE, true);
}
