//! Draws a view of a race: camera, sky, layers, terrain, gliders, wildlife,
//! effects, HUD and banners.

use super::art::*;
use super::course::*;
use super::flight::*;
use super::pilots::{palette, PILOTS};
use super::scenery::*;
use crate::engine::gfx::*;
use crate::engine::ui::{grad6, grad7, OUTLINE, SUNSET};

/// How to draw one pilot this frame.
#[derive(Clone, Copy, Debug)]
pub struct Look {
    pub pose: PPose,
    /// wing pitch in degrees (nose up positive)
    pub pitch: f32,
    /// turn frame 0..=6 (0 facing `flip`'s direction, 6 the other way)
    pub turn: usize,
    pub flip: bool,
    /// no wing (paddling, crashed, cheering)
    pub wing: bool,
    /// wing lying on the ground next to the pilot (after a crash)
    pub wreck: bool,
}

impl Look {
    pub fn flying(pitch: f32, flip: bool) -> Self {
        Look { pose: PPose::Prone, pitch, turn: 0, flip, wing: true, wreck: false }
    }
}

pub fn look_for(p: &Pilot, c: &super::course::Course, t: u32) -> Look {
    let flip = p.face < 0.0;
    let run = |a: f32| [PPose::Run1, PPose::Run2, PPose::Run3, PPose::Run4][(a as usize) % 4];
    let mut l = Look::flying(p.pitch * 22.0, flip);
    match p.state {
        State::Wait => {
            l.pose = if p.anim.fract() > 0.01 && (p.anim as usize) % 2 == 1 { PPose::Walk1 } else { PPose::Stand };
            l.pitch = 14.0;
        }
        State::Run | State::Hop(_) => {
            l.pose = run(p.anim);
            l.pitch = 10.0 + p.run * 6.0;
        }
        State::Wobble(k) => {
            l.pose = if (k / 6) % 2 == 0 { PPose::Upright } else { PPose::Prone };
            l.pitch = ((k as f32) * 0.5).sin() * 30.0;
        }
        State::Fly => {
            let agl = p.agl(c);
            l.pose = if p.tuck {
                PPose::Tuck
            } else if agl < 22.0 {
                PPose::Upright
            } else if (t / 20 + p.def as u32) % 2 == 0 {
                PPose::Prone
            } else {
                PPose::Prone2
            };
            if p.turn > 0 {
                let k = p.turn_phase();
                l.turn = ((k * 6.0).round() as usize).min(6);
                // the turn starts facing the old direction
                l.flip = p.face < 0.0;
            }
            if p.dazed > 0 {
                l.pitch = ((p.dazed as f32) * 0.4).sin() * 30.0;
            }
        }
        State::Stall(_) => {
            l.pitch = -36.0;
        }
        State::Suck(k) => {
            l.pitch = ((k as f32) * 0.7).sin() * 36.0;
        }
        State::Flare(_) => {
            l.pose = PPose::Flare;
            l.pitch = 30.0;
        }
        State::Landed(kind, k) => match kind {
            Landing::Crash => {
                l.pose = if k < 40 { PPose::Down } else { PPose::Sit };
                l.wing = false;
                l.wreck = true;
            }
            Landing::Splash => {
                l.pose = if (k / 8) % 2 == 0 { PPose::Paddle1 } else { PPose::Paddle2 };
                l.wing = false;
                l.wreck = true;
            }
            Landing::Belly => {
                l.pose = if k < 30 { PPose::Prone } else { PPose::Stand };
                l.pitch = if k < 30 { -6.0 } else { 14.0 };
            }
            _ => {
                l.pose = if p.slide > 0.1 { run(p.anim) } else { PPose::Stand };
                l.pitch = 18.0;
            }
        },
        State::Paddle(_) => {
            l.pose = if (p.anim as usize) % 2 == 0 { PPose::Paddle1 } else { PPose::Paddle2 };
            l.wing = false;
            l.wreck = true;
        }
        State::Walk(_) => {
            l.pose = if (p.anim as usize) % 2 == 0 { PPose::Walk1 } else { PPose::Walk2 };
            l.pitch = 20.0;
        }
        State::Done => {
            let good = matches!(p.landing, Some(Landing::Perfect | Landing::Soft)) && c.in_field(p.land_x);
            if good && (t / 30 + p.def as u32) % 3 == 0 {
                l.pose = PPose::Cheer;
                l.wing = false;
                l.wreck = true;
            } else if p.landing == Some(Landing::Crash) {
                l.pose = PPose::Sit;
                l.wing = false;
                l.wreck = true;
            } else {
                l.pose = PPose::Stand;
                l.pitch = 16.0;
            }
        }
    }
    l
}

/// Draw a pilot (with wing) whose lowest point is at (sx, sy).
pub fn draw_pilot(f: &mut Frame, art: &Art, def: usize, sx: i32, sy: i32, l: &Look) {
    let pal = palette(def);
    let gpal = glider_palette(def);
    let dir = if l.flip { -1 } else { 1 };
    if l.wreck {
        // the glider lies crumpled next to the pilot
        let w = art.wing_pitch(def, -36.0);
        blit(f, w, sx - dir * 14, sy - 8, !l.flip, &gpal);
    }
    if !l.wing {
        blit(f, art.pilot(def, l.pose), sx, sy, l.flip, &pal);
        return;
    }
    let (hx, hy) = hang(l.pose);
    let (ax, ay) = (sx + hx * dir, sy + hy);
    let hnd = hand(l.pose);
    let mid_turn = (2..=4).contains(&l.turn);
    // which way the wing is drawn and which frame
    let (wing, wflip) = match l.turn {
        0 => (art.wing_pitch(def, l.pitch), l.flip),
        1..=3 => (&art.wings[def].yaw[l.turn], l.flip),
        _ => (&art.wings[def].yaw[6 - l.turn], !l.flip),
    };
    let yaw = match l.turn {
        0 => 0.0,
        k => -30.0 * k as f32,
    };
    let fr = if mid_turn { aframe(yaw, (0, 16)) } else { aframe(yaw, (hnd.0, hnd.1 - hy)) };
    let pt = |q: (i32, i32)| (ax + q.0 * dir, ay + q.1);
    let tube = rgb(184, 192, 208);
    let tube_d = rgb(104, 112, 136);
    // far upright behind everything
    let (b1, b2) = (pt(fr[1]), pt(fr[2]));
    f.line(ax, ay, b2.0, b2.1, tube_d);
    blit(f, wing, ax, ay, wflip, &gpal);
    if mid_turn {
        let fs = &art.fronts[def];
        blit(f, fs, ax, ay + 15, false, &pal);
        f.line(b1.0, b1.1, b2.0, b2.1, tube);
        f.line(ax, ay, b1.0, b1.1, tube);
    } else {
        // hang strap
        let strap = if l.pose.prone() { (ax, ay + 10) } else { (ax, ay + 4) };
        f.line(ax, ay + 1, strap.0, strap.1, rgb(48, 48, 64));
        let s = art.pilot(def, l.pose);
        let flip_p = if l.turn >= 5 { !l.flip } else { l.flip };
        blit(f, s, sx, sy, flip_p, &pal);
        f.line(b1.0, b1.1, b2.0, b2.1, tube);
        f.line(ax, ay, b1.0, b1.1, tube);
    }
}

/// Every sprite on a few sheets (to judge the art at 1x and the animation).
pub fn sprite_sheet(art: &Art) -> Vec<Frame> {
    let mut out = Vec::new();
    for def in 0..PILOTS.len() {
        let mut f = Frame::new();
        f.clear(rgb(112, 168, 224));
        f.rect(0, 150, W, H - 150, rgb(96, 168, 80));
        f.text_sh(4, 4, PILOTS[def].name, rgb(248, 248, 248), OUTLINE, false);
        // pitch frames
        for (k, p) in super::art::PITCHES.iter().enumerate() {
            let l = Look::flying(*p, false);
            draw_pilot(&mut f, art, def, 20 + k as i32 * 34, 60, &l);
        }
        // a turn, frame by frame
        for k in 0..=6 {
            let mut l = Look::flying(0.0, false);
            l.turn = k;
            draw_pilot(&mut f, art, def, 20 + k as i32 * 34, 108, &l);
        }
        // every pose with its wing
        for (k, pose) in ALL_PPOSES.iter().enumerate() {
            let mut l = Look::flying(if pose.prone() { 0.0 } else { 16.0 }, false);
            l.pose = *pose;
            l.wing = pose.carries();
            let x = 14 + (k as i32 % 9) * 27;
            let y = if k < 9 { 178 } else { 216 };
            draw_pilot(&mut f, art, def, x, y, &l);
        }
        out.push(f);
    }
    let mut f = Frame::new();
    f.clear(rgb(112, 168, 224));
    f.rect(0, 120, W, H - 120, rgb(96, 168, 80));
    let c = &art.critters;
    for (k, p) in c.sheep.iter().enumerate() {
        draw_pic(&mut f, p, 16 + k as i32 * 22, 140, false);
    }
    draw_pic_vflip(&mut f, &c.sheep[0], 130, 140, false);
    for (k, p) in c.cow.iter().enumerate() {
        draw_pic(&mut f, p, 160 + k as i32 * 26, 140, false);
    }
    for (k, p) in c.stork.iter().enumerate() {
        draw_pic(&mut f, p, 20 + k as i32 * 28, 20, false);
    }
    for (k, p) in c.stork_bundle.iter().enumerate() {
        draw_pic(&mut f, p, 110 + k as i32 * 28, 20, false);
    }
    for (k, p) in c.vulture.iter().enumerate() {
        draw_pic(&mut f, p, 20 + k as i32 * 28, 60, false);
    }
    for (k, p) in c.goose.iter().enumerate() {
        draw_pic(&mut f, p, 120 + k as i32 * 20, 60, false);
    }
    for (k, p) in c.gull.iter().enumerate() {
        draw_pic(&mut f, p, 180 + k as i32 * 14, 60, false);
    }
    out.push(f);
    out
}


// ------------------------------------------------------------------ views

/// A camera on the world: centre (x, y) shows at screen row `cy`; the view
/// covers rows `top..vh`.
#[derive(Clone, Copy, Debug)]
pub struct Cam {
    pub x: f32,
    pub y: f32,
    pub cy: i32,
    pub vh: i32,
}

impl Cam {
    #[inline]
    pub fn sx(&self, x: f32) -> i32 {
        (x - self.x).round() as i32 + 128
    }
    #[inline]
    pub fn sy(&self, y: f32) -> i32 {
        self.cy - (y - self.y).round() as i32
    }
    pub fn pos(&self) -> (f32, f32) {
        (self.x, self.y)
    }
    fn visible(&self, x: f32, margin: f32) -> bool {
        (x - self.x).abs() < 128.0 + margin
    }
}

/// Where the camera wants to be for a pilot: centred when high, and lower
/// on screen when close to the ground so the ground stays in view.
pub fn cam_target(p: &Pilot, c: &Course, vh: i32, top: i32) -> (f32, f32) {
    let lead = if p.airborne() { p.face * 36.0 } else { p.face * 20.0 };
    let x = (p.x + lead).clamp(128.0, LEN as f32 - 128.0);
    let cy = top + (vh - top) / 2;
    let g = c.floor_at(p.x).min(c.floor_at(p.x + p.face * 40.0));
    let agl = (p.y - g).max(0.0);
    let hi = (top + 48) as f32;
    let raw = (vh - 26) as f32 - agl;
    // close to the ground: pilot higher on screen so the ground shows;
    // too high for that: drift back to the centre
    let r = if raw >= hi { raw.min((vh - 52) as f32) } else { hi + (cy as f32 - hi) * ((hi - raw) / 70.0).min(1.0) };
    (x, p.y + (r - cy as f32))
}

pub fn draw_view(f: &mut Frame, race: &Race, sc: &Scenery, art: &Art, cam: &Cam, viewer: Option<usize>, t: u32) {
    let c = &race.course;
    let pos = cam.pos();
    sc.draw_sky(f, pos, cam.cy, cam.vh, t);
    sc.draw_back(f, pos, cam.cy, cam.vh, t);
    // thermal clouds mark every thermal
    for th in &c.thermals {
        if !cam.visible(th.x, 90.0) {
            continue;
        }
        let k = if th.w < 39.0 { 0 } else if th.w < 44.0 { 1 } else { 2 };
        draw_pic_clip(f, &sc.cumulus[k], cam.sx(th.x), cam.sy(th.top + 8.0), cam.vh, false);
    }
    if let (Some(st), Some(pic)) = (c.storm, &sc.storm) {
        if cam.visible(st.x, 140.0) {
            let shake = if race.flash > 0 { (t % 3) as i32 - 1 } else { 0 };
            draw_pic_clip(f, pic, cam.sx(st.x) + shake, cam.sy(st.base), cam.vh, false);
            if race.flash > 0 {
                lightning(f, cam.sx(st.x) + ((t / 7 % 5) as i32 - 2) * 18, cam.sy(st.base), cam.sy(c.ground_at(st.x)), t, cam.vh);
            }
        }
    }
    draw_family(f, race, sc, cam, t);
    draw_bundle(f, race, art, cam, t);
    sc.draw_ground(f, c, pos, cam.cy, cam.vh, t);
    draw_ramp(f, c, cam);
    let cheer = race.pilots.iter().any(|p| matches!(p.state, State::Landed(k, n) if n < 70 && matches!(k, Landing::Perfect | Landing::Soft)));
    draw_props(f, c, cheer, sc, art, cam, t);
    draw_target(f, c, cam, t);
    for s in &race.sheep {
        if !cam.visible(s.x, 20.0) {
            continue;
        }
        let g = c.ground_at(s.x);
        let (x, y) = (cam.sx(s.x), cam.sy(g + s.h));
        let cr = &art.critters;
        let flip = s.face < 0.0;
        match s.mode {
            SheepMode::Fly => {
                if (s.spin as i32) % 2 == 0 {
                    draw_pic_vflip(f, &cr.sheep[3], x, y - 9, flip);
                } else {
                    draw_pic(f, &cr.sheep[3], x, y, flip);
                }
            }
            SheepMode::Dizzy => {
                draw_pic(f, &cr.sheep[4], x, y, flip);
                stars(f, x + if flip { -6 } else { 6 }, y - 9, t);
            }
            SheepMode::Run => draw_pic(f, &cr.sheep[if (t / 4) % 2 == 0 { 3 } else { 1 }], x, y, flip),
            SheepMode::Walk => draw_pic(f, &cr.sheep[if (t / 10) % 2 == 0 { 0 } else { 1 }], x, y, flip),
            SheepMode::Graze => draw_pic(f, &cr.sheep[if (s.t / 40) % 3 == 0 { 0 } else { 2 }], x, y, flip),
        }
    }
    draw_markers(f, race, art, cam, t);
    draw_geese(f, race, art, cam, t);
    draw_ufo(f, race, cam, t);
    for (k, g) in c.gates.iter().enumerate() {
        if cam.visible(g.x, 40.0) {
            let made = viewer.map(|v| race.pilots[v].gates & (1 << k) != 0).unwrap_or(false);
            draw_gate(f, sc, g, k, made, cam, t, true);
        }
    }
    draw_streaks(f, c, cam, t);
    // shadows on the ground
    for p in &race.pilots {
        if !p.airborne() || !cam.visible(p.x, 30.0) {
            continue;
        }
        let agl = p.agl(c);
        if agl < 140.0 {
            let r = (9.0 - agl / 18.0).max(2.0) as i32;
            let gy = cam.sy(c.floor_at(p.x));
            if gy < cam.vh {
                f.shade_ellipse(cam.sx(p.x), gy, r, 1, 5, 8);
            }
        }
    }
    // pilots: CPUs, then the viewer on top
    let mut order: Vec<usize> = (0..race.pilots.len()).collect();
    order.sort_by_key(|&i| (Some(i) == viewer, race.pilots[i].view.is_some()));
    for &i in &order {
        let p = &race.pilots[i];
        if !cam.visible(p.x, 40.0) || matches!(p.state, State::Suck(_)) {
            continue;
        }
        let (x, y) = (cam.sx(p.x), cam.sy(p.y));
        if y < -40 || y > cam.vh + 40 {
            continue;
        }
        let l = look_for(p, c, t);
        let wet = matches!(p.state, State::Paddle(_) | State::Landed(Landing::Splash, _));
        draw_pilot(f, art, p.def, x, y + if wet { 7 } else { 0 }, &l);
        if wet {
            // water line over the legs, and ripples
            let wy = y;
            f.hline(x - 9, x + 9, wy, rgb(200, 232, 248));
            for k in 0..3 {
                let r = 4 + ((t as i32 / 4 + k * 5) % 14);
                f.set(x - r, wy, rgb(232, 248, 248));
                f.set(x + r, wy, rgb(232, 248, 248));
            }
        }
        if p.dazed > 0 || matches!(p.state, State::Landed(Landing::Crash, _)) || (p.state == State::Done && p.landing == Some(Landing::Crash)) {
            stars(f, x, y - 26, t);
        }
        if let Some((txt, _)) = p.say {
            bubble(f, x, y - 44, txt);
        }
        // tag over human pilots
        if let Ctrl::Human(k) = p.ctrl {
            if p.airborne() || matches!(p.state, State::Wait | State::Run | State::Hop(_)) {
                // keep the tag under the HUD band
                let top = if cam.vh < 200 { 14 } else { 36 };
                tag(f, x, (y - if p.airborne() { 52 } else { 60 }).max(top), k, t);
            }
        }
    }
    for (k, g) in c.gates.iter().enumerate() {
        if cam.visible(g.x, 40.0) {
            let made = viewer.map(|v| race.pilots[v].gates & (1 << k) != 0).unwrap_or(false);
            draw_gate(f, sc, g, k, made, cam, t, false);
        }
    }
    // inside the storm: only a shaking cloud
    if let Some(st) = c.storm {
        for p in &race.pilots {
            if let State::Suck(k) = p.state {
                if cam.visible(st.x, 60.0) && (k / 3) % 2 == 0 {
                    stars(f, cam.sx(st.x) + ((k * 7) % 40) - 20, cam.sy(p.y), t);
                }
            }
        }
    }
    draw_fx(f, race, cam);
    for p in &race.popups {
        if !cam.visible(p.x, 40.0) || (p.life < 12 && (p.life / 2) % 2 == 0) {
            continue;
        }
        let w = text_width(&p.text, true);
        let y = cam.sy(p.y);
        if y > 0 && y < cam.vh - 8 {
            f.text_grad(cam.sx(p.x) - w / 2, y, &p.text, &grad7(p.color), OUTLINE, true);
        }
    }
    if race.flash > 0 {
        f.shade_rect(0, 0, W, cam.vh, 6, 5);
    }
}

fn stars(f: &mut Frame, x: i32, y: i32, t: u32) {
    for k in 0..3 {
        let a = t as f32 * 0.15 + k as f32 * 2.1;
        let sx = x + (a.cos() * 7.0) as i32;
        let sy = y + (a.sin() * 2.0) as i32;
        f.set(sx, sy, rgb(248, 248, 96));
        f.set(sx + 1, sy, rgb(248, 248, 96));
        f.set(sx, sy - 1, rgb(248, 200, 48));
    }
}

fn tag(f: &mut Frame, x: i32, y: i32, k: usize, t: u32) {
    let bob = ((t / 8) % 2) as i32;
    let y = y - bob;
    let col = if k == 0 { rgb(248, 232, 64) } else { rgb(96, 224, 248) };
    f.rect(x - 9, y - 2, 19, 11, OUTLINE);
    f.rect(x - 8, y - 1, 17, 9, col);
    f.text(x - 5, y, if k == 0 { "1P" } else { "2P" }, rgb(16, 16, 48));
    for k in 0..4 {
        f.hline(x - 3 + k, x + 3 - k, y + 9 + k, OUTLINE);
    }
    for k in 0..3 {
        f.hline(x - 2 + k, x + 2 - k, y + 9 + k, col);
    }
}

/// A speech bubble whose tail points down at (x, y).
pub fn bubble(f: &mut Frame, x: i32, y: i32, txt: &str) {
    let w = text_width(txt, false) + 8;
    let bx = (x - w / 2).clamp(2, W - w - 2);
    f.rect(bx - 1, y - 12, w + 2, 13, OUTLINE);
    f.rect(bx, y - 11, w, 11, rgb(248, 248, 248));
    f.text(bx + 4, y - 9, txt, rgb(24, 24, 48));
    for k in 0..4 {
        f.hline(x - 3 + k, x + 3 - k, y + 1 + k, OUTLINE);
    }
    for k in 0..3 {
        f.hline(x - 2 + k, x + 2 - k, y + k, rgb(248, 248, 248));
    }
}

fn lightning(f: &mut Frame, x: i32, y0: i32, y1: i32, t: u32, vh: i32) {
    let mut x = x;
    let mut y = y0;
    let mut k = t;
    while y < y1.min(vh) {
        k = k.wrapping_mul(1103515245).wrapping_add(12345);
        let nx = x + ((k >> 16) % 13) as i32 - 6;
        let ny = y + 8 + ((k >> 8) % 8) as i32;
        f.thick_line(x, y, nx, ny, 3, rgb(184, 168, 248));
        f.line(x, y, nx, ny, rgb(248, 248, 255));
        x = nx;
        y = ny;
    }
}

fn draw_family(f: &mut Frame, race: &Race, sc: &Scenery, cam: &Cam, t: u32) {
    let c = &race.course;
    let span = LEN as f32 - 600.0;
    let x = 600.0 + (700.0 + t as f32 * 0.09).rem_euclid(span);
    let base = c.launch.y0 + 50.0 + (t as f32 * 0.01).sin() * 10.0;
    if !cam.visible(x, 40.0) {
        return;
    }
    let (sx, sy) = (cam.sx(x), cam.sy(base));
    draw_pic_clip(f, &sc.family, sx, sy - 12, cam.vh, false);
    // ropes and basket
    for dx in [-4, 4] {
        f.line(sx + dx, sy - 12, sx + dx / 2 * 3, sy - 4, rgb(96, 64, 40));
    }
    f.rect(sx - 7, sy - 4, 14, 7, OUTLINE);
    f.rect(sx - 6, sy - 3, 12, 5, rgb(176, 120, 64));
    f.hline(sx - 6, sx + 5, sy - 1, rgb(136, 88, 48));
    // mum, dad and the kid wave when someone flies by
    let near = race.pilots.iter().any(|p| p.view.is_some() && p.airborne() && (p.x - x).abs() < 70.0 && (p.y - base).abs() < 60.0);
    let heads = [(-4, rgb(248, 200, 160), rgb(136, 80, 32)), (0, rgb(224, 160, 104), rgb(248, 216, 72)), (4, rgb(248, 200, 160), rgb(200, 72, 32))];
    for (k, (dx, skin, hair)) in heads.iter().enumerate() {
        let hy = sy - 6 + if k == 1 { 1 } else { 0 };
        f.rect(sx + dx - 1, hy - 1, 3, 3, *skin);
        f.hline(sx + dx - 1, sx + dx + 1, hy - 2, *hair);
        let wave = near && ((t / 6) as usize + k) % 2 == 0;
        if near || k == 2 {
            let ay = if wave { hy - 4 } else { hy - 2 };
            f.line(sx + dx + 1, hy, sx + dx + 2, ay, *skin);
        }
    }
    if near && (t / 40) % 2 == 0 {
        bubble(f, sx + 10, sy - 50, "HI!");
    }
}

fn draw_bundle(f: &mut Frame, race: &Race, art: &Art, cam: &Cam, t: u32) {
    let c = &race.course;
    let span = LEN as f32 + 900.0;
    let x = (t as f32 * 0.85 + 1200.0).rem_euclid(span) - 300.0;
    let y = c.launch.y0 + 110.0 + (t as f32 * 0.03).sin() * 6.0;
    if cam.visible(x, 20.0) {
        let fr = [0, 1, 0, 2][((t / 9) % 4) as usize];
        draw_pic(f, &art.critters.stork_bundle[fr], cam.sx(x), cam.sy(y), false);
    }
}

fn draw_ramp(f: &mut Frame, c: &Course, cam: &Cam) {
    let l = c.launch;
    if !cam.visible((l.x0 + l.x1) / 2.0, 60.0) {
        return;
    }
    let (ax, ay) = (cam.sx(l.x0 - 6.0), cam.sy(l.y0));
    let (bx, by) = (cam.sx(l.x1), cam.sy(l.y1));
    let wood = rgb(176, 120, 72);
    let wood_d = rgb(120, 80, 48);
    // legs down to the hill
    let mut x = l.x0 + 6.0;
    while x < l.x1 {
        let top = cam.sy(l.y_at(x)) + 3;
        let bot = cam.sy(c.ground_at(x));
        if bot > top {
            f.rect(cam.sx(x) - 1, top, 3, bot - top + 1, OUTLINE);
            f.line(cam.sx(x), top, cam.sx(x), bot, wood_d);
        }
        x += 16.0;
    }
    f.thick_line(ax, ay + 1, bx, by + 1, 5, OUTLINE);
    f.thick_line(ax, ay + 1, bx, by + 1, 3, wood);
    f.line(ax, ay, bx, by, rgb(224, 176, 112));
    // the launch zone is painted yellow, the edge red and white
    let (z0, _) = l.zone();
    let (zx, zy) = (cam.sx(z0), cam.sy(l.y_at(z0)));
    f.line(zx, zy, bx, by, rgb(248, 216, 48));
    for k in 0..3 {
        let cx = zx + 4 + k * 7;
        let cy2 = cam.sy(l.y_at(z0 + 4.0 + k as f32 * 7.0)) + 2;
        f.set(cx, cy2, rgb(248, 216, 48));
        f.set(cx + 1, cy2 + 1, rgb(248, 216, 48));
        f.set(cx, cy2 + 2, rgb(248, 216, 48));
    }
    f.rect(bx - 1, by - 1, 3, 5, rgb(248, 248, 248));
    f.set(bx, by, rgb(232, 48, 48));
    f.set(bx, by + 2, rgb(232, 48, 48));
}

pub fn draw_props(f: &mut Frame, c: &Course, cheer: bool, sc: &Scenery, art: &Art, cam: &Cam, t: u32) {
    for (i, pr) in c.props.iter().enumerate() {
        if !cam.visible(pr.x, 60.0) {
            continue;
        }
        let (x, y) = (cam.sx(pr.x), cam.sy(pr.y));
        if let Some(pic) = &sc.props[i] {
            draw_pic(f, pic, x, y + 1, false);
        }
        match pr.kind {
            PropKind::Windmill => {
                let (ax, ay) = (x, y - 37);
                let a0 = t as f32 * 0.025 + pr.var as f32;
                for k in 0..4 {
                    let a = a0 + k as f32 * std::f32::consts::FRAC_PI_2;
                    let (ex, ey) = (ax + (a.cos() * 19.0) as i32, ay + (a.sin() * 19.0) as i32);
                    f.thick_line(ax, ay, ex, ey, 3, OUTLINE);
                    f.line(ax, ay, ex, ey, rgb(120, 80, 48));
                    let (px, py) = (-(a.sin()), a.cos());
                    for s in 5..18 {
                        let (sx, sy) = (ax as f32 + a.cos() * s as f32, ay as f32 + a.sin() * s as f32);
                        let c2 = if s % 3 == 0 { rgb(176, 136, 96) } else { rgb(248, 240, 224) };
                        for w in 1..4 {
                            f.set((sx + px * w as f32) as i32, (sy + py * w as f32) as i32, c2);
                        }
                    }
                }
                f.fill_ellipse(ax, ay, 1, 1, OUTLINE);
            }
            PropKind::Lighthouse => {
                if (t / 8) % 16 < 3 {
                    let (lx, ly) = (x, y - 56);
                    let dir = if (t / 128) % 2 == 0 { 1 } else { -1 };
                    f.fill_poly(&[(lx, ly), (lx + dir * 40, ly - 7), (lx + dir * 40, ly + 7)], rgb(248, 248, 200));
                }
            }
            PropKind::Windsock => windsock(f, x, y, c.wind, t),
            PropKind::Cow => {
                let fr = if (t / 50 + pr.var) % 3 == 0 { 1 } else { 0 };
                draw_pic(f, &art.critters.cow[fr], x, y + 1, pr.var % 2 == 1);
            }
            PropKind::KiteSurfer => kite_surfer(f, x, cam.sy(6.0), pr.var, t),
            PropKind::Crowd => crowd(f, cheer, x, y, t),
            PropKind::Boat => {}
            _ => {}
        }
    }
}

fn windsock(f: &mut Frame, x: i32, y: i32, wind: f32, t: u32) {
    f.rect(x - 1, y - 20, 3, 21, OUTLINE);
    f.line(x, y - 20, x, y, rgb(200, 200, 216));
    let lift = ((t as f32 * 0.2).sin() * 1.5) as i32;
    let len = 6 + (wind * 40.0) as i32;
    for k in 0..len {
        let c = if (k / 3) % 2 == 0 { rgb(248, 104, 32) } else { rgb(248, 248, 248) };
        let r = 2 - k / 5;
        let yy = y - 19 + (k * lift) / len.max(1) + (k / 4);
        f.rect(x + 1 + k, yy - r.max(1), 1, r.max(1) * 2, c);
    }
}

fn kite_surfer(f: &mut Frame, x: i32, wy: i32, var: u32, t: u32) {
    let ph = t as f32 * 0.03 + var as f32 * 2.0;
    let sx = x + (ph.sin() * 20.0) as i32;
    let cols = [rgb(248, 72, 168), rgb(64, 224, 232), rgb(248, 216, 48)];
    let col = cols[var as usize % 3];
    // board and spray
    f.rect(sx - 4, wy - 1, 9, 2, OUTLINE);
    f.hline(sx - 3, sx + 3, wy - 1, col);
    if (t / 4) % 2 == 0 {
        f.set(sx - 6, wy - 2, rgb(248, 248, 248));
        f.set(sx - 7, wy - 3, rgb(248, 248, 248));
    }
    // rider
    f.rect(sx - 1, wy - 8, 3, 6, rgb(40, 40, 64));
    f.rect(sx - 1, wy - 11, 3, 3, rgb(232, 176, 128));
    // the kite in a figure eight
    let kx = sx + 10 + ((ph * 2.0).sin() * 18.0) as i32;
    let ky = wy - 46 + ((ph * 4.0).sin() * 6.0) as i32;
    f.line(sx + 1, wy - 7, kx - 6, ky + 2, rgb(200, 200, 216));
    f.line(sx + 1, wy - 7, kx + 6, ky + 2, rgb(200, 200, 216));
    for k in -8i32..=8 {
        let yy = ky - (8 - k.abs()) / 3;
        f.rect(kx + k, yy, 1, 3, if k.abs() < 3 { rgb(248, 248, 248) } else { col });
    }
    f.hline(kx - 9, kx + 9, ky + 3, OUTLINE);
}

fn crowd(f: &mut Frame, cheer: bool, x: i32, y: i32, t: u32) {
    let shirts = [rgb(232, 56, 64), rgb(56, 136, 232), rgb(248, 200, 48), rgb(120, 216, 96), rgb(232, 104, 200), rgb(248, 136, 32)];
    // fence
    for k in 0..9 {
        let px = x - 28 + k * 7;
        let jump = if cheer && ((t / 5) as i32 + k) % 2 == 0 { 2 } else { 0 };
        let py = y - jump;
        f.rect(px - 1, py - 10, 4, 6, OUTLINE);
        f.rect(px, py - 9, 2, 4, shirts[(k as usize) % 6]);
        f.rect(px - 1, py - 13, 4, 4, OUTLINE);
        f.rect(px, py - 12, 2, 2, if k % 3 == 0 { rgb(176, 112, 72) } else { rgb(248, 200, 160) });
        if cheer || k % 4 == 1 {
            f.set(px - 1, py - 12, rgb(248, 200, 160));
            f.set(px + 2, py - 12, rgb(248, 200, 160));
        }
        f.rect(px, py - 5, 2, 5, rgb(48, 48, 72));
        if k % 3 == 1 {
            // a flag on a stick
            f.line(px + 2, py - 12, px + 2, py - 20, rgb(96, 64, 40));
            let wave = ((t / 8) as i32 + k) % 2;
            f.rect(px + 3, py - 20 + wave, 5, 3, shirts[(k as usize + 2) % 6]);
        }
    }
    f.hline(x - 32, x + 34, y - 4, rgb(248, 248, 248));
    f.hline(x - 32, x + 34, y - 2, rgb(248, 248, 248));
    for k in 0..6 {
        f.rect(x - 32 + k * 13, y - 6, 1, 6, rgb(248, 248, 248));
    }
}

fn draw_target(f: &mut Frame, c: &Course, cam: &Cam, t: u32) {
    let tx = c.target;
    if !cam.visible(tx, 50.0) {
        return;
    }
    let gy = cam.sy(c.ground_at(tx));
    let x = cam.sx(tx);
    // flattened rings on the grass
    let [r0, r1, r2] = RINGS.map(|r| r.0 as i32);
    for (r, col) in [(r2, rgb(248, 248, 248)), (r2 - 4, rgb(232, 56, 56)), (r1, rgb(248, 248, 248)), (r1 - 4, rgb(232, 56, 56)), (r0, rgb(248, 216, 48))] {
        f.fill_ellipse(x, gy + 1, r, 2, col);
    }
    f.line(x - 4, gy, x + 4, gy + 2, OUTLINE);
    f.line(x - 4, gy + 2, x + 4, gy, OUTLINE);
    // distance pennants: gold, silver, bronze
    for (d, col) in [(r0, rgb(248, 216, 48)), (r1, rgb(200, 208, 224)), (r2, rgb(208, 136, 72))] {
        for s in [-1, 1] {
            let px = x + s * d;
            f.line(px, gy, px, gy - 7, rgb(96, 96, 112));
            f.rect(px + 1, gy - 7, 3, 2, col);
        }
    }
    // the target flag
    let wave = ((t / 10) % 2) as i32;
    f.rect(x - 1, gy - 34, 3, 34, OUTLINE);
    f.line(x, gy - 33, x, gy, rgb(248, 248, 248));
    f.rect(x + 1, gy - 34 + wave, 18, 11, OUTLINE);
    f.rect(x + 2, gy - 33 + wave, 16, 9, rgb(232, 56, 56));
    f.line(x + 5, gy - 32 + wave, x + 14, gy - 26 + wave, rgb(248, 248, 248));
    f.line(x + 5, gy - 26 + wave, x + 14, gy - 32 + wave, rgb(248, 248, 248));
}

fn draw_markers(f: &mut Frame, race: &Race, art: &Art, cam: &Cam, t: u32) {
    let c = &race.course;
    for th in &c.thermals {
        if !cam.visible(th.x, th.w + 20.0) {
            continue;
        }
        let g = c.ground_at(th.x);
        let tf = t as f32;
        match th.marker {
            Marker::Storks => {
                let birds = if c.venue == Venue::Canyon { &art.critters.vulture } else { &art.critters.stork };
                for k in 0..3 {
                    let ph = tf * 0.011 + k as f32 * 2.1 + th.phase;
                    let x = th.x + ph.sin() * th.w * 0.75;
                    let y = g + (th.top - g) * (0.32 + 0.22 * k as f32) + (tf * 0.008 + k as f32).sin() * 14.0;
                    let flip = ph.cos() < 0.0;
                    let flap = ((t / 7) as usize + k * 5) % 30;
                    let fr = if flap < 4 { [1, 0, 2, 0][flap] } else { 0 };
                    draw_pic(f, &birds[fr], cam.sx(x), cam.sy(y), flip);
                }
            }
            Marker::Leaves => {
                let h = (th.top - g - 30.0).max(40.0);
                for j in 0..16 {
                    let jf = j as f32;
                    let x = th.x + (tf * 0.03 + jf * 1.7).sin() * th.w * 0.7;
                    let y = g + 6.0 + (tf * 0.32 + jf * 47.0).rem_euclid(h);
                    let (sx, sy) = (cam.sx(x), cam.sy(y));
                    let col = match c.venue {
                        Venue::Dunes => [rgb(248, 248, 240), rgb(232, 232, 216)][j % 2],
                        _ => [rgb(248, 152, 32), rgb(248, 216, 48), rgb(216, 72, 40)][j % 3],
                    };
                    let flick = ((t / 6) as usize + j) % 2 == 0;
                    f.set(sx, sy, col);
                    if flick {
                        f.set(sx + 1, sy, col);
                    } else {
                        f.set(sx, sy - 1, col);
                    }
                    if c.venue == Venue::Dunes {
                        f.set(sx, sy + 1, rgb(184, 184, 168));
                    }
                }
            }
            Marker::Dust => {
                // a dust devil at the bottom of the thermal
                for j in 0..44 {
                    let hj = j as f32 * 2.2;
                    let r = 3.0 + hj * 0.16;
                    let a = tf * 0.22 + j as f32 * 0.7;
                    let x = th.x + a.cos() * r + (tf * 0.02).sin() * 4.0;
                    let col = if j % 3 == 0 { rgb(232, 192, 136) } else { rgb(208, 160, 104) };
                    let (sx, sy) = (cam.sx(x), cam.sy(g + hj));
                    f.set(sx, sy, col);
                    if a.sin() > 0.0 {
                        f.set(sx + 1, sy, col);
                    }
                }
                let h = (th.top - g - 30.0).max(40.0);
                for j in 0..10 {
                    let jf = j as f32;
                    let x = th.x + (tf * 0.025 + jf * 2.3).sin() * th.w * 0.6;
                    let y = g + 100.0 + (tf * 0.3 + jf * 53.0).rem_euclid((h - 100.0).max(20.0));
                    f.set(cam.sx(x), cam.sy(y), rgb(232, 200, 152));
                }
                let birds = &art.critters.vulture;
                let ph = tf * 0.01 + th.phase;
                let x = th.x + ph.sin() * th.w * 0.7;
                let y = g + (th.top - g) * 0.7;
                draw_pic(f, &birds[0], cam.sx(x), cam.sy(y), ph.cos() < 0.0);
            }
        }
    }
}

fn draw_geese(f: &mut Frame, race: &Race, art: &Art, cam: &Cam, t: u32) {
    for fl in &race.flocks {
        let Some((lx, ly)) = fl.pos(race.t, race.course.wind) else { continue };
        if !cam.visible(lx - 30.0, 60.0) {
            continue;
        }
        let mut birds = vec![(lx, ly)];
        for k in 1..=3 {
            let kf = k as f32;
            birds.push((lx - 13.0 * kf, ly + 6.0 * kf));
            birds.push((lx - 13.0 * kf + 3.0, ly - 6.0 * kf));
        }
        for (k, (x, y)) in birds.iter().enumerate() {
            let fr = ((t / 9) as usize + k) % 2;
            draw_pic(f, &art.critters.goose[fr], cam.sx(*x), cam.sy(*y), false);
        }
    }
}

fn draw_ufo(f: &mut Frame, race: &Race, cam: &Cam, t: u32) {
    let Some(u) = &race.ufo else { return };
    if !cam.visible(u.x, 40.0) {
        return;
    }
    let (x, y) = (cam.sx(u.x), cam.sy(u.y));
    if u.beaming() {
        // tractor beam: a lightened cone with moving stripes
        let bottom = cam.sy(u.y - 170.0).min(cam.vh - 1);
        for yy in (y + 3).max(0)..bottom {
            let k = (yy - y) as f32 / 170.0;
            let half = 5 + (k * 12.0) as i32;
            let stripe = ((yy + (t as i32) / 2) / 3) % 3 == 0;
            for xx in x - half..=x + half {
                if (0..W).contains(&xx) && (xx + yy) & 1 == 0 {
                    let i = (yy * W + xx) as usize;
                    f.px[i] = if stripe { rgb(200, 248, 200) } else { mix(f.px[i], rgb(184, 248, 184), 0.5) };
                }
            }
        }
    }
    f.fill_ellipse(x, y - 4, 6, 5, OUTLINE);
    f.fill_ellipse(x, y - 4, 5, 4, rgb(160, 232, 232));
    f.rect(x - 1, y - 6, 3, 3, rgb(96, 216, 96));
    f.set(x - 1, y - 6, OUTLINE);
    f.set(x + 1, y - 6, OUTLINE);
    f.fill_ellipse(x, y, 14, 4, OUTLINE);
    f.fill_ellipse(x, y, 13, 3, rgb(168, 168, 192));
    f.hline(x - 12, x + 12, y - 1, rgb(216, 216, 232));
    for k in 0..5 {
        let lx = x - 10 + k * 5;
        let on = ((t / 6) as i32 + k) % 3 == 0;
        f.set(lx, y + 1, if on { rgb(248, 248, 96) } else { rgb(232, 72, 72) });
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_gate(f: &mut Frame, sc: &Scenery, g: &Gate, k: usize, made: bool, cam: &Cam, t: u32, back: bool) {
    let x = cam.sx(g.x);
    let (top, bot) = (cam.sy(g.hi), cam.sy(g.lo));
    let cy = (top + bot) / 2;
    let ry = (bot - top) / 2;
    let rx = 7;
    let (a, b) = if made { (rgb(248, 216, 48), rgb(248, 248, 160)) } else { (rgb(232, 48, 64), rgb(248, 248, 248)) };
    if back {
        // the balloon that holds the hoop
        let bt = top - 26;
        let bob = ((t as f32 * 0.04 + k as f32).sin() * 2.0) as i32;
        draw_pic_clip(f, &sc.envelopes[k], x, bt - 12 + bob, cam.vh, false);
        f.line(x - 5, bt - 12 + bob, x - 3, bt - 4 + bob, rgb(96, 64, 40));
        f.line(x + 5, bt - 12 + bob, x + 3, bt - 4 + bob, rgb(96, 64, 40));
        f.rect(x - 5, bt - 5 + bob, 11, 7, OUTLINE);
        f.rect(x - 4, bt - 4 + bob, 9, 5, rgb(176, 120, 64));
        f.hline(x - 4, x + 4, bt - 2 + bob, rgb(136, 88, 48));
        // ropes down to the hoop
        f.line(x - 3, bt + 2 + bob, x - 1, top, rgb(120, 96, 72));
        f.line(x + 3, bt + 2 + bob, x + 1, top, rgb(120, 96, 72));
        // gate number pennant
        let n = format!("{}", k + 1);
        f.rect(x + 7, bt - 6 + bob, 9, 11, OUTLINE);
        f.rect(x + 8, bt - 5 + bob, 7, 9, if made { rgb(248, 216, 48) } else { rgb(248, 248, 248) });
        f.text(x + 9, bt - 4 + bob, &n, rgb(24, 24, 48));
    }
    // hoop: the left half behind the pilots, the right half in front
    let n = 96;
    for i in 0..n {
        let ang = i as f32 / n as f32 * std::f32::consts::TAU;
        let (ca, sa) = (ang.cos(), ang.sin());
        if (ca < 0.0) != back {
            continue;
        }
        let px = x + (ca * rx as f32).round() as i32;
        let py = cy + (sa * ry as f32).round() as i32;
        if py < 0 || py >= cam.vh {
            continue;
        }
        f.rect(px - 2, py - 1, 4, 3, OUTLINE);
    }
    for i in 0..n {
        let ang = i as f32 / n as f32 * std::f32::consts::TAU;
        let (ca, sa) = (ang.cos(), ang.sin());
        if (ca < 0.0) != back {
            continue;
        }
        let px = x + (ca * rx as f32).round() as i32;
        let py = cy + (sa * ry as f32).round() as i32;
        if py < 0 || py >= cam.vh {
            continue;
        }
        let col = if (i / 6) % 2 == 0 { a } else { b };
        let col = if back { shade(col, 3, 4) } else { col };
        f.set(px, py, col);
        f.set(px + if ca < 0.0 { 1 } else { -1 }, py, col);
    }
}

/// Wind lines that follow the ground: they climb on slopes with ridge lift.
/// Over sink (lakes, canyon floors) the air visibly falls.
fn draw_streaks(f: &mut Frame, c: &Course, cam: &Cam, t: u32) {
    for k in 0..22u32 {
        let wx = cam.x - 128.0 + ((k * 61) as f32 + t as f32 * 0.4).rem_euclid(300.0) - 20.0;
        if c.sink_at(wx) < 0.05 {
            continue;
        }
        let dy = ((k * 37) as f32 - t as f32 * 0.9).rem_euclid(220.0) - 110.0;
        let (sx, sy) = (cam.sx(wx), cam.cy + dy as i32);
        if sy > 0 && sy < cam.vh - 4 && sy < cam.sy(c.floor_at(wx)) - 4 {
            f.line(sx, sy, sx, sy + 3, rgb(88, 104, 152));
            f.set(sx, sy + 4, rgb(120, 136, 184));
        }
    }
    for k in 0..26u32 {
        let speed = 1.3 + (k % 3) as f32 * 0.3;
        let span = 340.0;
        let ox = (k as f32 * 53.0 + t as f32 * speed).rem_euclid(span) - 40.0;
        let wx = cam.x - 128.0 + ox;
        let h0 = 6.0 + ((k * 23) % 64) as f32;
        let wy = c.ground_at(wx + 20.0).max(c.ground_at(wx)) + h0;
        let lift = c.ridge_lift(wx, wy);
        if lift < 0.05 && k % 3 != 0 {
            continue;
        }
        let (sx, sy) = (cam.sx(wx), cam.sy(wy));
        if sy < 0 || sy >= cam.vh {
            continue;
        }
        let len = 4 + (k % 4) as i32;
        let rise = (c.ground_at(wx + len as f32 + 20.0) - c.ground_at(wx + 20.0)).round() as i32;
        let col = if lift > 0.05 { rgb(248, 248, 248) } else { rgb(200, 216, 232) };
        f.line(sx, sy, sx + len, sy - rise, col);
    }
}

fn draw_fx(f: &mut Frame, race: &Race, cam: &Cam) {
    for fx in &race.fx {
        let (x, y) = (cam.sx(fx.x), cam.sy(fx.y));
        if y < 0 || y >= cam.vh {
            continue;
        }
        match fx.kind {
            FxKind::Dust => {
                let r = 1 + fx.life * 3 / fx.max.max(1);
                f.fill_ellipse(x, y, r, r, fx.color);
            }
            FxKind::Drop => {
                f.set(x, y, fx.color);
                f.set(x, y + 1, rgb(96, 168, 232));
            }
            FxKind::Star => {
                f.set(x, y, fx.color);
                f.set(x - 1, y, fx.color);
                f.set(x + 1, y, fx.color);
                f.set(x, y - 1, fx.color);
                f.set(x, y + 1, fx.color);
            }
            FxKind::Wool => {
                f.fill_ellipse(x, y, 1, 1, fx.color);
            }
            FxKind::Spark => {
                let cols = [rgb(248, 72, 72), rgb(248, 216, 48), rgb(96, 224, 248), rgb(120, 248, 96)];
                let col = cols[(fx.life as usize + (fx.x as usize)) % 4];
                f.set(x, y, col);
                f.set(x + 1, y, col);
            }
            FxKind::Feather => f.set(x, y, fx.color),
        }
    }
}

// ------------------------------------------------------------------ HUD

pub fn ordinal(n: usize) -> &'static str {
    match n {
        1 => "1ST",
        2 => "2ND",
        3 => "3RD",
        _ => "4TH",
    }
}

pub fn clock_text(frames: u32) -> String {
    let s = frames / 60;
    format!("{}:{:02}", s / 60, s % 60)
}

/// Position of a pilot in the race right now (by score, then distance).
pub fn place(race: &Race, i: usize) -> usize {
    let me = &race.pilots[i];
    let key = |p: &Pilot| p.gate_pts + p.landing_pts + p.bonus - p.penalty + p.time_bonus;
    1 + race.pilots.iter().enumerate().filter(|(k, p)| *k != i && (key(p) > key(me) || (key(p) == key(me) && p.x > me.x))).count()
}

/// Full HUD band for one player (1P), rows 0..32.
pub fn draw_hud(f: &mut Frame, race: &Race, i: usize, t: u32) {
    let p = &race.pilots[i];
    f.rect(0, 0, W, 31, rgb(16, 16, 48));
    f.hline(0, W - 1, 31, rgb(248, 176, 32));
    f.hline(0, W - 1, 32, rgb(176, 32, 88));
    // altimeter
    f.text_sh(4, 2, "ALT", rgb(160, 168, 216), OUTLINE, false);
    let alt = (p.y.max(0.0) * 3.0) as i32;
    f.text_grad(4, 13, &format!("{alt:>4}"), &grad7(rgb(96, 224, 248)), OUTLINE, true);
    f.text_sh(38, 13, "M", rgb(160, 168, 216), OUTLINE, false);
    // variometer: a bar that grows up (green) or down (red)
    let vx = 54;
    f.text_sh(vx, 2, "VARIO", rgb(160, 168, 216), OUTLINE, false);
    vario_bar(f, vx + 36, 3, 24, p.vario, t);
    let ms = p.vario * 10.0;
    let vtxt = format!("{}{:.1}", if ms >= 0.0 { "+" } else { "-" }, ms.abs());
    let vcol = if ms > 0.3 { rgb(120, 248, 120) } else if ms < -2.0 { rgb(248, 96, 96) } else { rgb(248, 248, 248) };
    f.text_grad(vx, 13, &vtxt, &grad7(vcol), OUTLINE, true);
    // clock
    f.text_center(128, 2, "TIME", rgb(160, 168, 216), OUTLINE, false);
    let tc = clock_text(p.shown_time());
    let sunset = p.sunset && (t / 8) % 2 == 0;
    f.text_grad_center(128, 13, &tc, &grad7(if sunset { rgb(248, 96, 64) } else { rgb(248, 224, 72) }), OUTLINE, true);
    // score, gates, place
    f.text_sh(180, 2, "SCORE", rgb(160, 168, 216), OUTLINE, false);
    let sc = p.gate_pts + p.landing_pts + p.bonus + p.time_bonus - p.penalty;
    f.text_sh(212, 2, &format!("{:>5}", sc.max(0)), rgb(248, 248, 248), OUTLINE, false);
    for k in 0..race.course.gates.len() {
        let made = p.gates & (1 << k) != 0;
        mini_balloon(f, 184 + k as i32 * 10, 13, made, k);
    }
    let pl = place(race, i);
    f.text_grad(226, 13, ordinal(pl), &grad7(if pl == 1 { rgb(248, 216, 48) } else { rgb(200, 200, 232) }), OUTLINE, false);
    if p.v < PILOTS[p.def].stall_speed() + 0.07 && p.state == State::Fly && (t / 6) % 2 == 0 {
        f.text_sh(4, 23, "STALL!", rgb(248, 72, 72), OUTLINE, false);
    } else if p.tuck {
        f.text_sh(4, 23, "TUCK", rgb(248, 216, 48), OUTLINE, false);
    }
    minimap(f, race, Some(i), 92, 22, 160, t);
}

fn mini_balloon(f: &mut Frame, x: i32, y: i32, made: bool, k: usize) {
    let col = if made { [rgb(232, 48, 48), rgb(48, 120, 232), rgb(56, 192, 88), rgb(184, 72, 216)][k % 4] } else { rgb(48, 48, 88) };
    f.fill_ellipse(x + 3, y + 3, 4, 4, OUTLINE);
    f.fill_ellipse(x + 3, y + 3, 3, 3, col);
    if made {
        f.set(x + 2, y + 1, rgb(248, 248, 248));
    }
    f.rect(x + 2, y + 7, 3, 2, if made { rgb(176, 120, 64) } else { rgb(48, 48, 88) });
}

pub fn vario_bar(f: &mut Frame, x: i32, y: i32, h: i32, v: f32, t: u32) {
    f.rect(x - 1, y - 1, 8, h + 2, OUTLINE);
    f.rect(x, y, 6, h, rgb(32, 32, 72));
    let mid = y + h / 2;
    f.hline(x, x + 5, mid, rgb(120, 120, 168));
    let n = ((v * 10.0 / 4.0) * (h / 2) as f32).clamp(-(h / 2) as f32, (h / 2) as f32) as i32;
    if n > 0 {
        for k in 0..n {
            f.hline(x + 1, x + 4, mid - 1 - k, if k > h / 4 { rgb(160, 248, 120) } else { rgb(96, 216, 96) });
        }
    } else if n < 0 {
        for k in 0..-n {
            f.hline(x + 1, x + 4, mid + 1 + k, if k > h / 4 { rgb(248, 72, 72) } else { rgb(216, 120, 72) });
        }
    }
    if v > 0.05 && (t / 6) % 2 == 0 {
        f.hline(x + 2, x + 3, y - 3, rgb(160, 248, 120));
        f.hline(x + 1, x + 4, y - 2, rgb(160, 248, 120));
    }
}

/// Course profile with gates, field and everyone's dot.
pub fn minimap(f: &mut Frame, race: &Race, me: Option<usize>, x0: i32, y0: i32, w: i32, t: u32) {
    let c = &race.course;
    let h = 8;
    let sx = |x: f32| x0 + (x / LEN as f32 * (w - 1) as f32) as i32;
    let maxa = 640.0;
    let sy = |y: f32| y0 + h - 1 - ((y / maxa).clamp(0.0, 1.0) * (h - 1) as f32) as i32;
    f.rect(x0 - 1, y0 - 1, w + 2, h + 2, OUTLINE);
    f.rect(x0, y0, w, h, rgb(40, 56, 120));
    for k in 0..w {
        let wx = k as f32 / (w - 1) as f32 * LEN as f32;
        let gy = sy(c.ground_at(wx).max(0.0));
        let col = if c.water_at(wx).is_some() { rgb(96, 168, 232) } else if c.in_field(wx) { rgb(160, 232, 96) } else { rgb(96, 152, 80) };
        for yy in gy..y0 + h {
            f.set(x0 + k, yy, col);
        }
    }
    for (k, g) in c.gates.iter().enumerate() {
        let made = me.map(|m| race.pilots[m].gates & (1 << k) != 0).unwrap_or(false);
        let col = if made { rgb(248, 216, 48) } else { rgb(248, 248, 248) };
        f.line(sx(g.x), sy(g.hi), sx(g.x), sy(g.lo), col);
    }
    f.set(sx(c.target), y0, rgb(232, 48, 48));
    f.set(sx(c.target), y0 + 1, rgb(232, 48, 48));
    for (i, p) in race.pilots.iter().enumerate() {
        if Some(i) == me {
            continue;
        }
        f.set(sx(p.x), sy(p.y), PILOTS[p.def].ui);
    }
    if let Some(m) = me {
        let p = &race.pilots[m];
        if (t / 8) % 3 != 0 {
            f.rect(sx(p.x) - 1, sy(p.y) - 1, 3, 3, rgb(248, 248, 248));
        }
    }
}

/// Compact HUD strip for split screen, rows y..y+12.
pub fn draw_hud_small(f: &mut Frame, race: &Race, i: usize, y: i32, t: u32) {
    let p = &race.pilots[i];
    f.rect(0, y, W, 12, rgb(16, 16, 48));
    f.hline(0, W - 1, y + 12, rgb(248, 176, 32));
    let k = match p.ctrl {
        Ctrl::Human(k) => k,
        Ctrl::Cpu => 0,
    };
    let col = if k == 0 { rgb(248, 232, 64) } else { rgb(96, 224, 248) };
    f.rect(2, y + 2, 13, 9, col);
    f.text(3, y + 3, if k == 0 { "1P" } else { "2P" }, rgb(16, 16, 48));
    let alt = (p.y.max(0.0) * 3.0) as i32;
    f.text_sh(20, y + 3, &format!("ALT{alt:>5}M"), rgb(96, 224, 248), OUTLINE, false);
    vario_bar(f, 80, y + 2, 8, p.vario, t);
    f.text_sh(94, y + 3, &clock_text(p.shown_time()), rgb(248, 224, 72), OUTLINE, false);
    let sc = p.gate_pts + p.landing_pts + p.bonus + p.time_bonus - p.penalty;
    f.text_sh(130, y + 3, &format!("{:>5}", sc.max(0)), rgb(248, 248, 248), OUTLINE, false);
    for g in 0..race.course.gates.len() {
        mini_balloon(f, 166 + g as i32 * 9, y + 1, p.gates & (1 << g) != 0, g);
    }
    let pl = place(race, i);
    f.text_sh(208, y + 3, ordinal(pl), if pl == 1 { rgb(248, 216, 48) } else { rgb(200, 200, 232) }, OUTLINE, false);
    if p.v < PILOTS[p.def].stall_speed() + 0.07 && p.state == State::Fly && (t / 6) % 2 == 0 {
        f.text_sh(228, y + 3, "STALL", rgb(248, 72, 72), OUTLINE, false);
    }
}

/// Where the next gate is when it is off screen (right edge marker).
pub fn gate_pointer(f: &mut Frame, race: &Race, i: usize, cam: &Cam, top: i32, t: u32) {
    let p = &race.pilots[i];
    if !p.airborne() {
        return;
    }
    let c = &race.course;
    let Some((k, g)) = c.gates.iter().enumerate().find(|(k, g)| p.gates & (1 << k) == 0 && g.x > p.x - 4.0) else { return };
    let x = cam.sx(g.x);
    if x < W - 4 || g.x - p.x > 900.0 {
        return;
    }
    let (a, b) = (cam.sy(g.hi).clamp(top + 2, cam.vh - 3), cam.sy(g.lo).clamp(top + 2, cam.vh - 3));
    let col = if (t / 10) % 2 == 0 { rgb(248, 216, 48) } else { rgb(248, 136, 32) };
    f.rect(W - 4, a, 3, (b - a).max(1), OUTLINE);
    f.line(W - 3, a, W - 3, b, col);
    let n = format!("{}", k + 1);
    let ty = ((a + b) / 2 - 3).clamp(top + 2, cam.vh - 10);
    f.text_sh(W - 10, ty, &n, col, OUTLINE, false);
    if cam.sy(g.hi) < top + 2 {
        f.text_sh(W - 10, top + 3, "+", col, OUTLINE, false);
    }
    if cam.sy(g.lo) > cam.vh - 3 {
        f.text_sh(W - 10, cam.vh - 10, "-", col, OUTLINE, false);
    }
}

pub fn draw_banner(f: &mut Frame, b: &Banner, cy: i32) {
    let n = b.text.chars().count() as i32;
    let scale = if n * 18 < 236 { 3 } else { 2 };
    let x = if b.t < 10 { 128 + (10 - b.t) * 30 } else { 128 };
    let y = cy - 7 * scale / 2 + if (10..16).contains(&b.t) { (16 - b.t) / 2 } else { 0 };
    if b.t > b.life - 8 && (b.t / 2) % 2 == 0 {
        return;
    }
    let grad: Vec<u32> = if b.color == rgb(248, 248, 248) { SUNSET.to_vec() } else { grad6(b.color).to_vec() };
    f.logo(x, y, &b.text, scale, &grad, OUTLINE, rgb(8, 8, 24));
}
