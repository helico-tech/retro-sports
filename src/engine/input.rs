//! The pad every game reads: four directions, three buttons and start.
//! On the keyboard: arrows + Z X C + Enter (P1), WASD + F G H + Tab (P2).

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Pad {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    /// main action (dodgeball: throw / catch)
    pub a: bool,
    /// jump
    pub b: bool,
    /// second action (dodgeball: pass / duck)
    pub c: bool,
    pub start: bool,
}

/// Buttons that went down this tick.
pub fn edge(now: Pad, prev: Pad) -> Pad {
    Pad {
        up: now.up && !prev.up,
        down: now.down && !prev.down,
        left: now.left && !prev.left,
        right: now.right && !prev.right,
        a: now.a && !prev.a,
        b: now.b && !prev.b,
        c: now.c && !prev.c,
        start: now.start && !prev.start,
    }
}

pub fn any(p: Pad) -> bool {
    p.a || p.b || p.c || p.start || p.up || p.down || p.left || p.right
}
