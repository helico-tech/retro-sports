//! Scenes and flow: title, thief select, the job card, the street, results,
//! continue, game over and the champion screen. 1P is a ladder of three
//! jobs; 2P is two rounds of thief versus BOA with the roles swapped.

use super::art::*;
use super::audio::Track;
use super::bot::BotKind;
use super::data::*;
use super::render::*;
use super::scenery::*;
use super::world::*;
use crate::engine::gfx::*;
use crate::engine::input::{any, edge, Pad};
use crate::engine::sprites as sp;
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
/// How long the attract demo runs.
pub const DEMO_LEN: u32 = 60 * 45;
/// The 2P street.
pub const STAGE_2P: usize = 1;
/// 2P split screen: the rows of each view that are shown.
pub const CROP0: i32 = 101;
pub const CROP_H: i32 = 102;

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
    /// chosen thief per player (index into THIEVES)
    pub cursor: [usize; 2],
    pub locked: [bool; 2],
    pub stage: usize,
    /// 2P: round 0 (1P steals) or 1 (2P steals)
    pub round: usize,
    pub world: Option<World>,
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
    /// money per player over the whole game
    pub score: [i32; 2],
    pub tally: [Vec<Line>; 2],
    pub cleared: bool,
    pub outcome: Option<Outcome>,
    pub tries: usize,
    seed2p: u64,
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
            round: 0,
            world: None,
            scenery: None,
            title_sc: Scenery::new(Venue::Gracht, 768.0, 11),
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
            tally: [Vec::new(), Vec::new()],
            cleared: false,
            outcome: None,
            tries: 0,
            seed2p: 1,
            scratch: RefCell::new(Frame::new()),
        }
    }

    pub fn music(&self) -> Option<Track> {
        match self.scene {
            Scene::Title | Scene::Select | Scene::Intro | Scene::Results => Some(Track::Title),
            Scene::Play => {
                if self.world.as_ref().map(|w| w.chasing).unwrap_or(false) {
                    Some(Track::Chase)
                } else {
                    Some(Track::Sneak)
                }
            }
            Scene::Champion => Some(Track::Fanfare),
            Scene::Continue | Scene::GameOver => None,
        }
    }

    fn go(&mut self, s: Scene) {
        self.scene = s;
        self.t = 0;
    }

    fn stage_now(&self) -> usize {
        if self.mode == Mode::Two {
            STAGE_2P
        } else {
            self.stage.min(STAGES.len() - 1)
        }
    }

    fn ensure_scenery(&mut self, stage: usize) {
        let def = &STAGES[stage];
        if self.scenery.as_ref().map(|s| s.venue != def.venue || s.len != def.len as i32).unwrap_or(true) {
            self.scenery = Some(Scenery::new(def.venue, def.len, 0x51 + stage as u64 * 7));
        }
    }

    /// 2P: who steals this round.
    pub fn thief_player(&self) -> usize {
        self.round % 2
    }

    pub fn start_play(&mut self) {
        let st = self.stage_now();
        let w = match self.mode {
            Mode::One => World::new(st, THIEVES[self.cursor[0]], Ctrl::Human(0), None, self.rng.next() as u64, None),
            Mode::Two => {
                let tp = self.thief_player();
                World::new(st, THIEVES[self.cursor[tp]], Ctrl::Human(tp), Some(1 - tp), self.seed2p, None)
            }
            Mode::Demo => World::new(st, THIEVES[self.rng.int(4) as usize], Ctrl::Cpu, None, self.rng.next() as u64, Some(BotKind::Cpu(0.75))),
        };
        self.world = Some(w);
        self.ensure_scenery(st);
        self.paused = false;
        self.go(Scene::Play);
    }

    /// For the screenshot script: straight to this stage's card.
    pub fn go_intro_for_shots(&mut self) {
        self.world = None;
        self.go(Scene::Intro);
    }

    /// For the screenshot script: straight to the champion screen.
    pub fn go_champion_for_shots(&mut self) {
        self.make_confetti();
        self.go(Scene::Champion);
    }

    pub fn start_demo(&mut self) {
        self.mode = Mode::Demo;
        self.stage = self.rng.int(3) as usize;
        self.start_play();
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
                    self.world = None;
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
                for (pi, &p) in e.iter().enumerate().take(players) {
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
                            self.sfx.push(Sfx::Unlock);
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
                    self.round = 0;
                    self.tries = 0;
                    self.score = [0; 2];
                    self.seed2p = self.rng.next() as u64;
                    self.go(Scene::Intro);
                }
            }
            Scene::Intro => {
                if self.t == 2 {
                    let st = self.stage_now();
                    self.ensure_scenery(st);
                }
                if self.t > 600 || (self.t > 40 && (ok(e[0]) || ok(e[1]))) {
                    self.sfx.push(Sfx::Confirm);
                    self.start_play();
                }
            }
            Scene::Play => {
                if self.mode == Mode::Demo {
                    if any(e[0]) || any(e[1]) {
                        self.world = None;
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
                let Some(w) = self.world.as_mut() else {
                    self.go(Scene::Title);
                    return;
                };
                let pads_in = if self.mode == Mode::Demo { [Pad::default(); 2] } else { pads };
                w.update(&pads_in);
                self.sfx.append(&mut w.sfx);
                if self.mode == Mode::Demo && (self.t > DEMO_LEN || w.finished()) {
                    self.world = None;
                    self.go(Scene::Title);
                    return;
                }
                if w.finished() {
                    self.make_results();
                    self.go(Scene::Results);
                }
            }
            Scene::Results => {
                let n = self.tally[0].len().max(self.tally[1].len()) as u32;
                let done_at = 20 + n * 30 + 30;
                if self.t > 20 && self.t <= 20 + n * 30 && (self.t - 20).is_multiple_of(30) {
                    self.sfx.push(Sfx::Blip);
                }
                if self.t == done_at {
                    self.sfx.push(if self.cleared || self.mode == Mode::Two { Sfx::Cash } else { Sfx::Busted });
                }
                if self.t > done_at + 30 && ok(e[0]) || self.t > done_at + 600 {
                    self.after_results();
                } else if self.t > 30 && self.t < done_at && ok(e[0]) {
                    self.t = done_at;
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

    /// The stage's money, line by line, per player.
    fn make_results(&mut self) {
        let Some(w) = self.world.as_ref() else { return };
        let th = &w.thief;
        let cleared = w.result == Some(Outcome::Cleared);
        let mut thief = Vec::new();
        for &(kind, _, on) in &th.delivered {
            let v = kind.def().value;
            if on {
                thief.push(Line { label: kind.def().name.to_string(), pts: v });
            } else {
                thief.push(Line { label: format!("{} +", kind.def().name), pts: v / 2 });
            }
        }
        if cleared {
            thief.push(Line { label: "BESTELD!".into(), pts: 200 });
            let s = w.time_left / 60;
            thief.push(Line { label: format!("TIJD {}S", s), pts: s as i32 * 2 });
            if th.stats.chases == 0 {
                thief.push(Line { label: "NOOIT GEZIEN".into(), pts: 150 });
            }
        }
        if th.fines > 0 {
            thief.push(Line { label: format!("BOETES X {}", th.fines), pts: -(th.fines as i32) * FINE });
        }
        if th.stats.swims > 0 {
            thief.push(Line { label: "ZWEMDIPLOMA".into(), pts: 25 });
        }
        if thief.is_empty() {
            thief.push(Line { label: "NIKS GEJAT".into(), pts: 0 });
        }
        self.cleared = cleared;
        self.outcome = w.result;
        match self.mode {
            Mode::Two => {
                let tp = self.thief_player();
                let bp = 1 - tp;
                let mut boa = Vec::new();
                if let Some(h) = w.human_boa {
                    let b = &w.boas[h];
                    boa.push(Line { label: format!("BOETES X {}", b.fines), pts: b.fines as i32 * FINE });
                    if b.complaints > 0 {
                        boa.push(Line { label: format!("KLACHTEN X {}", b.complaints), pts: -(b.complaints as i32) * COMPLAINT });
                    }
                }
                let mut tally: [Vec<Line>; 2] = [Vec::new(), Vec::new()];
                tally[tp] = thief;
                tally[bp] = boa;
                for (s, l) in self.score.iter_mut().zip(tally.iter()) {
                    *s += l.iter().map(|x| x.pts).sum::<i32>();
                }
                self.tally = tally;
            }
            _ => {
                if cleared {
                    self.score[0] += thief.iter().map(|x| x.pts).sum::<i32>();
                }
                self.tally = [thief, Vec::new()];
            }
        }
    }

    fn after_results(&mut self) {
        match self.mode {
            Mode::One => {
                if self.cleared {
                    self.stage += 1;
                    self.tries = 0;
                    if self.stage >= STAGES.len() {
                        self.make_confetti();
                        self.go(Scene::Champion);
                    } else {
                        self.go(Scene::Intro);
                    }
                } else {
                    self.tries += 1;
                    self.count = 9;
                    self.go(Scene::Continue);
                }
            }
            Mode::Two => {
                self.round += 1;
                if self.round >= 2 {
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
        let (Some(w), Some(sc)) = (&self.world, &self.scenery) else { return };
        if w.views.len() >= 2 {
            // split screen: player 1 on top, whatever their role
            let mut s = self.scratch.borrow_mut();
            let tp = self.thief_player();
            let order = if tp == 0 { [0usize, 1] } else { [1, 0] };
            for (slot, &v) in order.iter().enumerate() {
                draw_view(&mut s, w, sc, &self.art, v, t, (CROP0, CROP0 + CROP_H));
                let dst = if slot == 0 { 9 } else { 122 };
                f.copy_band(&s, CROP0, dst, CROP_H);
                draw_mini_hud(f, w, v, if slot == 0 { 0 } else { 113 }, slot, t);
                if let Some(b) = &w.banners[v] {
                    draw_banner(f, b, dst + CROP_H / 2 - 10, true);
                }
                if w.views[v] == View::Thief && w.thief.state == TState::Pick {
                    draw_lock_panel(f, w, &self.art, dst + CROP_H - 28, t);
                }
            }
            f.hline(0, W - 1, 111, rgb(248, 136, 24));
            f.hline(0, W - 1, 112, OUTLINE);
        } else {
            draw_view(f, w, sc, &self.art, 0, t, (34, H));
            if w.shake > 0 {
                let dx = if (w.shake / 2) % 2 == 0 { 1 } else { -1 };
                for y in 34..H {
                    let row = &mut f.px[(y * W) as usize..((y + 1) * W) as usize];
                    if dx > 0 {
                        row.rotate_right(1);
                    } else {
                        row.rotate_left(1);
                    }
                }
            }
            draw_hud(f, w, &self.art, t);
            if w.thief.state == TState::Pick {
                draw_lock_panel(f, w, &self.art, 196, t);
            }
            if let Some(b) = &w.banners[0] {
                draw_banner(f, b, 70, false);
            }
        }
        if w.phase == Phase::Countdown && self.mode == Mode::One {
            self.draw_controls_tip(f);
        }
        if self.paused {
            f.shade_rect(0, 0, W, H, 1, 2);
            f.logo(128, 80, "PAUZE", 4, &SUNSET, OUTLINE, LOGO_SHADOW);
            f.text_center(128, 124, "PUSH START", white(), OUTLINE, true);
            f.text_center(128, 140, "ESC: BACK TO THE TITLE", rgb(184, 184, 216), OUTLINE, false);
        }
        if self.mode == Mode::Demo {
            f.rect(0, 212, W, 12, rgb(16, 16, 48));
            if (t / 30).is_multiple_of(2) {
                f.text_center(128, 214, "DEMO PLAY - PUSH START", rgb(248, 232, 64), OUTLINE, true);
            }
        }
    }

    fn draw_controls_tip(&self, f: &mut Frame) {
        let y = 194;
        f.shade_rect(4, y - 2, 248, 30, 1, 3);
        f.rect_outline(4, y - 2, 248, 30, rgb(248, 136, 24));
        f.text_sh(10, y + 1, "ARROWS WALK  HOLD X RUN  Z STEAL/THROW", white(), OUTLINE, false);
        f.text_sh(10, y + 13, "C: ACT CASUAL, OR HIDE BY A TREE", rgb(248, 232, 96), OUTLINE, false);
    }

    // ------------------------------------------------------------ title

    fn draw_title(&self, f: &mut Frame, t: u32) {
        let cam = 120.0 + ((t as f32 * 0.004).sin() * 60.0);
        let sc = &self.title_sc;
        sc.draw_back(f, cam, t, (0, H));
        sc.draw_street(f, cam, (0, H));
        sc.draw_water(f, cam, t, (0, H));
        sc.draw_houseboats(f, cam, t, (0, H), &|f, x, y| self.art.cat.draw(f, x, y, &PROPS, false));
        // a row of parked bikes on the quay
        for k in 0..9 {
            let x = 18 + k * 30;
            let kind = [Kind::Oma, Kind::Race, Kind::Oma, Kind::Bak, Kind::Oma, Kind::Race, Kind::Oma, Kind::Oma, Kind::Wreck][k as usize];
            f.shade_ellipse(x, sy(QUAY_ROW), 11, 2, 5, 8);
            let lock = if k == 3 { Lock::Chain } else { kind.def().lock };
            draw_bike(f, &self.art, kind, (k * 3) as usize % 8, x, sy(QUAY_ROW), k % 2 == 1, 0, lock, false, t);
        }
        // Joost picks a lock; Henk walks by with his coffee, looking the other way
        let jp = thief_palette(Who::Joost);
        let ppl = self.art.style(Who::Joost.def().style);
        let k = ((t / 6) % 2) as usize;
        blit_person(f, &ppl.pick[k], 104, sy(QUAY_ROW - 4.0), false, &jp);
        let bx = 230 - ((t as i32 / 3) % 330);
        let boa = self.art.style(2);
        f.shade_ellipse(bx, sy(30.0), 7, 2, 5, 8);
        blit_person(f, &boa.walk[((t / 9) % 4) as usize], bx, sy(30.0), true, &boa_palette(0));
        if (bx - 104).abs() < 70 {
            bubble(f, bx, sy(30.0) - 33, "?");
        }
        f.shade_rect(0, 0, W, 92, 3, 4);
        // logo drops in
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
        let steel = [rgb(248, 248, 248), rgb(208, 216, 232), rgb(168, 184, 208), rgb(128, 144, 176), rgb(96, 104, 136), rgb(64, 72, 104)];
        // the second word trembles: nerves
        let nerves = if st > 40 && (t / 40).is_multiple_of(3) { ((t / 2) % 3) as i32 - 1 } else { 0 };
        f.logo(120, drop(6, 0), "STALEN", 5, &steel, OUTLINE, rgb(24, 8, 40));
        f.logo(134 + nerves, drop(50, 8), "ZENUWEN!", 4, &SUNSET, OUTLINE, rgb(24, 8, 40));
        if st > 36 {
            f.rect(0, 90, W, 11, rgb(24, 16, 56));
            f.hline(0, W - 1, 89, rgb(248, 136, 24));
            f.hline(0, W - 1, 101, rgb(248, 136, 24));
            f.text_grad_center(128, 92, "AMSTERDAM'S FASTEST HANDS", &grad7(white()), OUTLINE, false);
        }
        if !self.menu_open {
            if (t / 24).is_multiple_of(2) && st > 36 {
                f.text_grad_center(128, 108, "PUSH START", &grad7(white()), OUTLINE, true);
            }
        } else {
            f.shade_rect(72, 104, 112, 26, 1, 3);
            f.rect_outline(72, 104, 112, 26, rgb(248, 136, 24));
            for (k, s) in ["1 PLAYER", "2 PLAYERS"].iter().enumerate() {
                let y = 107 + k as i32 * 11;
                let sel = self.menu == k;
                let col = if sel { rgb(248, 232, 64) } else { rgb(184, 184, 216) };
                f.text_sh(100, y, s, col, OUTLINE, true);
                if sel {
                    self.art.cutter.draw(f, 80, y, &PROPS, false);
                }
            }
        }
        f.rect(0, 200, W, 24, rgb(16, 16, 48));
        f.hline(0, W - 1, 199, rgb(248, 136, 24));
        let tw = text_width(TICKER, false) + 6;
        let off = (t as i32 / 2) % tw;
        for k in 0..3 {
            f.text_sh(k * tw - off, 202, TICKER, rgb(248, 216, 48), OUTLINE, false);
        }
        f.text_center(128, 213, "(C) 1992 HELICO SOFT", rgb(248, 232, 200), rgb(96, 48, 32), false);
    }

    // ------------------------------------------------------------ select

    fn draw_select(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(16, 24, 72), rgb(24, 32, 96));
        f.text_grad_center(128, 4, "KIES JE DIEF!", &grad7(rgb(248, 176, 48)), OUTLINE, true);
        f.text_center(128, 14, "CHOOSE YOUR THIEF", rgb(160, 168, 216), OUTLINE, false);
        for k in 0..4 {
            let (cx, cy) = (6 + (k % 2) as i32 * 124, 24 + (k / 2) as i32 * 88);
            let who = THIEVES[k];
            let d = who.def();
            let hover: Vec<usize> = (0..2).filter(|&p| (p == 0 || self.mode == Mode::Two) && self.cursor[p] == k).collect();
            let accent = d.color;
            f.rect(cx, cy, 120, 84, rgb(8, 8, 32));
            f.rect(cx + 2, cy + 56, 116, 26, rgb(16, 16, 48));
            f.rect(cx + 2, cy + 2, 116, 11, shade(accent, 1, 2));
            f.rect_outline(cx, cy, 120, 84, shade(accent, 2, 3));
            f.text_grad(cx + 5, cy + 4, d.name, &grad7(mix(accent, 0xFFFFFF, 0.4)), OUTLINE, false);
            // bricks, a bike, and the thief
            f.rect(cx + 2, cy + 50, 56, 5, rgb(144, 88, 72));
            f.hline(cx + 2, cx + 57, cy + 50, rgb(104, 64, 56));
            let active = !hover.is_empty();
            let locked = hover.iter().any(|&p| self.locked[p]);
            let ppl = self.art.style(d.style);
            let pal = thief_palette(who);
            let kind = [Kind::Oma, Kind::Oma, Kind::Race, Kind::Bak][k];
            // the bakfiets is long: keep its box off the stat labels
            let bx = if kind == Kind::Bak { cx + 30 } else { cx + 38 };
            if locked {
                draw_rider(f, &self.art, d.style, &pal, kind, Some(bike_pal(kind, k * 2 + 1)), bx - 6, cy + 52, false, ((t / 4) % 8) as usize, ((t / 3) % 4) as usize);
            } else {
                draw_bike(f, &self.art, kind, k * 2 + 1, bx, cy + 52, false, 0, kind.def().lock, false, t);
                let s = if active { &ppl.pick[((t / 6) % 2) as usize] } else { &ppl.casual };
                blit_person(f, s, cx + 22, cy + 52, false, &pal);
            }
            for (r, lab) in ["SNEAK", "HANDS", "LEGS", "BIKE"].iter().enumerate() {
                let y = cy + 16 + r as i32 * 9;
                f.text(cx + 60, y, lab, rgb(200, 200, 232));
                for pip in 0..5 {
                    let c = if pip < d.stats[r] { mix(accent, 0xFFFFFF, 0.3) } else { rgb(40, 40, 72) };
                    f.rect(cx + 94 + pip * 5, y, 5, 6, OUTLINE);
                    f.rect(cx + 95 + pip * 5, y + 1, 3, 4, c);
                }
            }
            for (k, l) in d.bio.iter().enumerate() {
                f.text(cx + 4, cy + 58 + k as i32 * 8, l, rgb(176, 184, 224));
            }
            for &p in &hover {
                let col = if p == 0 { rgb(248, 232, 64) } else { rgb(96, 224, 248) };
                let locked = self.locked[p];
                if locked || (t / 8).is_multiple_of(2) {
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
        let st_i = self.stage_now();
        let st = &STAGES[st_i];
        let cam = (t as f32 * 0.3) % (st.len - 256.0);
        match &self.scenery {
            Some(sc) if sc.venue == st.venue => {
                sc.draw_back(f, cam, t, (0, H));
                sc.draw_street(f, cam, (0, H));
                sc.draw_water(f, cam, t, (0, H));
            }
            _ => stripes(f, t, rgb(16, 24, 72), rgb(24, 32, 96)),
        }
        f.shade_rect(0, 0, W, H, 1, 2);
        let slide = (30 - self.t as i32).max(0) * 9;
        let label = if self.mode == Mode::Two { format!("RONDE {}", self.round + 1) } else { format!("KLUS {}", self.stage + 1) };
        f.text_grad_center(128 - slide, 5, &label, &grad7(rgb(248, 176, 48)), OUTLINE, true);
        let scale = if st.name.len() > 13 { 2 } else if st.name.len() > 9 { 3 } else { 4 };
        f.logo(128 + slide, if scale == 2 { 20 } else { 15 }, st.name, scale, &SUNSET, OUTLINE, LOGO_SHADOW);
        f.text_center(128, 48, st.tagline[0], white(), OUTLINE, false);
        f.text_center(128, 57, st.tagline[1], rgb(200, 208, 240), OUTLINE, false);
        // the order, on Sjaak's note
        let (nx, ny) = (12, 68);
        f.rect(nx - 1, ny - 1, 114, 78, OUTLINE);
        f.rect(nx, ny, 112, 76, rgb(232, 224, 192));
        f.text(nx + 4, ny + 3, "BESTELLING:", rgb(120, 40, 40));
        for (k, &kind) in st.order.iter().enumerate() {
            let y = ny + 13 + k as i32 * 11;
            bike_icon(f, kind, nx + 4, y, false);
            f.text(nx + 24, y + 1, kind.def().name, rgb(40, 40, 64));
        }
        f.text(nx + 62, ny + 67, "- SJAAK", rgb(120, 40, 40));
        // who you face
        let bx = 190;
        let by = 124;
        let boss_kind = st.boas.iter().find(|b| st.boss.ends_with(b.name)).map(|b| b.kind).unwrap_or(BoaKind::Foot);
        f.shade_ellipse(bx, by, 9, 2, 5, 8);
        if boss_kind == BoaKind::Bike {
            draw_rider(f, &self.art, 2, &boa_palette(st_i + 2), Kind::Oma, Some(boa_bike_pal()), bx, by, true, ((t / 5) % 8) as usize, ((t / 4) % 4) as usize);
        } else {
            let ppl = self.art.style(2);
            let s = if (t / 40).is_multiple_of(3) { &ppl.whistle } else { &ppl.stand };
            blit_person(f, s, bx, by, true, &boa_palette(st_i));
            if boss_kind == BoaKind::Chief {
                self.art.dog[((t / 10) % 2) as usize].draw_foot(f, bx - 20, by + 1, &PROPS, false);
            }
        }
        f.text_center(bx, by + 4, st.boss, rgb(120, 168, 248), OUTLINE, false);
        if self.t > 40 {
            let h = st.heckles[((t / 150) as usize) % st.heckles.len()];
            bubble(f, bx.min(200), by - 34, h);
        }
        // tip box
        let (tx, ty) = (8, 152);
        f.rect(tx, ty, 240, 46, rgb(16, 16, 48));
        f.rect_outline(tx, ty, 240, 46, rgb(248, 136, 24));
        if self.mode == Mode::Two {
            let tp = self.thief_player();
            let (a, b) = if tp == 0 { ("1P", "2P") } else { ("2P", "1P") };
            f.text_sh(tx + 6, ty + 5, &format!("{} = DIEF: STEAL SJAAK'S ORDER", a), rgb(248, 232, 64), OUTLINE, false);
            f.text_sh(tx + 6, ty + 15, &format!("{} = BOA: Z GRABS. ONLY IN THE ACT!", b), rgb(96, 224, 248), OUTLINE, false);
            f.text_sh(tx + 6, ty + 25, "BOA: HOLD X RUN, C WHISTLE FOR HELP", rgb(200, 200, 232), OUTLINE, false);
            f.text_sh(tx + 6, ty + 35, "GRAB AN INNOCENT? KLACHT! -50", rgb(232, 120, 96), OUTLINE, false);
        } else {
            f.rect(tx + 6, ty - 4, 26, 9, rgb(16, 16, 48));
            f.text_sh(tx + 8, ty - 3, "TIP", rgb(248, 136, 24), OUTLINE, false);
            f.text_sh(tx + 6, ty + 6, st.tip[0], white(), OUTLINE, false);
            f.text_sh(tx + 6, ty + 15, st.tip[1], white(), OUTLINE, false);
            f.text_sh(tx + 6, ty + 26, "Z AT A BIKE: PICK THE LOCK. X: CUTTER", rgb(248, 232, 96), OUTLINE, false);
            f.text_sh(tx + 6, ty + 35, "Z BY SJAAK'S BOAT: THROW IT ON!", rgb(248, 232, 96), OUTLINE, false);
        }
        if (t / 20).is_multiple_of(2) && self.t > 40 {
            f.text_center(128, 204, "PUSH START", white(), OUTLINE, true);
        }
    }

    // ------------------------------------------------------------ results

    fn draw_results(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(40, 16, 8), rgb(56, 24, 8));
        let st = &STAGES[self.stage_now()];
        let head = if self.mode == Mode::Two {
            "AFREKENING"
        } else if self.cleared {
            "GELUKT!"
        } else {
            "MISLUKT..."
        };
        f.logo(128, 4, head, 3, &if self.cleared || self.mode == Mode::Two { SUNSET.to_vec() } else { grad6(rgb(216, 48, 64)).to_vec() }, OUTLINE, LOGO_SHADOW);
        let why = match (self.mode, self.outcome) {
            (Mode::Two, _) => st.name.to_string(),
            (_, Some(Outcome::Cleared)) => format!("{}: ALLES BEZORGD", st.name),
            (_, Some(Outcome::Busted)) => "OPGEPAKT! DRIE BOETES.".to_string(),
            _ => "TE LAAT! SJAAK IS AL WEG.".to_string(),
        };
        f.text_center(128, 34, &why, rgb(200, 192, 232), OUTLINE, false);
        let n = self.tally[0].len().max(self.tally[1].len()) as u32;
        let shown = if self.t < 20 { 0 } else { ((self.t - 20) / 30 + 1).min(n) } as usize;
        let done = self.t >= 20 + n * 30 + 30;
        let players: Vec<usize> = if self.mode == Mode::Two { vec![0, 1] } else { vec![0] };
        for &p in &players {
            let (x0, w) = if players.len() == 2 { (8 + p as i32 * 122, 118) } else { (36, 184) };
            let y0 = 48;
            let col = if p == 0 { rgb(248, 232, 64) } else { rgb(96, 224, 248) };
            f.rect(x0, y0, w, 140, rgb(16, 16, 40));
            f.rect_outline(x0, y0, w, 140, col);
            if self.mode == Mode::Two {
                let role = if p == self.thief_player() { "DIEF" } else { "BOA" };
                f.text_sh(x0 + 5, y0 + 4, &format!("{}P - {}", p + 1, role), col, OUTLINE, false);
            }
            let lines = &self.tally[p];
            for (k, l) in lines.iter().enumerate().take(shown) {
                let y = y0 + 16 + k as i32 * 11;
                f.text_sh(x0 + 5, y, &l.label, rgb(216, 216, 232), OUTLINE, false);
                let s = format!("{}", l.pts);
                let c = if l.pts < 0 { rgb(232, 96, 80) } else { white() };
                f.text_sh(x0 + w - 5 - text_width(&s, false), y, &s, c, OUTLINE, false);
            }
            let sum: i32 = lines.iter().take(shown).map(|l| l.pts).sum();
            f.hline(x0 + 4, x0 + w - 5, y0 + 112, col);
            let s = format!("EUR {}", sum);
            f.text_sh(x0 + 5, y0 + 117, "TOTAAL", col, OUTLINE, false);
            f.text_grad(x0 + w - 5 - text_width(&s, true), y0 + 116, &s, &grad7(col), OUTLINE, true);
            if done {
                let s = format!("SPAARPOT EUR {}", self.score[p]);
                f.text_sh(x0 + 5, y0 + 129, &s, rgb(184, 184, 216), OUTLINE, false);
            }
        }
        if done && (t / 20).is_multiple_of(2) {
            let msg = match self.mode {
                Mode::One if self.cleared => "SJAAK IS BLIJ! PUSH START",
                Mode::One => "SJAAK IS NIET BLIJ... PUSH START",
                _ => "PUSH START",
            };
            f.text_center(128, 204, msg, white(), OUTLINE, true);
        }
    }

    // ------------------------------------------------------------ endings

    fn draw_continue(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(16, 32, 48), rgb(24, 48, 64));
        f.logo(128, 22, "CONTINUE?", 3, &SUNSET, OUTLINE, rgb(0, 0, 0));
        let n = format!("{}", self.count.max(0));
        f.logo(128, 54, &n, 6, &grad6(rgb(248, 232, 64)), OUTLINE, rgb(0, 0, 0));
        // you on the curb; a BOA writes the fine
        let gy = 176;
        f.rect(0, gy, W, H - gy, rgb(144, 88, 72));
        f.hline(0, W - 1, gy, rgb(104, 64, 56));
        let who = THIEVES[self.cursor[0]];
        let ppl = self.art.style(who.def().style);
        f.shade_ellipse(112, gy + 2, 9, 2, 5, 8);
        blit_person(f, &ppl.sit, 112, gy + 2, false, &thief_palette(who));
        let boa = self.art.style(2);
        f.shade_ellipse(146, gy + 2, 7, 2, 5, 8);
        blit_person(f, &boa.write, 146, gy + 2, true, &boa_palette(1));
        self.art.ticket.draw(f, 136, gy - 18, &PROPS, true);
        if (t / 40).is_multiple_of(2) {
            bubble(f, 150, gy - 34, "NAAM? ADRES?");
        } else {
            bubble(f, 108, gy - 34, "HET WAS MIJN FIETS!");
        }
        f.text_center(128, 118, "THE ORDER IS STILL WAITING", rgb(248, 216, 96), OUTLINE, false);
        if (t / 20).is_multiple_of(2) {
            f.text_center(128, 196, "PUSH Z TO TRY AGAIN", white(), OUTLINE, false);
        }
        f.text_center(128, 210, "SJAAK DOESN'T WAIT FOREVER", rgb(160, 200, 216), OUTLINE, false);
    }

    fn draw_game_over(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(16, 8, 32), rgb(24, 16, 48));
        f.logo(128, 50, "GAME", 4, &grad6(rgb(216, 48, 64)), OUTLINE, rgb(0, 0, 0));
        f.logo(128, 88, "OVER", 4, &grad6(rgb(216, 48, 64)), OUTLINE, rgb(0, 0, 0));
        f.text_center(128, 140, "YOUR BIKE IS STILL GONE.", rgb(200, 200, 232), OUTLINE, false);
        // a BOA pushes a confiscated bike past
        let x = (t as i32) % 360 - 60;
        let gy = 192;
        f.shade_ellipse(x, gy, 7, 2, 5, 8);
        draw_bike(f, &self.art, Kind::Oma, 2, x + 16, gy, false, ((t / 6) % 4) as usize, Lock::None, false, t);
        let boa = self.art.style(2);
        blit_person(f, &boa.walk[((t / 9) % 4) as usize], x, gy, false, &boa_palette(2));
        if (t / 50).is_multiple_of(2) {
            bubble(f, x + 4, gy - 34, "IN BESLAG!");
        }
    }

    fn draw_champion(&self, f: &mut Frame, t: u32) {
        let cam = 200.0 + (t as f32 * 0.2).min(120.0);
        let sc = &self.title_sc;
        sc.draw_back(f, cam, t, (0, H));
        sc.draw_street(f, cam, (0, H));
        sc.draw_water(f, cam, t, (0, H));
        f.shade_rect(0, 0, W, 72, 2, 3);
        // Sjaak's boat, piled high
        let bx = 150;
        let wl = WATER + 8;
        // the loot, stacked: a back row on top of a front row
        let pile = [(-12, -9, Kind::Race, 2), (12, -9, Kind::Oma, 6), (-22, -2, Kind::Oma, 1), (0, -2, Kind::Gold, 5), (22, -2, Kind::Oma, 3)];
        for (k, &(dx, dy, kind, col)) in pile.iter().enumerate() {
            draw_bike(f, &self.art, kind, col, bx + dx, wl + dy, k % 2 == 1, k % 4, Lock::None, false, t);
        }
        f.fill_poly(&[(bx - 33, wl - 5), (bx + 33, wl - 5), (bx + 28, wl + 5), (bx - 28, wl + 5)], OUTLINE);
        f.fill_poly(&[(bx - 32, wl - 4), (bx + 32, wl - 4), (bx + 27, wl + 4), (bx - 27, wl + 4)], rgb(160, 168, 176));
        let sjp = sjaak_palette();
        let s2 = self.art.style(2);
        blit_person_clip(f, if (t / 10).is_multiple_of(2) { &s2.victory } else { &s2.cheer }, bx + 26, wl + 2, true, &sjp, wl - 4);
        // the winner on the quay
        let winner = if self.mode == Mode::Two && self.score[1] > self.score[0] { 1 } else { 0 };
        let who = THIEVES[self.cursor[winner]];
        let ppl = self.art.style(who.def().style);
        let hop = (((t as i32) % 30) - 15).abs() / 4;
        let s = if hop > 1 { &ppl.victory } else { &ppl.cheer };
        f.shade_ellipse(70, sy(QUAY_ROW), 8, 2, 5, 8);
        blit_person(f, s, 70, sy(QUAY_ROW) - hop, false, &thief_palette(who));
        if self.mode == Mode::Two {
            let other = THIEVES[self.cursor[1 - winner]];
            let op = self.art.style(other.def().style);
            blit_person(f, &op.sit, 40, sy(QUAY_ROW), false, &thief_palette(other));
        }
        for &(x, y, _, c) in &self.confetti {
            f.rect(x as i32, y as i32, 2, 2, c);
        }
        f.logo(128, 6, "MEESTERDIEF!", 3, &SUNSET, OUTLINE, LOGO_SHADOW);
        let (line, sub) = if self.mode == Mode::Two {
            (if winner == 0 { "SPELER 1 WINT!" } else { "SPELER 2 WINT!" }, format!("EUR {} - EUR {}", self.score[0], self.score[1]))
        } else {
            ("SJAAK IS PROUD OF YOU", format!("SPAARPOT EUR {}", self.score[0]))
        };
        f.text_grad_center(128, 36, line, &grad7(rgb(96, 224, 248)), OUTLINE, true);
        f.text_center(128, 48, &sub, white(), OUTLINE, false);
        if (t / 60) % 2 == 1 {
            bubble(f, bx + 22, wl - 40, "TOP SPUL!");
        }
        f.text_center(128, 212, "AND NOW BUY A GOOD LOCK.", rgb(248, 136, 24), OUTLINE, false);
        let _ = sp::HAIR_STYLES;
    }
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
