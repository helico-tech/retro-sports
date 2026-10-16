//! Scenes and flow: title, bike select, stage card, race, results,
//! continue, game over and the champion screen. Pure logic plus drawing
//! into a Frame; the window, keyboard and speakers live in main.rs.

use super::art::*;
use super::audio::Track;
use super::data::*;
use super::race::*;
use super::render::*;
use super::scenery::*;
use crate::engine::gfx::*;
use crate::engine::input::{any, edge};
use crate::engine::sprites::{self as sp, Sprite};
use crate::engine::ui::{grad6, grad7, LOGO_SHADOW, OUTLINE, SUNSET};
use std::cell::RefCell;

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

/// Idle frames on the title before the demo starts.
pub const IDLE_DEMO: u32 = 900;
/// 2P split screen: the rows of each view that are shown.
pub const CROP0: i32 = 104;
pub const CROP_H: i32 = 99;

/// One line of the score tally.
#[derive(Clone, Debug)]
pub struct Line {
    pub label: String,
    pub pts: i32,
}

pub struct App {
    pub scene: Scene,
    pub t: u32,
    pub frame: u32,
    pub mode: Mode,
    pub menu: usize,
    pub menu_open: bool,
    /// chosen bike per player (index into BIKES)
    pub cursor: [usize; 2],
    pub locked: [bool; 2],
    pub stage: usize,
    pub race: Option<Race>,
    pub scenery: Option<Scenery>,
    pub title_sc: Scenery,
    pub art: Art,
    pub sfx: Vec<Sfx>,
    prev: [Pad; 2],
    rng: Rng,
    pub quit: bool,
    pub paused: bool,
    idle: u32,
    pub count: i32,
    confetti: Vec<(f32, f32, f32, u32)>,
    /// tour points per player
    pub score: [i32; 2],
    /// this stage: ranking (rider, frames, dnf) and the tally per human
    pub results: Vec<(usize, u32, bool)>,
    pub tally: [Vec<Line>; 2],
    pub names: Vec<(Who, Bike, Ctrl)>,
    pub last_place: usize,
    scratch: RefCell<Frame>,
}

impl App {
    pub fn new(seed: u64) -> Self {
        App {
            scene: Scene::Title,
            t: 0,
            frame: 0,
            mode: Mode::One,
            menu: 0,
            menu_open: false,
            cursor: [0, 1],
            locked: [false; 2],
            stage: 0,
            race: None,
            scenery: None,
            title_sc: Scenery::new(Venue::Polder, 7),
            art: Art::new(),
            sfx: Vec::new(),
            prev: [Pad::default(); 2],
            rng: Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1),
            quit: false,
            paused: false,
            idle: 0,
            count: 0,
            confetti: Vec::new(),
            score: [0; 2],
            results: Vec::new(),
            tally: [Vec::new(), Vec::new()],
            names: Vec::new(),
            last_place: 0,
            scratch: RefCell::new(Frame::new()),
        }
    }

    pub fn music(&self) -> Option<Track> {
        match self.scene {
            Scene::Title | Scene::Select | Scene::Intro | Scene::Results => Some(Track::Title),
            Scene::Play => {
                let st = self.race.as_ref().map(|r| r.stage).unwrap_or(self.stage);
                Some(if STAGES[st.min(2)].venue == Venue::Dijk { Track::Storm } else { Track::Polka })
            }
            Scene::Champion => Some(Track::Fanfare),
            Scene::Continue | Scene::GameOver => None,
        }
    }

    fn go(&mut self, s: Scene) {
        self.scene = s;
        self.t = 0;
    }

    fn ensure_scenery(&mut self, stage_i: usize, seed: u64) {
        let venue = STAGES[stage_i].venue;
        if self.scenery.as_ref().map(|s| s.venue != venue).unwrap_or(true) {
            self.scenery = Some(Scenery::new(venue, seed));
        }
    }

    pub fn field(&self, stage_i: usize) -> Vec<(Who, Bike, Ctrl, f32)> {
        let sk = STAGES[stage_i].cpu_skill;
        match self.mode {
            Mode::One => {
                let mut f = vec![(Who::Kees, BIKES[self.cursor[0]], Ctrl::Human(0), 0.5)];
                for w in RIVALS {
                    f.push((w, w.def().bike, Ctrl::Cpu, sk));
                }
                f
            }
            Mode::Two => {
                let pair = [[Who::Oma, Who::Dennis], [Who::Dennis, Who::DeVries], [Who::Oma, Who::DeVries]][stage_i % 3];
                let sk = 0.55 + 0.1 * stage_i as f32;
                vec![
                    (Who::Kees, BIKES[self.cursor[0]], Ctrl::Human(0), 0.5),
                    (Who::Anouk, BIKES[self.cursor[1]], Ctrl::Human(1), 0.5),
                    (pair[0], pair[0].def().bike, Ctrl::Cpu, sk),
                    (pair[1], pair[1].def().bike, Ctrl::Cpu, sk),
                ]
            }
            Mode::Demo => {
                let bike = BIKES[self.rng.0 as usize % 4];
                let mut f = vec![(Who::Kees, bike, Ctrl::Cpu, 0.7)];
                for w in RIVALS {
                    f.push((w, w.def().bike, Ctrl::Cpu, 0.7));
                }
                f
            }
        }
    }

    pub fn start_race(&mut self, stage_i: usize) {
        let seed = self.rng.next() as u64;
        let field = self.field(stage_i);
        self.names = field.iter().map(|f| (f.0, f.1, f.2)).collect();
        self.race = Some(Race::new(stage_i, &field, seed));
        self.ensure_scenery(stage_i, seed ^ 0x55);
        self.paused = false;
        self.go(Scene::Play);
    }

    pub fn start_demo(&mut self) {
        self.mode = Mode::Demo;
        let st = self.rng.int(3) as usize;
        self.stage = st;
        self.start_race(st);
    }

    pub fn update(&mut self, pads: [Pad; 2], esc: bool) {
        self.frame = self.frame.wrapping_add(1);
        self.t += 1;
        let prev = self.prev;
        let e = [edge(pads[0], prev[0]), edge(pads[1], prev[1])];
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
                    self.race = None;
                    self.menu_open = false;
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
                if self.idle > IDLE_DEMO {
                    self.menu_open = false;
                    self.idle = 0;
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
                            self.locked[pi] = true;
                            self.sfx.push(Sfx::Bell);
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
                    self.stage = 0;
                    self.score = [0; 2];
                    self.go(Scene::Intro);
                }
            }
            Scene::Intro => {
                if self.t == 2 {
                    // build the stage backdrop while the card is up
                    let seed = self.rng.next() as u64;
                    self.ensure_scenery(self.stage, seed);
                }
                if self.t > 420 || (self.t > 40 && (ok(e[0]) || ok(e[1]))) {
                    self.sfx.push(Sfx::Confirm);
                    self.start_race(self.stage);
                }
            }
            Scene::Play => {
                if self.mode == Mode::Demo {
                    if any(e[0]) || any(e[1]) {
                        self.race = None;
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
                let Some(r) = self.race.as_mut() else {
                    self.go(Scene::Title);
                    return;
                };
                let pads_in = if self.mode == Mode::Demo { [Pad::default(); 2] } else { pads };
                r.update(&pads_in, &prev);
                self.sfx.append(&mut r.sfx);
                if r.finished() {
                    if self.mode == Mode::Demo {
                        self.race = None;
                        self.go(Scene::Title);
                    } else {
                        self.make_results();
                        self.go(Scene::Results);
                    }
                }
            }
            Scene::Results => {
                let n = self.tally[0].len().max(self.tally[1].len()) as u32;
                // one line every 30 frames, then the total
                let done_at = 20 + n * 30 + 30;
                if self.t > 20 && self.t <= 20 + n * 30 && (self.t - 20) % 30 == 0 {
                    self.sfx.push(Sfx::Blip);
                }
                if self.t == done_at {
                    self.sfx.push(Sfx::Confirm);
                }
                if self.t > done_at + 30 && ok(e[0]) || self.t > done_at + 600 {
                    self.after_results();
                } else if self.t > 30 && self.t < done_at && ok(e[0]) {
                    // skip the count
                    self.t = done_at;
                }
            }
            Scene::Continue => {
                if self.t % 60 == 0 {
                    self.count -= 1;
                    self.sfx.push(Sfx::Blip);
                }
                if ok(e[0]) && self.t > 20 {
                    self.sfx.push(Sfx::Confirm);
                    self.go(Scene::Intro);
                } else if self.count < 0 {
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
                for c in self.confetti.iter_mut() {
                    c.1 += c.2;
                    c.0 += ((c.1 * 0.05) + c.2 * 9.0).sin() * 0.4;
                    if c.1 > 224.0 {
                        c.1 -= 230.0;
                    }
                }
                if self.t > 900 || (self.t > 120 && ok(e[0])) {
                    self.menu_open = false;
                    self.go(Scene::Title);
                }
            }
        }
    }

    fn make_results(&mut self) {
        let Some(r) = self.race.as_ref() else { return };
        self.results = r.results();
        let st = r.def();
        let mut tally: [Vec<Line>; 2] = [Vec::new(), Vec::new()];
        for (place, &(i, _, dnf)) in self.results.iter().enumerate() {
            let Ctrl::Human(p) = r.riders[i].ctrl else { continue };
            let rd = &r.riders[i];
            let b = rd.bike.def();
            let mut v = Vec::new();
            if dnf {
                v.push(Line { label: "BEZEMWAGEN".into(), pts: 0 });
            } else {
                v.push(Line { label: format!("{} PLACE", ordinal(place)), pts: PLACE_PTS[place.min(3)] });
            }
            v.push(Line { label: format!("{} X {}", st.cargo.one(), rd.cargo), pts: rd.cargo * b.cargo_pts });
            v.push(Line { label: format!("PERFECT X {}", rd.stats.perfect), pts: rd.stats.perfect as i32 * 2 });
            if rd.stats.eaten > 0 {
                v.push(Line { label: format!("SNACKS X {}", rd.stats.eaten), pts: rd.stats.eaten as i32 * 25 });
            }
            if rd.stats.crashes > 0 {
                v.push(Line { label: "ZWEMDIPLOMA".into(), pts: 50 });
            }
            tally[p.min(1)] = v;
            if p == 0 {
                self.last_place = if dnf { 3 } else { place };
            }
        }
        for p in 0..2 {
            let sum: i32 = tally[p].iter().map(|l| l.pts).sum();
            self.score[p] += sum;
        }
        self.tally = tally;
    }

    fn after_results(&mut self) {
        match self.mode {
            Mode::One => {
                let need = if self.stage + 1 >= STAGES.len() { 1 } else { 2 };
                if self.last_place < need {
                    self.stage += 1;
                    if self.stage >= STAGES.len() {
                        self.make_confetti();
                        self.go(Scene::Champion);
                    } else {
                        self.go(Scene::Intro);
                    }
                } else {
                    // this stage's points don't count if you try again
                    let sum: i32 = self.tally[0].iter().map(|l| l.pts).sum();
                    self.score[0] -= sum;
                    self.count = 9;
                    self.go(Scene::Continue);
                }
            }
            Mode::Two => {
                self.stage += 1;
                if self.stage >= STAGES.len() {
                    self.make_confetti();
                    self.go(Scene::Champion);
                } else {
                    self.go(Scene::Intro);
                }
            }
            Mode::Demo => self.go(Scene::Title),
        }
    }

    fn make_confetti(&mut self) {
        self.confetti.clear();
        let cols = [rgb(248, 136, 24), rgb(248, 248, 248), rgb(200, 32, 48), rgb(32, 72, 160), rgb(248, 216, 48)];
        for _ in 0..100 {
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
            Scene::Play => self.draw_play(f, t),
            Scene::Results => self.draw_results(f, t),
            Scene::Continue => self.draw_continue(f, t),
            Scene::GameOver => self.draw_game_over(f, t),
            Scene::Champion => self.draw_champion(f, t),
        }
    }

    fn draw_play(&self, f: &mut Frame, t: u32) {
        let (Some(r), Some(sc)) = (&self.race, &self.scenery) else { return };
        if r.views.len() >= 2 {
            // split screen: each player's view into the scratch frame, then a band of it
            let mut s = self.scratch.borrow_mut();
            for (v, &i) in r.views.iter().enumerate().take(2) {
                let clip = (CROP0, CROP0 + CROP_H);
                draw_view(&mut s, r, sc, &self.art, i, t, clip);
                if r.shake[v] > 0 {
                    let dx = if (r.shake[v] / 2) % 2 == 0 { 1 } else { -1 } * (r.shake[v].min(4) / 2 + 1);
                    shake_rows(&mut s, clip, dx);
                }
                let dst = if v == 0 { 13 } else { 125 };
                f.copy_band(&s, CROP0, dst, CROP_H);
                draw_mini_hud(f, r, i, if v == 0 { 0 } else { 112 }, t);
                if let Some(b) = &r.banners[v] {
                    draw_banner(f, b, dst + CROP_H / 2, true);
                }
            }
            f.hline(0, W - 1, 111, rgb(176, 32, 88));
        } else {
            let i = r.views[0];
            draw_view(f, r, sc, &self.art, i, t, (33, H));
            if r.shake[0] > 0 {
                let dx = if (r.shake[0] / 2) % 2 == 0 { 1 } else { -1 } * (r.shake[0].min(4) / 2 + 1);
                shake_rows(f, (33, H), dx);
            }
            draw_hud(f, r, &self.art, i, t);
            if let Some(b) = &r.banners[0] {
                // above the road, so you can still see what is coming
                draw_banner(f, b, if b.pop { 100 } else { 78 }, false);
            }
        }
        if r.phase == Phase::Countdown && self.mode != Mode::Demo && r.phase_t < COUNTDOWN {
            self.draw_controls_tip(f, t);
        }
        if self.paused {
            f.shade_rect(0, 0, W, H, 1, 2);
            f.logo(128, 80, "PAUZE", 4, &SUNSET, OUTLINE, LOGO_SHADOW);
            f.text_center(128, 124, "PUSH START", rgb(248, 248, 248), OUTLINE, true);
            f.text_center(128, 140, "ESC: BACK TO THE TITLE", rgb(184, 184, 216), OUTLINE, false);
        }
        if self.mode == Mode::Demo {
            f.rect(0, 212, W, 12, rgb(16, 16, 48));
            if (t / 30) % 2 == 0 {
                f.text_center(128, 214, "DEMO PLAY - PUSH START", rgb(248, 232, 64), OUTLINE, true);
            }
        }
    }

    /// The keys, shown while the countdown runs.
    fn draw_controls_tip(&self, f: &mut Frame, _t: u32) {
        if self.mode == Mode::Two {
            return;
        }
        let y = 186;
        f.shade_rect(4, y - 2, 248, 30, 1, 3);
        f.rect_outline(4, y - 2, 248, 30, rgb(248, 136, 24));
        f.text_sh(10, y + 1, "< >  PEDAL IN RHYTHM   UP/DOWN: LANE", rgb(248, 248, 248), OUTLINE, false);
        f.text_sh(10, y + 13, "Z BELL   X HOP   HOLD C TUCK", rgb(248, 232, 96), OUTLINE, false);
    }

    // ------------------------------------------------------------ title

    fn draw_title(&self, f: &mut Frame, t: u32) {
        let cam = t as f32 * 0.6;
        self.title_sc.draw(f, &self.art, cam, t, 0.7, 0.4, (0, H));
        self.title_sc.draw_road(f, cam, (0, H), |_| false);
        self.title_sc.draw_water(f, cam, t, (0, H));
        f.shade_rect(0, 0, W, 92, 3, 4);
        // the four riders pedal by, against the wind
        let riders = [(Who::Oma, Bike::Oma, 0.0f32, 0.0f32), (Who::Dennis, Bike::Race, 1.0, 40.0), (Who::Kees, Bike::Oma, 0.0, 80.0), (Who::DeVries, Bike::Bak, 1.0, 130.0)];
        for (k, &(who, bike, lane, x0)) in riders.iter().enumerate() {
            let x = 40.0 + x0 + ((t as f32 * 0.02 + k as f32 * 1.7).sin() * 10.0);
            let gy = ground_y(lane) as i32;
            f.shade_ellipse(x as i32 + 2, gy, 13, 2, 5, 8);
            let crank = ((t / 4 + k as u32 * 3) % 8) as usize;
            let cargo = if who == Who::Kees { Some((Cargo::Cheese, 3)) } else { None };
            draw_bike_rider(f, &self.art, who, bike, x as i32, gy, crank, t as f32 * 1.7, k == 1, false, false, cargo, t);
        }
        // wind streaks
        for k in 0..10 {
            let y = 100 + (k * 37) % 90;
            let x = 256 - ((t as i32 * (3 + k % 3) + k * 61) % 300);
            for d in (0..10).step_by(2) {
                f.set(x + d, y, rgb(232, 240, 248));
            }
        }
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
        let orange = [rgb(248, 248, 200), rgb(248, 216, 120), rgb(248, 176, 64), rgb(248, 136, 24), rgb(224, 96, 16), rgb(176, 56, 16)];
        // the second word shakes in the wind
        let gust = if st > 40 && (t / 50) % 4 == 0 { ((t / 2) % 3) as i32 - 1 } else { 0 };
        f.logo(122, drop(6, 0), "TEGEN", 5, &orange, OUTLINE, rgb(24, 8, 40));
        f.logo(136 + gust, drop(48, 8), "WIND!", 5, &grad6(rgb(64, 152, 232)), OUTLINE, rgb(24, 8, 40));
        if st > 36 {
            f.rect(0, 92, W, 11, rgb(24, 16, 56));
            f.hline(0, W - 1, 91, rgb(248, 136, 24));
            f.hline(0, W - 1, 103, rgb(248, 136, 24));
            f.text_grad_center(128, 94, "HOLLAND'S TOUGHEST BIKE RACE", &grad7(rgb(248, 248, 248)), OUTLINE, false);
        }
        if !self.menu_open {
            if (t / 24) % 2 == 0 && st > 36 {
                f.text_grad_center(128, 110, "PUSH START", &grad7(rgb(248, 248, 248)), OUTLINE, true);
            }
        } else {
            f.shade_rect(72, 106, 112, 26, 1, 3);
            f.rect_outline(72, 106, 112, 26, rgb(248, 136, 24));
            for (k, s) in ["1 PLAYER", "2 PLAYERS"].iter().enumerate() {
                let y = 109 + k as i32 * 11;
                let sel = self.menu == k;
                let col = if sel { rgb(248, 232, 64) } else { rgb(184, 184, 216) };
                f.text_sh(100, y, s, col, OUTLINE, true);
                if sel {
                    let wheel = self.art.wheel(4, false, (t / 3) as usize);
                    wheel.draw(f, 84, y - 2, &PROPS, false);
                }
            }
        }
        // footer: scrolling ticker and the copyright
        f.rect(0, 200, W, 24, rgb(16, 16, 48));
        f.hline(0, W - 1, 199, rgb(248, 136, 24));
        let w = text_width(TICKER, false) + 6;
        let off = (t as i32 / 2) % w;
        for k in 0..3 {
            f.text_sh(k * w - off, 202, TICKER, rgb(248, 216, 48), OUTLINE, false);
        }
        f.text_center(128, 213, "(C) 1992 HELICO SOFT", rgb(248, 232, 200), rgb(96, 48, 32), false);
    }

    // ------------------------------------------------------------ select

    fn draw_select(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(16, 24, 72), rgb(24, 32, 96));
        f.text_grad_center(128, 4, "KIES JE FIETS!", &grad7(rgb(248, 176, 48)), OUTLINE, true);
        f.text_center(128, 14, "CHOOSE YOUR BIKE", rgb(160, 168, 216), OUTLINE, false);
        for k in 0..4 {
            let (cx, cy) = (6 + (k % 2) as i32 * 124, 24 + (k / 2) as i32 * 88);
            let bd = BIKES[k].def();
            let hover: Vec<usize> = (0..2).filter(|&p| (p == 0 || self.mode == Mode::Two) && self.cursor[p] == k).collect();
            let accent = [rgb(232, 64, 48), rgb(248, 72, 168), rgb(48, 120, 216), rgb(248, 136, 24)][k];
            f.rect(cx, cy, 120, 84, rgb(8, 8, 32));
            f.rect(cx + 2, cy + 52, 116, 30, rgb(16, 16, 48));
            f.rect(cx + 2, cy + 2, 116, 11, shade(accent, 1, 2));
            f.rect_outline(cx, cy, 120, 84, shade(accent, 2, 3));
            f.text_grad(cx + 5, cy + 4, bd.name, &grad7(mix(accent, 0xFFFFFF, 0.4)), OUTLINE, false);
            // a strip of road under the bike
            f.rect(cx + 2, cy + 47, 56, 4, rgb(176, 72, 64));
            f.hline(cx + 2, cx + 57, cy + 47, rgb(200, 96, 80));
            let who = if hover.contains(&1) && !hover.contains(&0) { Who::Anouk } else { Who::Kees };
            let active = !hover.is_empty();
            let crank = if active { ((t / 4) % 8) as usize } else { 0 };
            let wheel = if active { t as f32 * 1.5 } else { 0.0 };
            draw_bike_rider(f, &self.art, who, BIKES[k], cx + 30, cy + 49, crank, wheel, false, false, false, Some((Cargo::Cheese, bd.cargo_n.min(4))), t);
            for (r, lab) in ["SPD", "WIND", "CARGO", "HOP"].iter().enumerate() {
                let y = cy + 17 + r as i32 * 8;
                f.text(cx + 62, y, lab, rgb(200, 200, 232));
                for pip in 0..5 {
                    let c = if pip < bd.stats[r] { mix(accent, 0xFFFFFF, 0.3) } else { rgb(40, 40, 72) };
                    f.rect(cx + 92 + pip * 5, y, 5, 6, OUTLINE);
                    f.rect(cx + 93 + pip * 5, y + 1, 3, 4, c);
                }
            }
            for (k, l) in bd.bio.iter().enumerate() {
                f.text(cx + 4, cy + 56 + k as i32 * 9, l, rgb(176, 184, 224));
            }
            for &p in &hover {
                let col = if p == 0 { rgb(248, 232, 64) } else { rgb(96, 224, 248) };
                let locked = self.locked[p];
                if locked || (t / 8) % 2 == 0 {
                    f.rect_outline(cx - 1, cy - 1, 122, 86, col);
                    f.rect_outline(cx, cy, 120, 84, col);
                }
                let tag = if p == 0 { "1P" } else { "2P" };
                let tx = if p == 0 { cx + 88 } else { cx + 103 };
                f.rect(tx - 1, cy + 2, 14, 9, col);
                f.text(tx, cy + 3, tag, rgb(16, 16, 48));
                if locked {
                    f.text_grad_center(cx + 30, cy + 18, "OK!", &grad7(col), OUTLINE, true);
                }
            }
        }
        if self.mode == Mode::Two {
            f.text_center(128, 202, "1P: ARROWS + , . /  (OR Z X C)", rgb(248, 232, 64), OUTLINE, false);
            f.text_center(128, 212, "2P: W A S D + F G H", rgb(96, 224, 248), OUTLINE, false);
        } else {
            f.text_center(128, 208, "ARROWS: PICK   Z: OK   X: BACK", rgb(200, 200, 232), OUTLINE, false);
        }
    }

    // ------------------------------------------------------------ intro

    fn draw_intro(&self, f: &mut Frame, t: u32) {
        let st = &STAGES[self.stage.min(2)];
        // the stage's backdrop, darkened
        if let Some(sc) = &self.scenery {
            if sc.venue == st.venue {
                sc.draw(f, &self.art, t as f32 * 0.3, t, 0.6, 0.3, (0, H));
                sc.draw_road(f, t as f32 * 0.3, (0, H), |_| false);
                sc.draw_water(f, t as f32 * 0.3, t, (0, H));
            } else {
                stripes(f, t, rgb(16, 24, 72), rgb(24, 32, 96));
            }
        } else {
            stripes(f, t, rgb(16, 24, 72), rgb(24, 32, 96));
        }
        f.shade_rect(0, 0, W, H, 1, 2);
        let slide = (30 - self.t as i32).max(0) * 9;
        let label = format!("ETAPPE {}", self.stage + 1);
        f.text_grad_center(128 - slide, 6, &label, &grad7(rgb(248, 176, 48)), OUTLINE, true);
        let scale = if st.name.len() > 9 { 3 } else { 4 };
        f.logo(128 + slide, 16, st.name, scale, &SUNSET, OUTLINE, LOGO_SHADOW);
        f.text_center(128, 50, st.tagline[0], rgb(248, 248, 248), OUTLINE, false);
        f.text_center(128, 59, st.tagline[1], rgb(200, 208, 240), OUTLINE, false);
        // the field: you and who you face, lined up on a strip of road
        let field = self.field_preview();
        let n = field.len() as i32;
        f.rect(0, 108, W, 6, rgb(176, 72, 64));
        f.hline(0, W - 1, 108, rgb(200, 96, 80));
        f.hline(0, W - 1, 114, rgb(144, 56, 56));
        let talker = ((t / 90) as usize) % n as usize;
        for (k, &(who, bike, human)) in field.iter().enumerate() {
            let x = 128 - (n - 1) * 31 + k as i32 * 62 + if k % 2 == 0 { -slide } else { slide };
            let y = 111;
            let crank = ((t / 5 + k as u32 * 2) % 8) as usize;
            let cargo = if human { Some((st.cargo, bike.def().cargo_n)) } else { None };
            f.shade_ellipse(x + 2, y, 13, 2, 5, 8);
            draw_bike_rider(f, &self.art, who, bike, x, y, crank, t as f32, false, false, false, cargo, t);
            let name = who.def().short;
            let col = if human { rgb(248, 232, 64) } else { who.def().color };
            f.text_center(x, y + 6, name, col, OUTLINE, false);
            if !human && k == talker && self.t > 40 {
                let h = who.def().heckles;
                bubble(f, x + 4, y - 30, h[((t / 270) as usize + k) % h.len()]);
            }
        }
        // cargo and tip box
        let cargo = format!("CARGO: {} {}", BIKES[self.cursor[0]].def().cargo_n, st.cargo.name());
        f.text_center(128, 128, &cargo, rgb(248, 216, 96), OUTLINE, false);
        let need = match self.mode {
            Mode::One if self.stage + 1 < STAGES.len() => "FINISH IN THE TOP 2 TO GO ON",
            Mode::One => "WIN THIS ONE AND YOU ARE CHAMPION",
            _ => "MOST POINTS AFTER 3 STAGES WINS",
        };
        f.text_center(128, 140, need, rgb(184, 192, 232), OUTLINE, false);
        let (bx, by) = (8, 156);
        f.rect(bx, by, 240, 40, rgb(16, 16, 48));
        f.rect_outline(bx, by, 240, 40, rgb(248, 136, 24));
        f.rect_outline(bx + 1, by + 1, 238, 38, rgb(96, 64, 32));
        f.text_sh(bx + 6, by + 4, "TIP:", rgb(248, 136, 24), OUTLINE, false);
        if self.stage == 0 {
            f.text_sh(bx + 32, by + 4, "LEFT, RIGHT, LEFT... IN RHYTHM!", rgb(248, 248, 248), OUTLINE, false);
            f.text_sh(bx + 32, by + 14, "TOO FAST AND YOUR LEGS JUST SPIN.", rgb(248, 248, 248), OUTLINE, false);
            f.text_sh(bx + 32, by + 24, "Z BELL  X HOP  HOLD C TUCK", rgb(248, 232, 96), OUTLINE, false);
        } else {
            f.text_sh(bx + 32, by + 6, st.tip[0], rgb(248, 248, 248), OUTLINE, false);
            f.text_sh(bx + 32, by + 16, st.tip[1], rgb(248, 248, 248), OUTLINE, false);
            f.text_sh(bx + 32, by + 26, "BOTH PEDALS HELD = BRAKE", rgb(248, 232, 96), OUTLINE, false);
        }
        if (t / 20) % 2 == 0 && self.t > 40 {
            f.text_center(128, 208, "PUSH START", rgb(248, 248, 248), OUTLINE, true);
        }
    }

    fn field_preview(&self) -> Vec<(Who, Bike, bool)> {
        match self.mode {
            Mode::Two => {
                let pair = [[Who::Oma, Who::Dennis], [Who::Dennis, Who::DeVries], [Who::Oma, Who::DeVries]][self.stage % 3];
                vec![(Who::Kees, BIKES[self.cursor[0]], true), (pair[0], pair[0].def().bike, false), (pair[1], pair[1].def().bike, false), (Who::Anouk, BIKES[self.cursor[1]], true)]
            }
            _ => {
                let mut v = vec![(Who::Kees, BIKES[self.cursor[0]], true)];
                for w in RIVALS {
                    v.push((w, w.def().bike, false));
                }
                v
            }
        }
    }

    // ------------------------------------------------------------ results

    fn draw_results(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(40, 16, 8), rgb(56, 24, 8));
        let st = &STAGES[self.stage.min(2)];
        f.text_grad_center(128, 4, "UITSLAG", &grad7(rgb(248, 176, 48)), OUTLINE, true);
        f.text_center(128, 14, st.name, rgb(200, 192, 232), OUTLINE, false);
        // ranking board
        let (bx, by) = (8, 26);
        f.rect(bx, by, 240, 66, rgb(16, 16, 40));
        f.rect_outline(bx, by, 240, 66, rgb(248, 136, 24));
        let best = self.results.first().map(|e| e.1).unwrap_or(0);
        for (k, &(i, fr, dnf)) in self.results.iter().enumerate() {
            let y = by + 5 + k as i32 * 15;
            let Some(&(who, bike, ctrl)) = self.names.get(i) else { continue };
            let col = match ctrl {
                Ctrl::Human(0) => rgb(248, 232, 64),
                Ctrl::Human(_) => rgb(96, 224, 248),
                Ctrl::Cpu => rgb(216, 216, 232),
            };
            let pc = [rgb(248, 216, 48), rgb(216, 216, 232), rgb(232, 136, 72), rgb(160, 160, 184)][k.min(3)];
            f.text_grad(bx + 6, y + 2, ordinal(k), &grad7(pc), OUTLINE, true);
            // a small head
            let s = &self.art.set(who, bike).ride[0][0];
            let pal = rider_palette(who);
            for yy in 0..11 {
                for xx in 0..12 {
                    let v = s.get(xx + match bike {
                        Bike::Lig => 0,
                        Bike::Race => 11,
                        _ => 6,
                    }, yy + match bike {
                        Bike::Lig => 7,
                        Bike::Race => 3,
                        _ => 0,
                    });
                    if v != sp::T {
                        f.set(bx + 30 + xx, y + yy, pal[v as usize]);
                    }
                }
            }
            let name = match ctrl {
                Ctrl::Human(_) => format!("{} ({})", who.def().short, bike.def().name),
                Ctrl::Cpu => who.def().name.to_string(),
            };
            f.text_sh(bx + 46, y + 3, &name, col, OUTLINE, false);
            let time = if dnf {
                "DNF".to_string()
            } else if k == 0 {
                fmt_time(fr)
            } else {
                format!("+{}", fmt_gap(fr.saturating_sub(best)))
            };
            f.text_sh(bx + 234 - text_width(&time, false), y + 3, &time, rgb(248, 248, 248), OUTLINE, false);
        }
        // tally
        let n = self.tally[0].len().max(self.tally[1].len()) as u32;
        let shown = if self.t < 20 { 0 } else { ((self.t - 20) / 30 + 1).min(n) } as usize;
        let done = self.t >= 20 + n * 30 + 30;
        let players: Vec<usize> = if self.mode == Mode::Two { vec![0, 1] } else { vec![0] };
        for &p in &players {
            let (x0, w) = if players.len() == 2 { (8 + p as i32 * 122, 118) } else { (40, 176) };
            let y0 = 98;
            let col = if p == 0 { rgb(248, 232, 64) } else { rgb(96, 224, 248) };
            f.rect(x0, y0, w, 96, rgb(16, 16, 40));
            f.rect_outline(x0, y0, w, 96, col);
            let lines = &self.tally[p];
            for (k, l) in lines.iter().enumerate().take(shown) {
                let y = y0 + 5 + k as i32 * 11;
                f.text_sh(x0 + 5, y, &l.label, rgb(216, 216, 232), OUTLINE, false);
                let s = format!("{}", l.pts);
                f.text_sh(x0 + w - 5 - text_width(&s, false), y, &s, rgb(248, 248, 248), OUTLINE, false);
            }
            let sum: i32 = lines.iter().take(shown).map(|l| l.pts).sum();
            f.hline(x0 + 4, x0 + w - 5, y0 + 72, col);
            let s = format!("{}", sum);
            f.text_sh(x0 + 5, y0 + 76, "STAGE", col, OUTLINE, false);
            f.text_grad(x0 + w - 5 - text_width(&s, true), y0 + 75, &s, &grad7(col), OUTLINE, true);
            if done {
                let s = format!("TOUR {}", self.score[p]);
                f.text_sh(x0 + 5, y0 + 86, &s, rgb(184, 184, 216), OUTLINE, false);
            }
        }
        if done && (t / 20) % 2 == 0 {
            let msg = match self.mode {
                Mode::One => {
                    let need = if self.stage + 1 >= STAGES.len() { 1 } else { 2 };
                    if self.last_place < need { "LEKKER BEZIG! PUSH START" } else { "DOE NORMAAL... PUSH START" }
                }
                _ => "PUSH START",
            };
            f.text_center(128, 204, msg, rgb(248, 248, 248), OUTLINE, true);
        }
    }

    // ------------------------------------------------------------ endings

    fn draw_continue(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(16, 32, 48), rgb(24, 48, 64));
        f.logo(128, 26, "CONTINUE?", 3, &SUNSET, OUTLINE, rgb(0, 0, 0));
        let n = format!("{}", self.count.max(0));
        f.logo(128, 64, &n, 6, &grad6(rgb(248, 232, 64)), OUTLINE, rgb(0, 0, 0));
        // you, in the canal, next to the quay where your bike lies
        let wy = 160;
        f.rect(0, wy - 14, W, 14, rgb(120, 64, 48));
        for y in (wy - 14..wy).step_by(4) {
            for x in (0..W).step_by(10) {
                let o = if (y / 4) % 2 == 0 { 0 } else { 5 };
                f.rect(x + o, y, 9, 3, rgb(152, 80, 56));
            }
        }
        f.rect(0, wy - 16, W, 2, rgb(184, 176, 168));
        let bike = BIKES[self.cursor[0]];
        draw_bike_only(f, &self.art, Who::Kees, bike, 200, wy - 16, 0.0);
        f.rect(0, wy, W, H - wy, rgb(48, 96, 104));
        f.dither_rect(0, wy, W, 2, rgb(88, 144, 136), rgb(48, 96, 104));
        for k in 0..10 {
            let x = (k * 29 + (t as i32 / 4)) % 270 - 10;
            f.hline(x, x + 4, wy + 8 + (k * 7) % 20, rgb(88, 144, 136));
        }
        let bob = ((t / 12) % 2) as i32;
        let pal = rider_palette(Who::Kees);
        let s = &self.art.people(Who::Kees).swim;
        blit_person_clip(f, s, 110, wy + 16 + bob, false, &pal, wy + 1);
        f.hline(98, 122, wy + 1, rgb(200, 232, 240));
        self.art.duck.draw(f, 128, wy - 4 + bob, &PROPS, true);
        self.art.cargo_bmp(STAGES[self.stage.min(2)].cargo).draw(f, 60, wy - 2 - bob, &PROPS, false);
        if (t / 40) % 2 == 0 {
            bubble(f, 112, wy - 34, "BRRR!");
        }
        if (t / 20) % 2 == 0 {
            f.text_center(128, 196, "PUSH Z TO GET BACK ON YOUR BIKE", rgb(248, 248, 248), OUTLINE, false);
        }
        f.text_center(128, 210, "THE WIND IS WAITING FOR YOU", rgb(160, 200, 216), OUTLINE, false);
    }

    fn draw_game_over(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(16, 8, 32), rgb(24, 16, 48));
        f.logo(128, 60, "GAME", 4, &grad6(rgb(216, 48, 64)), OUTLINE, rgb(0, 0, 0));
        f.logo(128, 98, "OVER", 4, &grad6(rgb(216, 48, 64)), OUTLINE, rgb(0, 0, 0));
        f.text_center(128, 150, "THE WIND WON. IT ALWAYS DOES.", rgb(200, 200, 232), OUTLINE, false);
        // Oma Riek rides past, says nothing
        let x = (t as i32 * 2) % 360 - 60;
        let crank = ((t / 4) % 8) as usize;
        f.shade_ellipse(x + 2, 190, 13, 2, 5, 8);
        draw_bike_rider(f, &self.art, Who::Oma, Bike::Oma, x, 190, crank, t as f32 * 2.0, false, false, false, None, t);
        if (t / 30) % 2 == 0 {
            bubble(f, x + 4, 156, "...");
        }
    }

    fn draw_champion(&self, f: &mut Frame, t: u32) {
        let cam = t as f32 * 0.3;
        self.title_sc.draw(f, &self.art, cam, t, 0.5, 0.0, (0, H));
        self.title_sc.draw_road(f, cam, (0, H), |_| false);
        self.title_sc.draw_water(f, cam, t, (0, H));
        f.shade_rect(0, 0, W, 72, 2, 3);
        // podium on the road
        let winner = if self.mode == Mode::Two && self.score[1] > self.score[0] { 1 } else { 0 };
        let first = if winner == 0 { Who::Kees } else { Who::Anouk };
        let others: [Who; 2] = if self.mode == Mode::Two { [if winner == 0 { Who::Anouk } else { Who::Kees }, Who::Oma] } else { [Who::Oma, Who::Dennis] };
        let base = 186;
        let steps = [(128, 26, first, "1"), (90, 16, others[0], "2"), (166, 10, others[1], "3")];
        for &(x, h, who, num) in &steps {
            f.rect(x - 18, base - h, 36, h, OUTLINE);
            f.rect(x - 17, base - h + 1, 34, h - 1, rgb(232, 232, 240));
            f.rect(x - 17, base - h + 1, 34, 3, rgb(248, 136, 24));
            f.text_center(x, base - h + 6, num, rgb(48, 48, 80), rgb(184, 184, 200), true);
            let hop = if who == first { (((t as i32) % 30) - 15).abs() / 4 } else { 0 };
            let ppl = self.art.people(who);
            let s: &Sprite = if who == first {
                if hop > 1 { &ppl.victory } else { &ppl.cheer }
            } else if (t / 20) % 2 == 0 {
                &ppl.cheer
            } else {
                &ppl.stand
            };
            blit_person(f, s, x, base - h - hop, x > 128, &rider_palette(who));
        }
        // Oma Riek has opinions about this
        if (t / 60) % 3 == 1 {
            bubble(f, if self.mode == Mode::Two { 170 } else { 94 }, 132, "...");
        }
        // tulips along the front
        for k in 0..16 {
            let x = 8 + k * 16;
            let c = [rgb(232, 48, 64), rgb(248, 216, 48), rgb(248, 136, 24), rgb(232, 96, 168)][(k % 4) as usize];
            f.line(x, 222, x, 212, rgb(64, 144, 64));
            f.rect(x - 2, 208, 5, 5, c);
            f.set(x - 2, 207, c);
            f.set(x + 2, 207, c);
            f.set(x, 207, c);
        }
        for &(x, y, _, c) in &self.confetti {
            f.rect(x as i32, y as i32, 2, 2, c);
        }
        f.logo(128, 6, "KAMPIOEN!", 4, &SUNSET, OUTLINE, LOGO_SHADOW);
        let (line, sub) = if self.mode == Mode::Two {
            (if winner == 0 { "SPELER 1 WINT DE TOUR!" } else { "SPELER 2 WINT DE TOUR!" }, format!("{} - {} PTS", self.score[0], self.score[1]))
        } else {
            ("YOU BEAT THE WIND!", format!("TOUR TOTAL {} PTS", self.score[0]))
        };
        f.text_grad_center(128, 44, line, &grad7(rgb(96, 224, 248)), OUTLINE, true);
        f.text_center(128, 56, &sub, rgb(248, 248, 248), OUTLINE, false);
        f.text_center(128, 196, "HUP HOLLAND HUP!", rgb(248, 136, 24), OUTLINE, true);
    }
}

pub fn fmt_time(frames: u32) -> String {
    let s = frames as f32 / 60.0;
    let m = (s / 60.0) as u32;
    format!("{}:{:04.1}", m, s - m as f32 * 60.0)
}

pub fn fmt_gap(frames: u32) -> String {
    format!("{:.1}", frames as f32 / 60.0)
}

pub fn stripes(f: &mut Frame, t: u32, a: u32, b: u32) {
    for y in 0..H {
        let row = &mut f.px[(y * W) as usize..((y + 1) * W) as usize];
        for (x, p) in row.iter_mut().enumerate() {
            let k = ((x as i32 + y + (t / 2) as i32) / 8) % 2;
            *p = if k == 0 { a } else { b };
        }
    }
}
