//! TIENEN! CPU kids: field play (fetch, dribble, cross, receive and pick a
//! shot that fits the keeper's points), keeping (read the shot, move, jump,
//! dive, catch or punch), the latjetrap and the billenbrand. Probe bots
//! play a human's kid with human habits.

use super::data::*;
use super::world::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BotKind {
    /// plays it sensibly
    Decent,
    /// a lazy keeper: late, never dives
    Lazy,
    /// always goes for the bicycle kick
    Greedy,
}

#[derive(Clone, Debug)]
pub struct Brain {
    pub rng: Rng,
    /// set by the world when a swing starts: where to aim, who to cross to
    pub aim: f32,
    pub to: Option<usize>,
    /// the touch count this brain last planned for
    seen: u32,
    want: Option<Kind>,
    react: u32,
    delay: u32,
    err: i32,
    hold: u32,
    jumped: bool,
    pressed: bool,
    wob: (f32, f32),
    turn: usize,
    target: f32,
}

impl Default for Brain {
    fn default() -> Self {
        Brain::new(1)
    }
}

impl Brain {
    pub fn new(seed: u64) -> Self {
        Brain {
            rng: Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1),
            aim: 0.0,
            to: None,
            seen: u32::MAX,
            want: None,
            react: 0,
            delay: 10,
            err: 0,
            hold: 30,
            jumped: false,
            pressed: false,
            wob: (0.0, 0.0),
            turn: usize::MAX,
            target: 0.5,
        }
    }
    pub fn reset(&mut self) {
        self.seen = u32::MAX;
        self.want = None;
        self.jumped = false;
        self.pressed = false;
    }
    fn gauss(&mut self) -> f32 {
        (self.rng.f() + self.rng.f() + self.rng.f() + self.rng.f() - 2.0) * 1.7
    }
}


/// Steer toward a point: full speed far away, slower close by.
fn go(me: &Kid, tx: f32, ty: f32) -> (f32, f32) {
    let (dx, dy) = (tx - me.x, (ty - me.y) * 1.3);
    let d = (dx * dx + dy * dy).sqrt();
    if d < 1.5 {
        return (0.0, 0.0);
    }
    let s = (d / 8.0).min(1.0);
    (dx / d * s, dy / d * s)
}

pub fn think(w: &World, k: usize, b: &mut Brain) -> Intent {
    match w.phase {
        Phase::Latje => latje(w, k, b),
        Phase::Brand => brand(w, k, b),
        Phase::Play => {
            if k == w.keeper {
                keeper(w, k, b)
            } else {
                field(w, k, b)
            }
        }
        _ => Intent::default(),
    }
}

// ------------------------------------------------------------ latjetrap

fn latje(w: &World, k: usize, b: &mut Brain) -> Intent {
    let l = &w.latje;
    if l.left.get(l.turn) != Some(&k) || l.stage != 0 {
        return Intent::default();
    }
    let me = &w.kids[k];
    let id = (l.round * 8 + l.turn) as usize;
    if b.turn != id {
        b.turn = id;
        let sd = 0.03 + 0.09 * (1.0 - me.skill) + 0.01 * (5 - schot(me.id)) as f32;
        b.target = 0.5 + b.gauss() * sd;
        b.hold = 30 + b.rng.int(50) as u32;
    }
    let mut it = Intent::default();
    if l.st > b.hold && (l.needle - b.target).abs() < 0.03 {
        it.a = true;
    }
    if l.st > 400 {
        it.a = true;
    }
    it
}

// ------------------------------------------------------------ billenbrand

fn brand(w: &World, k: usize, b: &mut Brain) -> Intent {
    let br = &w.brand;
    let mut it = Intent::default();
    if br.shooters.get(br.turn) == Some(&k) && br.stage == 0 {
        if b.turn != 100 + br.turn {
            b.turn = 100 + br.turn;
            b.target = b.gauss() * 0.3;
            b.hold = 40 + b.rng.int(40) as u32;
        }
        if br.st > b.hold && (br.aim - b.target).abs() < 0.08 {
            it.a = true;
        }
    }
    if w.loser == Some(k) {
        // a bot that lost wiggles a little
        it.my = ((w.t / 25) % 3) as f32 - 1.0;
    }
    it
}

// ------------------------------------------------------------ field

/// How good a spot is to receive a cross: close to the goal, outside the
/// keeper's area, in front of it.
fn spot_score(x: f32, y: f32) -> f32 {
    let dx = (x - 92.0).abs();
    let dy = (y - goal_mid()).abs();
    let mut s = 100.0 - dx * 0.8 - dy * 1.2;
    if in_area(x, y) {
        s -= 60.0;
    }
    if x > 170.0 {
        s -= 30.0;
    }
    s
}

/// The kind a smart kid wants against a keeper with `pts` points.
fn choose_kind(w: &World, k: usize, b: &mut Brain) -> Kind {
    let me = &w.kids[k];
    let pts = w.kids[w.keeper].pts;
    if w.bot_kind(k) == Some(BotKind::Greedy) {
        return Kind::Bicycle;
    }
    let acro = kid(me.id).acrobat;
    let smart = b.rng.f() < 0.3 + 0.7 * me.skill;
    if smart && pts <= 5 {
        return match pts {
            1 => Kind::Volley,
            2 => {
                if kop(me.id) >= schot(me.id) || b.rng.f() < 0.6 {
                    Kind::Header
                } else {
                    Kind::Heel
                }
            }
            5 if acro || b.rng.f() < 0.4 * me.skill => Kind::Bicycle,
            _ => {
                if kop(me.id) >= 3 && b.rng.f() < 0.6 {
                    Kind::Header
                } else {
                    Kind::Volley
                }
            }
        };
    }
    let r = b.rng.f();
    let bike = if acro { 0.3 } else { 0.06 + 0.06 * me.skill };
    if r < bike && pts > 5 {
        Kind::Bicycle
    } else if r < bike + 0.35 + 0.05 * kop(me.id) as f32 {
        Kind::Header
    } else if r < 0.85 {
        Kind::Volley
    } else {
        Kind::Heel
    }
}

fn contact_height(kind: Kind) -> f32 {
    match kind {
        Kind::Volley | Kind::Heel => 6.0,
        Kind::Chest => 16.0,
        Kind::Header => 25.0,
        Kind::Bicycle => 38.0,
    }
}

fn field(w: &World, k: usize, b: &mut Brain) -> Intent {
    let me = &w.kids[k];
    let ball = &w.ball;
    let mut it = Intent { face: -1, ..Default::default() };
    let skill = me.skill;
    if b.seen != w.touches {
        b.seen = w.touches;
        b.want = None;
        b.jumped = false;
        b.pressed = false;
        b.react = 0;
        b.err = (b.gauss() * (1.0 - skill) * 3.0).round() as i32;
        b.hold = 15 + b.rng.int(40) as u32 + ((1.0 - skill) * 30.0) as u32;
        b.wob = (b.rng.range(-8.0, 8.0), b.rng.range(-6.0, 6.0));
    }
    b.react += 1;
    let mates: Vec<usize> = w.field().filter(|&m| m != k).collect();

    // 1. the ball at my feet: get to a good place, then cross
    if ball.owner == Some(k) {
        let tx = me.x.clamp(110.0, 160.0);
        let ty = me.y.clamp(14.0, 58.0);
        let (mx, my) = go(me, tx, ty);
        it.mx = mx;
        it.my = my;
        it.face = 0;
        if b.react > b.hold || me.holding > 200 {
            let best = mates.iter().copied().max_by(|&a, &c| {
                let s = |m: usize| spot_score(w.kids[m].x, w.kids[m].y) + if w.kids[m].human().is_some() { 10.0 } else { 0.0 };
                s(a).partial_cmp(&s(c)).unwrap_or(std::cmp::Ordering::Equal)
            });
            it.to = best;
            it.c = true;
        }
        return it;
    }

    // 2. a cross is coming to me
    if ball.cross_to == Some(k) && ball.air && ball.loose() {
        let land = ball.land.unwrap_or((ball.x, ball.y));
        let far = land.0 > 165.0;
        if b.want.is_none() {
            b.want = Some(if far { Kind::Volley } else { choose_kind(w, k, b) });
        }
        let mut kind = b.want.unwrap_or(Kind::Volley);
        // too low for a bicycle kick: head it
        if kind == Kind::Bicycle && ball.z + ball.vz * ball.vz / (2.0 * G) < 42.0 && ball.vz <= 0.5 {
            kind = Kind::Header;
            b.want = Some(kind);
        }
        let face = if matches!(kind, Kind::Heel | Kind::Bicycle) { 1 } else { -1 };
        let mut hz = contact_height(kind);
        if ball.when_at(hz).is_none() && kind != Kind::Volley {
            // it is already lower than that: take it on the volley
            kind = if face == 1 { Kind::Heel } else { Kind::Volley };
            b.want = Some(kind);
            hz = contact_height(kind);
        }
        let face = if matches!(kind, Kind::Heel | Kind::Bicycle) { 1 } else { -1 };
        let (cx, cy, t) = ball.when_at(hz).unwrap_or((ball.x, ball.y, 0.0));
        // stand so the ball meets the foot or head
        let (mx, my) = go(me, cx - face as f32 * 3.0, cy);
        it.mx = mx;
        it.my = my;
        it.face = face;
        let aim_far = w.kids[w.keeper].y > goal_mid();
        it.aim = if aim_far { -0.8 } else { 0.8 } * (0.4 + 0.6 * skill);
        if far {
            // too far to shoot: take it down
            it.face = -1;
            return it;
        }
        let lead = (3 + b.err).max(1);
        if kind == Kind::Bicycle {
            if !b.jumped && t <= 10.0 + b.err as f32 {
                it.b = true;
                b.jumped = true;
            }
            if b.jumped && me.air() && !b.pressed && t <= lead as f32 + 1.0 {
                it.a = true;
                b.pressed = true;
            }
        } else if !b.pressed && t <= lead as f32 {
            it.a = true;
            b.pressed = true;
        }
        return it;
    }

    // 3. a rebound in the air near me: shoot it first time
    if ball.air && ball.loose() && ball.from == Some(w.keeper) && ball.cross_to.is_none() && ball.shot.is_some() {
        if let Some((cx, cy, t)) = ball.when_at(6.0) {
            let d = ((cx - me.x).powi(2) + (cy - me.y).powi(2)).sqrt();
            let closest = w.field().all(|m| {
                let o = &w.kids[m];
                m == k || ((cx - o.x).powi(2) + (cy - o.y).powi(2)).sqrt() >= d
            });
            if closest && d < me.speed() * t + 6.0 && cx > AREA_X - 4.0 && cx < 150.0 {
                let (mx, my) = go(me, cx + 3.0, cy);
                it.mx = mx;
                it.my = my;
                if t <= 3.0 + b.err.max(0) as f32 && skill > 0.25 {
                    it.a = true;
                }
                return it;
            }
        }
    }

    // 4. a loose ball: the closest kid fetches it, unless the keeper will
    let (rx, ry) = if ball.air {
        ball.when_at(0.0).map(|(x, y, _)| (x, y)).unwrap_or((ball.x, ball.y))
    } else {
        (ball.x + ball.vx * 25.0, ball.y + ball.vy * 25.0)
    };
    let shot_flying = ball.shot.is_some() && ball.air;
    let fetch_ok = ball.loose() && !shot_flying && (ball.cross_to.is_none() || !ball.air) && !in_area(rx, ry) && w.dog.as_ref().map(|d| !d.has).unwrap_or(true);
    if fetch_ok {
        let dist = |m: usize| {
            let o = &w.kids[m];
            ((o.x - rx).powi(2) + ((o.y - ry) * 1.3).powi(2)).sqrt()
        };
        let mut best = k;
        if let Some(m) = ball.cross_to.filter(|&m| m != w.keeper) {
            // a pass along the ground: it is that kid's ball
            best = m;
        }
        for m in w.field() {
            if ball.cross_to.is_some() {
                break;
            }
            // a human closer by gets the ball, unless they leave it lying
            let counts = w.kids[m].human().is_none() || w.stuck_for() < 120;
            if dist(m) < dist(best) && counts {
                best = m;
            }
        }
        if best == k {
            let (mx, my) = go(me, rx.clamp(GOAL_X + 6.0, 248.0), ry.clamp(-2.0, DEPTH + 2.0));
            it.mx = mx;
            it.my = my;
            it.face = 0;
            return it;
        }
    }
    // the poodle has it: chase
    if let Some(d) = &w.dog {
        if d.has {
            let closest = w.field().min_by(|&a, &c| {
                let da = (w.kids[a].x - d.x).abs() + (w.kids[a].y - d.y).abs();
                let dc = (w.kids[c].x - d.x).abs() + (w.kids[c].y - d.y).abs();
                da.partial_cmp(&dc).unwrap_or(std::cmp::Ordering::Equal)
            });
            if closest == Some(k) {
                let (mx, my) = go(me, d.x, d.y);
                it.mx = mx;
                it.my = my;
                it.face = 0;
                return it;
            }
        }
    }

    // 5. otherwise: get open in front of the goal, spread out
    let order: Vec<usize> = w.field().filter(|&m| ball.owner != Some(m)).collect();
    let slot = order.iter().position(|&m| m == k).unwrap_or(0);
    let spots = [(84.0, 22.0), (92.0, 52.0), (124.0, 37.0)];
    let (sx, sy) = spots[slot % 3];
    let (mx, my) = go(me, sx + b.wob.0, sy + b.wob.1);
    it.mx = mx * 0.8;
    it.my = my * 0.8;
    it
}

// ------------------------------------------------------------ keeper

fn keeper(w: &World, k: usize, b: &mut Brain) -> Intent {
    let me = &w.kids[k];
    let ball = &w.ball;
    let lazy = w.bot_kind(k) == Some(BotKind::Lazy);
    let skill = if lazy { 0.15 } else { me.skill };
    let mut it = Intent::default();
    if b.seen != w.touches {
        b.seen = w.touches;
        b.react = 0;
        b.pressed = false;
        b.jumped = false;
        b.delay = (20.0 - 14.0 * skill + b.rng.range(0.0, 6.0)) as u32 + if lazy { 18 } else { 0 };
        b.err = (b.gauss() * (1.0 - skill) * 3.0).round() as i32;
        b.hold = 40 + b.rng.int(40) as u32 + if lazy { 60 } else { 0 };
    }
    b.react += 1;
    let mid = goal_mid();
    if ball.held {
        // roll it out after a moment
        let (mx, my) = go(me, GOAL_X + 10.0, mid);
        it.mx = mx;
        it.my = my;
        if me.at > b.hold {
            it.a = true;
            it.my = b.rng.range(-1.0, 1.0);
        }
        return it;
    }
    // base position: on the line between ball and goal
    let bx = ball.x.max(GOAL_X + 10.0);
    let near = ((bx - GOAL_X) / 120.0).clamp(0.0, 1.0);
    let mut tx = GOAL_X + 6.0 + (1.0 - near) * 6.0;
    let mut ty = mid + (ball.y - mid) * (0.55 - 0.25 * near);
    // a slow ball in the area: pick it up
    if ball.loose() && ball.z < 3.0 && ball.speed() < 1.4 && in_area(ball.x, ball.y) {
        let (mx, my) = go(me, ball.x - 3.0, ball.y);
        it.mx = mx;
        it.my = my;
        if (me.x + 3.0 - ball.x).abs() < 7.0 && (me.y - ball.y).abs() < 5.0 && me.act == Act::Idle && b.react > b.delay / 2 {
            it.a = true;
        }
        return it;
    }
    // a cross that comes down in the area: claim it
    if ball.loose() && ball.air && ball.cross_to.is_some() && ball.shot.is_none() {
        if let Some((cx, cy, t)) = ball.when_at(18.0) {
            let reach = ((cx - me.x).powi(2) + (cy - me.y).powi(2)).sqrt() < 1.4 * t + 6.0;
            if in_area(cx, cy) && reach && skill + 0.2 > (b.err.abs() as f32) * 0.1 && b.react > b.delay {
                let (mx, my) = go(me, cx - 3.0, cy);
                it.mx = mx;
                it.my = my;
                if t <= 4.0 && !b.pressed {
                    it.a = true;
                    b.pressed = true;
                }
                return it;
            }
        }
    }
    // a shot on its way
    if let Some(_s) = ball.shot {
        if ball.loose() && ball.vx < -0.4 && !ball.in_net {
            let plane = me.x + 3.0;
            let t = (plane - ball.x) / ball.vx;
            if t > 0.0 {
                let yc = ball.y + ball.vy * t;
                let zc = (ball.z + ball.vz * t - 0.5 * G * t * t).max(2.0);
                let on_target = yc > G_Y0 - 5.0 && yc < G_Y1 + 5.0 && zc < BAR + 8.0;
                if b.react > b.delay && on_target {
                    let ey = b.err as f32 * 1.5;
                    tx = GOAL_X + 6.0;
                    ty = yc + ey;
                    let gap = yc - me.y;
                    let walk = 1.3 * t;
                    if gap.abs() > walk + 4.0 && t < 12.0 && !lazy && !b.pressed {
                        it.a = true;
                        it.my = gap.signum();
                        b.pressed = true;
                    } else if zc > 27.0 && !b.jumped && t < 14.0 {
                        it.b = true;
                        b.jumped = true;
                    }
                    let when = 4 + b.err.max(-2) + if lazy { -3 } else { 0 };
                    if !b.pressed && t <= when.max(1) as f32 {
                        if zc > 26.0 || (ball.speed() > 4.6 && b.rng.f() < 0.3) {
                            it.c = true;
                        } else {
                            it.a = true;
                        }
                        b.pressed = true;
                    }
                }
            }
        }
    }
    let (mx, my) = go(me, tx, ty);
    it.mx = mx;
    it.my = my;
    it
}
