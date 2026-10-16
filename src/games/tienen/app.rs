//! Scenes and flow: title, kid select, the venue card, the game (with the
//! latjetrap before and the billenbrand after), results, continue, game
//! over and the champion screen. 1P is a ladder of three venues; 2P is one
//! game with two humans and two local kids.

use super::art::*;
use super::audio::Track;
use super::data::*;
use super::render::*;
use super::scenery::*;
use super::world::*;
use crate::engine::gfx::*;
use crate::engine::input::{any, edge, Pad};
use crate::engine::ui::{grad6, grad7, LOGO_SHADOW, OUTLINE, SUNSET};

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
    /// chosen kid per player (index into PICKS)
    pub cursor: [usize; 2],
    pub locked: [bool; 2],
    pub stage: usize,
    /// 2P: the venue
    pub venue2p: usize,
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
    pub score: [i32; 2],
    pub tally: [Vec<Line>; 2],
    pub cleared: bool,
    /// 2P: who won the game
    pub winner: usize,
    pub reason: String,
    pub tries: usize,
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
            venue2p: 1,
            world: None,
            scenery: None,
            title_sc: Scenery::new(Place::Flat, 3),
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
            winner: 0,
            reason: String::new(),
            tries: 0,
        }
    }

    pub fn music(&self) -> Option<Track> {
        match self.scene {
            Scene::Title | Scene::Select | Scene::Intro | Scene::Results => Some(Track::Title),
            Scene::Play => match self.world.as_ref().map(|w| w.phase) {
                Some(Phase::Over) | Some(Phase::Brand) | Some(Phase::Done) => Some(Track::Brand),
                _ => Some(Track::Play),
            },
            Scene::Champion => Some(Track::Fanfare),
            Scene::Continue | Scene::GameOver => None,
        }
    }

    fn go(&mut self, s: Scene) {
        self.scene = s;
        self.t = 0;
    }

    pub fn venue_now(&self) -> usize {
        match self.mode {
            Mode::Two => self.venue2p,
            _ => self.stage.min(VENUES.len() - 1),
        }
    }

    fn ensure_scenery(&mut self, v: usize) {
        let place = VENUES[v].place;
        if self.scenery.as_ref().map(|s| s.place != place).unwrap_or(true) {
            self.scenery = Some(Scenery::new(place, 0x51 + v as u64 * 7));
        }
    }

    pub fn start_play(&mut self) {
        let v = self.venue_now();
        let d = &VENUES[v];
        let roster = match self.mode {
            Mode::One => [(PICKS[self.cursor[0]], Ctrl::Human(0), 0.75), (d.kids[0], Ctrl::Cpu, d.skill), (d.kids[1], Ctrl::Cpu, d.skill), (d.kids[2], Ctrl::Cpu, d.skill)],
            Mode::Two => [(PICKS[self.cursor[0]], Ctrl::Human(0), 0.75), (PICKS[self.cursor[1]], Ctrl::Human(1), 0.75), (d.kids[0], Ctrl::Cpu, d.skill), (d.kids[1], Ctrl::Cpu, d.skill)],
            Mode::Demo => {
                let pick = PICKS[self.rng.int(4) as usize];
                [(pick, Ctrl::Cpu, 0.7), (d.kids[0], Ctrl::Cpu, d.skill), (d.kids[1], Ctrl::Cpu, d.skill), (d.kids[2], Ctrl::Cpu, d.skill)]
            }
        };
        let mut w = World::new(v, roster, self.rng.next() as u64);
        if self.mode == Mode::One {
            w.quota = Some(d.quota);
        }
        self.world = Some(w);
        self.ensure_scenery(v);
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
                            let other = 1 - pi;
                            if players == 2 && self.locked[other] && self.cursor[other] == self.cursor[pi] {
                                // taken
                                self.sfx.push(Sfx::Whiff);
                            } else {
                                self.locked[pi] = true;
                                self.sfx.push(Sfx::Kick);
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
                    self.tries = 0;
                    self.score = [0; 2];
                    self.go(Scene::Intro);
                }
            }
            Scene::Intro => {
                if self.mode == Mode::Two && (e[0].left || e[0].right) {
                    let n = VENUES.len();
                    self.venue2p = if e[0].left { (self.venue2p + n - 1) % n } else { (self.venue2p + 1) % n };
                    self.sfx.push(Sfx::Blip);
                }
                if self.t == 2 || self.mode == Mode::Two {
                    let v = self.venue_now();
                    self.ensure_scenery(v);
                }
                if self.t > 900 || (self.t > 40 && (ok(e[0]) || ok(e[1]))) {
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
                if w.finished() && w.pt > 40 {
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
                    self.sfx.push(if self.cleared || self.mode == Mode::Two { Sfx::Cheer } else { Sfx::Groan });
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

    /// One kid's lines on the results screen.
    fn lines_for(w: &World, k: usize) -> Vec<Line> {
        let kd = &w.kids[k];
        let mut v = Vec::new();
        for kind in KINDS {
            let n = kd.stats.goals[kind.index()];
            if n > 0 {
                v.push(Line { label: format!("{} X {}", kind.name(), n), pts: n as i32 * kind.value() * 100 });
            }
        }
        if kd.stats.saves > 0 {
            v.push(Line { label: format!("REDDINGEN X {}", kd.stats.saves), pts: kd.stats.saves as i32 * 50 });
        }
        if w.loser != Some(k) {
            v.push(Line { label: format!("PUNTEN OVER {}", kd.pts), pts: kd.pts * 30 });
        }
        if kd.stats.brand_hits > 0 {
            v.push(Line { label: format!("RAAK X {}", kd.stats.brand_hits), pts: kd.stats.brand_hits as i32 * 150 });
        }
        if w.loser == Some(k) {
            v.push(Line { label: "OP NUL!".into(), pts: 0 });
        } else {
            v.push(Line { label: "NIET OP NUL".into(), pts: 500 });
        }
        v
    }

    fn make_results(&mut self) {
        let Some(w) = self.world.as_ref() else { return };
        let lo = w.loser.unwrap_or(0);
        let lname = kid(w.kids[lo].id).name;
        self.reason = if w.dinner { format!("ETEN! {} HAD THE FEWEST POINTS", lname) } else { format!("{} IS OP NUL", lname) };
        match self.mode {
            Mode::Two => {
                let h: Vec<usize> = (0..2).map(|p| w.human_kid(p).unwrap_or(p)).collect();
                let lines: Vec<Vec<Line>> = h.iter().map(|&k| Self::lines_for(w, k)).collect();
                self.winner = if lo == h[0] {
                    1
                } else if lo == h[1] {
                    0
                } else {
                    let (a, b) = (&w.kids[h[0]], &w.kids[h[1]]);
                    if b.pts > a.pts || (b.pts == a.pts && b.stats.scored > a.stats.scored) {
                        1
                    } else {
                        0
                    }
                };
                for (s, l) in self.score.iter_mut().zip(lines.iter()) {
                    *s += l.iter().map(|x| x.pts).sum::<i32>();
                }
                self.tally = [lines[0].clone(), lines[1].clone()];
                self.cleared = true;
            }
            _ => {
                let me = w.human_kid(0).unwrap_or(0);
                let mut lines = Self::lines_for(w, me);
                let q = VENUES[w.venue].quota;
                let s = w.kids[me].stats.scored;
                let safe = w.loser != Some(me);
                self.cleared = safe && s >= q;
                if s >= q {
                    lines.push(Line { label: format!("RESPECT {}/{}", s, q), pts: 300 });
                } else {
                    lines.push(Line { label: format!("DOEL {}/{}", s, q), pts: 0 });
                    if safe {
                        self.reason = format!("ONLY {} OF {} GOAL POINTS. NO RESPECT.", s, q);
                    }
                }
                if self.cleared {
                    self.score[0] += lines.iter().map(|x| x.pts).sum::<i32>();
                }
                self.tally = [lines, Vec::new()];
            }
        }
    }

    fn after_results(&mut self) {
        match self.mode {
            Mode::One => {
                if self.cleared {
                    self.stage += 1;
                    self.tries = 0;
                    if self.stage >= VENUES.len() {
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
                self.make_confetti();
                self.go(Scene::Champion);
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
        draw_world(f, w, sc, &self.art, t);
        if w.shake > 0 {
            let dx = if (w.shake / 2) % 2 == 0 { 1 } else { -1 };
            for y in 28..H {
                let row = &mut f.px[(y * W) as usize..((y + 1) * W) as usize];
                if dx > 0 {
                    row.rotate_right(1);
                } else {
                    row.rotate_left(1);
                }
            }
        }
        draw_hud(f, w, &self.art, t);
        match w.phase {
            Phase::Latje => draw_latje(f, w, t),
            Phase::CountOut => draw_count_out(f, w, t),
            Phase::Brand => draw_brand(f, w, t),
            _ => {}
        }
        draw_finish_hint(f, w);
        if let Some(b) = &w.banner {
            draw_banner(f, b, 52);
        }
        if self.mode != Mode::Demo {
            self.draw_tips(f, w);
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

    /// The controls, the first time they matter: at the kick-off for a field
    /// player, and whenever a human has just gone in goal.
    fn draw_tips(&self, f: &mut Frame, w: &World) {
        let human_keeper = w.kids[w.keeper].human();
        let lines: Option<[&str; 2]> = if w.phase == Phase::Wissel && human_keeper.is_some() && w.pt > 20 {
            Some(["YOU KEEP! Z CATCH, Z+UP/DOWN DIVE", "X JUMP, C PUNCH. CATCH = WISSEL!"])
        } else if w.phase == Phase::Play && w.play_t < 420 && human_keeper.is_none() && self.stage == 0 && self.tries == 0 {
            Some(["C: CROSS TO A MATE  Z: SHOOT IT", "OUT OF THE AIR  X: JUMP  ARROWS AIM"])
        } else {
            None
        };
        if let Some(l) = lines {
            let y = 196;
            f.rect(4, y - 2, 248, 26, rgb(16, 16, 48));
            f.rect_outline(4, y - 2, 248, 26, rgb(248, 136, 24));
            f.text_sh(10, y + 1, l[0], white(), OUTLINE, false);
            f.text_sh(10, y + 12, l[1], rgb(248, 232, 96), OUTLINE, false);
        }
    }

    // ------------------------------------------------------------ title

    fn draw_title(&self, f: &mut Frame, t: u32) {
        self.title_sc.draw(f);
        goal_back(f);
        // Kevin keeps it up; Gijs waits in goal
        let kp = kid_palette(0);
        let ppl = self.art.kid(0);
        let phase = t % 40;
        let bz = 6.0 + (20.0 - (phase as f32 - 20.0).abs()) * 1.6;
        let (kx, ky) = proj(150.0, 46.0, 0.0);
        f.shade_ellipse(kx, ky, 7, 2, 5, 8);
        let s = if phase < 6 { &ppl.kick[0] } else { &ppl.stand };
        blit_person(f, s, kx, ky, true, &kp);
        let (bx, by) = proj(146.0, 46.0, bz);
        let (gx, gy) = proj(146.0, 46.0, 0.0);
        f.shade_ellipse(gx, gy, 3, 1, 1, 2);
        self.art.ball[((t / 4) % 2) as usize].draw(f, bx - 4, by - 7, &PROPS, false);
        let gp = kid_palette(2);
        let g = self.art.kid(2);
        let (gkx, gky) = proj(GOAL_X + 8.0, goal_mid(), 0.0);
        f.shade_ellipse(gkx, gky, 7, 2, 5, 8);
        blit_person(f, &g.ready, gkx, gky, false, &gp);
        goal_near(f);
        f.shade_rect(0, 0, W, 98, 3, 4);
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
        // the "10" bounces like a ball
        let hop = if st > 40 { (((t % 30) as i32 - 15).abs() - 15).abs() / 4 } else { 0 };
        f.logo(118, drop(12, 0), "TIENEN!", 5, &SUNSET, OUTLINE, rgb(24, 8, 40));
        f.logo(226, drop(48, 10) - hop, "10", 3, &grad6(rgb(248, 248, 248)), OUTLINE, rgb(24, 8, 40));
        if st > 36 {
            f.rect(0, 86, W, 11, rgb(24, 16, 56));
            f.hline(0, W - 1, 85, rgb(248, 136, 24));
            f.hline(0, W - 1, 97, rgb(248, 136, 24));
            f.text_grad_center(128, 88, "ONE GOAL. TEN POINTS. NO MERCY.", &grad7(white()), OUTLINE, false);
        }
        if !self.menu_open {
            if (t / 24).is_multiple_of(2) && st > 36 {
                f.text_grad_center(128, 106, "PUSH START", &grad7(white()), OUTLINE, true);
            }
        } else {
            f.shade_rect(72, 102, 112, 26, 1, 3);
            f.rect_outline(72, 102, 112, 26, rgb(248, 136, 24));
            for (k, s) in ["1 PLAYER", "2 PLAYERS"].iter().enumerate() {
                let y = 105 + k as i32 * 11;
                let sel = self.menu == k;
                let col = if sel { rgb(248, 232, 64) } else { rgb(184, 184, 216) };
                f.text_sh(100, y, s, col, OUTLINE, true);
                if sel {
                    self.art.ball[((t / 6) % 2) as usize].draw(f, 84, y - 1, &PROPS, false);
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
        stripes(f, t, rgb(16, 48, 32), rgb(24, 64, 40));
        f.text_grad_center(128, 4, "WIE DOET ER MEE?", &grad7(rgb(248, 176, 48)), OUTLINE, true);
        f.text_center(128, 14, "PICK YOUR KID", rgb(160, 216, 168), OUTLINE, false);
        for (k, &id) in PICKS.iter().enumerate() {
            let (cx, cy) = (6 + (k % 2) as i32 * 124, 24 + (k / 2) as i32 * 88);
            let d = kid(id);
            let hover: Vec<usize> = (0..2).filter(|&p| (p == 0 || self.mode == Mode::Two) && self.cursor[p] == k).collect();
            let accent = d.color;
            f.rect(cx, cy, 120, 84, rgb(8, 24, 16));
            f.rect(cx + 2, cy + 56, 116, 26, rgb(16, 40, 24));
            f.rect(cx + 2, cy + 2, 116, 11, shade(accent, 1, 2));
            f.rect_outline(cx, cy, 120, 84, shade(accent, 2, 3));
            f.text_grad(cx + 5, cy + 4, d.name, &grad7(mix(accent, 0xFFFFFF, 0.4)), OUTLINE, false);
            // tiles and the kid
            f.rect(cx + 2, cy + 50, 56, 5, rgb(168, 160, 152));
            f.hline(cx + 2, cx + 57, cy + 50, rgb(136, 128, 120));
            let active = !hover.is_empty();
            let locked = hover.iter().any(|&p| self.locked[p]);
            let ppl = self.art.style(d.style);
            let pal = kid_palette(id);
            let s = if locked {
                if (t / 8).is_multiple_of(2) {
                    &ppl.victory
                } else {
                    &ppl.cheer
                }
            } else if active {
                // their signature move
                match k {
                    0 => &ppl.kick[((t / 8) % 2) as usize],
                    1 => &ppl.header,
                    2 => &ppl.ready,
                    _ => &ppl.bike[((t / 10) % 2) as usize],
                }
            } else {
                &ppl.stand
            };
            blit_person(f, s, cx + 26, cy + 52, false, &pal);
            if active && !locked {
                let bz = match k {
                    0 => 8,
                    1 => 26,
                    2 => 18,
                    _ => 22,
                };
                self.art.ball[((t / 4) % 2) as usize].draw(f, cx + 38, cy + 46 - bz, &PROPS, false);
            }
            for (r, lab) in ["SCHOT", "KOP", "KEEP", "SNEL"].iter().enumerate() {
                let y = cy + 16 + r as i32 * 9;
                f.text(cx + 56, y, lab, rgb(200, 232, 208));
                for pip in 0..5 {
                    let c = if pip < d.stats[r] { mix(accent, 0xFFFFFF, 0.3) } else { rgb(40, 64, 48) };
                    f.rect(cx + 92 + pip * 5, y, 5, 6, OUTLINE);
                    f.rect(cx + 93 + pip * 5, y + 1, 3, 4, c);
                }
            }
            for (k, l) in d.bio.iter().enumerate() {
                f.text(cx + 4, cy + 58 + k as i32 * 8, l, rgb(176, 224, 184));
            }
            for &p in &hover {
                let col = if p == 0 { P1_COL } else { P2_COL };
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
                    f.text_grad_center(cx + 26, cy + 14, "OK!", &grad7(col), OUTLINE, true);
                }
            }
        }
        if self.mode == Mode::Two {
            f.text_center(128, 202, "1P: ARROWS + , . /  (OR Z X C)", P1_COL, OUTLINE, false);
            f.text_center(128, 212, "2P: W A S D + F G H", P2_COL, OUTLINE, false);
        } else {
            f.text_center(128, 208, "ARROWS: PICK   Z: OK   X: BACK", rgb(200, 232, 208), OUTLINE, false);
        }
    }

    // ------------------------------------------------------------ intro

    fn draw_intro(&self, f: &mut Frame, t: u32) {
        let v = self.venue_now();
        let d = &VENUES[v];
        match &self.scenery {
            Some(sc) if sc.place == d.place => sc.draw(f),
            _ => stripes(f, t, rgb(16, 48, 32), rgb(24, 64, 40)),
        }
        f.shade_rect(0, 0, W, H, 1, 2);
        let slide = (30 - self.t as i32).max(0) * 9;
        let label = if self.mode == Mode::Two { "2 PLAYERS".to_string() } else { format!("VELD {}", self.stage + 1) };
        f.text_grad_center(128 - slide, 5, &label, &grad7(rgb(248, 176, 48)), OUTLINE, true);
        let scale = if d.name.len() > 8 { 3 } else { 4 };
        f.logo(128 + slide, 15, d.name, scale, &SUNSET, OUTLINE, LOGO_SHADOW);
        if self.mode == Mode::Two && (t / 20).is_multiple_of(2) {
            f.text_sh(6, 24, "<", P1_COL, OUTLINE, true);
            f.text_sh(244, 24, ">", P1_COL, OUTLINE, true);
        }
        f.text_center(128, 48, d.tagline[0], white(), OUTLINE, false);
        f.text_center(128, 57, d.tagline[1], rgb(200, 208, 240), OUTLINE, false);
        // the locals, lined up
        let gy = 128;
        let locals: Vec<usize> = if self.mode == Mode::Two { d.kids[..2].to_vec() } else { d.kids.to_vec() };
        f.text_center(128, 70, "TEGEN", rgb(248, 176, 48), OUTLINE, true);
        for (k, &id) in locals.iter().enumerate() {
            let x = 128 + (k as i32 - (locals.len() as i32 - 1) / 2) * 56 - if locals.len() == 2 { 28 } else { 0 };
            let ppl = self.art.kid(id);
            f.shade_ellipse(x, gy, 8, 2, 5, 8);
            let s = if kid(id).name == d.rival && (t / 30).is_multiple_of(2) { &ppl.cheer } else { &ppl.stand };
            blit_person(f, s, x, gy, k == 0, &kid_palette(id));
            f.text_center(x, gy + 4, kid(id).name, kid(id).color, OUTLINE, false);
            if kid(id).name == d.rival && self.t > 40 {
                let h = d.heckles[((t / 150) as usize) % d.heckles.len()];
                bubble(f, x, gy - 36, h);
            }
        }
        // tip box
        let (tx, ty) = (8, 150);
        f.rect(tx, ty, 240, 46, rgb(16, 16, 48));
        f.rect_outline(tx, ty, 240, 46, rgb(248, 136, 24));
        if self.mode == Mode::Two {
            f.text_sh(tx + 6, ty + 5, "1P AND 2P PLAY TIENEN WITH TWO LOCALS.", white(), OUTLINE, false);
            f.text_sh(tx + 6, ty + 15, "WHOEVER HITS ZERO LOSES. A LOCAL ON", white(), OUTLINE, false);
            f.text_sh(tx + 6, ty + 25, "ZERO? MOST POINTS LEFT WINS.", white(), OUTLINE, false);
            f.text_sh(tx + 6, ty + 35, "LEFT/RIGHT: PICK THE PITCH", P1_COL, OUTLINE, false);
        } else {
            f.rect(tx + 6, ty - 4, 26, 9, rgb(16, 16, 48));
            f.text_sh(tx + 8, ty - 3, "TIP", rgb(248, 136, 24), OUTLINE, false);
            f.text_sh(tx + 6, ty + 6, d.tip[0], white(), OUTLINE, false);
            f.text_sh(tx + 6, ty + 15, d.tip[1], white(), OUTLINE, false);
            f.text_sh(tx + 6, ty + 26, &format!("SCORE {} GOAL POINTS FOR RESPECT,", d.quota), rgb(248, 232, 96), OUTLINE, false);
            f.text_sh(tx + 6, ty + 35, "AND DON'T BE THE ONE ON ZERO!", rgb(248, 232, 96), OUTLINE, false);
        }
        if (t / 20).is_multiple_of(2) && self.t > 40 {
            f.text_center(128, 204, "PUSH START", white(), OUTLINE, true);
        }
    }

    // ------------------------------------------------------------ results

    fn draw_results(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(40, 16, 8), rgb(56, 24, 8));
        let head = if self.mode == Mode::Two {
            "UITSLAG"
        } else if self.cleared {
            "GEWONNEN!"
        } else if self.reason.starts_with("ONLY") {
            "GEEN RESPECT"
        } else {
            "OP NUL..."
        };
        let grad = if self.cleared || self.mode == Mode::Two { SUNSET.to_vec() } else { grad6(rgb(216, 48, 64)).to_vec() };
        f.logo(128, 4, head, 3, &grad, OUTLINE, LOGO_SHADOW);
        f.text_center(128, 34, &self.reason, rgb(200, 192, 232), OUTLINE, false);
        let n = self.tally[0].len().max(self.tally[1].len()) as u32;
        let shown = if self.t < 20 { 0 } else { ((self.t - 20) / 30 + 1).min(n) } as usize;
        let done = self.t >= 20 + n * 30 + 30;
        let players: Vec<usize> = if self.mode == Mode::Two { vec![0, 1] } else { vec![0] };
        for &p in &players {
            let (x0, w) = if players.len() == 2 { (8 + p as i32 * 122, 118) } else { (36, 184) };
            let y0 = 48;
            let col = if p == 0 { P1_COL } else { P2_COL };
            f.rect(x0, y0, w, 140, rgb(16, 16, 40));
            f.rect_outline(x0, y0, w, 140, col);
            let name = kid(PICKS[self.cursor[p]]).name;
            if self.mode == Mode::Two {
                f.text_sh(x0 + 5, y0 + 4, &format!("{}P {}", p + 1, name), col, OUTLINE, false);
                if done && self.winner == p {
                    f.text_sh(x0 + w - 40, y0 + 4, "WINT!", P1_COL, OUTLINE, true);
                }
            } else {
                f.text_sh(x0 + 5, y0 + 4, name, col, OUTLINE, false);
            }
            let lines = &self.tally[p];
            for (k, l) in lines.iter().enumerate().take(shown) {
                let y = y0 + 16 + k as i32 * 11;
                f.text_sh(x0 + 5, y, &l.label, rgb(216, 216, 232), OUTLINE, false);
                let s = format!("{}", l.pts);
                f.text_sh(x0 + w - 5 - text_width(&s, false), y, &s, white(), OUTLINE, false);
            }
            let sum: i32 = lines.iter().take(shown).map(|l| l.pts).sum();
            f.hline(x0 + 4, x0 + w - 5, y0 + 112, col);
            let s = format!("{}", sum);
            f.text_sh(x0 + 5, y0 + 117, "TOTAAL", col, OUTLINE, false);
            f.text_grad(x0 + w - 5 - text_width(&s, true), y0 + 116, &s, &grad7(col), OUTLINE, true);
            if done {
                let s = if self.mode == Mode::One && !self.cleared { "NOT ADDED TO THE RUN".to_string() } else { format!("RUN SCORE {}", self.score[p]) };
                f.text_sh(x0 + 5, y0 + 129, &s, rgb(184, 184, 216), OUTLINE, false);
            }
        }
        if done && (t / 20).is_multiple_of(2) {
            let msg = match self.mode {
                Mode::One if self.cleared => "ON TO THE NEXT PITCH! PUSH START",
                Mode::One if self.reason.starts_with("ONLY") => "THEY WON'T PICK YOU... PUSH START",
                Mode::One => "YOUR BOTTOM HURTS... PUSH START",
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
        // you on the kerb, rubbing your bottom; the ball rolls by
        let gy = 176;
        f.rect(0, gy, W, H - gy, rgb(88, 88, 96));
        f.rect(0, gy - 3, W, 3, rgb(208, 200, 192));
        let id = PICKS[self.cursor[0]];
        let ppl = self.art.kid(id);
        f.shade_ellipse(112, gy + 2, 9, 2, 5, 8);
        let s = if (t / 30).is_multiple_of(2) { &ppl.sit } else { &ppl.ouch };
        blit_person(f, s, 112, gy + if (t / 30).is_multiple_of(2) { 2 } else { -2 }, false, &kid_palette(id));
        let bx = ((t as i32 * 2) % 320) - 30;
        self.art.ball[((t / 4) % 2) as usize].draw(f, bx, gy - 5, &PROPS, false);
        if (t / 50).is_multiple_of(2) {
            let s = if self.reason.starts_with("ONLY") { "NEXT TIME I SCORE!" } else { "AU... MIJN BILLEN" };
            bubble(f, 112, gy - 34, s);
        }
        f.text_center(128, 118, "ONE MORE GAME BEFORE DINNER?", rgb(248, 216, 96), OUTLINE, false);
        if (t / 20).is_multiple_of(2) {
            f.text_center(128, 196, "PUSH Z TO PLAY AGAIN", white(), OUTLINE, false);
        }
        f.text_center(128, 210, "NOG EVEN, MAM!", rgb(160, 200, 216), OUTLINE, false);
    }

    fn draw_game_over(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(16, 8, 32), rgb(24, 16, 48));
        f.logo(128, 50, "GAME", 4, &grad6(rgb(216, 48, 64)), OUTLINE, rgb(0, 0, 0));
        f.logo(128, 88, "OVER", 4, &grad6(rgb(216, 48, 64)), OUTLINE, rgb(0, 0, 0));
        f.text_center(128, 128, "MUM CALLED. DINNER IS READY.", rgb(200, 200, 232), OUTLINE, false);
        // walking home with the ball under the arm
        let x = (t as i32) % 360 - 60;
        let gy = 192;
        let id = PICKS[self.cursor[0]];
        let ppl = self.art.kid(id);
        f.shade_ellipse(x, gy, 7, 2, 5, 8);
        blit_person(f, &ppl.hold, x, gy, false, &kid_palette(id));
        self.art.ball[0].draw(f, x + 2, gy - 18, &PROPS, false);
        if (t / 50).is_multiple_of(2) {
            bubble(f, x + 4, gy - 34, "MORGEN WEER!");
        }
    }

    fn draw_champion(&self, f: &mut Frame, t: u32) {
        let sc = &self.title_sc;
        sc.draw(f);
        f.shade_rect(0, 0, W, 72, 2, 3);
        // the winner on the shoulders of the others
        let winner = if self.mode == Mode::Two { self.winner } else { 0 };
        let id = PICKS[self.cursor[winner]];
        let (cx, gy) = (128, 170);
        let others: Vec<usize> = if self.mode == Mode::Two { vec![PICKS[self.cursor[1 - winner]], 9] } else { vec![4, 5, 6] };
        for (k, &o) in others.iter().enumerate() {
            let x = cx - 20 + k as i32 * 20 - if others.len() == 2 { -10 } else { 0 };
            let ppl = self.art.kid(o);
            f.shade_ellipse(x, gy, 7, 2, 5, 8);
            blit_person(f, &ppl.victory, x, gy, k % 2 == 1, &kid_palette(o));
        }
        let ppl = self.art.kid(id);
        let hop = (((t as i32) % 30) - 15).abs() / 5;
        blit_person(f, &ppl.victory, cx, gy - 26 - hop, false, &kid_palette(id));
        self.art.ball[0].draw(f, cx + 6, gy - 62 - hop, &PROPS, false);
        for &(x, y, _, c) in &self.confetti {
            f.rect(x as i32, y as i32, 2, 2, c);
        }
        f.logo(128, 6, "KONING VAN", 3, &SUNSET, OUTLINE, LOGO_SHADOW);
        f.logo(128, 32, "HET PLEIN!", 3, &SUNSET, OUTLINE, LOGO_SHADOW);
        let (line, sub) = if self.mode == Mode::Two {
            (if winner == 0 { "SPELER 1 WINT!" } else { "SPELER 2 WINT!" }, format!("{} - {}", self.score[0], self.score[1]))
        } else {
            ("NOBODY GOT YOUR BOTTOM", format!("SCORE {}", self.score[0]))
        };
        f.text_grad_center(128, 62, line, &grad7(P2_COL), OUTLINE, true);
        f.text_center(128, 74, &sub, white(), OUTLINE, false);
        f.text_center(128, 212, "SAME TIME TOMORROW. BRING THE BALL.", rgb(248, 136, 24), OUTLINE, false);
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
