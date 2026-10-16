//! Scenes and flow: title (with attract demo), pilot select, stage intro
//! card, the flight (1P or split screen 2P), results tally, continue, game
//! over and the champion screen.

use super::ai::Style;
use super::art::*;
use super::audio::Track;
use super::course::*;
use super::flight::*;
use super::pilots::{palette, PILOTS};
use super::render::*;
use super::scenery::*;
use crate::engine::gfx::*;
use crate::engine::input::{any, edge, Pad};
use crate::engine::rng::Rng;
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

/// CPU skill of the three rivals on each stage of the 1P ladder.
pub const LADDER: [[f32; 3]; 3] = [[-0.2, -0.1, 0.05], [0.25, 0.38, 0.5], [0.38, 0.5, 0.62]];
/// Skill of the CPU pilots that fly along in 2P and in the demo.
pub const FILLER_SKILL: f32 = 0.5;
const IDLE_DEMO: u32 = 900;
pub const HUD_H: i32 = 33;
/// Split screen: each view is this tall, the minimap sits between them.
pub const SPLIT_H: i32 = 106;
pub const SPLIT_GAP: i32 = 12;

pub struct App {
    pub scene: Scene,
    pub t: u32,
    pub frame: u32,
    pub mode: Mode,
    pub menu: usize,
    pub menu_open: bool,
    pub cursor: [usize; 2],
    pub locked: [bool; 2],
    pub stage: usize,
    pub race: Option<Race>,
    pub scenery: Option<Scenery>,
    pub art: Art,
    title_course: Course,
    title_sc: Scenery,
    pub cams: [Cam; 2],
    pub sfx: Vec<Sfx>,
    prev: [Pad; 2],
    rng: Rng,
    pub quit: bool,
    pub paused: bool,
    idle: u32,
    pub count: i32,
    confetti: Vec<(f32, f32, f32, u32)>,
    /// 2P: running totals per player
    pub totals: [i32; 2],
    /// results tally progress (frames)
    pub tally: i32,
    /// after you land in 1P: hold A to fast-forward the others
    pub fast: bool,
    scratch: RefCell<Frame>,
    /// the pilot the camera follows in each view
    pub follow: [usize; 2],
    pub done_t: u32,
}

impl App {
    pub fn new(seed: u64) -> Self {
        let title_course = Course::new(Venue::Dunes, 11);
        let title_sc = Scenery::new(&title_course, 5);
        let cam = Cam { x: 400.0, y: 200.0, cy: 128, vh: H };
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
            art: Art::new(),
            title_course,
            title_sc,
            cams: [cam; 2],
            sfx: Vec::new(),
            prev: [Pad::default(); 2],
            rng: Rng(seed | 1),
            quit: false,
            paused: false,
            idle: 0,
            count: 0,
            confetti: Vec::new(),
            totals: [0; 2],
            tally: 0,
            fast: false,
            scratch: RefCell::new(Frame::new()),
            follow: [0, 1],
            done_t: 0,
        }
    }

    pub fn music(&self) -> Option<Track> {
        match self.scene {
            Scene::Title | Scene::Select | Scene::Intro => Some(Track::Title),
            Scene::Play => Some(Track::Flight),
            Scene::Results => Some(Track::Title),
            Scene::Champion => Some(Track::Win),
            Scene::Continue | Scene::GameOver => None,
        }
    }

    fn go(&mut self, s: Scene) {
        self.scene = s;
        self.t = 0;
    }

    pub fn venue(&self) -> Venue {
        VENUES[self.stage.min(2)]
    }

    /// The pilots in this stage's race.
    fn entries(&self) -> Vec<Entry> {
        match self.mode {
            Mode::One => {
                let me = self.cursor[0];
                let mut e: Vec<Entry> = vec![(me, Ctrl::Human(0), 0.0, Style::Normal)];
                for k in 0..3 {
                    e.push(((me + 1 + k) % 4, Ctrl::Cpu, LADDER[self.stage][k], Style::Normal));
                }
                e
            }
            Mode::Two => {
                let (a, b) = (self.cursor[0], self.cursor[1]);
                let mut e: Vec<Entry> = vec![(a, Ctrl::Human(0), 0.0, Style::Normal), (b, Ctrl::Human(1), 0.0, Style::Normal)];
                for d in 0..4 {
                    if d != a && d != b {
                        e.push((d, Ctrl::Cpu, FILLER_SKILL, Style::Normal));
                    }
                }
                e
            }
            Mode::Demo => (0..4).map(|d| (d, Ctrl::Cpu, 0.45 + 0.1 * d as f32, Style::Normal)).collect(),
        }
    }

    /// Build the course for the current stage (the intro card shows it).
    pub fn prepare_stage(&mut self) {
        let seed = self.rng.next() as u64;
        let race = Race::new(self.venue(), &self.entries(), seed);
        let sc = Scenery::new(&race.course, seed ^ 0x55);
        self.race = Some(race);
        self.scenery = Some(sc);
        self.paused = false;
        self.fast = false;
        self.follow = [0, 1];
        self.done_t = 0;
        self.snap_cams();
    }

    pub fn snap_cams(&mut self) {
        let Some(r) = &self.race else { return };
        for v in 0..2 {
            let (top, vh, cy) = self.view_layout(v);
            let i = self.follow[v].min(r.pilots.len() - 1);
            let (x, y) = cam_target(&r.pilots[i], &r.course, vh, top);
            self.cams[v] = Cam { x, y, cy, vh };
        }
    }

    /// (first row under the HUD, view height, centre row) of a view, as
    /// drawn into its own frame.
    fn view_layout(&self, _v: usize) -> (i32, i32, i32) {
        if self.mode == Mode::Two {
            (12, SPLIT_H, 12 + (SPLIT_H - 12) / 2)
        } else {
            (HUD_H, H, HUD_H + (H - HUD_H) / 2)
        }
    }

    pub fn start_demo(&mut self) {
        self.mode = Mode::Demo;
        self.stage = self.rng.int(3) as usize;
        self.prepare_stage();
        self.go(Scene::Play);
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
            Scene::Select => self.update_select(e),
            Scene::Intro => {
                let wait = if self.stage == 0 { 60 } else { 30 };
                if self.t > 420 || (self.t > wait && (ok(e[0]) || ok(e[1]))) {
                    self.sfx.push(Sfx::Confirm);
                    self.go(Scene::Play);
                }
            }
            Scene::Play => self.update_play(pads, e),
            Scene::Results => {
                self.tally += 1;
                if self.tally % 6 == 0 && self.tally < 6 * 30 {
                    self.sfx.push(Sfx::Blip);
                }
                if self.t > 60 && (ok(e[0]) || ok(e[1])) {
                    if self.tally < 6 * 30 {
                        self.tally = 6 * 30;
                    } else {
                        self.sfx.push(Sfx::Confirm);
                        self.after_results();
                    }
                }
            }
            Scene::Continue => {
                if self.t % 60 == 0 {
                    self.count -= 1;
                    self.sfx.push(Sfx::Blip);
                }
                if ok(e[0]) && self.t > 20 {
                    self.sfx.push(Sfx::Confirm);
                    self.prepare_stage();
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

    fn update_select(&mut self, e: [Pad; 2]) {
        let ok = |p: Pad| p.a || p.start;
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
                        self.sfx.push(Sfx::Baa);
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
            self.stage = 0;
            self.totals = [0; 2];
            self.prepare_stage();
            self.go(Scene::Intro);
        }
    }

    fn update_play(&mut self, pads: [Pad; 2], e: [Pad; 2]) {
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
        let mode = self.mode;
        let Some(race) = self.race.as_mut() else {
            self.go(Scene::Title);
            return;
        };
        let humans_done = race.pilots.iter().filter(|p| matches!(p.ctrl, Ctrl::Human(_))).all(|p| p.state == State::Done);
        // once you are down, A fast-forwards the rest of the field
        self.fast = mode == Mode::One && humans_done && (pads[0].a || pads[0].b);
        let steps = if self.fast { 8 } else { 1 };
        for _ in 0..steps {
            race.update(&pads);
            if race.finished() {
                break;
            }
        }
        self.sfx.append(&mut race.sfx);
        // the camera: your pilot, or the best rival still flying once you are down
        match mode {
            Mode::One | Mode::Demo => {
                let me = 0;
                let mut f = me;
                if race.pilots[me].state == State::Done {
                    if self.done_t == 0 {
                        self.done_t = race.t;
                    }
                    if race.t > self.done_t + 150 {
                        if let Some(k) = (0..race.pilots.len()).filter(|&k| race.pilots[k].airborne()).max_by(|&a, &b| race.pilots[a].x.partial_cmp(&race.pilots[b].x).unwrap()) {
                            f = k;
                        }
                    }
                }
                self.follow[0] = f;
            }
            Mode::Two => self.follow = [0, 1],
        }
        let views = if mode == Mode::Two { 2 } else { 1 };
        for v in 0..views {
            let (top, vh, cy) = if mode == Mode::Two { (12, SPLIT_H, 12 + (SPLIT_H - 12) / 2) } else { (HUD_H, H, HUD_H + (H - HUD_H) / 2) };
            let p = &race.pilots[self.follow[v]];
            let (tx, ty) = cam_target(p, &race.course, vh, top);
            let c = &mut self.cams[v];
            let k = if self.fast { 0.4 } else { 0.12 };
            c.x += (tx - c.x) * k;
            c.y += (ty - c.y) * k * 0.8;
            c.cy = cy;
            c.vh = vh;
        }
        let finished = race.finished();
        if mode == Mode::Demo {
            if self.t > 60 * 50 || finished {
                self.race = None;
                self.go(Scene::Title);
            }
            return;
        }
        if finished {
            if self.done_t == 0 || race.t > self.done_t {
                self.done_t = race.t;
            }
            if self.t > 30 && race.t >= self.done_t {
                self.fast = false;
                self.tally = 0;
                if mode == Mode::Two {
                    for k in 0..2 {
                        self.totals[k] += race.pilots[k].total();
                    }
                }
                self.go(Scene::Results);
            }
        }
    }

    /// Did player 1 win this stage?
    pub fn won_stage(&self) -> bool {
        self.race.as_ref().map(|r| r.ranking()[0] == 0).unwrap_or(false)
    }

    fn after_results(&mut self) {
        match self.mode {
            Mode::One => {
                if self.won_stage() {
                    self.stage += 1;
                    if self.stage >= 3 {
                        self.make_confetti();
                        self.go(Scene::Champion);
                    } else {
                        self.prepare_stage();
                        self.go(Scene::Intro);
                    }
                } else {
                    self.count = 9;
                    self.go(Scene::Continue);
                }
            }
            Mode::Two => {
                self.stage += 1;
                if self.stage >= 3 {
                    self.make_confetti();
                    self.go(Scene::Champion);
                } else {
                    self.prepare_stage();
                    self.go(Scene::Intro);
                }
            }
            Mode::Demo => self.go(Scene::Title),
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
            Scene::Play => self.draw_play(f, t),
            Scene::Results => self.draw_results(f, t),
            Scene::Continue => self.draw_continue(f, t),
            Scene::GameOver => self.draw_game_over(f, t),
            Scene::Champion => self.draw_champion(f, t),
        }
    }

    fn draw_play(&self, f: &mut Frame, t: u32) {
        let (Some(race), Some(sc)) = (&self.race, &self.scenery) else {
            f.clear(rgb(16, 16, 48));
            return;
        };
        if self.mode == Mode::Two {
            let mut s = self.scratch.borrow_mut();
            for v in 0..2 {
                let cam = &self.cams[v];
                draw_view(&mut s, race, sc, &self.art, cam, Some(v), t);
                gate_pointer(&mut s, race, v, cam, 12, t);
                if let Some(b) = &race.banners[v] {
                    draw_banner(&mut s, b, 34);
                }
                draw_hud_small(&mut s, race, v, 0, t);
                let dst = if v == 0 { 0 } else { SPLIT_H + SPLIT_GAP };
                f.copy_band(&s, 0, dst, SPLIT_H.min(H - dst));
            }
            // the divider holds the minimap of the whole course
            let y = SPLIT_H;
            f.rect(0, y, W, SPLIT_GAP, rgb(16, 16, 48));
            f.hline(0, W - 1, y, rgb(176, 32, 88));
            minimap(f, race, None, 40, y + 2, 176, t);
            for (v, col) in [(0, rgb(248, 232, 64)), (1, rgb(96, 224, 248))] {
                let p = &race.pilots[v];
                let x = 40 + (p.x / LEN as f32 * 175.0) as i32;
                let yy = y + 2 + 7 - ((p.y / 640.0).clamp(0.0, 1.0) * 7.0) as i32;
                f.rect(x - 1, yy - 1, 3, 3, col);
            }
            f.text_sh(4, y + 3, "1P", rgb(248, 232, 64), OUTLINE, false);
            f.text_sh(240, y + 3, "2P", rgb(96, 224, 248), OUTLINE, false);
        } else {
            let cam = &self.cams[0];
            let viewer = if self.mode == Mode::Demo { None } else { Some(0) };
            draw_view(f, race, sc, &self.art, cam, viewer.or(Some(self.follow[0])), t);
            if self.mode == Mode::One {
                gate_pointer(f, race, 0, cam, HUD_H, t);
            }
            draw_hud(f, race, if self.mode == Mode::Demo { self.follow[0] } else { 0 }, t);
            if let Some(b) = &race.banners[0] {
                draw_banner(f, b, HUD_H + 26);
            }
            if self.mode == Mode::One {
                let me = &race.pilots[0];
                if self.follow[0] != 0 {
                    let name = PILOTS[race.pilots[self.follow[0]].def].name;
                    f.rect(0, HUD_H + 1, W, 10, rgb(16, 16, 48));
                    f.text_center(128, HUD_H + 2, &format!("WATCHING {name}"), rgb(200, 200, 232), OUTLINE, false);
                }
                if me.state == State::Done && !race.finished() && (t / 30) % 2 == 0 {
                    f.rect(0, 212, W, 12, rgb(16, 16, 48));
                    f.text_center(128, 214, "HOLD Z TO FAST FORWARD", rgb(248, 232, 64), OUTLINE, true);
                }
                self.draw_prompts(f, race, t);
            }
        }
        if self.paused {
            f.shade_rect(0, 0, W, H, 1, 2);
            f.logo(128, 90, "PAUSE", 3, &SUNSET, OUTLINE, rgb(8, 8, 24));
            f.text_center(128, 130, "PUSH START", rgb(248, 248, 248), OUTLINE, true);
        }
        if self.mode == Mode::Demo {
            f.rect(0, 212, W, 12, rgb(16, 16, 48));
            if (t / 30) % 2 == 0 {
                f.text_center(128, 214, "DEMO PLAY - PUSH START", rgb(248, 232, 64), OUTLINE, true);
            }
        }
    }

    /// On-screen help the first time things matter (1P).
    fn draw_prompts(&self, f: &mut Frame, race: &Race, t: u32) {
        let p = &race.pilots[0];
        let c = &race.course;
        let blink = (t / 16) % 2 == 0;
        let (z0, _) = c.launch.zone();
        let msg = match p.state {
            State::Wait => Some("GET READY..."),
            State::Run | State::Hop(_) if p.x >= z0 => Some("NOW! PRESS X!"),
            State::Run | State::Hop(_) => Some("TAP LEFT, RIGHT, LEFT, RIGHT!"),
            State::Fly if p.agl(c) < FLARE_MAX_AGL && p.agl(c) > 2.0 && p.vy < 0.0 && race.pilots[0].clock > 120 => Some("PRESS X TO FLARE!"),
            _ => None,
        };
        if let Some(m) = msg {
            if blink || matches!(p.state, State::Wait) {
                f.rect(0, 200, W, 12, rgb(16, 16, 48));
                f.text_center(128, 202, m, rgb(248, 248, 248), OUTLINE, true);
            }
        }
    }

    fn draw_title(&self, f: &mut Frame, t: u32) {
        let cam = Cam { x: 214.0, y: 42.0, cy: 150, vh: H };
        let sc = &self.title_sc;
        let c = &self.title_course;
        sc.draw_sky(f, (cam.x, 260.0), cam.cy, cam.vh, t);
        // the layers sit low behind the beach: placed by hand for the title
        sc.far.draw(f, (cam.x + 2000.0, 150.0), cam.cy, H);
        draw_pic_clip(f, &sc.cumulus[2], 196, 126, H, false);
        draw_pic_clip(f, &sc.cow_cloud, 60, 60, H, false);
        sc.mid.draw(f, (cam.x + 600.0, 182.0), cam.cy, H);
        sc.draw_ground(f, c, cam.pos(), cam.cy, cam.vh, t);
        draw_props(f, c, false, sc, &self.art, &cam, t);
        for k in 0..3 {
            let a = t as f32 * 0.012 + k as f32 * 2.1;
            let x = 196.0 + a.sin() * 34.0;
            let y = 136.0 + k as f32 * 9.0;
            let fr = if ((t / 7) as usize + k * 5) % 30 < 4 { 1 } else { 0 };
            draw_pic(f, &self.art.critters.stork[fr], x as i32, y as i32, a.cos() < 0.0);
        }
        // gliders soaring past
        let paths = [(0usize, 0.0f32, 128.0f32, 150.0f32, 110.0f32), (2, 2.4, 100.0, 162.0, 80.0), (3, 4.0, 150.0, 140.0, 120.0)];
        for &(def, ph, cx, cy, rx) in &paths {
            let a = t as f32 * 0.006 + ph;
            let x = cx + a.cos() * rx;
            let y = cy + (a * 2.0).sin() * 10.0;
            let flip = a.sin() > 0.0;
            let mut l = Look::flying(-(a * 2.0).cos() * 10.0, flip);
            let edge = a.sin().abs();
            if edge < 0.16 {
                // turning at the end of each pass
                let k = ((0.16 - edge) / 0.16 * 3.0) as usize;
                l.turn = if a.cos() > 0.0 { k.min(3) } else { (6 - k).max(3) };
                l.flip = if a.cos() > 0.0 { a.sin() > 0.0 } else { a.sin() <= 0.0 };
            }
            draw_pilot(f, &self.art, def, x as i32, y as i32, &l);
        }
        f.shade_rect(0, 0, W, 92, 3, 4);
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
        f.logo(112, drop(6, 0), "HANG", 5, &SUNSET, OUTLINE, rgb(24, 8, 40));
        f.logo(146, drop(48, 8), "TIME!", 5, &grad6(rgb(64, 200, 248)), OUTLINE, rgb(8, 16, 48));
        if st > 36 {
            f.rect(0, 93, W, 11, rgb(24, 16, 56));
            f.hline(0, W - 1, 92, rgb(248, 176, 32));
            f.hline(0, W - 1, 104, rgb(248, 176, 32));
            f.text_grad_center(128, 95, "THE GREAT HELICO SKY RALLY", &grad7(rgb(248, 216, 96)), OUTLINE, false);
        }
        if !self.menu_open {
            if (t / 24) % 2 == 0 && st > 36 {
                f.text_grad_center(128, 112, "PUSH START", &grad7(rgb(248, 248, 248)), OUTLINE, true);
            }
        } else {
            f.shade_rect(72, 108, 112, 24, 1, 2);
            f.rect_outline(72, 108, 112, 24, rgb(248, 176, 32));
            for (k, s) in ["1 PLAYER", "2 PLAYERS"].iter().enumerate() {
                let y = 110 + k as i32 * 11;
                let sel = self.menu == k;
                let col = if sel { rgb(248, 232, 64) } else { rgb(184, 184, 216) };
                f.text_sh(100, y, s, col, OUTLINE, true);
                if sel {
                    mini_glider(f, &self.art, 88, y + 4, t);
                }
            }
        }
        f.rect(0, 200, W, 24, rgb(16, 16, 48));
        f.hline(0, W - 1, 199, rgb(248, 176, 32));
        let msg = "FIND THE LIFT, DODGE THE SHEEP, LAND ON THE CROSS.      STORKS KNOW WHERE THE THERMALS ARE.      NEVER TRUST A CLOUD THAT LOOKS HUNGRY.      GEESE: NATURE'S ESCALATOR.      ";
        let w = text_width(msg, false) + 6;
        let off = (t as i32 / 2) % w;
        for k in 0..3 {
            f.text_sh(k * w - off, 202, msg, rgb(248, 216, 48), OUTLINE, false);
        }
        f.text_center(128, 213, "(C) 1992 HELICO SOFT", rgb(248, 232, 200), rgb(96, 48, 32), false);
    }

    fn draw_select(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(16, 32, 88), rgb(24, 48, 112));
        f.text_grad_center(128, 6, "CHOOSE YOUR PILOT", &grad7(rgb(248, 200, 48)), OUTLINE, true);
        for k in 0..4 {
            let (cx, cy) = (6 + (k % 2) as i32 * 124, 20 + (k / 2) as i32 * 92);
            let d = &PILOTS[k];
            let acc = d.ui;
            let hover: Vec<usize> = (0..2).filter(|&p| (p == 0 || self.mode == Mode::Two) && self.cursor[p] == k).collect();
            f.rect(cx, cy, 120, 88, rgb(8, 16, 48));
            f.rect(cx + 2, cy + 2, 116, 12, shade(d.suit.1, 2, 3));
            f.rect_outline(cx, cy, 120, 88, shade(acc, 2, 3));
            f.text_grad(cx + 5, cy + 4, d.name, &grad7(acc), OUTLINE, false);
            // a little sky with the pilot flying in it
            f.rect(cx + 3, cy + 16, 58, 46, rgb(88, 152, 232));
            f.rect(cx + 3, cy + 46, 58, 16, rgb(120, 184, 240));
            f.dither_rect(cx + 3, cy + 44, 58, 2, rgb(88, 152, 232), rgb(120, 184, 240));
            let active = !hover.is_empty();
            let bob = if active { ((t as f32 * 0.12).sin() * 3.0) as i32 } else { 0 };
            let mut l = Look::flying(if active { ((t as f32 * 0.12).cos() * 12.0).round() } else { 0.0 }, false);
            if active && (t / 90) % 3 == 2 {
                let k2 = ((t % 90) as f32 / 90.0 * 6.0) as usize;
                l.turn = k2.min(6);
            }
            draw_pilot(f, &self.art, k, cx + 30, cy + 54 + bob, &l);
            for (r, (lab, v)) in [("GLD", d.glide), ("CLB", d.climb), ("HND", d.handling), ("NRV", d.nerve)].iter().enumerate() {
                let y = cy + 18 + r as i32 * 11;
                f.text(cx + 64, y, lab, rgb(200, 200, 232));
                for pip in 0..5 {
                    let c = if pip < *v { acc } else { rgb(40, 40, 72) };
                    f.rect(cx + 83 + pip * 7, y, 6, 7, OUTLINE);
                    f.rect(cx + 84 + pip * 7, y + 1, 4, 5, c);
                }
            }
            f.text(cx + 5, cy + 66, d.bio[0], rgb(176, 184, 224));
            f.text(cx + 5, cy + 76, d.bio[1], rgb(176, 184, 224));
            for &p in &hover {
                let col = if p == 0 { rgb(248, 232, 64) } else { rgb(96, 224, 248) };
                let locked = self.locked[p];
                if locked || (t / 8) % 2 == 0 {
                    f.rect_outline(cx - 1, cy - 1, 122, 90, col);
                    f.rect_outline(cx, cy, 120, 88, col);
                }
                let tag = if p == 0 { "1P" } else { "2P" };
                let tx = if p == 0 { cx + 4 } else { cx + 19 };
                f.rect(tx - 1, cy + 18, 14, 9, col);
                f.text(tx, cy + 19, tag, rgb(16, 16, 48));
                if locked {
                    f.text_grad_center(cx + 32, cy + 22, "OK!", &grad7(col), OUTLINE, true);
                }
            }
        }
        if self.mode == Mode::Two {
            f.text_center(128, 208, "1P: ARROWS + , . /  (OR Z X C)", rgb(248, 232, 64), OUTLINE, false);
            f.text_center(128, 216, "2P: W A S D + F G H", rgb(96, 224, 248), OUTLINE, false);
        } else {
            f.text_center(128, 212, "ARROWS: PICK   Z: OK   X: BACK", rgb(200, 200, 232), OUTLINE, false);
        }
    }

    fn draw_intro(&self, f: &mut Frame, t: u32) {
        let (Some(race), Some(sc)) = (&self.race, &self.scenery) else { return };
        let c = &race.course;
        let cam = Cam { x: c.launch.x0 + 60.0, y: c.launch.y0 + 40.0, cy: 112, vh: H };
        sc.draw_sky(f, cam.pos(), cam.cy, H, t);
        sc.draw_back(f, cam.pos(), cam.cy, H, t);
        sc.draw_ground(f, c, cam.pos(), cam.cy, H, t);
        f.shade_rect(0, 0, W, H, 1, 2);
        let slide = (30 - self.t as i32).max(0) * 8;
        let title = if self.mode == Mode::Two { format!("STAGE {} OF 3", self.stage + 1) } else { format!("STAGE {}", self.stage + 1) };
        f.logo(128 - slide, 6, &title, 3, &SUNSET, OUTLINE, rgb(8, 8, 24));
        f.text_grad_center(128 + slide, 36, self.venue().name(), &grad7(rgb(96, 224, 248)), OUTLINE, true);
        f.text_center(128, 48, self.venue().tagline(), rgb(232, 232, 248), OUTLINE, false);
        // who flies today
        let others: Vec<&Pilot> = race.pilots.iter().filter(|p| p.ctrl == Ctrl::Cpu).collect();
        let label = if self.mode == Mode::One { "YOUR RIVALS" } else { "FLYING WITH YOU" };
        f.text_center(128, 62, label, rgb(248, 216, 48), OUTLINE, false);
        let n = others.len() as i32;
        for (k, p) in others.iter().enumerate() {
            let x = 128 + (k as i32 * 2 - (n - 1)) * 40;
            let bob = ((t as f32 * 0.08 + k as f32 * 2.0).sin() * 2.0) as i32;
            let l = Look::flying(0.0, true);
            draw_pilot(f, &self.art, p.def, x, 112 + bob, &l);
            let name = PILOTS[p.def].short;
            f.text_center(x, 118, name, rgb(248, 248, 248), OUTLINE, false);
            if self.mode == Mode::One {
                let sk = p.brain.as_ref().map(|b| b.skill).unwrap_or(0.5);
                let stars = (1 + (sk * 4.0) as i32).max(1);
                for s in 0..5 {
                    let col = if s < stars { rgb(248, 216, 48) } else { rgb(64, 64, 96) };
                    f.rect(x - 14 + s * 6, 128, 4, 4, col);
                }
            }
        }
        // the strongest rival heckles
        if let Some(r) = others.iter().max_by(|a, b| {
            let sa = a.brain.as_ref().map(|x| x.skill).unwrap_or(0.0);
            let sb = b.brain.as_ref().map(|x| x.skill).unwrap_or(0.0);
            sa.partial_cmp(&sb).unwrap()
        }) {
            if self.t > 40 {
                let k = others.iter().position(|p| p.def == r.def).unwrap_or(0) as i32;
                let x = 128 + (k * 2 - (n - 1)) * 40;
                bubble(f, x, 82, PILOTS[r.def].quote);
            }
        }
        // the tip box
        let y = 140;
        f.rect(6, y, 244, 58, rgb(16, 16, 48));
        f.rect_outline(6, y, 244, 58, rgb(248, 176, 32));
        let tips: [&str; 5] = if self.stage == 0 {
            [
                "RUN: TAP LEFT + RIGHT. X AT THE EDGE!",
                "UP: SLOW   DOWN: FAST   C: TUCK",
                "LEFT/RIGHT: TURN. WEAVE IN THE LIFT.",
                "STORKS, LEAVES AND CLOUDS MARK LIFT.",
                "FLY THROUGH 4 HOOPS. X TO FLARE + LAND",
            ]
        } else if self.stage == 1 {
            ["THERMALS GROW OVER SUNNY SLOPES.", "THE LAKE SINKS YOU. CROSS IT HIGH.", "UP TOO LONG = STALL. DON'T.", "COWS ARE NOT LANDING PADS.", "NEITHER ARE SHEEP."]
        } else {
            ["STRONG LIFT, STRONG SINK.", "DUST DEVILS = THERMALS.", "THE STORM CLOUD SUCKS YOU UP", "AND SPITS YOU OUT. DIZZY.", "FLY IT IF YOU DARE."]
        };
        for (k, s) in tips.iter().enumerate() {
            f.text_sh(12, y + 4 + k as i32 * 10, s, if k == 0 { rgb(248, 232, 64) } else { rgb(232, 232, 248) }, OUTLINE, false);
        }
        if self.t > 60 && (t / 20) % 2 == 0 {
            f.text_center(128, 208, "PUSH START", rgb(248, 248, 248), OUTLINE, true);
        }
        if self.mode == Mode::Two {
            f.text_center(128, 218, &format!("TOTAL  1P {}   2P {}", self.totals[0], self.totals[1]), rgb(176, 184, 224), OUTLINE, false);
        }
    }

    fn draw_results(&self, f: &mut Frame, t: u32) {
        let (Some(race), Some(sc)) = (&self.race, &self.scenery) else { return };
        let c = &race.course;
        let cam = Cam { x: c.target, y: c.ground_at(c.target) + 80.0, cy: 112, vh: H };
        sc.draw_sky(f, cam.pos(), cam.cy, H, t);
        sc.draw_back(f, cam.pos(), cam.cy, H, t);
        sc.draw_ground(f, c, cam.pos(), cam.cy, H, t);
        f.shade_rect(0, 0, W, H, 1, 2);
        f.logo(128, 4, "RESULTS", 3, &SUNSET, OUTLINE, rgb(8, 8, 24));
        f.text_center(128, 32, self.venue().name(), rgb(96, 224, 248), OUTLINE, true);
        let step = self.tally / 30;
        let sub = (self.tally % 30) as f32 / 30.0;
        let humans: Vec<usize> = (0..race.pilots.len()).filter(|&i| matches!(race.pilots[i].ctrl, Ctrl::Human(_))).collect();
        let cols = humans.len() as i32;
        for (n, &i) in humans.iter().enumerate() {
            let p = &race.pilots[i];
            let x0 = if cols == 1 { 40 } else { 6 + n as i32 * 126 };
            let w = if cols == 1 { 176 } else { 118 };
            let rows: [(&str, i32, String); 5] = [
                ("GATES", p.gate_pts, format!("{} X {}", p.gates_made(), GATE_PTS)),
                ("TIME", p.time_bonus, clock_text(p.clock + p.walk_frames)),
                ("LANDING", p.landing_pts, landing_text(p, c).to_string()),
                ("BONUS", p.bonus, String::new()),
                ("PENALTY", -p.penalty, String::new()),
            ];
            f.rect(x0, 46, w, 76, rgb(16, 16, 48));
            f.rect_outline(x0, 46, w, 76, PILOTS[p.def].ui);
            f.text_sh(x0 + 4, 49, PILOTS[p.def].name, PILOTS[p.def].ui, OUTLINE, false);
            for (k, (lab, v, note)) in rows.iter().enumerate() {
                if step < k as i32 {
                    break;
                }
                let shown = if step == k as i32 { (*v as f32 * sub) as i32 } else { *v };
                let y = 60 + k as i32 * 10;
                f.text_sh(x0 + 4, y, lab, rgb(200, 200, 232), OUTLINE, false);
                if cols == 1 {
                    f.text_sh(x0 + 56, y, note, rgb(144, 152, 200), OUTLINE, false);
                }
                let s = format!("{:>5}", shown);
                f.text_sh(x0 + w - 34, y, &s, if *v < 0 { rgb(248, 120, 120) } else { rgb(248, 248, 248) }, OUTLINE, false);
            }
            if step >= 5 {
                f.hline(x0 + 4, x0 + w - 4, 110, rgb(248, 176, 32));
                f.text_sh(x0 + 4, 112, "TOTAL", rgb(248, 216, 48), OUTLINE, true);
                f.text_sh(x0 + w - 46, 112, &format!("{:>5}", p.total()), rgb(248, 216, 48), OUTLINE, true);
            }
        }
        if step >= 6 {
            // the ranking
            let rank = race.ranking();
            f.rect(40, 128, 176, 52, rgb(16, 16, 48));
            f.rect_outline(40, 128, 176, 52, rgb(248, 176, 32));
            for (k, &i) in rank.iter().enumerate() {
                let p = &race.pilots[i];
                let y = 132 + k as i32 * 12;
                let me = matches!(p.ctrl, Ctrl::Human(_));
                let col = if me { if (t / 10) % 2 == 0 { rgb(248, 232, 64) } else { rgb(248, 248, 248) } } else { rgb(200, 200, 232) };
                f.text_sh(46, y, ordinal(k + 1), if k == 0 { rgb(248, 216, 48) } else { rgb(176, 184, 224) }, OUTLINE, false);
                f.text_sh(72, y, PILOTS[p.def].name, col, OUTLINE, false);
                f.text_sh(176, y, &format!("{:>5}", p.total()), col, OUTLINE, false);
            }
            let msg = match self.mode {
                Mode::One if self.won_stage() && self.stage == 2 => "YOU WIN THE RALLY!".to_string(),
                Mode::One if self.won_stage() => format!("NEXT: {}", VENUES[self.stage + 1].name()),
                Mode::One => format!("BEATEN BY {}", PILOTS[race.pilots[race.ranking()[0]].def].short),
                _ => format!("TOTAL  1P {}   2P {}", self.totals[0], self.totals[1]),
            };
            f.text_grad_center(128, 186, &msg, &grad7(if self.mode != Mode::One || self.won_stage() { rgb(120, 232, 96) } else { rgb(248, 104, 96) }), OUTLINE, true);
            if (t / 20) % 2 == 0 {
                f.text_center(128, 204, "PUSH START", rgb(248, 248, 248), OUTLINE, true);
            }
        }
    }

    fn draw_continue(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(32, 8, 24), rgb(48, 16, 40));
        f.logo(128, 26, "CONTINUE?", 3, &SUNSET, OUTLINE, rgb(0, 0, 0));
        let n = format!("{}", self.count.max(0));
        f.logo(128, 64, &n, 6, &grad6(rgb(248, 232, 64)), OUTLINE, rgb(0, 0, 0));
        // your pilot, stuck in a tree
        tree_with_pilot(f, &self.art, self.cursor[0], 128, 196, t);
        if (t / 20) % 2 == 0 {
            f.text_center(128, 204, "PUSH Z TO TRY AGAIN", rgb(248, 248, 248), OUTLINE, false);
        }
    }

    fn draw_game_over(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(16, 8, 32), rgb(24, 16, 48));
        f.logo(128, 40, "GAME", 4, &grad6(rgb(216, 48, 64)), OUTLINE, rgb(0, 0, 0));
        f.logo(128, 78, "OVER", 4, &grad6(rgb(216, 48, 64)), OUTLINE, rgb(0, 0, 0));
        f.rect(0, 160, W, 64, rgb(96, 168, 80));
        f.hline(0, W - 1, 160, rgb(152, 224, 96));
        // the sheep celebrate
        for k in 0..5 {
            let hop = if ((t / 8) as i32 + k) % 4 == 0 { 3 } else { 0 };
            draw_pic(f, &self.art.critters.sheep[0], 50 + k * 38, 178 - hop, k % 2 == 1);
        }
        f.text_center(128, 196, "THE SHEEP WIN THIS TIME", rgb(248, 248, 248), OUTLINE, true);
    }

    fn draw_champion(&self, f: &mut Frame, t: u32) {
        let cam = Cam { x: 4700.0, y: 70.0, cy: 140, vh: H };
        let sc = &self.title_sc;
        let c = &self.title_course;
        sc.draw_sky(f, cam.pos(), cam.cy, H, t);
        sc.draw_back(f, cam.pos(), cam.cy, H, t);
        sc.draw_ground(f, c, cam.pos(), cam.cy, H, t);
        for (i, pr) in c.props.iter().enumerate() {
            if (pr.x - cam.x).abs() < 170.0 {
                if let Some(p) = &sc.props[i] {
                    draw_pic(f, p, cam.sx(pr.x), cam.sy(pr.y) + 1, false);
                }
            }
        }
        // podium: the champion on top, two rivals beside
        let champ = self.champion_def();
        let order = [(champ, 128, 40, rgb(248, 216, 48)), ((champ + 1) % 4, 88, 26, rgb(200, 208, 224)), ((champ + 2) % 4, 168, 16, rgb(208, 136, 72))];
        let gy = cam.sy(c.ground_at(cam.x));
        for (k, &(def, x, h, col)) in order.iter().enumerate() {
            f.rect(x - 19, gy - h - 1, 38, h + 2, OUTLINE);
            f.rect(x - 18, gy - h, 36, h, col);
            f.rect(x + 8, gy - h, 10, h, shade(col, 3, 4));
            f.text_center(x, gy - h + 4, &format!("{}", k + 1), rgb(24, 24, 48), col, true);
            let hop = if k == 0 { (((t as i32) % 30) - 15).abs() / 3 } else { 0 };
            let pose = if k == 0 || (t / 20) % 2 == 0 { PPose::Cheer } else { PPose::Stand };
            let l = Look { pose, pitch: 14.0, turn: 0, flip: false, wing: pose != PPose::Cheer, wreck: false };
            draw_pilot(f, &self.art, def, x, gy - h - 1 - hop, &l);
        }
        for &(x, y, _, col) in &self.confetti {
            f.rect(x as i32, y as i32, 2, 2, col);
        }
        let title = if champ == 2 { "SKY QUEEN!" } else { "SKY KING!" };
        f.logo(128, 6, title, 4, &SUNSET, OUTLINE, rgb(8, 8, 24));
        let s = format!("{} RULES THE SKY!", PILOTS[champ].name);
        f.text_grad_center(128, 44, &s, &grad7(rgb(96, 224, 248)), OUTLINE, true);
        if self.mode == Mode::Two {
            f.text_center(128, 58, &format!("1P {}   2P {}", self.totals[0], self.totals[1]), rgb(248, 248, 248), OUTLINE, false);
        }
        f.rect(0, 206, W, 18, rgb(16, 16, 48));
        f.text_center(128, 210, "THANKS FOR FLYING HELICO AIR", rgb(248, 248, 248), OUTLINE, true);
    }

    pub fn champion_def(&self) -> usize {
        match self.mode {
            Mode::Two => {
                if self.totals[1] > self.totals[0] {
                    self.cursor[1]
                } else {
                    self.cursor[0]
                }
            }
            _ => self.cursor[0],
        }
    }
}

fn landing_text(p: &Pilot, c: &Course) -> &'static str {
    match p.landing {
        Some(Landing::Crash) => "CRASH",
        Some(Landing::Splash) => "SPLASH",
        _ if !c.in_field(p.land_x) => "OUTLANDING",
        _ if p.sheep_hit => "ON A SHEEP",
        _ if p.landing_pts >= 500 => "BULLSEYE",
        _ if p.landing_pts >= 300 => "CLOSE",
        _ if p.landing_pts >= 150 => "ON TARGET",
        _ => "IN THE FIELD",
    }
}

fn stripes(f: &mut Frame, t: u32, a: u32, b: u32) {
    for y in 0..H {
        for x in 0..W {
            let k = ((x + y + (t / 2) as i32) / 8) % 2;
            f.px[(y * W + x) as usize] = if k == 0 { a } else { b };
        }
    }
}

/// A tiny glider for the menu cursor.
fn mini_glider(f: &mut Frame, _art: &Art, x: i32, y: i32, t: u32) {
    let bob = ((t / 10) % 2) as i32;
    let y = y + bob;
    f.fill_poly(&[(x - 7, y), (x + 6, y - 3), (x + 2, y + 2)], OUTLINE);
    f.fill_poly(&[(x - 6, y), (x + 5, y - 2), (x + 2, y + 1)], rgb(248, 216, 48));
    f.line(x, y + 1, x, y + 4, rgb(200, 200, 216));
    f.rect(x - 2, y + 4, 4, 2, rgb(48, 96, 224));
}

/// Continue screen: a big tree with your glider stuck in it.
fn tree_with_pilot(f: &mut Frame, art: &Art, def: usize, x: i32, y: i32, t: u32) {
    f.rect(0, y, W, H - y, rgb(80, 136, 64));
    f.rect(x - 6, y - 60, 12, 60, OUTLINE);
    f.rect(x - 5, y - 60, 10, 60, rgb(120, 80, 48));
    f.rect(x + 1, y - 60, 4, 60, rgb(96, 64, 40));
    for (dx, dy, r) in [(0, -78, 34), (-28, -64, 22), (28, -66, 22), (-14, -96, 20), (16, -94, 20)] {
        f.fill_ellipse(x + dx, y + dy, r + 1, r * 3 / 4 + 1, OUTLINE);
    }
    for (dx, dy, r) in [(0, -78, 34), (-28, -64, 22), (28, -66, 22), (-14, -96, 20), (16, -94, 20)] {
        f.fill_ellipse(x + dx, y + dy, r, r * 3 / 4, rgb(56, 136, 64));
        f.fill_ellipse(x + dx - r / 3, y + dy - r / 4, r / 3, r / 4, rgb(88, 168, 80));
    }
    let gp = glider_palette(def);
    blit(f, art.wing_pitch(def, -24.0), x + 8, y - 88, false, &gp);
    let sway = ((t as f32 * 0.05).sin() * 2.0) as i32;
    let pal = palette(def);
    f.line(x + 8, y - 88, x + 10 + sway, y - 62, rgb(48, 48, 64));
    blit(f, art.pilot(def, PPose::Flare), x + 10 + sway, y - 34, false, &pal);
}
