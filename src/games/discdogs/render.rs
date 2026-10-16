//! Draws a match: the stage, owners, dogs, disc, critters, effects, the
//! aim and power meters, the HUD, banners and the comment box.

use super::data::{Stage, PAIRS};
use super::scenery::{sx, Scenery, GROUND_Y};
use super::sim::*;
use super::sprites::*;
use crate::engine::gfx::*;
use crate::engine::sprites::blit;
use crate::engine::ui::{grad6, grad7, OUTLINE, SUNSET};

pub struct Art {
    pub dogs: DogArt,
    pub owners: OwnerArt,
    pub props: Props,
}

impl Art {
    pub fn new() -> Self {
        Art { dogs: DogArt::new(), owners: OwnerArt::new(), props: Props::new() }
    }
}

impl Default for Art {
    fn default() -> Self {
        Self::new()
    }
}

/// The collar (and HUD) color of a side: yellow for 1P, cyan for 2P, red for the CPU.
pub fn side_color(m: &Match, side: usize) -> u32 {
    match m.sides[side].ctrl {
        Ctrl::Human(0) => rgb(248, 216, 48),
        Ctrl::Human(_) => rgb(64, 208, 248),
        Ctrl::Cpu => rgb(232, 64, 72),
    }
}

/// The disc takes the thrower's pair color.
pub fn disc_color(pair: usize) -> u32 {
    PAIRS[pair].color
}

fn sy(z: f32) -> i32 {
    GROUND_Y - z.round() as i32
}

pub fn dog_frame(d: &Dog, t: u32) -> DogFrame {
    let run = [DogFrame::Run0, DogFrame::Run1, DogFrame::Run2, DogFrame::Run3];
    match d.state {
        DogState::Sit => {
            if (t / 10) % 2 == 0 {
                DogFrame::SitWag
            } else {
                DogFrame::Sit
            }
        }
        DogState::Stand => DogFrame::Stand,
        DogState::Run | DogState::Swim => run[(d.anim as usize) % 4],
        DogState::Air => {
            if d.flip_t >= 0 {
                [DogFrame::Curl0, DogFrame::Curl1, DogFrame::Curl2, DogFrame::Curl3][((d.flip_t * 4 / FLIP_LEN) as usize).min(3)]
            } else if d.vz > 0.4 {
                DogFrame::Leap
            } else {
                DogFrame::Reach
            }
        }
        DogState::Tumble => DogFrame::Stand,
        DogState::Stare => DogFrame::Stare,
        DogState::Happy => {
            if d.z > 0.0 || (t / 8) % 2 == 0 {
                DogFrame::Happy
            } else {
                DogFrame::Stand
            }
        }
        DogState::Guilty => DogFrame::Guilty,
        DogState::Lie => DogFrame::Lie,
        DogState::Shake => {
            if (t / 3) % 2 == 0 {
                DogFrame::Shake0
            } else {
                DogFrame::Shake1
            }
        }
        DogState::Bark => {
            if (t / 10) % 2 == 0 {
                DogFrame::Bark
            } else {
                DogFrame::Stand
            }
        }
    }
}

/// Draw a dog at world position, with the disc in its mouth if it has it.
pub fn draw_dog(f: &mut Frame, art: &Art, d: &Dog, cam: f32, t: u32, collar: u32, disc_col: u32, carried: bool, wet: bool) {
    draw_dog_at(f, art, d, sx(d.x, cam), GROUND_Y, t, collar, disc_col, carried, wet);
}

/// Draw a dog with its ground point at screen (x, gy).
#[allow(clippy::too_many_arguments)]
pub fn draw_dog_at(f: &mut Frame, art: &Art, d: &Dog, x: i32, gy: i32, t: u32, collar: u32, disc_col: u32, carried: bool, wet: bool) {
    let sy = |z: f32| gy - z.round() as i32;
    let y = sy(d.z);
    let fr = dog_frame(d, t);
    let s = art.dogs.get(d.pair, fr);
    let flip = d.face < 0.0;
    let mut pal = dog_pal(d.pair, collar);
    if wet {
        pal = wet_pal(&pal);
    }
    // shadow
    let r = (9 - (d.z / 8.0) as i32).max(3);
    f.shade_ellipse(x, gy + 1, r, 2, 5, 8);
    match d.state {
        DogState::Tumble => {
            draw_vflip(f, s, x, gy + 1, flip, &pal);
            for k in 0..3 {
                let a = t as f32 * 0.2 + k as f32 * 2.1;
                let (px, py) = (x + (a.cos() * 7.0) as i32, gy - 22 + (a.sin() * 2.0) as i32);
                f.set(px, py, rgb(248, 248, 96));
                f.set(px + 1, py, rgb(248, 248, 96));
            }
        }
        DogState::Swim => {
            let cut = gy - 6;
            draw_clipped(f, s, x, y + 5, flip, &pal, cut);
            let w = 8 + ((t / 6) % 3) as i32;
            f.hline(x - w, x + w, cut, rgb(200, 232, 248));
            f.hline(x - w + 3, x + w - 3, cut + 1, rgb(144, 200, 240));
        }
        DogState::Air if d.flip_t >= 0 => {
            // spinning: draw around the body centre
            let cy = sy(d.z + 9.0);
            draw(f, s, x, cy, flip, &pal);
            // speed lines
            for k in 0..3 {
                let a = (d.flip_t as f32 / FLIP_LEN as f32) * std::f32::consts::TAU + k as f32 * 2.1;
                let (px, py) = (x + (a.cos() * 13.0) as i32, cy + (a.sin() * 13.0) as i32);
                f.set(px, py, rgb(248, 248, 248));
                f.set(px, py + 1, rgb(200, 216, 248));
            }
        }
        _ => draw(f, s, x, y, flip, &pal),
    }
    if carried {
        let (mx, mz) = d.mouth();
        draw_disc(f, x + (mx - d.x).round() as i32, sy(mz) + 1, disc_col, if d.flip_t >= 0 { 1 } else { 0 }, 0.0);
    }
}

fn owner_pose(m: &Match, active: bool) -> OwnerPose {
    if !active {
        return OwnerPose::Sit;
    }
    match m.phase {
        Phase::Ready => OwnerPose::Stand,
        Phase::Aim => OwnerPose::Aim,
        Phase::Power => OwnerPose::Coil,
        Phase::Release => OwnerPose::Fling,
        Phase::Flight => {
            if m.dog.state == DogState::Stare {
                OwnerPose::Whistle
            } else if m.throw_anim < 14 {
                OwnerPose::Follow
            } else {
                OwnerPose::Stand
            }
        }
        Phase::Result | Phase::Over => {
            if m.owner_mood > 0 {
                if (m.phase_t / 12) % 2 == 0 {
                    OwnerPose::Cheer
                } else {
                    OwnerPose::Stand
                }
            } else if m.owner_mood < 0 {
                OwnerPose::Facepalm
            } else {
                OwnerPose::Stand
            }
        }
    }
}

/// The side that waits its turn: owner on a cooler, dog lying beside it.
fn draw_waiting(f: &mut Frame, m: &Match, art: &Art, cam: f32, t: u32) {
    let side = 1 - m.thrower();
    let pair = m.sides[side].pair;
    let ox = sx(-62.0, cam);
    if ox < -40 {
        return;
    }
    let y = GROUND_Y - 7;
    // cooler
    f.rect(ox - 7, y - 8, 14, 9, OUTLINE);
    f.rect(ox - 6, y - 7, 12, 7, rgb(56, 136, 216));
    f.rect(ox - 6, y - 7, 12, 2, rgb(232, 240, 248));
    blit(f, art.owners.get(pair, OwnerPose::Sit), ox, y - 2, false, &owner_pal(pair));
    let mut d = Dog::new(pair);
    d.state = DogState::Lie;
    let s = art.dogs.get(pair, if (t / 90) % 4 == 0 { DogFrame::Sit } else { DogFrame::Lie });
    let pal = dog_pal(pair, side_color(m, side));
    f.shade_ellipse(ox + 20, y + 1, 9, 2, 5, 8);
    draw(f, s, ox + 20, y, false, &pal);
    let _ = d;
}

pub fn draw_match(f: &mut Frame, m: &Match, sc: &Scenery, art: &Art, t: u32) {
    let cam = m.cam;
    sc.draw_back(f, cam, t, m.excite);
    let side = m.thrower();
    let pair = m.sides[side].pair;
    let dcol = disc_color(pair);

    // ---- world props that change
    for b in &m.blankets {
        let x = sx(b.x, cam);
        if !(-60..W + 60).contains(&x) {
            continue;
        }
        // the plate with what is left of the sausages
        let (px, py) = (x + 8, 189);
        f.fill_ellipse(px, py, 6, 2, OUTLINE);
        f.fill_ellipse(px, py, 5, 1, rgb(248, 248, 248));
        for k in 0..b.sausages {
            draw(f, &art.props.sausage, px - 3 + k * 3, py - 1 - k % 2, false, &prop_pal("sausage"));
        }
        if b.sleeper {
            let (zx, zy) = (x + 16, 192);
            if m.sleeper_awake > 0 {
                art.props.sleeper_up.draw(f, zx, zy, false);
            } else {
                art.props.sleeper.draw(f, zx, zy, false);
                let k = (t / 20) % 3;
                for j in 0..=k {
                    let s = if j == 2 { "Z" } else { "z" };
                    f.text(zx - 6 + j as i32 * 4, zy - 12 - j as i32 * 6, s, rgb(248, 248, 248));
                }
            }
        }
    }
    if m.kite.here {
        let kx = sx(m.kite.kid_x, cam);
        let ((ax, az), (bx, bz)) = m.kite.string(m.frame);
        let (x0, y0, x1, y1) = (sx(ax, cam), sy(az) - 6, sx(bx, cam), sy(bz));
        if x1 > -30 && kx < W + 30 {
            // the string sags a little
            let n = 24;
            let mut last = (x0, y0);
            for k in 1..=n {
                let u = k as f32 / n as f32;
                let x = x0 as f32 + (x1 - x0) as f32 * u;
                let y = y0 as f32 + (y1 - y0) as f32 * u + (u * (1.0 - u) * 10.0);
                let p = (x as i32, y as i32);
                f.line(last.0, last.1, p.0, p.1, rgb(248, 248, 248));
                last = p;
            }
            art.props.kid.draw(f, kx, GROUND_Y - 14, false);
            kite(f, x1, y1, t);
        }
    }

    // squirrel behind the dog's line
    let sq_draw = |f: &mut Frame, s: &Squirrel| {
        let x = sx(s.x, cam);
        let y = GROUND_Y + s.d as i32;
        let fr = if s.taunt > 0 && !s.gone { 2 } else { ((s.t / 5) % 2) as usize };
        f.shade_ellipse(x, y + 1, 5, 1, 5, 8);
        // it runs "into" the screen, so it faces the dog when taunting
        let flip = s.taunt > 0 && !s.gone && m.dog.x < s.x;
        draw(f, &art.props.squirrel[fr], x, y, flip || (s.gone && s.t % 2 == 3), &prop_pal("squirrel"));
        if s.taunt > 0 && !s.gone && (t / 8) % 2 == 0 {
            f.text_sh(x + 6, y - 16, "NYA!", rgb(248, 248, 248), OUTLINE, false);
        }
    };
    if let Some(s) = &m.squirrel {
        if s.d < 0.0 {
            sq_draw(f, s);
        }
    }

    // ---- the line: owners
    draw_waiting(f, m, art, cam, t);
    let ox = sx(OWNER_X, cam);
    let pose = owner_pose(m, true);
    f.shade_ellipse(ox, GROUND_Y + 1, 7, 2, 5, 8);
    blit(f, art.owners.get(pair, pose), ox, GROUND_Y + 1, false, &owner_pal(pair));
    if m.disc.state == DiscState::Held {
        let (hx, hy) = owner_hand(pose);
        let (px, py) = crate::engine::sprites::canvas_to_screen(hx, hy, ox, GROUND_Y + 1, false);
        draw_disc(f, px + 1, py, dcol, if pose == OwnerPose::Coil { -1 } else { 0 }, 0.0);
    }

    // ---- disc shadow
    if matches!(m.disc.state, DiscState::Flying | DiscState::Gulled) {
        let x = sx(m.disc.x, cam);
        let r = (5 - (m.disc.z / 30.0) as i32).max(2);
        f.shade_ellipse(x, GROUND_Y + 1, r, 1, 5, 8);
    }

    // ---- the dog
    let carried = m.dog.has_disc && m.disc.state == DiscState::Carried;
    let wet = m.dog.state == DogState::Shake && m.phase_t < 70 || m.dog.state == DogState::Swim;
    draw_dog(f, art, &m.dog, cam, t, side_color(m, side), dcol, carried, wet);
    if m.dog.state == DogState::Stare {
        let x = sx(m.dog.x, cam);
        bubble(f, x + 2, GROUND_Y - 38, if (t / 12) % 2 == 0 { "!" } else { "?!" });
        if m.human(side).is_some() && (t / 10) % 2 == 0 {
            f.text_center(x, GROUND_Y + 8, "TAP C!", rgb(248, 232, 64), OUTLINE, true);
        }
    }
    if m.dog.state == DogState::Lie && matches!(m.phase, Phase::Ready | Phase::Aim) {
        rain_cloud(f, sx(m.dog.x, cam), GROUND_Y - 36, t);
    }

    // ---- the disc
    let dx = sx(m.disc.x, cam);
    match m.disc.state {
        DiscState::Flying => {
            let wob = m.disc.wobble();
            let tilt = if wob > 0.15 { [(-1), 0, 1, 0][((t / 3) % 4) as usize] } else { 0 };
            // a short trail while it is fast
            if wob < 0.1 && m.disc.t > 2 {
                for k in 1..4 {
                    let tx = dx - (m.disc.vx * k as f32 * 2.0) as i32;
                    let ty = sy(m.disc.z - m.disc.vz * k as f32 * 2.0);
                    f.set(tx, ty, mix(dcol, 0xFFFFFF, 0.5));
                }
            }
            draw_disc(f, dx, sy(m.disc.z), dcol, tilt, m.disc.spin);
        }
        DiscState::Skid | DiscState::Stopped => draw_disc(f, dx, GROUND_Y - 1, dcol, 0, m.disc.spin),
        DiscState::Floating => {
            let bob = ((t / 15) % 2) as i32;
            draw_disc(f, dx, GROUND_Y - 4 + bob, dcol, 0, 0.0);
            f.hline(dx - 7, dx + 7, GROUND_Y - 2 + bob, rgb(200, 232, 248));
        }
        DiscState::Gulled => draw_disc(f, dx, sy(m.disc.z), dcol, 1, 0.0),
        _ => {}
    }

    if let Some(s) = &m.squirrel {
        if s.d >= 0.0 {
            sq_draw(f, s);
        }
    }

    // ---- the gull
    if let Some(g) = &m.gull {
        let fr = if g.has_disc || g.leaving { ((t / 5) % 3) as usize } else if (t / 8) % 2 == 0 { 1 } else { 0 };
        let (gx, gy) = (sx(g.x, cam), sy(g.z));
        draw(f, &art.props.gull[fr], gx, gy, g.vx > 0.0, &prop_pal("gull"));
        if !g.has_disc && !g.leaving && g.t < 40 && (t / 6) % 2 == 0 {
            bubble(f, gx, gy - 14, "!");
        }
    }

    // ---- effects
    for e in &m.fx {
        let (x, y) = (sx(e.x, cam), sy(e.z));
        match e.kind {
            FxKind::Dust => {
                let r = 1 + e.life * 3 / e.max.max(1);
                let c = match m.stage {
                    Stage::Pier => rgb(216, 176, 136),
                    _ => rgb(200, 232, 168),
                };
                f.fill_ellipse(x, y, r, r.max(1), c);
            }
            FxKind::Sparkle => {
                let c = if (e.life / 3) % 2 == 0 { rgb(248, 248, 200) } else { rgb(248, 216, 64) };
                f.set(x, y, c);
                f.set(x - 1, y, c);
                f.set(x + 1, y, c);
                f.set(x, y - 1, c);
                f.set(x, y + 1, c);
            }
            FxKind::Star => {
                f.set(x, y, rgb(248, 248, 96));
                f.set(x + 1, y, rgb(248, 200, 48));
            }
            FxKind::Drop => {
                f.set(x, y, rgb(160, 216, 248));
                f.set(x, y + 1, rgb(96, 168, 232));
            }
            FxKind::Feather => {
                f.hline(x - 1, x + 1, y, rgb(248, 248, 248));
                f.set(x + 2, y - 1, rgb(176, 184, 200));
            }
            FxKind::Crumb => f.set(x, y, rgb(200, 104, 72)),
            FxKind::Leaf => {
                f.set(x, y, rgb(72, 160, 64));
                f.set(x + 1, y + ((t / 6) % 2) as i32, rgb(48, 120, 48));
            }
        }
    }

    // ---- aim and power
    if matches!(m.phase, Phase::Aim | Phase::Power) {
        draw_aim(f, m, ox, t);
    }

    // ---- popups
    for p in &m.popups {
        if p.life > 8 || (p.life / 2) % 2 == 0 {
            let w = text_width(&p.text, true);
            let x = (sx(p.x, cam) - w / 2).clamp(2, W - w - 2);
            let y = sy(p.z).clamp(36, 200);
            f.text_grad(x, y, &p.text, &grad7(p.color), OUTLINE, true);
        }
    }

    // ---- things off screen
    if matches!(m.disc.state, DiscState::Flying | DiscState::Gulled) {
        let y = sy(m.disc.z);
        if y < 38 {
            let x = dx.clamp(8, W - 8);
            let blink = (t / 6) % 2 == 0;
            f.fill_poly(&[(x - 4, 42), (x + 4, 42), (x, 37)], if blink { rgb(248, 232, 64) } else { WHITE_C });
            let h = format!("{}", (m.disc.z / YD) as i32);
            f.text_center(x, 44, &h, rgb(248, 248, 248), OUTLINE, false);
        }
    }
    if m.phase == Phase::Flight {
        let x = sx(m.dog.x, cam);
        if x < -6 {
            arrow_left(f, 4, GROUND_Y - 12, t);
        } else if x > W + 6 {
            arrow_right(f, W - 5, GROUND_Y - 12, t);
        }
    }

    if m.shake > 0 {
        let d = if (m.shake / 2) % 2 == 0 { 2 } else { -2 };
        let src = f.clone();
        for y in 34..H {
            for x in 0..W {
                f.set(x, y, src.get((x - d).clamp(0, W - 1), y));
            }
        }
    }

    draw_hud(f, m, art, t);
    if let Some(b) = &m.banner {
        draw_banner(f, b);
    }
    if let Some(q) = &m.quip {
        draw_quip(f, q, art);
    }
}

const WHITE_C: u32 = rgb(248, 248, 248);

fn kite(f: &mut Frame, x: i32, y: i32, t: u32) {
    let pts = [(x, y - 9), (x + 6, y - 2), (x, y + 7), (x - 6, y - 2)];
    let out = [(x, y - 11), (x + 8, y - 2), (x, y + 9), (x - 8, y - 2)];
    f.fill_poly(&out, OUTLINE);
    f.fill_poly(&pts, rgb(232, 56, 64));
    f.fill_poly(&[(x, y - 9), (x + 6, y - 2), (x, y - 2)], rgb(248, 216, 48));
    f.fill_poly(&[(x, y - 2), (x - 6, y - 2), (x, y + 7)], rgb(248, 216, 48));
    // the tail flutters
    for k in 0..5 {
        let wob = (((t / 5) as i32 + k) % 3) - 1;
        let (tx, ty) = (x + wob - k, y + 9 + k * 3);
        f.set(tx, ty, rgb(248, 248, 248));
        if k % 2 == 0 {
            f.set(tx - 1, ty, rgb(64, 160, 232));
            f.set(tx + 1, ty, rgb(64, 160, 232));
        }
    }
}

fn bubble(f: &mut Frame, x: i32, y: i32, s: &str) {
    let w = text_width(s, true) + 6;
    f.rect(x - w / 2 - 1, y - 1, w + 2, 12, OUTLINE);
    f.rect(x - w / 2, y, w, 10, WHITE_C);
    f.fill_poly(&[(x - 2, y + 10), (x + 3, y + 10), (x, y + 14)], OUTLINE);
    f.fill_poly(&[(x - 1, y + 10), (x + 2, y + 10), (x, y + 12)], WHITE_C);
    f.text_bold(x - w / 2 + 3, y + 2, s, rgb(216, 40, 56));
}

fn rain_cloud(f: &mut Frame, x: i32, y: i32, t: u32) {
    f.fill_ellipse(x, y, 10, 4, OUTLINE);
    f.fill_ellipse(x - 4, y - 2, 6, 4, OUTLINE);
    f.fill_ellipse(x, y, 9, 3, rgb(120, 128, 152));
    f.fill_ellipse(x - 4, y - 2, 5, 3, rgb(144, 152, 176));
    for k in 0..4 {
        let ry = y + 5 + ((t as i32 / 2 + k * 5) % 14);
        f.set(x - 6 + k * 4, ry, rgb(120, 176, 248));
        f.set(x - 6 + k * 4, ry + 1, rgb(120, 176, 248));
    }
}

fn arrow_left(f: &mut Frame, x: i32, y: i32, t: u32) {
    let c = if (t / 6) % 2 == 0 { rgb(248, 232, 64) } else { WHITE_C };
    f.fill_poly(&[(x - 1, y), (x + 7, y - 6), (x + 7, y + 6)], OUTLINE);
    f.fill_poly(&[(x + 1, y), (x + 6, y - 4), (x + 6, y + 4)], c);
}

fn arrow_right(f: &mut Frame, x: i32, y: i32, t: u32) {
    let c = if (t / 6) % 2 == 0 { rgb(248, 232, 64) } else { WHITE_C };
    f.fill_poly(&[(x + 1, y), (x - 7, y - 6), (x - 7, y + 6)], OUTLINE);
    f.fill_poly(&[(x - 1, y), (x - 6, y - 4), (x - 6, y + 4)], c);
}

fn draw_aim(f: &mut Frame, m: &Match, ox: i32, t: u32) {
    let side = m.thrower();
    let human = m.human(side).is_some();
    // dotted arc showing the angle
    let (hx, hy) = (ox + 8, GROUND_Y - 20);
    let a = m.angle.to_radians();
    for k in 1..9 {
        let d = k as f32 * 5.0;
        let x = hx + (a.cos() * d) as i32;
        let y = hy - (a.sin() * d) as i32 + (k * k) as i32 / 12;
        let c = if (k + (t / 4) as i32) % 3 == 0 { rgb(248, 232, 64) } else { WHITE_C };
        f.rect(x - 1, y - 1, 3, 3, OUTLINE);
        f.set(x, y, c);
    }
    let s = format!("{}'", m.angle as i32);
    f.text_sh(hx + 30, hy - 30 - (m.angle * 0.4) as i32, &s, WHITE_C, OUTLINE, false);
    // the power meter
    let (mx, my, mh) = (ox - 22, GROUND_Y - 58, 48);
    f.rect(mx - 2, my - 2, 11, mh + 4, OUTLINE);
    f.rect(mx - 1, my - 1, 9, mh + 2, rgb(40, 40, 72));
    let top = (mh as f32 * (1.0 - PERFECT_FROM)) as i32;
    let v = if m.phase == Phase::Power { m.meter() } else { 0.0 };
    let fill = (v * mh as f32) as i32;
    for k in 0..fill {
        let y = my + mh - 1 - k;
        let u = k as f32 / mh as f32;
        let c = if u >= PERFECT_FROM {
            if (t / 3) % 2 == 0 { rgb(248, 248, 200) } else { rgb(248, 216, 48) }
        } else if u > 0.66 {
            rgb(248, 120, 48)
        } else if u > 0.33 {
            rgb(248, 200, 48)
        } else {
            rgb(96, 216, 72)
        };
        f.hline(mx, mx + 6, y, c);
    }
    // the perfect band
    f.rect_outline(mx - 1, my - 1, 9, top + 1, rgb(248, 216, 48));
    if v >= PERFECT_FROM && (t / 4) % 2 == 0 {
        f.text_sh(mx - 8, my - 12, "PERFECT", rgb(248, 248, 200), OUTLINE, false);
    }
    if m.c_held {
        f.text_sh(mx - 10, my + mh + 4, "FLOATER", rgb(160, 224, 248), OUTLINE, false);
    }
    if human && m.hint && (t / 20) % 2 == 0 {
        let msg = if m.phase == Phase::Aim { "UP/DOWN: AIM  HOLD Z: THROW" } else { "LET GO AT THE TOP!" };
        f.text_center(128, GROUND_Y + 12, msg, rgb(248, 248, 248), OUTLINE, false);
    }
}

/// A crop of the dog's head for the HUD.
pub fn dog_icon(f: &mut Frame, art: &Art, pair: usize, x: i32, y: i32, collar: u32, flip: bool) {
    let s = art.dogs.get(pair, DogFrame::Sit);
    let pal = dog_pal(pair, collar);
    // find the head: top rows, rightmost columns
    let mut y0 = s.h;
    let mut x1 = 0;
    for yy in 0..s.h {
        for xx in 0..s.w {
            if s.get(xx, yy) != T {
                y0 = y0.min(yy);
                x1 = x1.max(xx);
            }
        }
    }
    let (w, h) = (14, 13);
    for yy in 0..h {
        for xx in 0..w {
            let v = s.get(x1 - w + 1 + xx, y0 + yy);
            if v != T {
                let px = if flip { x + w - 1 - xx } else { x + xx };
                f.set(px, y + yy, pal[v as usize]);
            }
        }
    }
}

fn pip(f: &mut Frame, x: i32, y: i32, r: Option<&TossResult>, now: bool, t: u32) {
    f.rect(x - 1, y - 1, 9, 9, OUTLINE);
    match r {
        Some(r) if r.caught() => {
            let flip = matches!(r.outcome, Outcome::Catch(CatchKind::Flip));
            let jump = matches!(r.outcome, Outcome::Catch(CatchKind::Jump));
            let c = if flip { rgb(248, 96, 200) } else if jump { rgb(96, 224, 248) } else { rgb(96, 216, 72) };
            f.rect(x, y, 7, 7, c);
            f.rect(x, y, 7, 2, mix(c, 0xFFFFFF, 0.5));
            let n = match r.outcome {
                Outcome::Catch(k) => k.mult(),
                _ => 1,
            };
            if n > 1 {
                f.text(x + 1, y, &format!("{}", n), OUTLINE);
            }
        }
        Some(_) => {
            f.rect(x, y, 7, 7, rgb(72, 32, 48));
            f.line(x + 1, y + 1, x + 5, y + 5, rgb(248, 72, 72));
            f.line(x + 5, y + 1, x + 1, y + 5, rgb(248, 72, 72));
        }
        None => {
            let c = if now && (t / 8) % 2 == 0 { rgb(248, 232, 64) } else { rgb(48, 48, 88) };
            f.rect(x, y, 7, 7, c);
        }
    }
}

pub fn draw_hud(f: &mut Frame, m: &Match, art: &Art, t: u32) {
    f.rect(0, 0, W, 33, rgb(16, 16, 48));
    f.hline(0, W - 1, 33, rgb(248, 176, 32));
    f.hline(0, W - 1, 34, rgb(176, 32, 88));
    let active = m.thrower();
    for side in 0..2 {
        let pair = m.sides[side].pair;
        let col = side_color(m, side);
        let name = PAIRS[pair].dog.name;
        let short = if name.len() > 8 { &name[name.len() - 10..].trim_start() } else { name };
        let short = if pair == 2 { "SIR W." } else { short };
        let left = side == 0;
        let x0 = if left { 2 } else { 156 };
        // the active side's panel glows
        if active == side && m.phase != Phase::Over && (t / 10) % 2 == 0 {
            f.rect_outline(x0 - 1, 1, 100, 31, col);
        }
        let ix = if left { x0 + 2 } else { x0 + 82 };
        dog_icon(f, art, pair, ix, 4, col, !left);
        let tag = match m.sides[side].ctrl {
            Ctrl::Human(0) => "1P",
            Ctrl::Human(_) => "2P",
            Ctrl::Cpu => "CPU",
        };
        let tw = text_width(tag, false);
        let tx = if left { ix + 7 - tw / 2 } else { ix + 7 - tw / 2 };
        f.rect(tx - 1, 19, tw + 2, 9, col);
        f.text(tx, 20, tag, rgb(16, 16, 48));
        let nx = if left { x0 + 19 } else { x0 + 79 - text_width(short, true) };
        f.text_grad(nx, 3, short, &grad7(PAIRS[pair].color), OUTLINE, true);
        // score
        let total = format!("{}", m.sides[side].total());
        let sxp = if left { x0 + 19 + text_width(&total, true) / 2 + 6 } else { x0 + 79 - text_width(&total, true) / 2 - 6 };
        f.logo(sxp, 11, &total, 2, &grad6(rgb(248, 232, 96)), OUTLINE, rgb(8, 8, 24));
        // toss pips
        let n = (m.turns / 2).max(TOSSES);
        for k in 0..n.min(8) {
            let r = m.sides[side].results.get(k);
            let now = active == side && m.toss_no() == k + 1 && m.phase != Phase::Over;
            let px = if left { x0 + 52 + k as i32 * 9 } else { x0 + 40 - k as i32 * 9 };
            let px = if n > 5 { px - if left { 0 } else { 0 } } else { px };
            pip(f, px.clamp(x0, x0 + 92), 24, r, now, t);
        }
    }
    // centre: toss number and wind
    let toss = if m.sudden() { "SUDDEN".to_string() } else { format!("TOSS {}/{}", m.toss_no().min(TOSSES), TOSSES) };
    f.text_center(128, 3, &toss, rgb(160, 168, 216), OUTLINE, false);
    windsock(f, 116, 18, m.wind, t);
    let mph = (m.wind.abs() * 15.0).round() as i32;
    let ws = format!("{}", mph);
    f.text_sh(130, 14, &ws, rgb(248, 248, 248), OUTLINE, true);
    let arrow = if mph == 0 {
        "-"
    } else if m.wind > 0.0 {
        ">"
    } else {
        "<"
    };
    f.text_sh(130 + text_width(&ws, true) + 2, 14, arrow, if m.wind > 0.0 { rgb(120, 248, 120) } else { rgb(248, 120, 120) }, OUTLINE, true);
    f.text_center(128, 24, "MPH", rgb(120, 128, 176), OUTLINE, false);
    // zoomies meter while fetching
    if m.phase == Phase::Flight {
        let d = &m.dog;
        let k = (d.zoom / d.body.zoom_cap * 30.0) as i32;
        let (zx, zy) = (113, 31);
        f.rect(zx - 1, zy - 1, 32, 3, OUTLINE);
        let c = if d.zooming && (t / 3) % 2 == 0 { rgb(248, 248, 200) } else { rgb(248, 152, 32) };
        f.hline(zx, zx + k - 1, zy, c);
    }
}

pub fn windsock(f: &mut Frame, x: i32, y: i32, wind: f32, t: u32) {
    f.rect(x - 1, y - 9, 2, 14, OUTLINE);
    f.rect(x, y - 9, 1, 14, rgb(200, 200, 216));
    let dir = if wind >= 0.0 { 1 } else { -1 };
    let lift = (wind.abs() * 6.0).min(6.0) as i32;
    let flap = ((t / 5) % 2) as i32;
    for k in 0..4 {
        let sx0 = x + dir * (1 + k * 2);
        let droop = (6 - lift) * k / 3 + if k == 3 { flap } else { 0 };
        let c = if k % 2 == 0 { rgb(248, 96, 48) } else { rgb(248, 248, 248) };
        let hh = 3 - k / 2;
        f.rect(sx0.min(sx0 + dir), y - 9 + droop, 2, hh + 1, OUTLINE);
        f.rect(sx0.min(sx0 + dir), y - 8 + droop, 2, hh - 1, c);
    }
}

pub fn draw_banner(f: &mut Frame, b: &Banner) {
    let n = b.text.chars().count() as i32;
    let scale = if b.scale > 0 {
        b.scale
    } else if n * 18 < 236 {
        3
    } else {
        2
    };
    let target = 128;
    let y0 = 72;
    let (x, y) = if b.pop {
        (target, y0 - if b.t < 4 { 4 - b.t } else { 0 })
    } else {
        let x = if b.t < 10 { target + (10 - b.t) * 30 } else { target };
        (x, y0 + if (10..16).contains(&b.t) { (16 - b.t) / 2 } else { 0 })
    };
    // slide out at the end
    let x = if b.dur < 999 && b.t > b.dur - 8 { x - (b.t - (b.dur - 8)) * 34 } else { x };
    let grad: Vec<u32> = if b.color == rgb(248, 248, 248) { SUNSET.to_vec() } else { grad6(b.color).to_vec() };
    f.logo(x, y, &b.text, scale, &grad, OUTLINE, rgb(8, 8, 24));
}

/// Break text into lines of at most `n` characters, at spaces.
pub fn wrap(s: &str, n: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for w in s.split(' ') {
        if !cur.is_empty() && cur.len() + 1 + w.len() > n {
            lines.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(w);
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

/// The comment box at the bottom: who speaks, and the line typed out.
fn draw_quip(f: &mut Frame, q: &Quip, art: &Art) {
    let y = 199;
    f.rect(2, y, 252, 24, rgb(16, 16, 48));
    f.rect_outline(2, y, 252, 24, rgb(248, 248, 248));
    f.rect_outline(3, y + 1, 250, 22, rgb(96, 104, 168));
    match q.who {
        Some(pair) => {
            let s = art.owners.get(pair, OwnerPose::Stand);
            let pal = owner_pal(pair);
            for yy in 0..12 {
                for xx in 0..12 {
                    let v = s.get(6 + xx, yy);
                    if v != crate::engine::sprites::T {
                        f.set(7 + xx, y + 6 + yy, pal[v as usize]);
                    }
                }
            }
        }
        None => {
            // the announcer's microphone
            f.fill_ellipse(12, y + 9, 3, 4, OUTLINE);
            f.fill_ellipse(12, y + 9, 2, 3, rgb(176, 184, 200));
            f.rect(11, y + 13, 3, 6, OUTLINE);
            f.rect(12, y + 13, 1, 5, rgb(96, 96, 120));
        }
    }
    let shown = (q.t * 2).max(0) as usize;
    let lines = wrap(&q.text, 37);
    let mut left = shown;
    for (k, l) in lines.iter().take(2).enumerate() {
        let part: String = l.chars().take(left).collect();
        left = left.saturating_sub(l.len());
        f.text_sh(24, y + 4 + k as i32 * 9, &part, rgb(248, 248, 248), OUTLINE, false);
    }
}
