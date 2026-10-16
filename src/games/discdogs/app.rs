//! Scenes and flow: title, menu, attract demo, pup select, intro card,
//! play, pause, results tally, continue, game over and the champion screen.

use super::audio::Track;
use super::data::*;
use super::render::*;
use super::scenery::{Scenery, GROUND_Y};
use super::sim::*;
use super::sprites::*;
use crate::engine::gfx::*;
use crate::engine::input::any;
use crate::engine::sprites::blit;
use crate::engine::ui::{grad6, grad7, OUTLINE, SUNSET};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    One,
    Two,
    Demo,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scene {
    Title,
    Select,
    Intro,
    Play,
    Results,
    Continue,
    GameOver,
    Champion,
}

/// CPU skill per ladder step.
pub const CPU_SKILL: [f32; 3] = [0.4, 0.62, 0.84];

pub struct App {
    pub scene: Scene,
    pub t: u32,
    pub mode: Mode,
    pub menu: usize,
    pub menu_open: bool,
    pub cursor: [usize; 2],
    pub locked: [bool; 2],
    /// opponents in 1P, in ladder order
    pub ladder: Vec<usize>,
    /// ladder step in 1P (0..3)
    pub stage: usize,
    /// the stage picked for a 2P match
    pub stage2p: usize,
    pub game: Option<Match>,
    pub scenery: Option<Scenery>,
    pub art: Art,
    title_bg: Scenery,
    pub sfx: Vec<Sfx>,
    prev: [Pad; 2],
    rng: Rng,
    pub quit: bool,
    pub paused: bool,
    idle: u32,
    pub count: i32,
    pub frame: u32,
    confetti: Vec<(f32, f32, f32, u32)>,
    /// the finished match, for the results screen
    pub last: Option<Match>,
    demo_tip: usize,
}

impl App {
    pub fn new(seed: u64) -> Self {
        let blankets = [(150.0, 30.0, rgb(232, 56, 64))];
        App {
            scene: Scene::Title,
            t: 0,
            mode: Mode::One,
            menu: 0,
            menu_open: false,
            cursor: [0, 1],
            locked: [false; 2],
            ladder: Vec::new(),
            stage: 0,
            stage2p: 0,
            game: None,
            scenery: None,
            art: Art::new(),
            title_bg: Scenery::new(Stage::Park, Some((330.0, 374.0)), &blankets, 7),
            sfx: Vec::new(),
            prev: [Pad::default(); 2],
            rng: Rng(seed | 1),
            quit: false,
            paused: false,
            idle: 0,
            count: 0,
            frame: 0,
            confetti: Vec::new(),
            last: None,
            demo_tip: 0,
        }
    }

    pub fn music(&self) -> Option<Track> {
        match self.scene {
            Scene::Title | Scene::Select | Scene::Intro => Some(Track::Title),
            Scene::Play => Some(Track::Play),
            Scene::Results => Some(Track::Results),
            Scene::Champion => Some(Track::Champion),
            Scene::Continue | Scene::GameOver => None,
        }
    }

    fn go(&mut self, s: Scene) {
        self.scene = s;
        self.t = 0;
    }

    pub fn start_match(&mut self, pairs: [usize; 2], ctrls: [Ctrl; 2], skills: [f32; 2], stage: Stage) {
        let seed = self.rng.next() as u64;
        let m = Match::new(pairs, ctrls, skills, stage, seed);
        let blankets: Vec<(f32, f32, u32)> = m.blankets.iter().map(|b| (b.x, b.w, b.color)).collect();
        self.scenery = Some(Scenery::new(stage, m.pond, &blankets, seed ^ 0x55));
        self.game = Some(m);
        self.paused = false;
        self.go(Scene::Play);
    }

    pub fn start_demo(&mut self) {
        let a = self.rng.int(4) as usize;
        let b = (a + 1 + self.rng.int(3) as usize) % 4;
        let stage = STAGES[self.rng.int(3) as usize];
        self.mode = Mode::Demo;
        self.demo_tip = self.rng.int(DEMO_TIPS.len() as i32) as usize;
        self.start_match([a, b], [Ctrl::Cpu, Ctrl::Cpu], [0.7, 0.7], stage);
    }

    fn start_stage(&mut self) {
        let opp = self.ladder[self.stage];
        self.start_match([self.cursor[0], opp], [Ctrl::Human(0), Ctrl::Cpu], [0.5, CPU_SKILL[self.stage]], stage_for(self.stage));
    }

    /// The two pairs on the intro card and the stage.
    pub fn matchup(&self) -> (usize, usize, Stage) {
        match self.mode {
            Mode::One => (self.cursor[0], self.ladder.get(self.stage).copied().unwrap_or(1), stage_for(self.stage)),
            _ => (self.cursor[0], self.cursor[1], STAGES[self.stage2p % 3]),
        }
    }

    pub fn update(&mut self, pads: [Pad; 2], esc: bool) {
        self.frame = self.frame.wrapping_add(1);
        self.t += 1;
        let e = [edge(pads[0], self.prev[0]), edge(pads[1], self.prev[1])];
        self.prev = pads;
        let ok = |p: Pad| p.a || p.start;
        if any(e[0]) || any(e[1]) {
            self.idle = 0;
        } else {
            self.idle += 1;
        }

        if esc {
            match self.scene {
                Scene::Title => self.quit = true,
                _ => {
                    self.game = None;
                    self.menu_open = false;
                    self.paused = false;
                    self.go(Scene::Title);
                }
            }
            return;
        }

        match self.scene {
            Scene::Title => {
                if !self.menu_open {
                    if ok(e[0]) && self.t > 20 {
                        self.menu_open = true;
                        self.sfx.push(Sfx::Confirm);
                    }
                } else {
                    if e[0].up || e[0].down {
                        self.menu = 1 - self.menu;
                        self.sfx.push(Sfx::Blip);
                    }
                    if e[0].b {
                        self.menu_open = false;
                    }
                    if ok(e[0]) {
                        self.mode = if self.menu == 0 { Mode::One } else { Mode::Two };
                        self.locked = [false; 2];
                        self.cursor = [0, 1];
                        self.sfx.push(Sfx::Confirm);
                        self.go(Scene::Select);
                    }
                }
                if self.idle > 900 {
                    self.menu_open = false;
                    self.start_demo();
                }
            }
            Scene::Select => {
                let players = if self.mode == Mode::Two { 2 } else { 1 };
                for pi in 0..players {
                    let p = e[pi];
                    if !self.locked[pi] {
                        let c = self.cursor[pi];
                        let n = match (p.left, p.right, p.up, p.down) {
                            (true, ..) | (_, true, ..) => Some(c ^ 1),
                            (_, _, true, _) | (_, _, _, true) => Some(c ^ 2),
                            _ => None,
                        };
                        if let Some(n) = n {
                            self.cursor[pi] = n;
                            self.sfx.push(Sfx::Blip);
                        }
                        if ok(p) {
                            let other = 1 - pi;
                            if players == 2 && self.locked[other] && self.cursor[other] == self.cursor[pi] {
                                self.sfx.push(Sfx::WahWah);
                            } else {
                                self.locked[pi] = true;
                                self.sfx.push(Sfx::Bark);
                            }
                        }
                        if p.b && pi == 0 {
                            self.go(Scene::Title);
                            self.menu_open = true;
                            return;
                        }
                    } else if p.b {
                        self.locked[pi] = false;
                    }
                }
                if self.locked[..players].iter().all(|&l| l) && self.t > 10 {
                    if self.mode == Mode::One {
                        let me = self.cursor[0];
                        self.ladder = (1..4).map(|k| (me + k) % 4).collect();
                        self.stage = 0;
                    }
                    self.go(Scene::Intro);
                }
            }
            Scene::Intro => {
                if self.mode == Mode::Two && (e[0].left || e[0].right) {
                    self.stage2p = (self.stage2p + if e[0].right { 1 } else { 2 }) % 3;
                    self.sfx.push(Sfx::Blip);
                    self.t = self.t.min(60);
                }
                let wait = if self.mode == Mode::Two { 600 } else { 300 };
                if self.t > wait || (self.t > 40 && (ok(e[0]) || ok(e[1]))) {
                    self.sfx.push(Sfx::Whistle);
                    match self.mode {
                        Mode::One => self.start_stage(),
                        _ => {
                            let (a, b, st) = self.matchup();
                            self.start_match([a, b], [Ctrl::Human(0), Ctrl::Human(1)], [0.5, 0.5], st);
                        }
                    }
                }
            }
            Scene::Play => {
                if self.mode == Mode::Demo {
                    if any(e[0]) || any(e[1]) {
                        self.game = None;
                        self.go(Scene::Title);
                        return;
                    }
                } else if e[0].start || e[1].start {
                    self.paused = !self.paused;
                    self.sfx.push(Sfx::Blip);
                }
                if self.paused {
                    return;
                }
                let Some(g) = self.game.as_mut() else {
                    self.go(Scene::Title);
                    return;
                };
                g.update(&pads);
                self.sfx.append(&mut g.sfx);
                if g.finished() && g.phase_t > 150 {
                    if self.mode == Mode::Demo {
                        self.game = None;
                        self.go(Scene::Title);
                    } else {
                        self.last = self.game.take();
                        self.go(Scene::Results);
                    }
                }
            }
            Scene::Results => self.update_results(e),
            Scene::Continue => {
                if self.t % 60 == 0 {
                    self.count -= 1;
                    self.sfx.push(Sfx::Tick);
                }
                if ok(e[0]) && self.t > 20 {
                    self.sfx.push(Sfx::Confirm);
                    self.go(Scene::Intro);
                } else if self.count < 0 {
                    self.sfx.push(Sfx::WahWah);
                    self.go(Scene::GameOver);
                }
            }
            Scene::GameOver => {
                if self.t > 300 || (self.t > 60 && ok(e[0])) {
                    self.menu_open = false;
                    self.go(Scene::Title);
                }
            }
            Scene::Champion => {
                if self.t % 90 == 1 {
                    self.sfx.push(Sfx::Cheer);
                }
                if self.t % 50 == 25 {
                    self.sfx.push(Sfx::Bark);
                }
                for c in self.confetti.iter_mut() {
                    c.1 += c.2;
                    c.0 += ((c.1 * 0.05) + c.2 * 9.0).sin() * 0.4;
                    if c.1 > 224.0 {
                        c.1 -= 230.0;
                    }
                }
                if self.t > 1200 || (self.t > 120 && ok(e[0])) {
                    self.menu_open = false;
                    self.go(Scene::Title);
                }
            }
        }
    }

    /// Frames for the tally: rows appear one by one, then the totals count up.
    fn tally_len(&self) -> u32 {
        let n = self.last.as_ref().map(|m| m.sides[0].results.len().max(m.sides[1].results.len())).unwrap_or(5) as u32;
        30 + n * 16 + 60
    }

    fn update_results(&mut self, e: [Pad; 2]) {
        let ok = |p: Pad| p.a || p.start;
        let n = self.last.as_ref().map(|m| m.sides[0].results.len().max(m.sides[1].results.len())).unwrap_or(5) as u32;
        let t = self.t;
        if t >= 30 && t < 30 + n * 16 && (t - 30) % 16 == 0 {
            self.sfx.push(Sfx::Tick);
        }
        let end = self.tally_len();
        if t > 30 + n * 16 && t < end && t % 4 == 0 {
            self.sfx.push(Sfx::Blip);
        }
        if t == end {
            self.sfx.push(Sfx::Cheer);
        }
        let done = t > end + 400 || (t > end + 30 && (ok(e[0]) || ok(e[1])));
        if (ok(e[0]) || ok(e[1])) && t < end {
            // skip ahead
            self.t = end;
        }
        if !done {
            return;
        }
        let won = self.last.as_ref().map(|m| m.winner == Some(0)).unwrap_or(false);
        match self.mode {
            Mode::One => {
                if won {
                    self.stage += 1;
                    if self.stage >= self.ladder.len() {
                        self.make_confetti();
                        self.go(Scene::Champion);
                    } else {
                        self.go(Scene::Intro);
                    }
                } else {
                    self.count = 9;
                    self.go(Scene::Continue);
                }
            }
            _ => {
                self.menu_open = false;
                self.go(Scene::Title);
            }
        }
    }

    pub fn make_confetti(&mut self) {
        self.confetti.clear();
        let cols = [rgb(248, 56, 72), rgb(248, 216, 48), rgb(64, 200, 248), rgb(120, 248, 96), rgb(248, 104, 200)];
        for _ in 0..90 {
            let c = cols[self.rng.int(5) as usize];
            self.confetti.push((self.rng.range(0.0, 256.0), self.rng.range(-224.0, 224.0), self.rng.range(0.4, 1.2), c));
        }
    }

    // ---------------------------------------------------------------- draw

    pub fn draw(&self, f: &mut Frame) {
        let t = self.frame;
        match self.scene {
            Scene::Title => self.draw_title(f, t),
            Scene::Select => self.draw_select(f, t),
            Scene::Intro => self.draw_intro(f, t),
            Scene::Play => {
                if let (Some(g), Some(sc)) = (&self.game, &self.scenery) {
                    draw_match(f, g, sc, &self.art, t);
                    if self.paused {
                        f.shade_rect(0, 35, W, H - 35, 1, 2);
                        f.logo(128, 90, "PAUSE", 3, &SUNSET, OUTLINE, rgb(8, 8, 24));
                        f.text_center(128, 130, "PUSH START", rgb(248, 248, 248), OUTLINE, true);
                        f.text_center(128, 146, "ESC: BACK TO THE TITLE", rgb(184, 184, 216), OUTLINE, false);
                    }
                    if self.mode == Mode::Demo {
                        f.rect(0, 200, W, 24, rgb(16, 16, 48));
                        f.hline(0, W - 1, 199, rgb(248, 176, 32));
                        if (t / 30) % 2 == 0 {
                            f.text_center(128, 203, "DEMO PLAY - PUSH START", rgb(248, 232, 64), OUTLINE, true);
                        }
                        f.text_center(128, 214, DEMO_TIPS[self.demo_tip], rgb(184, 200, 248), OUTLINE, false);
                    }
                }
            }
            Scene::Results => self.draw_results(f, t),
            Scene::Continue => self.draw_continue(f, t),
            Scene::GameOver => self.draw_game_over(f, t),
            Scene::Champion => self.draw_champion(f, t),
        }
    }

    fn draw_title(&self, f: &mut Frame, t: u32) {
        // the park drifts by slowly
        let cam = -60.0 + ((t as f32) * 0.004).sin() * 50.0 + 50.0;
        self.title_bg.draw_back(f, cam, t, 0.0);
        f.shade_rect(0, 0, W, 98, 3, 4);
        // a little show on the grass: throw, run, leap, flip, catch
        title_show(f, &self.art, t);
        // logo drops in and bounces
        let st = self.t as i32;
        let drop = |target: i32, delay: i32| -> i32 {
            let k = st - delay;
            if k < 0 {
                -80
            } else if k < 20 {
                -80 + (target + 80) * k / 20
            } else if k < 30 {
                target - (6 - ((k - 25).abs() * 6 / 5))
            } else {
                target
            }
        };
        f.logo(110, drop(6, 0), "DISC", 6, &SUNSET, OUTLINE, rgb(24, 8, 40));
        f.logo(150, drop(54, 8), "DOGS!", 4, &grad6(rgb(64, 176, 248)), OUTLINE, rgb(8, 16, 48));
        // a paw print stamp on the logo
        if st > 40 {
            paw(f, 214, 20, rgb(248, 104, 168));
        }
        if st > 36 {
            f.rect(0, 99, W, 11, rgb(24, 16, 56));
            f.hline(0, W - 1, 98, rgb(248, 176, 32));
            f.hline(0, W - 1, 110, rgb(248, 176, 32));
            f.text_grad_center(128, 101, "THE GOODEST FRISBEE CHAMPIONSHIP", &grad7(rgb(248, 216, 96)), OUTLINE, false);
        }
        if !self.menu_open {
            if (t / 24) % 2 == 0 && st > 36 {
                f.text_grad_center(128, 120, "PUSH START", &grad7(rgb(248, 248, 248)), OUTLINE, true);
            }
        } else {
            f.shade_rect(72, 115, 112, 26, 1, 2);
            f.rect_outline(72, 115, 112, 26, rgb(248, 176, 32));
            for (k, s) in ["1 PLAYER", "2 PLAYERS"].iter().enumerate() {
                let y = 118 + k as i32 * 11;
                let sel = self.menu == k;
                let col = if sel { rgb(248, 232, 64) } else { rgb(184, 184, 216) };
                f.text_sh(100, y, s, col, OUTLINE, true);
                if sel {
                    draw_disc(f, 89, y + 3, rgb(248, 72, 168), 0, t as f32 * 0.5);
                }
            }
        }
        // footer: ticker and copyright
        f.rect(0, 200, W, 24, rgb(16, 16, 48));
        f.hline(0, W - 1, 199, rgb(248, 176, 32));
        let w = text_width(TICKER, false) + 6;
        let off = (t as i32 / 2) % w;
        for k in 0..2 {
            f.text_sh(k * w - off, 202, TICKER, rgb(248, 216, 48), OUTLINE, false);
        }
        f.text_center(128, 213, "(C) 1992 HELICO SOFT", rgb(248, 232, 200), rgb(96, 48, 32), false);
    }

    fn draw_select(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(16, 48, 40), rgb(24, 64, 52));
        f.text_grad_center(128, 4, "PICK YOUR PUP", &grad7(rgb(248, 200, 48)), OUTLINE, true);
        for k in 0..4 {
            let (cx, cy) = (5 + (k % 2) as i32 * 125, 16 + (k / 2) as i32 * 91);
            let pr = &PAIRS[k];
            let hover: Vec<usize> = (0..2).filter(|&p| (p == 0 || self.mode == Mode::Two) && self.cursor[p] == k).collect();
            let col = pr.color;
            f.rect(cx, cy, 121, 88, rgb(8, 16, 24));
            f.rect(cx + 2, cy + 2, 117, 12, shade(col, 1, 2));
            f.rect_outline(cx, cy, 121, 88, shade(col, 2, 3));
            let name = if k == 2 { "SIR WOOFINGTON" } else { pr.dog.name };
            f.text_grad(cx + 5, cy + 4, name, &grad7(mix(col, 0xFFFFFF, 0.3)), OUTLINE, k != 2);
            // grass strip with the pair on it
            f.rect(cx + 2, cy + 60, 52, 6, rgb(56, 136, 56));
            f.dither_rect(cx + 2, cy + 60, 52, 1, rgb(88, 176, 72), rgb(56, 136, 56));
            let active = !hover.is_empty();
            let opose = if active && (t / 16) % 2 == 0 { OwnerPose::Cheer } else { OwnerPose::Stand };
            blit(f, self.art.owners.get(k, opose), cx + 13, cy + 62, false, &owner_pal(k));
            let dfr = if active {
                [DogFrame::Run0, DogFrame::Run1, DogFrame::Run2, DogFrame::Run3][((t / 5) as usize + k) % 4]
            } else if (t / 12 + k as u32) % 2 == 0 {
                DogFrame::SitWag
            } else {
                DogFrame::Sit
            };
            let collar = if hover.contains(&1) && !hover.contains(&0) { rgb(64, 208, 248) } else { rgb(248, 216, 48) };
            draw(f, self.art.dogs.get(k, dfr), cx + 38, cy + 62, false, &dog_pal(k, collar));
            let d = &pr.dog;
            for (r, (lab, v)) in [("SPD", d.speed), ("JMP", d.jump), ("FOC", d.focus), ("ZOOM", d.zoomies)].iter().enumerate() {
                let y = cy + 19 + r as i32 * 10;
                f.text(cx + 58, y, lab, rgb(200, 216, 208));
                for pip in 0..5 {
                    let c = if pip < *v { mix(col, 0xFFFFFF, 0.2) } else { rgb(40, 48, 64) };
                    f.rect(cx + 84 + pip * 6, y, 6, 7, OUTLINE);
                    f.rect(cx + 85 + pip * 6, y + 1, 4, 5, c);
                }
            }
            f.text(cx + 5, cy + 70, d.breed_name, rgb(144, 168, 160));
            let with = format!("+ {}", pr.owner.name);
            f.text(cx + 5, cy + 78, &with, rgb(120, 136, 136));
            for &p in &hover {
                let hc = if p == 0 { rgb(248, 232, 64) } else { rgb(96, 224, 248) };
                let locked = self.locked[p];
                if locked || (t / 8) % 2 == 0 {
                    f.rect_outline(cx - 1, cy - 1, 123, 90, hc);
                    f.rect_outline(cx, cy, 121, 88, hc);
                }
                let tag = if p == 0 { "1P" } else { "2P" };
                let tx = if p == 0 { cx + 88 } else { cx + 104 };
                f.rect(tx - 1, cy + 59, 14, 9, hc);
                f.text(tx, cy + 60, tag, rgb(16, 16, 48));
                if locked {
                    f.text_grad_center(cx + 88, cy + 70, "WOOF!", &grad7(hc), OUTLINE, true);
                }
            }
        }
        f.rect(0, 199, W, 25, rgb(8, 16, 24));
        f.hline(0, W - 1, 199, rgb(248, 176, 32));
        let bio = |k: usize, alt: bool| -> &'static str { if alt { PAIRS[k].owner.bio } else { PAIRS[k].dog.bio } };
        let alt = (t / 180) % 2 == 1;
        if self.mode == Mode::Two {
            f.text_center(128, 202, bio(self.cursor[0], alt), rgb(248, 232, 64), OUTLINE, false);
            f.text_center(128, 212, bio(self.cursor[1], alt), rgb(96, 224, 248), OUTLINE, false);
        } else {
            f.text_center(128, 202, bio(self.cursor[0], alt), rgb(248, 248, 248), OUTLINE, false);
            f.text_center(128, 213, "ARROWS: PICK   Z: OK   X: BACK", rgb(160, 176, 200), OUTLINE, false);
        }
    }

    fn draw_intro(&self, f: &mut Frame, t: u32) {
        let (a, b, stage) = self.matchup();
        let (pa, pb) = (&PAIRS[a], &PAIRS[b]);
        for y in 0..H {
            let split = 128 + (100 - y) / 3;
            for x in 0..W {
                let stripe = ((x + y + t as i32) / 6) % 2 == 0;
                let base = if x < split { pa.color } else { pb.color };
                let c = if stripe { shade(base, 2, 5) } else { shade(base, 1, 3) };
                f.set(x, y, c);
            }
        }
        let slide = (36 - self.t as i32).max(0) * 7;
        // the pairs, big
        let ca = if self.mode == Mode::Two { rgb(248, 216, 48) } else { rgb(248, 216, 48) };
        let cb = if self.mode == Mode::Two { rgb(64, 208, 248) } else { rgb(232, 64, 72) };
        owner_big(f, &self.art, a, 34 - slide, 150, false);
        draw_scaled(f, self.art.dogs.get(a, DogFrame::Sit), 84 - slide, 150, false, &dog_pal(a, ca), 2);
        owner_big(f, &self.art, b, 222 + slide, 150, true);
        draw_scaled(f, self.art.dogs.get(b, DogFrame::Sit), 172 + slide, 150, true, &dog_pal(b, cb), 2);
        let na = pa.dog.name;
        f.text_grad(6 - slide, 8, na, &grad7(mix(pa.color, 0xFFFFFF, 0.4)), OUTLINE, true);
        f.text_sh(6 - slide, 19, &format!("+ {}", pa.owner.name), rgb(232, 232, 248), OUTLINE, false);
        let nb = pb.dog.name;
        let wb = text_width(nb, true);
        f.text_grad(250 - wb + slide, 156, nb, &grad7(mix(pb.color, 0xFFFFFF, 0.4)), OUTLINE, true);
        let ob = format!("+ {}", pb.owner.name);
        f.text_sh(250 - text_width(&ob, false) + slide, 167, &ob, rgb(232, 232, 248), OUTLINE, false);
        if self.t > 20 {
            let pulse = if self.t < 30 { 7 } else { 6 };
            f.logo(128, 34, "VS", pulse, &SUNSET, OUTLINE, rgb(8, 8, 24));
        }
        // the rival's heckle
        if self.t > 50 && self.mode == Mode::One {
            let h = pb.owner.heckle;
            let w = text_width(h[0], false).max(text_width(h[1], false)) + 8;
            let (bx, by) = (250 - w, 84);
            f.rect(bx - 1, by - 1, w + 2, 22, OUTLINE);
            f.rect(bx, by, w, 20, rgb(248, 248, 248));
            f.fill_poly(&[(bx + w - 30, by + 20), (bx + w - 20, by + 20), (bx + w - 26, by + 27)], OUTLINE);
            f.fill_poly(&[(bx + w - 29, by + 19), (bx + w - 21, by + 19), (bx + w - 26, by + 25)], rgb(248, 248, 248));
            let shown = ((self.t - 50) * 2) as usize;
            let l1: String = h[0].chars().take(shown).collect();
            let l2: String = h[1].chars().take(shown.saturating_sub(h[0].len())).collect();
            f.text(bx + 4, by + 2, &l1, rgb(16, 16, 48));
            f.text(bx + 4, by + 11, &l2, rgb(16, 16, 48));
        }
        // the stage
        f.rect(0, 176, W, 48, rgb(8, 8, 32));
        f.hline(0, W - 1, 176, rgb(248, 176, 32));
        let sname = if self.mode == Mode::Two && (t / 20) % 2 == 0 { format!("< {} >", stage.name()) } else { stage.name().to_string() };
        f.text_grad_center(128, 179, &sname, &grad7(rgb(248, 232, 200)), OUTLINE, true);
        let sub = match self.mode {
            Mode::One => format!("STAGE {} OF 3 - {}", self.stage + 1, stage.tagline()),
            _ => stage.tagline().to_string(),
        };
        f.text_center(128, 190, &sub, rgb(160, 168, 216), OUTLINE, false);
        // the controls, in the game's own words
        let tip = self.mode == Mode::Two || self.stage == 0;
        if tip {
            f.text_center(128, 201, "THROW: UP/DOWN AIM, HOLD Z, LET GO (+C: FLOATER)", rgb(248, 232, 96), OUTLINE, false);
            f.text_center(128, 211, "DOG: ARROWS RUN (TAP TWICE: ZOOMIES) X LEAP Z FLIP", rgb(96, 224, 248), OUTLINE, false);
        } else {
            f.text_center(128, 205, "ONE THROW AT A TIME. GOOD LUCK, HUMAN.", rgb(200, 200, 232), OUTLINE, false);
        }
    }

    fn draw_results(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(24, 16, 64), rgb(32, 24, 80));
        let Some(m) = &self.last else { return };
        f.logo(128, 4, "RESULTS", 3, &SUNSET, OUTLINE, rgb(8, 8, 24));
        f.text_center(128, 33, m.stage.name(), rgb(184, 192, 232), OUTLINE, false);
        let n = m.sides[0].results.len().max(m.sides[1].results.len());
        let rows_shown = if self.t < 30 { 0 } else { ((self.t - 30) / 16 + 1) as usize };
        let end = self.tally_len();
        for side in 0..2 {
            let pair = m.sides[side].pair;
            let x0 = 6 + side as i32 * 126;
            f.rect(x0, 44, 118, 112, rgb(8, 8, 32));
            f.rect_outline(x0, 44, 118, 112, PAIRS[pair].color);
            dog_icon(f, &self.art, pair, x0 + 4, 47, side_color(m, side), false);
            let name = if pair == 2 { "SIR W." } else { PAIRS[pair].dog.name };
            f.text_grad(x0 + 22, 49, name, &grad7(mix(PAIRS[pair].color, 0xFFFFFF, 0.3)), OUTLINE, true);
            for (k, r) in m.sides[side].results.iter().enumerate().take(rows_shown.min(n)) {
                let y = 66 + k as i32 * 10;
                let (label, c) = match r.outcome {
                    Outcome::Catch(kind) => (kind.short(), if kind.mult() > 1 { rgb(96, 224, 248) } else { rgb(232, 232, 248) }),
                    Outcome::Miss(mk) => (mk.short(), rgb(232, 96, 96)),
                };
                f.text(x0 + 4, y, &format!("{}", k + 1), rgb(120, 128, 176));
                f.text(x0 + 14, y, label, c);
                if r.caught() {
                    let mult = match r.outcome {
                        Outcome::Catch(kind) => kind.mult(),
                        _ => 1,
                    };
                    let yd = format!("{}X{}", r.yards, mult);
                    f.text(x0 + 56, y, &yd, rgb(184, 184, 216));
                }
                let pts = format!("{}", r.points());
                f.text(x0 + 114 - text_width(&pts, false), y, &pts, rgb(248, 232, 96));
            }
            // total counts up
            let total = m.sides[side].total();
            let k = if self.t < 30 + n as u32 * 16 { 0.0 } else { ((self.t - 30 - n as u32 * 16) as f32 / 60.0).min(1.0) };
            let shown = (total as f32 * k) as i32;
            f.hline(x0 + 4, x0 + 113, 133, rgb(96, 104, 168));
            f.text(x0 + 4, 138, "TOTAL", rgb(184, 192, 232));
            f.logo(x0 + 86, 134, &format!("{}", shown), 2, &grad6(rgb(248, 232, 96)), OUTLINE, rgb(8, 8, 24));
        }
        // the ranking
        if self.t >= end {
            let w = m.winner.unwrap_or(0);
            for side in 0..2 {
                let pair = m.sides[side].pair;
                let x = 64 + side as i32 * 128;
                let won = side == w;
                f.rect(x - 22, 186, 44, 8, OUTLINE);
                f.rect(x - 21, 187, 42, 6, if won { rgb(248, 200, 48) } else { rgb(176, 184, 200) });
                f.text_center(x, 196, if won { "1ST" } else { "2ND" }, if won { rgb(248, 232, 96) } else { rgb(200, 200, 216) }, OUTLINE, true);
                let mut d = Dog::new(pair);
                d.x = 0.0;
                d.state = if won { DogState::Happy } else { DogState::Lie };
                if won {
                    d.z = (((t / 4) % 8) as f32 - 4.0).abs();
                }
                draw_dog_at(f, &self.art, &d, x, 186, t, side_color(m, side), 0, false, false);
            }
            let wn = PAIRS[m.sides[w].pair].dog.name;
            let s = if m.coin_toss { format!("{} WINS THE COIN TOSS!", wn) } else { format!("{} WINS!", wn) };
            f.text_grad_center(128, 164, &s, &grad7(rgb(248, 216, 64)), OUTLINE, true);
            if self.t > end + 30 && (t / 20) % 2 == 0 {
                f.text_center(128, 212, "PUSH Z", rgb(248, 248, 248), OUTLINE, false);
            }
        }
    }

    fn draw_continue(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(32, 8, 24), rgb(48, 16, 40));
        f.logo(128, 30, "CONTINUE?", 3, &SUNSET, OUTLINE, rgb(0, 0, 0));
        let n = format!("{}", self.count.max(0));
        f.logo(128, 72, &n, 6, &grad6(rgb(248, 232, 64)), OUTLINE, rgb(0, 0, 0));
        // puppy eyes
        let me = self.cursor[0];
        f.shade_ellipse(128, 171, 26, 4, 1, 2);
        draw_scaled(f, self.art.dogs.get(me, DogFrame::Guilty), 128, 170, false, &dog_pal(me, rgb(248, 216, 48)), 2);
        let w = 46;
        f.rect(150, 118, w, 12, OUTLINE);
        f.rect(151, 119, w - 2, 10, rgb(248, 248, 248));
        f.text(155, 121, "PLEASE?", rgb(16, 16, 48));
        if (t / 20) % 2 == 0 {
            f.text_center(128, 188, "PUSH Z TO TRY AGAIN", rgb(248, 248, 248), OUTLINE, true);
        }
    }

    fn draw_game_over(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(16, 8, 32), rgb(24, 16, 48));
        f.logo(128, 50, "GAME", 4, &grad6(rgb(216, 48, 64)), OUTLINE, rgb(0, 0, 0));
        f.logo(128, 88, "OVER", 4, &grad6(rgb(216, 48, 64)), OUTLINE, rgb(0, 0, 0));
        let me = self.cursor[0];
        let mut d = Dog::new(me);
        d.state = DogState::Lie;
        draw_dog_at(f, &self.art, &d, 128, 180, t, rgb(248, 216, 48), 0, false, false);
        rain(f, 128, 150, t);
        let s = format!("{} WILL REMEMBER THIS.", PAIRS[me].dog.name);
        f.text_center(128, 196, &s, rgb(200, 200, 232), OUTLINE, false);
    }

    fn draw_champion(&self, f: &mut Frame, t: u32) {
        let cam = 120.0;
        self.title_bg.draw_back(f, cam, t, 1.0);
        f.shade_rect(0, 0, W, 112, 2, 3);
        let me = self.cursor[0];
        // the podium with the dog on top
        let (cx, cy) = (128, 176);
        f.rect(cx - 26, cy - 14, 52, 30, OUTLINE);
        f.rect(cx - 25, cy - 13, 50, 28, rgb(248, 248, 240));
        f.rect(cx - 25, cy - 13, 50, 3, rgb(248, 200, 48));
        f.logo(cx, cy - 8, "1", 2, &grad6(rgb(248, 200, 48)), OUTLINE, rgb(8, 8, 24));
        let hop = ((t as i32 % 30) - 15).abs();
        let mut d = Dog::new(me);
        d.state = DogState::Happy;
        d.z = ((15 - hop).max(0) / 2) as f32;
        let ground = cy - 14;
        let pal = dog_pal(me, rgb(248, 216, 48));
        f.shade_ellipse(cx, ground + 1, 9, 2, 5, 8);
        draw(f, self.art.dogs.get(me, if d.z > 0.0 { DogFrame::Leap } else { DogFrame::Happy }), cx, ground - d.z as i32, false, &pal);
        // the golden disc in its mouth
        let (mx, mz) = d.mouth();
        draw_disc(f, cx + mx as i32, ground - mz as i32 + 1, rgb(248, 200, 48), 0, t as f32);
        // owner cheering beside
        let op = if (t / 12) % 2 == 0 { OwnerPose::Cheer } else { OwnerPose::Stand };
        f.shade_ellipse(cx - 46, 191, 7, 2, 5, 8);
        blit(f, self.art.owners.get(me, op), cx - 46, 191, false, &owner_pal(me));
        // the trophy
        trophy(f, cx + 50, 190, t);
        for &(x, y, _, c) in &self.confetti {
            f.rect(x as i32, y as i32, 2, 2, c);
        }
        f.logo(128, 6, "BEST IN", 3, &SUNSET, OUTLINE, rgb(8, 8, 24));
        f.logo(128, 32, "SHOW!", 4, &SUNSET, OUTLINE, rgb(8, 8, 24));
        let s = format!("{} IS THE GOODEST!", PAIRS[me].dog.name);
        f.text_grad_center(128, 72, &s, &grad7(rgb(96, 224, 248)), OUTLINE, true);
        f.text_center(128, 86, "WHO'S A GOOD DOG? THAT'S RIGHT.", rgb(248, 248, 248), OUTLINE, false);
        f.rect(0, 202, W, 22, rgb(16, 16, 48));
        f.hline(0, W - 1, 201, rgb(248, 176, 32));
        f.text_center(128, 208, "THANKS FOR PLAYING", rgb(248, 248, 248), OUTLINE, true);
    }
}

fn stripes(f: &mut Frame, t: u32, a: u32, b: u32) {
    for y in 0..H {
        for x in 0..W {
            let k = ((x + y + (t / 2) as i32) / 8) % 2;
            f.set(x, y, if k == 0 { a } else { b });
        }
    }
}

/// An owner at 2x (intro card).
pub fn owner_big(f: &mut Frame, art: &Art, pair: usize, x: i32, y: i32, flip: bool) {
    let s = art.owners.get(pair, OwnerPose::Stand);
    let pal = owner_pal(pair);
    let k = 2;
    for sy in 0..s.h {
        for sx in 0..s.w {
            let v = s.px[(sy * s.w + sx) as usize];
            if v == crate::engine::sprites::T {
                continue;
            }
            let px = if flip { x + (crate::engine::sprites::OX - sx) * k } else { x + (sx - crate::engine::sprites::OX) * k };
            f.rect(px, y + (sy - crate::engine::sprites::OY) * k, k, k, pal[v as usize]);
        }
    }
}

fn paw(f: &mut Frame, x: i32, y: i32, c: u32) {
    f.fill_ellipse(x, y + 4, 5, 4, OUTLINE);
    f.fill_ellipse(x, y + 4, 4, 3, c);
    for (dx, dy) in [(-5, -2), (-2, -5), (2, -5), (5, -2)] {
        f.fill_ellipse(x + dx, y + dy, 2, 2, OUTLINE);
        f.fill_ellipse(x + dx, y + dy, 1, 1, c);
    }
}

fn rain(f: &mut Frame, x: i32, y: i32, t: u32) {
    f.fill_ellipse(x, y - 16, 16, 6, OUTLINE);
    f.fill_ellipse(x - 6, y - 20, 9, 6, OUTLINE);
    f.fill_ellipse(x, y - 16, 15, 5, rgb(112, 120, 144));
    f.fill_ellipse(x - 6, y - 20, 8, 5, rgb(136, 144, 168));
    for k in 0..6 {
        let ry = y - 8 + ((t as i32 / 2 + k * 5) % 24);
        f.set(x - 12 + k * 5, ry, rgb(120, 176, 248));
        f.set(x - 12 + k * 5, ry + 1, rgb(120, 176, 248));
    }
}

fn trophy(f: &mut Frame, cx: i32, base: i32, t: u32) {
    let gold = rgb(248, 200, 48);
    let gold_d = rgb(184, 120, 24);
    let cy = base - 14;
    f.rect(cx - 10, cy + 2, 20, 12, OUTLINE);
    f.rect(cx - 9, cy + 3, 18, 10, rgb(96, 56, 32));
    f.rect(cx - 9, cy + 3, 18, 2, rgb(136, 88, 48));
    f.fill_poly(&[(cx - 12, cy - 22), (cx + 12, cy - 22), (cx + 6, cy - 2), (cx - 6, cy - 2)], OUTLINE);
    f.fill_poly(&[(cx - 10, cy - 21), (cx + 10, cy - 21), (cx + 5, cy - 3), (cx - 5, cy - 3)], gold);
    f.fill_poly(&[(cx + 3, cy - 21), (cx + 10, cy - 21), (cx + 5, cy - 3), (cx + 3, cy - 3)], gold_d);
    f.circle_outline(cx - 12, cy - 16, 4, OUTLINE);
    f.circle_outline(cx + 12, cy - 16, 4, OUTLINE);
    f.circle_outline(cx - 12, cy - 16, 3, gold);
    f.circle_outline(cx + 12, cy - 16, 3, gold_d);
    f.rect(cx - 2, cy - 3, 4, 6, gold_d);
    // a golden disc on top
    draw_disc(f, cx, cy - 25, gold, 0, t as f32 * 0.3);
    if (t / 4) % 8 == 0 {
        f.set(cx - 6, cy - 18, rgb(248, 248, 248));
        f.set(cx - 5, cy - 18, rgb(248, 248, 248));
        f.set(cx - 6, cy - 17, rgb(248, 248, 248));
    }
}

/// The title's little show: the owner throws, the dog runs, leaps, flips
/// and catches, over and over.
fn title_show(f: &mut Frame, art: &Art, t: u32) {
    let cyc = 240;
    let k = (t % cyc) as f32;
    let pair = ((t / cyc) % 4) as usize;
    let col = PAIRS[pair].color;
    let ox = 40;
    let gy = GROUND_Y;
    let pose = if k < 20.0 {
        OwnerPose::Aim
    } else if k < 30.0 {
        OwnerPose::Coil
    } else if k < 40.0 {
        OwnerPose::Fling
    } else if k > 190.0 {
        OwnerPose::Cheer
    } else {
        OwnerPose::Stand
    };
    f.shade_ellipse(ox, gy + 1, 7, 2, 5, 8);
    blit(f, art.owners.get(pair, pose), ox, gy + 1, false, &owner_pal(pair));
    let mut d = Dog::new(pair);
    // disc path: from the hand, an arc to the catch point
    let (cx_catch, cz_catch) = (196.0, 30.0);
    let (dx, dz, flying) = if k < 34.0 {
        (ox as f32 + 8.0, 20.0, false)
    } else if k < 150.0 {
        let u = (k - 34.0) / 116.0;
        let x = ox as f32 + 8.0 + (cx_catch - ox as f32 - 8.0) * (1.0 - (1.0 - u) * (1.0 - u) * 0.6 - 0.4 * (1.0 - u)).min(1.0);
        let z = 20.0 + (cz_catch - 20.0) * u + (u * std::f32::consts::PI).sin() * 70.0;
        (x, z, true)
    } else {
        (0.0, 0.0, false)
    };
    // the dog: sits, runs, leaps at 128, flips, lands with the disc
    if k < 34.0 {
        d.x = ox as f32 + 18.0;
        d.state = DogState::Sit;
    } else if k < 120.0 {
        d.x = ox as f32 + 18.0 + (k - 34.0) * 1.6;
        d.state = DogState::Run;
        d.anim = k * 0.25;
    } else if k < 176.0 {
        let u = k - 120.0;
        d.x = ox as f32 + 18.0 + 86.0 * 1.6 + u * 0.5;
        d.state = DogState::Air;
        d.z = (2.9 * u - 0.5 * 0.115 * u * u).max(0.0);
        d.vz = 2.9 - 0.115 * u;
        if (34.0..34.0 + FLIP_LEN as f32).contains(&u) {
            d.flip_t = (u - 34.0) as i32;
        }
    } else {
        d.x = ox as f32 + 18.0 + 86.0 * 1.6 + 28.0;
        d.state = DogState::Happy;
    }
    let carried = k >= 150.0;
    d.has_disc = carried;
    draw_dog(f, art, &d, 0.0, t, rgb(248, 216, 48), col, carried, false);
    if flying {
        f.shade_ellipse(dx as i32, gy + 1, 3, 1, 5, 8);
        draw_disc(f, dx as i32, gy - dz as i32, col, 0, k);
    } else if k < 34.0 {
        draw_disc(f, ox + 9, gy - 16, col, 0, 0.0);
    }
    if (150.0..200.0).contains(&k) && (t / 4) % 2 == 0 {
        f.text_grad_center(d.x as i32, 140, "FLIP CATCH!", &grad7(rgb(248, 96, 200)), OUTLINE, true);
    }
}
