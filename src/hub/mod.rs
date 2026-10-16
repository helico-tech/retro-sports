//! The game selector: Helico Island. A little hiker walks a dotted path
//! between the venues; A or Start enters the game there.

mod audio;
mod map;

use crate::engine::gfx::*;
use crate::engine::input::{edge, Pad};
use crate::engine::rng::Rng;
use crate::engine::ui::*;
use crate::games::CARTS;
use map::*;
use std::f32::consts::TAU;

pub use audio::sounds;

/// Each game's venue: cart id, place name, index of its stop in `PATH_PTS`.
const VENUES: [(&str, &str, usize); 7] = [
    ("dodgeball", "BEACH COURT", 0),
    ("discdogs", "SUNNY HILL PARK", 3),
    ("zenuwen", "HELICODAM", 6),
    ("tienen", "HET TRAPVELDJE", 8),
    ("tegenwind", "THE POLDER", 10),
    ("trailmix", "MOUNT HELICO", 13),
    ("hangtime", "HELICO CLIFFS", 16),
];

const WALK: f32 = 1.3;
const IRIS: u32 = 32;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Phase {
    Roam,
    /// cheering, then the iris closes; then the game starts
    Enter(u32),
    /// iris opening after coming back from a game
    Back(u32),
}

/// A stop on the path: which cart, and how far along the path it is.
struct Stop {
    cart: usize,
    place: &'static str,
    s: f32,
}

pub struct Hub {
    /// The selected game (index into `CARTS`).
    pub cursor: usize,
    pub t: u32,
    pub sfx: Vec<usize>,
    /// Set when the player enters a game; the console takes it.
    pub start: Option<usize>,
    pub quit: bool,
    prev: [Pad; 2],
    phase: Phase,
    stops: Vec<Stop>,
    /// cumulative path length at each waypoint
    lens: Vec<f32>,
    stop: usize,
    /// hiker position along the path
    s: f32,
    face_left: bool,
    step_acc: f32,
    bg: Frame,
    glints: Vec<(i32, i32)>,
    rng: Rng,
    nessie_at: u32,
}

fn path_lens() -> Vec<f32> {
    let mut v = vec![0.0];
    for w in PATH_PTS.windows(2) {
        let (dx, dy) = ((w[1].0 - w[0].0) as f32, (w[1].1 - w[0].1) as f32);
        v.push(v.last().unwrap() + (dx * dx + dy * dy).sqrt());
    }
    v
}

impl Hub {
    pub fn new(seed: u64) -> Self {
        let lens = path_lens();
        let mut stops = Vec::new();
        for (id, place, k) in VENUES {
            if let Some(cart) = CARTS.iter().position(|c| c.id == id) {
                stops.push(Stop { cart, place, s: lens[k] });
            }
        }
        // a game without a venue yet waits at the end of the path
        for (cart, c) in CARTS.iter().enumerate() {
            if !VENUES.iter().any(|v| v.0 == c.id) {
                stops.push(Stop { cart, place: "NEW VENUE", s: *lens.last().unwrap() });
            }
        }
        let mut bg = Frame::new();
        let spots = map::build(&mut bg);
        let mut rng = Rng(seed | 1);
        let nessie_at = 900 + rng.int(1200) as u32;
        Hub {
            cursor: stops[0].cart,
            t: 0,
            sfx: Vec::new(),
            start: None,
            quit: false,
            prev: [Pad::default(); 2],
            phase: Phase::Roam,
            s: stops[0].s,
            stops,
            lens,
            stop: 0,
            face_left: false,
            step_acc: 0.0,
            bg,
            glints: spots.glints,
            rng,
            nessie_at,
        }
    }

    /// Back from game `i`: stand at its venue, iris opens.
    pub fn came_back(&mut self, i: usize) {
        if let Some(k) = self.stops.iter().position(|s| s.cart == i) {
            self.stop = k;
            self.s = self.stops[k].s;
            self.cursor = i;
        }
        self.phase = Phase::Back(0);
        self.prev = [Pad::default(); 2];
    }

    pub fn update(&mut self, pads: [Pad; 2], esc: bool) {
        self.t += 1;
        let e = [edge(pads[0], self.prev[0]), edge(pads[1], self.prev[1])];
        self.prev = pads;
        if self.t >= self.nessie_at + 420 {
            self.nessie_at = self.t + 1500 + self.rng.int(1800) as u32;
        }
        match self.phase {
            Phase::Enter(k) => {
                if k == 20 {
                    self.sfx.push(audio::ENTER);
                }
                if k >= 20 + IRIS {
                    self.start = Some(self.cursor);
                    self.phase = Phase::Roam;
                } else {
                    self.phase = Phase::Enter(k + 1);
                }
                return;
            }
            Phase::Back(k) => {
                self.phase = if k >= IRIS { Phase::Roam } else { Phase::Back(k + 1) };
            }
            Phase::Roam => {}
        }
        if esc {
            self.quit = true;
            return;
        }
        let n = self.stops.len();
        for p in e {
            if p.right || p.down {
                self.stop = (self.stop + 1) % n;
                self.sfx.push(audio::BLIP);
            } else if p.left || p.up {
                self.stop = (self.stop + n - 1) % n;
                self.sfx.push(audio::BLIP);
            }
        }
        self.cursor = self.stops[self.stop].cart;
        let target = self.stops[self.stop].s;
        let d = target - self.s;
        if d.abs() > 0.01 {
            let step = d.clamp(-WALK, WALK);
            self.s += step;
            self.face_left = self.dir_left(step);
            self.step_acc += step.abs();
            if self.step_acc > 9.0 {
                self.step_acc = 0.0;
                self.sfx.push(audio::STEP);
            }
            if (target - self.s).abs() < 0.01 {
                self.sfx.push(audio::ARRIVE);
            }
        } else if self.phase == Phase::Roam && (e[0].a || e[0].start || e[1].a || e[1].start) {
            self.phase = Phase::Enter(0);
            self.sfx.push(audio::ARRIVE);
        }
    }

    /// Standing at a venue and ready to take input that enters a game.
    pub fn ready(&self) -> bool {
        self.phase == Phase::Roam && !self.walking()
    }

    fn walking(&self) -> bool {
        (self.stops[self.stop].s - self.s).abs() > 0.01
    }

    /// Screen position of a point `s` along the path.
    fn at(&self, s: f32) -> (f32, f32) {
        let k = self.lens.iter().rposition(|&l| l <= s).unwrap_or(0).min(PATH_PTS.len() - 2);
        let seg = (self.lens[k + 1] - self.lens[k]).max(0.001);
        let u = ((s - self.lens[k]) / seg).clamp(0.0, 1.0);
        let (a, b) = (PATH_PTS[k], PATH_PTS[k + 1]);
        (a.0 as f32 + (b.0 - a.0) as f32 * u, a.1 as f32 + (b.1 - a.1) as f32 * u)
    }

    fn dir_left(&self, step: f32) -> bool {
        let (x0, _) = self.at(self.s - step);
        let (x1, _) = self.at(self.s);
        if (x1 - x0).abs() < 0.05 { self.face_left } else { x1 < x0 }
    }

    pub fn music(&self) -> Option<usize> {
        Some(audio::ISLAND)
    }

    // ---------------------------------------------------------------- draw

    pub fn draw(&self, f: &mut Frame) {
        let t = self.t;
        f.copy_from(&self.bg);
        self.draw_sea_life(f, t);
        self.draw_venues(f, t);
        self.draw_markers(f, t);
        self.draw_clouds(f, t);
        self.draw_hiker(f, t);
        self.draw_sky_life(f, t);
        self.draw_header(f, t);
        self.draw_panel(f, t);
        self.draw_iris(f);
    }

    fn draw_sea_life(&self, f: &mut Frame, t: u32) {
        for (k, &(x, y)) in self.glints.iter().enumerate() {
            let ph = (t / 6 + k as u32 * 7) % 24;
            if ph < 6 {
                let w = if ph < 3 { ph as i32 + 1 } else { 6 - ph as i32 };
                f.hline(x, x + w, y, FOAM);
            }
        }
        // foam pulses along the beach
        if (t / 20).is_multiple_of(2) {
            for x in (8..250).step_by(5) {
                for y in MAP_TOP..MAP_BOTTOM {
                    let d = island_d(x as f32, y as f32);
                    if (1.05..1.08).contains(&d) && is_sea(x, y) {
                        f.set(x, y, FOAM);
                        break;
                    }
                }
            }
        }
        // a sailboat drifts along the south coast
        let bx = ((t as f32 * 0.15) % 300.0) as i32 - 20;
        let by = 176 + ((t as f32 / 30.0).sin() * 1.0) as i32;
        f.fill_poly(&[(bx - 6, by - 1), (bx + 7, by - 1), (bx + 4, by + 2), (bx - 4, by + 2)], OUTLINE);
        f.fill_poly(&[(bx - 5, by), (bx + 6, by), (bx + 3, by + 1), (bx - 3, by + 1)], WOOD);
        f.fill_poly(&[(bx, by - 11), (bx, by - 2), (bx + 6, by - 2)], OUTLINE);
        f.fill_poly(&[(bx + 1, by - 9), (bx + 1, by - 3), (bx + 5, by - 3)], WALL);
        f.fill_poly(&[(bx - 1, by - 9), (bx - 1, by - 3), (bx - 5, by - 3)], rgb(232, 72, 72));
        f.hline(bx - 8, bx - 6, by + 2, FOAM);
        // Nessie, now and then, in the north-west bay
        if t >= self.nessie_at && t < self.nessie_at + 420 {
            let k = t - self.nessie_at;
            let rise = if k < 40 { k as i32 / 8 } else if k > 380 { (420 - k) as i32 / 8 } else { 5 };
            let (nx, ny) = (22 + (k / 8) as i32, 64);
            for (dx, h) in [(0, 2), (7, 3)] {
                let hh = (h * rise / 5).max(0);
                if hh > 0 {
                    f.fill_ellipse(nx + dx, ny, 3, hh, OUTLINE);
                    f.fill_ellipse(nx + dx, ny, 2, hh - 1, rgb(72, 176, 88));
                }
                f.hline(nx + dx - 4, nx + dx + 4, ny + 1, FOAM);
            }
            if rise >= 3 {
                let hy = ny - rise * 2;
                f.rect(nx + 12, hy, 3, rise * 2, OUTLINE);
                f.rect(nx + 13, hy + 1, 1, rise * 2 - 1, rgb(72, 176, 88));
                f.rect(nx + 12, hy - 2, 6, 4, OUTLINE);
                f.rect(nx + 13, hy - 1, 4, 2, rgb(96, 200, 104));
                f.set(nx + 15, hy - 1, OUTLINE);
                if k > 100 && k < 300 && (k / 40).is_multiple_of(2) {
                    bubble(f, nx + 16, hy - 12, "HI!");
                }
            }
        }
    }

    fn draw_venues(&self, f: &mut Frame, t: u32) {
        let tf = t as f32;
        // dodgeball: two tiny players and a ball flying between them
        let (cx, cy, cw, ch) = COURT;
        let u = (t % 70) as f32 / 70.0;
        let back = (t / 70) % 2 == 1;
        let (x0, x1) = ((cx + 6) as f32, (cx + cw - 6) as f32);
        let bx = if back { x1 + (x0 - x1) * u } else { x0 + (x1 - x0) * u };
        let by = (cy + ch / 2) as f32 - (u * TAU / 2.0).sin() * 9.0;
        tiny_person(f, cx + 5, cy + ch / 2 + 2, rgb(232, 64, 48), !back && u < 0.15);
        tiny_person(f, cx + cw - 5, cy + ch / 2 + 2, rgb(64, 160, 232), back && u < 0.15);
        f.shade_ellipse(bx as i32, cy + ch / 2 + 2, 1, 0, 5, 8);
        f.rect(bx as i32 - 1, by as i32 - 1, 3, 3, OUTLINE);
        f.set(bx as i32, by as i32, rgb(248, 64, 48));

        // disc dogs: a dog chases a disc across the meadow
        let k = (t % 150) as f32 / 150.0;
        let (sx, ex) = (52.0, 96.0);
        let disc_x = sx + (ex - sx) * k;
        let disc_y = 120.0 - (k * TAU / 2.0).sin() * 10.0;
        let dog_x = sx + (ex - sx) * (k * 1.08).min(1.0) - 6.0 + if k > 0.85 { 6.0 } else { 0.0 };
        let jump = if k > 0.8 && k < 0.95 { ((k - 0.8) / 0.15 * TAU / 2.0).sin() * 4.0 } else { 0.0 };
        if k < 0.9 {
            f.rect(disc_x as i32 - 2, disc_y as i32 - 1, 5, 3, OUTLINE);
            f.hline(disc_x as i32 - 1, disc_x as i32 + 1, disc_y as i32, rgb(248, 232, 64));
        }
        tiny_dog(f, dog_x as i32, 125 - jump as i32, t);

        // zenuwen: a thief rides off along the canal, a BOA runs after; Sjaak's boat waits
        let (cx, cy, cw) = CANAL;
        let k = (t % 320) as i32;
        let tx = cx + 2 + k * (cw - 4) / 320;
        tiny_bike(f, tx, cy + 8, t);
        if k > 50 {
            let bx = cx + 2 + (k - 50) * (cw - 14) / 270;
            tiny_person(f, bx, cy + 9, rgb(40, 56, 112), (t / 6).is_multiple_of(2));
            if (t / 10).is_multiple_of(2) {
                f.set(bx + 1, cy - 1, rgb(232, 64, 48));
                f.set(bx + 1, cy, rgb(232, 64, 48));
                f.set(bx + 1, cy + 2, rgb(232, 64, 48));
            }
        }
        let sx = cx + 16 + ((t / 40) % 6) as i32;
        f.rect(sx - 4, cy, 9, 3, OUTLINE);
        f.hline(sx - 3, sx + 3, cy + 1, rgb(176, 184, 192));
        f.set(sx - 1, cy - 1, rgb(248, 216, 48));
        f.set(sx + 1, cy - 1, rgb(232, 64, 48));

        // tienen: a cross, a header, the keeper dives the wrong way
        let (px, py, pw, ph) = PITCH;
        let k = (t % 180) as f32 / 180.0;
        let (kx, ky) = (px + pw - 4, py + ph - 2);
        let (hx, hy) = (px + 9, py + 5);
        let (bx, by) = if k < 0.5 {
            let u = k / 0.5;
            (kx as f32 + (hx - kx) as f32 * u, ky as f32 - 3.0 + (hy - ky) as f32 * u - (u * TAU / 2.0).sin() * 9.0)
        } else if k < 0.7 {
            let u = (k - 0.5) / 0.2;
            (hx as f32 + (px - hx) as f32 * u, (hy - 4) as f32 + 3.0 * u)
        } else {
            (px as f32 + 1.0, (py + ph / 2) as f32)
        };
        tiny_person(f, kx, ky, rgb(232, 64, 48), k < 0.1);
        tiny_person(f, hx, hy, rgb(248, 136, 24), (0.45..0.7).contains(&k));
        let dive = if k > 0.55 { 2 } else { 0 };
        tiny_person(f, px + 3, py + ph / 2 + 2 + dive, rgb(96, 176, 88), k > 0.55);
        f.rect(bx as i32 - 1, by as i32 - 1, 3, 3, OUTLINE);
        f.set(bx as i32, by as i32, rgb(248, 248, 248));
        if k > 0.7 && (t / 8).is_multiple_of(2) {
            f.text(px + 3, py + ph + 2, "GOAL!", rgb(248, 216, 48));
        }

        // tegenwind: windmill sails and a cyclist on the dike road
        let (mx, my) = MILL;
        let (hx, hy) = (mx, my - 2);
        let a0 = tf * 0.05;
        for s in 0..4 {
            let a = a0 + s as f32 * TAU / 4.0;
            let (ex, ey) = (hx + (a.cos() * 11.0) as i32, hy + (a.sin() * 11.0) as i32);
            f.thick_line(hx, hy, ex, ey, 3, OUTLINE);
        }
        for s in 0..4 {
            let a = a0 + s as f32 * TAU / 4.0;
            let (ex, ey) = (hx + (a.cos() * 10.0) as i32, hy + (a.sin() * 10.0) as i32);
            let (mx2, my2) = (hx + (a.cos() * 3.0) as i32, hy + (a.sin() * 3.0) as i32);
            f.line(mx2, my2, ex, ey, WALL);
        }
        f.rect(hx - 1, hy - 1, 3, 3, OUTLINE);
        f.set(hx, hy, WOOD);
        let rx = 168 - ((t as f32 * 0.35) % 90.0) as i32;
        tiny_bike(f, rx, 154, t);

        // trail mix: a goat bounces up the trail; a flag flutters on top
        let pts = [(186, 100), (200, 94), (184, 86), (202, 78), (190, 70), (196, 62)];
        let gk = (t / 4) as usize % 200;
        let seg = (gk / 40).min(pts.len() - 2);
        let gu = (gk % 40) as f32 / 40.0;
        let (gx, gy) = (
            pts[seg].0 as f32 + (pts[seg + 1].0 - pts[seg].0) as f32 * gu,
            pts[seg].1 as f32 + (pts[seg + 1].1 - pts[seg].1) as f32 * gu,
        );
        let hop = ((t as f32 * 0.4).sin().abs() * 2.0) as i32;
        tiny_goat(f, gx as i32, gy as i32 - hop, pts[seg + 1].0 < pts[seg].0);
        let (px, py) = PEAK;
        f.rect(px, py - 9, 1, 9, OUTLINE);
        let wave = ((t / 8) % 2) as i32;
        f.rect(px + 1, py - 9, 5, 3 + wave, OUTLINE);
        f.rect(px + 1, py - 8, 4, 1 + wave, rgb(232, 56, 56));

        // hang time: a glider soars in a figure eight over the cliffs
        let a = tf * 0.018;
        let (gx, gy) = (224.0 + a.sin() * 16.0, 98.0 + (2.0 * a).sin() * 5.0);
        let left = a.cos() < 0.0;
        tiny_glider(f, gx as i32, gy as i32, left);
        // lighthouse lamp
        let (lx, ly) = LIGHTHOUSE;
        if (t / 30).is_multiple_of(2) {
            f.rect(lx - 1, ly - 17, 3, 2, rgb(248, 248, 200));
            for k in 0..6 {
                f.set(lx - 4 - k * 2, ly - 16 + (k % 2), rgb(248, 248, 160));
            }
        }
    }

    /// A pad and a pennant at every venue, so you can see where games are.
    fn draw_markers(&self, f: &mut Frame, t: u32) {
        for (k, st) in self.stops.iter().enumerate() {
            let (x, y) = self.at(st.s);
            let (x, y) = (x as i32, y as i32);
            let c = venue_grad(CARTS[st.cart].id)[2];
            let sel = k == self.stop;
            f.fill_ellipse(x, y, 6, 3, OUTLINE);
            f.fill_ellipse(x, y, 5, 2, if sel && (t / 8).is_multiple_of(2) { rgb(248, 248, 168) } else { c });
            f.fill_ellipse(x - 1, y - 1, 2, 0, rgb(248, 248, 248));
            let (px, py) = (x + 7, y - 1);
            f.rect(px - 1, py - 12, 3, 13, OUTLINE);
            f.rect(px, py - 11, 1, 12, WALL);
            let wave = ((t / 10 + k as u32) % 2) as i32;
            f.fill_poly(&[(px + 1, py - 13), (px + 8, py - 10 + wave), (px + 1, py - 6)], OUTLINE);
            f.fill_poly(&[(px + 1, py - 12), (px + 6, py - 10 + wave), (px + 1, py - 8)], c);
        }
    }

    fn draw_clouds(&self, f: &mut Frame, t: u32) {
        for k in 0..3u32 {
            let speed = 0.06 + k as f32 * 0.025;
            let x = ((t as f32 * speed + k as f32 * 110.0) % 340.0) as i32 - 50;
            let y = 62 + k as i32 * 40;
            // shadow on the ground first
            f.shade_ellipse(x + 8, y + 16, 11, 3, 13, 16);
            cloud(f, x, y, 6 + k as i32 % 2 * 2);
        }
    }

    fn draw_sky_life(&self, f: &mut Frame, t: u32) {
        // gulls wheeling over the sea
        for k in 0..3 {
            let a = t as f32 * 0.012 + k as f32 * 2.1;
            let (cx, cy) = ([40.0, 150.0, 230.0][k], [70.0, 168.0, 60.0][k]);
            let (x, y) = (cx + a.cos() * 18.0, cy + a.sin() * 6.0);
            let up = ((t / 8) + k as u32).is_multiple_of(2);
            gull(f, x as i32, y as i32, up);
        }
        // the blimp crosses every so often, trailing a banner
        let period = 1500;
        let k = t % period;
        if k < 700 {
            let x = 290 - (k as f32 * 0.5) as i32;
            blimp(f, x, 52 + ((k as f32 / 40.0).sin() * 2.0) as i32);
        }
    }

    fn hiker_pos(&self) -> (i32, i32) {
        let (x, y) = self.at(self.s);
        (x as i32, y as i32)
    }

    fn draw_hiker(&self, f: &mut Frame, t: u32) {
        let (x, y) = self.hiker_pos();
        let pose = match self.phase {
            Phase::Enter(_) => Hero::Cheer,
            _ if self.walking() => {
                if (t / 6).is_multiple_of(2) { Hero::Walk1 } else { Hero::Walk2 }
            }
            _ => Hero::Stand,
        };
        let hop = match self.phase {
            Phase::Enter(k) if k < 20 => ((k as f32 / 20.0 * TAU / 2.0).sin() * 6.0) as i32,
            _ => 0,
        };
        f.shade_ellipse(x, y + 1, 4, 1, 1, 2);
        hero(f, x, y - hop, pose, self.face_left);
        // the venue's name, and an invitation
        if !self.walking() && self.phase == Phase::Roam {
            let st = &self.stops[self.stop];
            let txt = if (t / 40).is_multiple_of(2) { st.place } else { "PUSH START" };
            let bob = ((t / 10) % 2) as i32;
            bubble(f, x, y - 26 - bob, txt);
        }
    }

    fn draw_header(&self, f: &mut Frame, t: u32) {
        f.band_gradient(0, 34, &[rgb(40, 72, 176), rgb(64, 112, 208), rgb(96, 152, 232), rgb(136, 192, 240), rgb(176, 224, 248)]);
        let st = self.t as i32;
        let y = if st < 20 { -40 + st * 43 / 20 } else if st < 30 { 3 - (5 - ((st - 25).abs())) } else { 3 };
        f.logo(122, y, "RETRO SPORTS", 3, &SUNSET, OUTLINE, LOGO_SHADOW);
        f.rect(0, 34, W, 10, rgb(24, 16, 56));
        f.hline(0, W - 1, 33, rgb(248, 176, 32));
        f.hline(0, W - 1, 44, rgb(248, 176, 32));
        let _ = t;
        f.text_grad_center(128, 36, "GREETINGS FROM HELICO ISLAND!", &grad7(rgb(96, 224, 248)), OUTLINE, true);
    }

    fn draw_panel(&self, f: &mut Frame, t: u32) {
        f.rect(0, MAP_BOTTOM, W, H - MAP_BOTTOM, rgb(16, 16, 48));
        f.hline(0, W - 1, MAP_BOTTOM, rgb(248, 176, 32));
        let cart = &CARTS[self.cursor];
        let grad = venue_grad(cart.id);
        f.logo(128, MAP_BOTTOM + 2, cart.title, 2, &grad, OUTLINE, LOGO_SHADOW);
        let line = format!("{}  -  1 OR 2 PLAYERS", cart.sport);
        f.text_center(128, MAP_BOTTOM + 25, &line, rgb(160, 168, 216), OUTLINE, false);
        // ticker: the game's blurb, then island news
        f.rect(0, 213, W, 11, rgb(8, 8, 32));
        let msg = format!(
            "{}      ARROWS: WALK   Z OR ENTER: PLAY   ESC: BACK      ISLAND NEWS: SEA MONSTER SPOTTED AGAIN. TOURIST BOARD DENIES EVERYTHING.      ",
            cart.blurb
        );
        let w = text_width(&msg, false) + 6;
        let x = W - (t as i32 / 2) % (w + W);
        f.text(x, 215, &msg, rgb(248, 224, 72));
        // small arrows hint that there is more to the left and right
        if (t / 20).is_multiple_of(2) {
            f.text(6, MAP_BOTTOM + 10, "<", rgb(248, 176, 32));
            f.text(W - 11, MAP_BOTTOM + 10, ">", rgb(248, 176, 32));
        }
    }

    fn draw_iris(&self, f: &mut Frame) {
        let r = match self.phase {
            Phase::Enter(k) if k >= 20 => 300.0 * (1.0 - (k - 20) as f32 / IRIS as f32),
            Phase::Back(k) => 300.0 * k as f32 / IRIS as f32,
            _ => return,
        };
        let (cx, cy) = self.hiker_pos();
        let (cx, cy) = (cx as f32, cy as f32 - 8.0);
        let r2 = r * r;
        for y in 0..H {
            for x in 0..W {
                let (dx, dy) = (x as f32 - cx, y as f32 - cy);
                if dx * dx + dy * dy > r2 {
                    f.px[(y * W + x) as usize] = 0;
                }
            }
        }
    }
}

/// Logo colors per venue, for the panel.
fn venue_grad(id: &str) -> [u32; 6] {
    match id {
        "discdogs" => grad6(rgb(248, 200, 56)),
        "tegenwind" => grad6(rgb(248, 136, 32)),
        "trailmix" => grad6(rgb(120, 216, 88)),
        "hangtime" => grad6(rgb(96, 200, 248)),
        "zenuwen" => grad6(rgb(232, 88, 104)),
        "tienen" => grad6(rgb(248, 248, 248)),
        _ => SUNSET,
    }
}

fn bubble(f: &mut Frame, cx: i32, y: i32, s: &str) {
    let w = text_width(s, false) + 6;
    let x = (cx - w / 2).clamp(2, W - w - 2);
    f.rect(x - 1, y - 1, w + 2, 11, OUTLINE);
    f.rect(x, y, w, 9, rgb(248, 248, 240));
    let tx = cx.clamp(x + 3, x + w - 4);
    f.fill_poly(&[(tx - 3, y + 9), (tx + 3, y + 9), (tx, y + 13)], OUTLINE);
    f.fill_poly(&[(tx - 2, y + 8), (tx + 2, y + 8), (tx, y + 11)], rgb(248, 248, 240));
    f.text(x + 3, y + 1, s, OUTLINE);
}

/// A see-through cloud: dithered, so the venues under it stay visible.
fn cloud(f: &mut Frame, x: i32, y: i32, r: i32) {
    let blobs = [(0, 0, r), (r, -r / 2, r - 1), (2 * r, 0, r - 2)];
    let inside = |px: i32, py: i32, grow: i32| {
        blobs.iter().any(|&(dx, dy, rr)| {
            let (ex, ey) = ((px - x - dx) as f32 / (rr + grow) as f32, (py - y - dy) as f32 / (rr / 2 + 1 + grow) as f32);
            ex * ex + ey * ey <= 1.0
        })
    };
    for py in y - r - 2..y + r {
        for px in x - r - 2..x + 3 * r + 2 {
            if !inside(px, py, 0) || (px + py) & 1 == 1 {
                continue;
            }
            let lit = !inside(px, py + 2, 0);
            f.set(px, py, if lit { rgb(200, 208, 232) } else { rgb(248, 248, 248) });
        }
    }
}

fn gull(f: &mut Frame, x: i32, y: i32, up: bool) {
    let c = rgb(248, 248, 248);
    if up {
        f.set(x - 2, y - 1, c);
        f.set(x - 1, y, c);
        f.set(x, y, OUTLINE);
        f.set(x + 1, y, c);
        f.set(x + 2, y - 1, c);
    } else {
        f.hline(x - 2, x + 2, y, c);
        f.set(x, y, OUTLINE);
    }
}

fn blimp(f: &mut Frame, x: i32, y: i32) {
    // banner on a rope behind it
    let text = "HELICO SOFT";
    let bw = text_width(text, false) + 6;
    f.line(x + 14, y, x + 22, y, OUTLINE);
    f.rect(x + 22, y - 5, bw + 2, 11, OUTLINE);
    f.rect(x + 23, y - 4, bw, 9, rgb(248, 232, 64));
    f.text(x + 26, y - 3, text, rgb(176, 32, 88));
    f.fill_ellipse(x, y, 15, 6, OUTLINE);
    f.fill_ellipse(x, y, 14, 5, rgb(200, 200, 216));
    f.fill_ellipse(x - 2, y - 2, 10, 2, rgb(232, 232, 248));
    f.hline(x - 12, x + 12, y, rgb(232, 64, 64));
    f.fill_poly(&[(x + 11, y - 2), (x + 17, y - 6), (x + 17, y - 1)], OUTLINE);
    f.fill_poly(&[(x + 11, y + 2), (x + 17, y + 6), (x + 17, y + 1)], OUTLINE);
    f.rect(x - 3, y + 5, 6, 3, OUTLINE);
    f.rect(x - 2, y + 6, 4, 1, rgb(248, 248, 160));
}

fn tiny_person(f: &mut Frame, x: i32, y: i32, shirt: u32, arm_up: bool) {
    f.rect(x - 1, y - 6, 3, 7, OUTLINE);
    f.set(x, y - 5, rgb(248, 192, 144));
    f.rect(x, y - 4, 1, 2, shirt);
    f.set(x, y - 1, rgb(48, 48, 120));
    if arm_up {
        f.set(x + 1, y - 6, rgb(248, 192, 144));
    }
}

fn tiny_dog(f: &mut Frame, x: i32, y: i32, t: u32) {
    let leg = (t / 4).is_multiple_of(2);
    f.rect(x - 3, y - 4, 7, 4, OUTLINE);
    f.rect(x - 2, y - 3, 5, 2, rgb(200, 128, 64));
    f.rect(x + 2, y - 5, 3, 3, OUTLINE);
    f.set(x + 3, y - 4, rgb(200, 128, 64));
    f.set(x - 3, y - 4 - (t / 6 % 2) as i32, OUTLINE);
    f.set(x - 2 + leg as i32, y, OUTLINE);
    f.set(x + 2 - leg as i32, y, OUTLINE);
}

fn tiny_bike(f: &mut Frame, x: i32, y: i32, t: u32) {
    for wx in [x - 3, x + 3] {
        f.rect(wx - 1, y - 1, 3, 3, OUTLINE);
        f.set(wx, y, rgb(200, 200, 200));
    }
    f.line(x - 3, y, x + 3, y, OUTLINE);
    f.rect(x - 1, y - 6, 3, 5, OUTLINE);
    f.set(x, y - 5, rgb(248, 192, 144));
    f.rect(x, y - 4, 1, 2, rgb(248, 136, 32));
    f.set(x - 1 + ((t / 5) % 2) as i32 * 2, y - 1, rgb(48, 48, 120));
}

fn tiny_goat(f: &mut Frame, x: i32, y: i32, left: bool) {
    let d = if left { -1 } else { 1 };
    f.rect(x - 3, y - 4, 7, 4, OUTLINE);
    f.rect(x - 2, y - 3, 5, 2, rgb(240, 240, 232));
    f.rect(x + 2 * d - 1, y - 6, 3, 3, OUTLINE);
    f.set(x + 2 * d, y - 5, rgb(240, 240, 232));
    f.set(x + 2 * d - d, y - 7, OUTLINE);
}

fn tiny_glider(f: &mut Frame, x: i32, y: i32, left: bool) {
    let d = if left { -1 } else { 1 };
    f.fill_poly(&[(x - 7, y + 1), (x + 4 * d, y - 3), (x + 7, y + 1)], OUTLINE);
    f.fill_poly(&[(x - 6, y), (x + 4 * d, y - 2), (x + 6, y)], rgb(248, 72, 136));
    f.fill_poly(&[(x - 6, y), (x + 4 * d, y - 2), (x, y)], rgb(248, 232, 64));
    f.line(x, y, x, y + 3, OUTLINE);
    f.rect(x - 2, y + 3, 4, 2, OUTLINE);
    f.set(x - d, y + 3, rgb(64, 96, 224));
}

#[derive(Clone, Copy)]
enum Hero {
    Stand,
    Walk1,
    Walk2,
    Cheer,
}

/// The island hiker: cap, backpack, 12x16, facing right; feet at (x, y).
fn hero(f: &mut Frame, x: i32, y: i32, pose: Hero, flip: bool) {
    const STAND: [&str; 16] = [
        "...OOOOO....",
        "..OCCCCCO...",
        ".OCCCCCCOOO.",
        ".OccccCCCCCO",
        "OBOOSSSSOOO.",
        "OBOsSSKSO...",
        "OBOSSSSSSO..",
        "OBBOsSSSO...",
        "OBOTTTTTO...",
        "OBOTTTTTSO..",
        ".OOTTTTOSO..",
        "..OPPPPPOO..",
        "..OPPOPPO...",
        "..OPPOPPO...",
        "..OWWOWWO...",
        "..OOOOOOOO..",
    ];
    const WALK1: [&str; 16] = [
        "...OOOOO....",
        "..OCCCCCO...",
        ".OCCCCCCOOO.",
        ".OccccCCCCCO",
        "OBOOSSSSOOO.",
        "OBOsSSKSO...",
        "OBOSSSSSSO..",
        "OBBOsSSSO...",
        "OBOTTTTTOO..",
        "OBOTTTTTOSO.",
        ".OOTTTTO.OO.",
        "..OPPPPPO...",
        ".OPPOOPPO...",
        "OPPO..OPPO..",
        "OWWO..OWWO..",
        "OOOO..OOOO..",
    ];
    const WALK2: [&str; 16] = [
        "............",
        "...OOOOO....",
        "..OCCCCCO...",
        ".OCCCCCCOOO.",
        ".OccccCCCCCO",
        "OBOOSSSSOOO.",
        "OBOsSSKSO...",
        "OBOSSSSSSO..",
        "OBBOsSSSO...",
        "OBOTTTTTO...",
        "OBOTTTTSO...",
        ".OOTTTTSO...",
        "..OPPPPPO...",
        "...OPPPO....",
        "...OWWWO....",
        "...OOOOO....",
    ];
    const CHEER: [&str; 16] = [
        "OSO.OOOOO.OS",
        "OSOOCCCCCOOS",
        ".OSCCCCCCOSO",
        ".OSccCCCCSO.",
        "OBOSSSSSSO..",
        "OBOsSSKSO...",
        "OBOSSKKSO...",
        "OBBOsSSSO...",
        "OBOTTTTTO...",
        "OBOTTTTTO...",
        ".OOTTTTO....",
        "..OPPPPPO...",
        "..OPPOPPO...",
        "..OPPOPPO...",
        "..OWWOWWO...",
        "..OOOOOOOO..",
    ];
    let rows = match pose {
        Hero::Stand => &STAND,
        Hero::Walk1 => &WALK1,
        Hero::Walk2 => &WALK2,
        Hero::Cheer => &CHEER,
    };
    for (ry, row) in rows.iter().enumerate() {
        for (rx, ch) in row.bytes().enumerate() {
            let c = match ch {
                b'O' => OUTLINE,
                b'C' => rgb(232, 56, 56),
                b'c' => rgb(168, 32, 48),
                b'S' => rgb(248, 192, 144),
                b's' => rgb(208, 144, 104),
                b'K' => OUTLINE,
                b'T' => rgb(248, 216, 64),
                b'B' => rgb(64, 120, 200),
                b'P' => rgb(64, 72, 152),
                b'W' => rgb(248, 248, 248),
                _ => continue,
            };
            let dx = rx as i32 - 6;
            let px = if flip { x - dx } else { x + dx };
            f.set(px, y - 16 + ry as i32, c);
        }
    }
}

// ---------------------------------------------------------------- headless

pub fn shots(dir: &str) {
    use crate::engine::png::save_png;
    let dir = std::path::Path::new(dir);
    std::fs::create_dir_all(dir).unwrap();
    let mut hub = Hub::new(42);
    let snap = |hub: &Hub, name: &str| {
        let mut f = Frame::new();
        hub.draw(&mut f);
        save_png(&f, &dir.join(name), 3);
        println!("wrote hub/{name}");
    };
    let idle = |hub: &mut Hub, n: u32| {
        for _ in 0..n {
            hub.update([Pad::default(); 2], false);
        }
    };
    let press = |hub: &mut Hub, p: Pad| {
        hub.update([p, Pad::default()], false);
        hub.update([Pad::default(); 2], false);
    };
    idle(&mut hub, 12);
    snap(&hub, "01_logo_drop.png");
    idle(&mut hub, 80);
    snap(&hub, "02_island.png");
    for k in 1..hub.stops.len() {
        press(&mut hub, Pad { right: true, ..Pad::default() });
        idle(&mut hub, 20);
        if k == 1 {
            snap(&hub, "03_walking.png");
        }
        idle(&mut hub, 100);
        snap(&hub, &format!("{:02}_{}.png", 3 + k, CARTS[hub.cursor].id));
    }
    press(&mut hub, Pad { start: true, ..Pad::default() });
    idle(&mut hub, 10);
    snap(&hub, "10_enter_cheer.png");
    idle(&mut hub, 30);
    snap(&hub, "11_enter_iris.png");
    // the rare visitors
    let mut hub = Hub::new(42);
    hub.t = hub.nessie_at + 150;
    hub.update([Pad::default(); 2], false);
    snap(&hub, "12_nessie.png");
    let mut hub = Hub::new(42);
    hub.t = 300;
    hub.update([Pad::default(); 2], false);
    snap(&hub, "13_blimp.png");
}
