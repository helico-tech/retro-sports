//! Scenes and flow: title (with attract demo), runner select, stage intro
//! card, the race, results tally, continue, game over and the champion
//! screen. Pure logic + drawing into a Frame.

use super::audio::Track;
use super::course::{Course, Item, START_X, STAGE_NAMES};
use super::headless::{lineup, CPU_SKILL, GOAT_FAC};
use super::race::*;
use super::render::*;
use super::runners::{self, GERALD, RUNNERS};
use super::scenery::{profile, PropArt, Scenery};
use super::sprites::{Art, TPose};
use crate::engine::gfx::*;
use crate::engine::input::any;
use crate::engine::sprites::blit;
use crate::engine::ui::{grad6, grad7, OUTLINE, SUNSET};
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

/// One line of the results table.
#[derive(Clone, Copy, Debug)]
pub struct Standing {
    pub def: usize,
    pub human: Option<usize>,
    pub time: u32,
    pub finished: bool,
    pub stats: RunStats,
}

/// 2P cup points for 1st..4th.
pub const POINTS: [u32; 4] = [10, 6, 3, 1];

const TAGLINES: [&str; 3] = ["GENTLE HILLS. ANGRY ROOTS. ONE GOAT.", "THE COWS WERE HERE FIRST.", "THE AIR IS THIN. GERALD IS NOT."];
const TIPS: [[&str; 2]; 3] = [
    ["LEFT/RIGHT: PACE  X: JUMP  DOWN: DUCK", "Z AT A TABLE: FOOD. Z AT FANS: HYPE"],
    ["STEEP? TAP Z, C, Z, C TO POWER HIKE", "COWS DON'T MOVE. WALK PAST THEM."],
    ["SNOW DOWNHILL: HOLD DOWN TO GLISSADE", "TOO FAST ON ROCK? HOLD DOWN TO BRAKE"],
];
const SPECIALS: [&str; 4] = ["SPECIAL: HARDLY EVER BONKS", "SPECIAL: GOOD AT EVERYTHING", "SPECIAL: FAST. FALLS OVER A LOT.", "SPECIAL: FLIES DOWNHILL"];
const TICKER: &str = "TODAY'S FORECAST: SUNNY WITH A CHANCE OF GOAT.     PICKLE JUICE IS NOT A SMOOTHIE.     COWS HAVE RIGHT OF WAY.     GERALD HAS NEVER LOST A RACE (HE SAYS).     STROOPWAFELS: 100% TRAIL LEGAL.     DO NOT HIGH FIVE THE MARMOTS.     ";

pub struct App {
    pub scene: Scene,
    pub t: u32,
    pub mode: Mode,
    pub menu: usize,
    pub menu_open: bool,
    pub cursor: [usize; 2],
    pub locked: [bool; 2],
    pub stage: usize,
    pub race: Option<Race>,
    pub scenery: Scenery,
    pub profile: Vec<u32>,
    /// the current stage's course, for the intro card
    preview: Course,
    pub art: Art,
    pub props: PropArt,
    title_race: Race,
    title_sc: Scenery,
    pub sfx: Vec<Sfx>,
    prev: [Pad; 2],
    rng: Rng,
    pub quit: bool,
    pub paused: bool,
    idle: u32,
    pub count: i32,
    pub frame: u32,
    confetti: Vec<(f32, f32, f32, u32)>,
    pub results: Vec<Standing>,
    pub points: [u32; 2],
    pub qualified: bool,
    scratch: RefCell<Frame>,
}

impl App {
    pub fn new(seed: u64) -> Self {
        let title_race = Race::new(1, &[(0, Ctrl::Cpu(Strategy::Smart, 0.5))], 1);
        let preview = Course::new(0);
        let profile_px = profile(&preview, 88, 10);
        App {
            scene: Scene::Title,
            t: 0,
            mode: Mode::One,
            menu: 0,
            menu_open: false,
            cursor: [0, 1],
            locked: [false; 2],
            stage: 0,
            race: None,
            scenery: Scenery::new(0),
            profile: profile_px,
            preview,
            art: Art::new(),
            props: PropArt::new(),
            title_race,
            title_sc: Scenery::new(1),
            sfx: Vec::new(),
            prev: [Pad::default(); 2],
            rng: Rng(seed | 1),
            quit: false,
            paused: false,
            idle: 0,
            count: 0,
            frame: 0,
            confetti: Vec::new(),
            results: Vec::new(),
            points: [0; 2],
            qualified: false,
            scratch: RefCell::new(Frame::new()),
        }
    }

    pub fn music(&self) -> Option<Track> {
        match self.scene {
            Scene::Title | Scene::Select | Scene::Intro => Some(Track::Title),
            Scene::Play => Some(Track::Race),
            Scene::Champion => Some(Track::Win),
            Scene::Results | Scene::Continue | Scene::GameOver => None,
        }
    }

    fn go(&mut self, s: Scene) {
        self.scene = s;
        self.t = 0;
    }

    fn set_stage(&mut self, stage: usize) {
        if self.preview.stage != stage {
            self.scenery = Scenery::new(stage);
            self.preview = Course::new(stage);
            self.profile = profile(&self.preview, 88, 10);
        }
        self.stage = stage;
    }

    pub fn start_race(&mut self, racers: &[(usize, Ctrl)]) {
        let seed = self.rng.next() as u64;
        let stage = self.stage;
        self.set_stage(stage);
        self.race = Some(Race::new(stage, racers, seed));
        self.paused = false;
        self.go(Scene::Play);
    }

    pub fn start_demo(&mut self) {
        self.mode = Mode::Demo;
        let stage = self.rng.int(3) as usize;
        self.set_stage(stage);
        let a = self.rng.int(4) as usize;
        let racers = vec![
            (a, Ctrl::Cpu(Strategy::Smart, 0.7)),
            (GERALD, Ctrl::Goat(GOAT_FAC[stage])),
            ((a + 1) % 4, Ctrl::Cpu(Strategy::Smart, 0.6)),
            ((a + 2) % 4, Ctrl::Cpu(Strategy::Smart, 0.8)),
        ];
        self.start_race(&racers);
    }

    fn racers(&self) -> Vec<(usize, Ctrl)> {
        match self.mode {
            Mode::Two => {
                let (a, b) = (self.cursor[0], self.cursor[1]);
                let other = (0..4).find(|&k| k != a && k != b).unwrap_or(0);
                vec![
                    (other, Ctrl::Cpu(Strategy::Smart, CPU_SKILL[self.stage][0])),
                    (GERALD, Ctrl::Goat(GOAT_FAC[self.stage])),
                    (b, Ctrl::Human(1)),
                    (a, Ctrl::Human(0)),
                ]
            }
            _ => lineup(self.stage, self.cursor[0], Ctrl::Human(0)),
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
            // Esc counts as input, or the title would restart the demo at once
            self.idle = 0;
            match self.scene {
                Scene::Title => self.quit = true,
                _ => {
                    self.race = None;
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
                if self.idle > 900 && self.scene == Scene::Title {
                    self.menu_open = false;
                    self.start_demo();
                }
            }
            Scene::Select => {
                let players = if self.mode == Mode::Two { 2 } else { 1 };
                for (pi, &p) in e.iter().enumerate().take(players) {
                    if !self.locked[pi] {
                        let c = self.cursor[pi];
                        if p.left || p.right {
                            self.cursor[pi] = if p.left { (c + 3) % 4 } else { (c + 1) % 4 };
                            self.sfx.push(Sfx::Blip);
                        }
                        if ok(p) {
                            let other = 1 - pi;
                            if players == 2 && self.locked[other] && self.cursor[other] == self.cursor[pi] {
                                self.sfx.push(Sfx::Moo);
                            } else {
                                self.locked[pi] = true;
                                self.sfx.push(Sfx::Confirm);
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
                    self.set_stage(0);
                    self.points = [0; 2];
                    self.go(Scene::Intro);
                }
            }
            Scene::Intro => {
                if self.t > 300 || (self.t > 40 && (ok(e[0]) || ok(e[1]))) {
                    let r = self.racers();
                    self.start_race(&r);
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
                let Some(race) = self.race.as_mut() else {
                    self.go(Scene::Title);
                    return;
                };
                race.update(&pads);
                self.sfx.append(&mut race.sfx);
                if race.finished() && race.phase_t > 150 {
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
                let ready = self.t > 330;
                if ready && self.t == 331 {
                    self.sfx.push(if self.qualified { Sfx::Crowd } else { Sfx::Baa });
                }
                if (ready && ok(e[0])) || self.t > 900 || (self.t > 60 && ok(e[0]) && self.t < 330) {
                    if self.t < 330 {
                        // skip the tally
                        self.t = 330;
                        return;
                    }
                    self.after_results();
                } else if self.t < 330 && self.t.is_multiple_of(30) && self.t <= 240 {
                    self.sfx.push(Sfx::Blip);
                }
            }
            Scene::Continue => {
                if self.t.is_multiple_of(60) {
                    self.count -= 1;
                    self.sfx.push(Sfx::Blip);
                }
                if ok(e[0]) && self.t > 20 {
                    self.sfx.push(Sfx::Confirm);
                    self.go(Scene::Intro);
                } else if self.count < 0 {
                    self.go(Scene::GameOver);
                    self.sfx.push(Sfx::Baa);
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
                    self.sfx.push(Sfx::Crowd);
                }
                if self.t % 150 == 70 {
                    self.sfx.push(Sfx::Baa);
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

    fn make_results(&mut self) {
        let Some(race) = &self.race else { return };
        let rank = race.ranking();
        self.results = rank
            .iter()
            .map(|&i| {
                let r = &race.runners[i];
                Standing { def: r.def, human: r.human(), time: race.result_time(i), finished: r.finished.is_some(), stats: r.stats }
            })
            .collect();
        let place = |p: usize| self.results.iter().position(|s| s.human == Some(p)).unwrap_or(3);
        match self.mode {
            Mode::One => {
                let pl = place(0);
                self.qualified = if self.stage < 2 { pl < 2 } else { pl == 0 };
            }
            Mode::Two => {
                for p in 0..2 {
                    self.points[p] += POINTS[place(p).min(3)];
                }
                self.qualified = true;
            }
            Mode::Demo => {}
        }
    }

    fn after_results(&mut self) {
        match self.mode {
            Mode::One => {
                if self.qualified {
                    if self.stage >= 2 {
                        self.make_confetti();
                        self.go(Scene::Champion);
                    } else {
                        let s = self.stage + 1;
                        self.set_stage(s);
                        self.go(Scene::Intro);
                    }
                } else {
                    self.count = 9;
                    self.go(Scene::Continue);
                }
            }
            Mode::Two => {
                if self.stage >= 2 {
                    self.make_confetti();
                    self.go(Scene::Champion);
                } else {
                    let s = self.stage + 1;
                    self.set_stage(s);
                    self.go(Scene::Intro);
                }
            }
            Mode::Demo => self.go(Scene::Title),
        }
        self.race = None;
    }

    fn make_confetti(&mut self) {
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
            Scene::Play => self.draw_play(f, t),
            Scene::Results => self.draw_results(f, t),
            Scene::Continue => self.draw_continue(f, t),
            Scene::GameOver => self.draw_game_over(f, t),
            Scene::Champion => self.draw_champion(f, t),
        }
    }

    fn ctx<'a>(&'a self, race: &'a Race, sc: &'a Scenery) -> Ctx<'a> {
        Ctx { race, sc, art: &self.art, props: &self.props, profile: &self.profile }
    }

    fn draw_play(&self, f: &mut Frame, t: u32) {
        let Some(race) = &self.race else { return };
        let cx = self.ctx(race, &self.scenery);
        if race.views.len() >= 2 && !race.cpu_only {
            let v0 = View::band(race.views[0], 0);
            let v1 = View::band(race.views[1], 1);
            draw_view(f, &cx, &v0, t);
            {
                let mut s = self.scratch.borrow_mut();
                draw_view(&mut s, &cx, &v1, t);
                f.copy_band(&s, v1.y0, v1.y0, v1.h);
            }
            draw_mini_hud(f, &cx, race.views[0], 0, t);
            draw_mini_hud(f, &cx, race.views[1], 112, t);
            f.hline(0, W - 1, 111, OUTLINE);
        } else {
            let v = View::full(race.views[0]);
            draw_view(f, &cx, &v, t);
            draw_hud(f, &cx, race.views[0], t);
        }
        if race.phase == Phase::Countdown && !race.cpu_only {
            self.draw_go_card(f, race);
        }
        if self.paused {
            f.shade_rect(0, 33, W, H - 33, 1, 2);
            f.logo(128, 90, "PAUSE", 3, &SUNSET, OUTLINE, rgb(8, 8, 24));
            f.text_center(128, 130, "PUSH START", WHITE, OUTLINE, true);
            f.text_center(128, 146, "ESC: BACK TO THE TITLE", rgb(184, 184, 216), OUTLINE, false);
        }
        if self.mode == Mode::Demo {
            f.rect(0, 212, W, 12, rgb(16, 16, 48));
            if (t / 30).is_multiple_of(2) {
                f.text_center(128, 214, "DEMO PLAY - PUSH START", YELLOW, OUTLINE, true);
            }
        }
    }

    /// Before GO: the pace you start with.
    fn draw_go_card(&self, f: &mut Frame, race: &Race) {
        let y = if race.views.len() >= 2 { 90 } else { 182 };
        let s = "SET YOUR PACE: LEFT / RIGHT";
        let w = text_width(s, false) + 10;
        f.shade_rect(128 - w / 2, y, w, 12, 1, 3);
        f.rect_outline(128 - w / 2 - 1, y - 1, w + 2, 14, rgb(248, 176, 32));
        f.text_center(128, y + 3, s, WHITE, OUTLINE, false);
    }

    fn draw_title(&self, f: &mut Frame, t: u32) {
        let race = &self.title_race;
        let cx = self.ctx(race, &self.title_sc);
        // a slow pan along the meadow
        let base = race.course.spawns.first().map(|s| s.x - 180.0).unwrap_or(2000.0);
        let pan = (t as f32 * 0.35) % 900.0;
        let v = View::backdrop(base + pan, 0, H);
        draw_view(f, &cx, &v, t);
        let cam = camera(race, &v);
        // runners on the trail, and Gerald leaping after them
        let c = &race.course;
        for (k, (def, sx)) in [(0usize, 112i32), (1, 150)].iter().enumerate() {
            let wx = cam.x + *sx as f32;
            let sy = cam.sy(c.height(wx)) + 5 + k as i32 * 2;
            let s = c.slope(wx);
            let poses = if s > 0.15 {
                [TPose::Up1, TPose::Up2, TPose::Up3, TPose::Up4]
            } else if s < -0.15 {
                [TPose::Down1, TPose::Down2, TPose::Down3, TPose::Down4]
            } else {
                [TPose::Run1, TPose::Run2, TPose::Run3, TPose::Run4]
            };
            let p = poses[((t / 7 + k as u32 * 2) % 4) as usize];
            f.shade_ellipse(*sx, sy, 6, 1, 5, 8);
            blit(f, self.art.runner(*def, p), *sx, sy, false, &runners::palette(*def));
        }
        let gx = 66;
        let wx = cam.x + gx as f32;
        let hop = ((t % 40) as f32 / 40.0 * std::f32::consts::PI).sin() * 14.0;
        let gy = cam.sy(c.height(wx)) + 3 - hop as i32;
        self.art.goat[if hop > 3.0 { 2 } else { 0 }].draw(f, gx, gy, false, &self.art.goat_pal);
        if (t / 90) % 3 == 1 {
            bubble(f, gx + 4, gy - 26, "BAAAH!");
        }
        f.shade_rect(0, 0, W, 88, 3, 4);
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
        let alp = [rgb(248, 248, 200), rgb(176, 240, 120), rgb(104, 208, 88), rgb(56, 168, 72), rgb(40, 128, 72), rgb(24, 88, 64)];
        f.logo(116, drop(6, 0), "TRAIL", 5, &alp, OUTLINE, rgb(8, 24, 24));
        f.logo(150, drop(48, 8), "MIX!", 4, &SUNSET, OUTLINE, rgb(24, 8, 40));
        if st > 36 {
            f.rect(0, 89, W, 11, rgb(16, 40, 40));
            f.hline(0, W - 1, 88, rgb(248, 176, 32));
            f.hline(0, W - 1, 100, rgb(248, 176, 32));
            f.text_grad_center(128, 91, "THE ULTRA TRAIL OF MT HELICO", &grad7(rgb(160, 232, 248)), OUTLINE, true);
        }
        if !self.menu_open {
            if (t / 24).is_multiple_of(2) && st > 36 {
                f.text_grad_center(128, 184, "PUSH START", &grad7(WHITE), OUTLINE, true);
            }
        } else {
            f.shade_rect(72, 175, 112, 24, 1, 2);
            f.rect_outline(72, 175, 112, 24, rgb(248, 176, 32));
            for (k, s) in ["1 PLAYER", "2 PLAYERS"].iter().enumerate() {
                let y = 177 + k as i32 * 11;
                let sel = self.menu == k;
                let col = if sel { YELLOW } else { rgb(184, 184, 216) };
                f.text_sh(100, y, s, col, OUTLINE, true);
                if sel {
                    let bob = ((t / 6) % 2) as i32;
                    head_icon(f, &self.art, GERALD, 84 + bob, y - 3);
                }
            }
        }
        f.rect(0, 200, W, 24, rgb(16, 16, 48));
        f.hline(0, W - 1, 199, rgb(248, 176, 32));
        let w = text_width(TICKER, false) + 6;
        let off = (t as i32 / 2) % w;
        for k in 0..3 {
            f.text_sh(k * w - off, 202, TICKER, rgb(248, 216, 48), OUTLINE, false);
        }
        f.text_center(128, 213, "(C) 1992 HELICO SOFT", rgb(248, 232, 200), rgb(96, 48, 32), false);
    }

    fn draw_select(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(16, 48, 48), rgb(24, 64, 56));
        f.text_grad_center(128, 4, "CHOOSE YOUR RUNNER", &grad7(rgb(248, 200, 48)), OUTLINE, true);
        let two = self.mode == Mode::Two;
        for k in 0..4 {
            let d = &RUNNERS[k];
            let (cx, cy) = (6 + k as i32 * 62, 16);
            let hover: Vec<usize> = (0..2).filter(|&p| (p == 0 || two) && self.cursor[p] == k).collect();
            f.rect(cx, cy, 58, 78, rgb(8, 24, 32));
            f.rect(cx + 2, cy + 2, 54, 11, shade(d.shirt.0, 1, 2));
            f.rect_outline(cx, cy, 58, 78, shade(d.shirt.0, 2, 3));
            f.text_grad(cx + 29 - text_width(d.short, false) / 2, cy + 4, d.short, &grad7(d.shirt.0), OUTLINE, false);
            // a little mountain view behind them
            for yy in cy + 14..cy + 62 {
                let c = if yy < cy + 30 || (yy == cy + 30 && (yy & 1) == 0) { rgb(88, 152, 216) } else { rgb(136, 184, 232) };
                f.hline(cx + 2, cx + 55, yy, c);
            }
            let (mx, my) = (cx + 22 + (k as i32 * 7) % 16, cy + 24);
            f.fill_poly(&[(cx + 2, cy + 62), (mx, my), (cx + 56, cy + 62)], rgb(112, 128, 184));
            f.fill_poly(&[(mx, my), (mx + 8, my + 9), (mx + 3, my + 8), (mx - 1, my + 11), (mx - 6, my + 7)], rgb(240, 248, 248));
            f.fill_poly(&[(cx + 2, cy + 62), (cx + 2, cy + 48), (cx + 20, cy + 56), (cx + 40, cy + 50), (cx + 56, cy + 54), (cx + 56, cy + 62)], rgb(64, 144, 80));
            // a little trail under them
            f.rect(cx + 2, cy + 62, 54, 4, rgb(184, 136, 88));
            f.rect(cx + 2, cy + 66, 54, 10, rgb(136, 88, 56));
            f.hline(cx + 2, cx + 55, cy + 62, rgb(216, 168, 112));
            let active = !hover.is_empty();
            let pose = if active {
                [TPose::Run1, TPose::Run2, TPose::Run3, TPose::Run4][((t / 6) as usize + k) % 4]
            } else {
                TPose::Stand
            };
            blit(f, self.art.runner(k, pose), cx + 29, cy + 64, false, &runners::palette(k));
            for &p in &hover {
                let col = if p == 0 { YELLOW } else { CYAN };
                let locked = self.locked[p];
                if locked || (t / 8).is_multiple_of(2) {
                    f.rect_outline(cx - 1, cy - 1, 60, 80, col);
                    f.rect_outline(cx, cy, 58, 78, col);
                }
                let tx = if p == 0 { cx + 3 } else { cx + 40 };
                f.rect(tx - 1, cy + 15, 15, 9, col);
                f.text(tx, cy + 16, if p == 0 { "1P" } else { "2P" }, rgb(16, 16, 48));
                if locked {
                    f.text_grad_center(cx + 29, cy + 34, "OK!", &grad7(col), OUTLINE, true);
                }
            }
        }
        // detail panels
        let panels: Vec<(usize, i32, i32)> = if two { vec![(0, 4, 122), (1, 130, 122)] } else { vec![(0, 8, 240)] };
        for &(p, x, w) in &panels {
            let k = self.cursor[p];
            let d = &RUNNERS[k];
            let col = if p == 0 { YELLOW } else { CYAN };
            let y = 100;
            f.rect(x, y, w, 96, rgb(8, 16, 32));
            f.rect_outline(x, y, w, 96, col);
            f.text_grad(x + 5, y + 4, if two { d.short } else { d.name }, &grad7(d.shirt.0), OUTLINE, true);
            let stats = [("CLIMB", d.climb), ("DESCEND", d.descend), ("STAMINA", d.endurance), ("AGILITY", d.agility)];
            for (r, (lab, val)) in stats.iter().enumerate() {
                let yy = y + 17 + r as i32 * 9;
                f.text(x + 5, yy, lab, rgb(184, 200, 216));
                let px = x + if two { 52 } else { 56 };
                for pip in 0..5 {
                    let c = if pip < *val { d.acc.0.max(rgb(96, 96, 96)) } else { rgb(40, 40, 72) };
                    let c = if pip < *val && d.acc.0 == rgb(248, 248, 248) { d.shirt.0 } else { c };
                    f.rect(px + pip * 7, yy, 6, 7, OUTLINE);
                    f.rect(px + pip * 7 + 1, yy + 1, 4, 5, c);
                }
            }
            let chars = ((w - 10) / 6) as usize;
            let mut yy = y + 55;
            for line in wrap(d.bio, chars).iter().take(3) {
                f.text_sh(x + 5, yy, line, WHITE, OUTLINE, false);
                yy += 9;
            }
            if !two {
                f.text_sh(x + 5, y + 71, SPECIALS[k], rgb(160, 232, 248), OUTLINE, false);
            }
            f.text_sh(x + 5, y + 85, d.fuel, rgb(248, 176, 32), OUTLINE, false);
            if !two {
                // Gerald peeks in
                let gx = x + w - 22;
                self.art.goat[3].draw(f, gx, y + 46, true, &self.art.goat_pal);
                if (t / 60).is_multiple_of(2) {
                    bubble(f, gx - 30, y + 18, "NOT ME?");
                }
            }
        }
        if two {
            f.text_center(128, 205, "1P: ARROWS + Z   2P: A D + F", rgb(200, 200, 232), OUTLINE, false);
            f.text_center(128, 214, "SAME RUNNER? NOPE. THE COW SAYS NO.", rgb(144, 168, 184), OUTLINE, false);
        } else {
            f.text_center(128, 210, "LEFT/RIGHT: PICK   Z: OK   X: BACK", rgb(200, 200, 232), OUTLINE, false);
        }
    }

    fn draw_intro(&self, f: &mut Frame, t: u32) {
        let sc = &self.scenery;
        sc.draw_back(f, 300.0 + self.t as f32 * 0.6, 0.0, 0, H, t, 150);
        f.shade_rect(0, 0, W, H, 1, 2);
        let slide = (30 - self.t as i32).max(0) * 8;
        let stage_txt = ["STAGE 1", "STAGE 2", "STAGE 3"][self.stage.min(2)];
        f.logo(128 - slide, 4, stage_txt, 2, &grad6(rgb(248, 248, 248)), OUTLINE, rgb(8, 8, 24));
        let name = STAGE_NAMES[self.stage.min(2)];
        let words: Vec<&str> = name.split(' ').collect();
        f.logo(128 + slide, 24, words[0], 4, &SUNSET, OUTLINE, rgb(8, 8, 24));
        if words.len() > 1 {
            f.logo(128 - slide, 58, words[1], 4, &SUNSET, OUTLINE, rgb(8, 8, 24));
        }
        // the course profile
        let (px, py, pw, ph) = (40, 94, 176, 24);
        f.rect(px - 3, py - 3, pw + 6, ph + 6, rgb(16, 16, 48));
        f.rect_outline(px - 3, py - 3, pw + 6, ph + 6, rgb(248, 176, 32));
        let course = &self.preview;
        let (lo, hi) = (course.min_h, course.max_h.max(course.min_h + 1.0));
        let shown = ((self.t as i32 * 3).min(pw)) as usize;
        for x in 0..shown as i32 {
            let wx = START_X + (course.finish - START_X) * x as f32 / pw as f32;
            let top = py + ph - 1 - ((course.height(wx) - lo) / (hi - lo) * (ph - 2) as f32) as i32;
            for y in top..py + ph {
                f.set(px + x, y, if y == top { rgb(160, 224, 120) } else if (x + y) % 2 == 0 { rgb(56, 104, 72) } else { rgb(48, 88, 64) });
            }
        }
        for a in &course.aids {
            let x = px + ((a.x - START_X) / (course.finish - START_X) * pw as f32) as i32;
            f.text(x - 2, py + ph - 8, "+", if a.item == Item::Pickle { rgb(120, 232, 96) } else { WHITE });
        }
        let dist = format!("{:.1} KM", (course.finish - START_X) / 1000.0);
        f.text_sh(px + 2, py + 1, &dist, rgb(200, 200, 232), OUTLINE, false);
        let goal = match self.mode {
            Mode::One if self.stage < 2 => "TOP 2 GO ON",
            Mode::One => "WIN IT ALL. BEAT GERALD.",
            _ => "CUP RACE: 10, 6, 3, 1 POINTS",
        };
        f.text_center(128, py + ph + 5, goal, YELLOW, OUTLINE, false);
        // the line-up
        let racers = self.racers();
        let n = racers.len() as i32;
        for (k, (def, ctrl)) in racers.iter().rev().enumerate() {
            let x = 128 - (n - 1) * 30 + k as i32 * 60;
            let y = 150;
            let you = matches!(ctrl, Ctrl::Human(_));
            f.rect(x - 16, y, 32, 30, rgb(16, 16, 48));
            f.rect_outline(x - 16, y, 32, 30, if you { YELLOW } else { rgb(96, 104, 160) });
            if *def == GERALD {
                let fr = if (t / 20).is_multiple_of(2) { 3 } else { 0 };
                self.art.goat[fr].draw(f, x, y + 26, true, &self.art.goat_pal);
            } else {
                blit(f, self.art.runner(*def, TPose::Ready), x, y + 28, false, &runners::palette(*def));
            }
            let label = match ctrl {
                Ctrl::Human(0) => "1P",
                Ctrl::Human(_) => "2P",
                _ => runners::name(*def),
            };
            f.text_center(x, y + 32, label, if you { YELLOW } else { WHITE }, OUTLINE, false);
        }
        // tagline and tips
        if self.t > 40 {
            f.text_center(128, 138, TAGLINES[self.stage.min(2)], rgb(248, 232, 200), OUTLINE, false);
        }
        f.rect(0, 194, W, 30, rgb(16, 16, 48));
        f.hline(0, W - 1, 193, rgb(248, 176, 32));
        for (k, line) in TIPS[self.stage.min(2)].iter().enumerate() {
            f.text_center(128, 198 + k as i32 * 11, line, if k == 0 { WHITE } else { rgb(160, 232, 248) }, OUTLINE, false);
        }
    }

    fn draw_results(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(16, 24, 56), rgb(24, 32, 72));
        let title = format!("STAGE {} RESULTS", self.stage + 1);
        f.text_grad_center(128, 4, &title, &grad7(rgb(248, 200, 48)), OUTLINE, true);
        // tally for the player(s)
        let humans: Vec<&Standing> = {
            let mut v: Vec<&Standing> = self.results.iter().filter(|s| s.human.is_some()).collect();
            v.sort_by_key(|s| s.human);
            v
        };
        let st = self.t as i32;
        for (k, s) in humans.iter().enumerate() {
            let x = if humans.len() == 2 { 8 + k as i32 * 124 } else { 40 };
            let w = if humans.len() == 2 { 116 } else { 176 };
            let y = 18;
            let col = if s.human == Some(0) { YELLOW } else { CYAN };
            f.rect(x, y, w, 86, rgb(8, 8, 32));
            f.rect_outline(x, y, w, 86, col);
            head_icon(f, &self.art, s.def, x + 4, y + 3);
            f.text_grad(x + 19, y + 4, runners::name(s.def), &grad7(RUNNERS[s.def].shirt.0), OUTLINE, true);
            let lines: [(&str, i64, i64); 6] = [
                ("TIME", s.time as i64, 0),
                ("CLEAN JUMPS", s.stats.clean as i64, 10),
                ("HIGH FIVES", s.stats.fives as i64, 50),
                ("SNACKS", s.stats.aid as i64, 20),
                ("FACEPLANTS", s.stats.falls as i64, 0),
                ("STYLE", s.stats.style as i64, 0),
            ];
            for (r, (lab, val, _)) in lines.iter().enumerate() {
                let show_at = 20 + r as i32 * 30;
                if st < show_at {
                    continue;
                }
                let yy = y + 18 + r as i32 * 11;
                let prog = ((st - show_at) as f32 / 25.0).min(1.0);
                f.text_sh(x + 4, yy, lab, rgb(184, 200, 232), OUTLINE, false);
                let txt = if r == 0 {
                    time_str((*val as f32 * prog) as u32)
                } else {
                    format!("{}", (*val as f32 * prog) as i64)
                };
                let c = if r == 4 && *val > 0 { rgb(248, 136, 72) } else if r == 5 { YELLOW } else { WHITE };
                f.text_sh(x + w - 4 - text_width(&txt, false), yy, &txt, c, OUTLINE, false);
            }
        }
        // the ranking
        if st > 200 {
            let y0 = 110;
            f.rect(24, y0 - 2, 208, 54, rgb(8, 8, 32));
            f.rect_outline(24, y0 - 2, 208, 54, rgb(248, 176, 32));
            for (k, s) in self.results.iter().enumerate() {
                let y = y0 + 1 + k as i32 * 12;
                if st < 200 + k as i32 * 12 {
                    continue;
                }
                let you = s.human.is_some();
                let col = match s.human {
                    Some(0) => YELLOW,
                    Some(_) => CYAN,
                    None if s.def == GERALD => WHITE,
                    None => rgb(184, 184, 216),
                };
                let place = format!("{}.", k + 1);
                f.text_sh(30, y, &place, col, OUTLINE, true);
                head_icon(f, &self.art, s.def, 46, y - 3);
                let name = runners::name(s.def);
                f.text_sh(62, y, name, col, OUTLINE, true);
                let tm = if s.finished { time_str(s.time) } else { format!("~{}", time_str(s.time)) };
                f.text_sh(150, y, &tm, col, OUTLINE, false);
                if self.mode == Mode::Two {
                    if s.human.is_some() {
                        let pts = format!("+{}", POINTS[k.min(3)]);
                        f.text_sh(206, y, &pts, col, OUTLINE, false);
                    }
                } else if self.mode == Mode::One && ((self.stage < 2 && k < 2) || (self.stage == 2 && k == 0)) {
                    f.text_sh(210, y, "Q", rgb(120, 232, 96), OUTLINE, true);
                }
                if you && (t / 10).is_multiple_of(2) {
                    f.text_sh(16, y, ">", col, OUTLINE, true);
                }
            }
        }
        if st > 330 {
            let (txt, col) = match self.mode {
                Mode::Two => ("", WHITE),
                Mode::One if self.qualified && self.stage == 2 => ("KING OF THE MOUNTAIN!", YELLOW),
                Mode::One if self.qualified => ("QUALIFIED!", rgb(120, 232, 96)),
                _ => ("NOT QUALIFIED", rgb(248, 72, 72)),
            };
            if self.mode == Mode::Two {
                let s = format!("CUP POINTS  1P: {}   2P: {}", self.points[0], self.points[1]);
                f.text_grad_center(128, 172, &s, &grad7(YELLOW), OUTLINE, true);
            } else {
                let scale = if txt.len() > 12 { 2 } else { 3 };
                f.logo(128, 168, txt, scale, &grad6(col), OUTLINE, rgb(8, 8, 24));
            }
            let goat_won = self.results.first().map(|s| s.def == GERALD).unwrap_or(false);
            let k = (self.stage + self.results.iter().map(|s| s.time as usize).sum::<usize>()) % 3;
            let quip = if goat_won {
                ["GERALD: BAAAH! (THAT MEANS 'TOO SLOW')", "GERALD: FOUR LEGS. JUST SAYING.", "GERALD: I ALSO ATE A SIGN."][k]
            } else {
                ["GERALD: I WAS EATING. IT DOESN'T COUNT.", "GERALD: A COW WAS IN MY WAY. BAAH.", "GERALD: REMATCH. TOMORROW. 5 AM."][k]
            };
            f.text_center(128, 198, quip, rgb(200, 200, 232), OUTLINE, false);
            if (t / 20).is_multiple_of(2) {
                f.text_center(128, 212, "PUSH Z", WHITE, OUTLINE, true);
            }
        }
    }

    fn draw_continue(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(32, 8, 24), rgb(48, 16, 40));
        f.logo(128, 30, "CONTINUE?", 3, &SUNSET, OUTLINE, rgb(0, 0, 0));
        let n = format!("{}", self.count.max(0));
        f.logo(128, 72, &n, 6, &grad6(rgb(248, 232, 64)), OUTLINE, rgb(0, 0, 0));
        let me = self.cursor[0];
        f.rect(0, 170, W, 3, rgb(96, 56, 40));
        blit(f, self.art.runner(me, TPose::Faceplant), 120, 170, false, &runners::muddy(&runners::palette(me)));
        let fr = if (t / 15).is_multiple_of(2) { 3 } else { 1 };
        self.art.goat[fr].draw(f, 160, 171, true, &self.art.goat_pal);
        if (t / 40).is_multiple_of(2) {
            bubble(f, 150, 136, "GET UP! BAAAH!");
        }
        if (t / 20).is_multiple_of(2) {
            f.text_center(128, 192, "PUSH Z TO TRY AGAIN", WHITE, OUTLINE, false);
        }
    }

    fn draw_game_over(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(16, 8, 32), rgb(24, 16, 48));
        f.logo(128, 40, "GAME", 4, &grad6(rgb(216, 48, 64)), OUTLINE, rgb(0, 0, 0));
        f.logo(128, 78, "OVER", 4, &grad6(rgb(216, 48, 64)), OUTLINE, rgb(0, 0, 0));
        // Gerald on top of his mountain
        f.fill_poly(&[(96, 190), (128, 150), (160, 190)], rgb(120, 120, 144));
        f.fill_poly(&[(128, 150), (160, 190), (136, 190)], rgb(96, 96, 120));
        f.fill_poly(&[(122, 158), (128, 150), (134, 158)], WHITE);
        let fr = if (t / 20).is_multiple_of(2) { 3 } else { 1 };
        self.art.goat[fr].draw(f, 128, 152, false, &self.art.goat_pal);
        f.text_center(128, 196, "GERALD KEEPS THE MOUNTAIN.", rgb(200, 200, 232), OUTLINE, false);
        f.text_center(128, 208, "FOR NOW.", rgb(144, 144, 184), OUTLINE, false);
    }

    fn draw_champion(&self, f: &mut Frame, t: u32) {
        // the village finish as backdrop
        let race_c = &self.title_race;
        let sc = &self.title_sc;
        let cx = self.ctx(race_c, sc);
        let fin = race_c.course.finish;
        let v = View::backdrop(fin - 206.0, 0, H);
        draw_view(f, &cx, &v, t);
        f.shade_rect(0, 0, W, 64, 2, 3);
        let cam = camera(race_c, &v);
        let ground = cam.sy(race_c.course.height(fin)) + 6;
        let (px, py) = (100, ground);
        self.props.podium.draw(f, px, py);
        let (winner, second, third) = self.podium();
        let hop = |k: u32| -> i32 { (((t as i32 + k as i32 * 11) % 30) - 15).abs() };
        for (k, (def, x, step)) in [(winner, px, 30), (second, px - 30, 20), (third, px + 30, 12)].iter().enumerate() {
            let y = py - step;
            if *def == GERALD {
                let fr = if (t / 12).is_multiple_of(2) { 3 } else { 1 };
                self.art.goat[fr].draw(f, *x, y + 1, k > 0, &self.art.goat_pal);
            } else if *def < 4 {
                let h = if k == 0 { (15 - hop(0)).max(0) / 2 } else { 0 };
                let pose = if k == 0 {
                    if hop(0) < 8 { TPose::Victory } else { TPose::Stand }
                } else if (t / 30 + k as u32).is_multiple_of(2) {
                    TPose::Cheer1
                } else {
                    TPose::Stand
                };
                blit(f, self.art.runner(*def, pose), *x, y - h, false, &runners::palette(*def));
            }
        }
        for &(x, y, _, c) in &self.confetti {
            f.rect(x as i32, y as i32, 2, 2, c);
        }
        let (l1, l2) = if self.mode == Mode::Two { ("CUP", "CHAMPION!") } else { ("KING OF THE", "MOUNTAIN!") };
        f.logo(128, 4, l1, 3, &SUNSET, OUTLINE, rgb(8, 8, 24));
        f.logo(128, 30, l2, 3, &SUNSET, OUTLINE, rgb(8, 8, 24));
        let who = if winner < 4 { RUNNERS[winner].name } else { "GERALD" };
        let s = if self.mode == Mode::Two {
            let p = if self.points[0] >= self.points[1] { 1 } else { 2 };
            format!("{}P {} WINS THE CUP!", p, RUNNERS[self.cursor[p - 1]].short)
        } else {
            format!("{}!", who)
        };
        f.text_grad_center(128, 62, &s, &grad7(CYAN), OUTLINE, true);
        f.rect(0, 200, W, 24, rgb(16, 16, 48));
        f.hline(0, W - 1, 199, rgb(248, 176, 32));
        f.text_center(128, 203, "GERALD DEMANDS A REMATCH.", rgb(200, 200, 232), OUTLINE, false);
        f.text_center(128, 213, "THANKS FOR PLAYING", WHITE, OUTLINE, true);
    }

    /// Who stands on the podium: (1st, 2nd, 3rd) runner indexes or GERALD.
    fn podium(&self) -> (usize, usize, usize) {
        let mut order: Vec<usize> = self.results.iter().map(|s| s.def).collect();
        if self.mode == Mode::Two {
            let (a, b) = (self.cursor[0], self.cursor[1]);
            let (w, l) = if self.points[0] >= self.points[1] { (a, b) } else { (b, a) };
            order = vec![w, GERALD, l];
        } else {
            // 1P: you won the last stage, so you stand on top
            let me = self.cursor[0];
            order.retain(|&d| d != me);
            order.insert(0, me);
        }
        while order.len() < 3 {
            order.push(GERALD);
        }
        (order[0], order[1], order[2])
    }
}

pub fn bubble(f: &mut Frame, x: i32, y: i32, text: &str) {
    let w = text_width(text, false) + 8;
    let bx = (x - w / 2).clamp(2, W - w - 2);
    f.rect(bx - 1, y - 1, w + 2, 13, OUTLINE);
    f.rect(bx, y, w, 11, WHITE);
    let tx = x.clamp(bx + 3, bx + w - 4);
    for k in 0..4 {
        f.hline(tx - 2 + k, tx + 1 - k / 2, y + 11 + k, if k == 3 { OUTLINE } else { WHITE });
        f.set(tx - 3 + k, y + 11 + k, OUTLINE);
    }
    f.text(bx + 4, y + 2, text, rgb(16, 16, 48));
}

const WHITE: u32 = rgb(248, 248, 248);

fn stripes(f: &mut Frame, t: u32, a: u32, b: u32) {
    for y in 0..H {
        let row = (y * W) as usize;
        for x in 0..W {
            let k = ((x + y + (t / 2) as i32) / 8) % 2;
            f.px[row + x as usize] = if k == 0 { a } else { b };
        }
    }
}

/// Break text into lines of at most `n` characters, on spaces.
pub fn wrap(s: &str, n: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for w in s.split(' ') {
        if !line.is_empty() && line.len() + 1 + w.len() > n {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(w);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}
