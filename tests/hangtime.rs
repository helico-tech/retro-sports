use retro_sports::games::hangtime::{ai::Style, course::Venue, headless};

#[test]
#[ignore]
fn trace_print() {
    let v = match std::env::var("V").as_deref() { Ok("1") => Venue::Alps, Ok("2") => Venue::Canyon, _ => Venue::Dunes };
    let sk: f32 = std::env::var("SK").ok().and_then(|s| s.parse().ok()).unwrap_or(0.5);
    let seed: u64 = std::env::var("SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(3);
    let st = if std::env::var("CAUT").is_ok() { Style::Cautious } else { Style::Normal };
    headless::trace(v, sk, st, seed);
}

#[test]
#[ignore]
fn losses_print() {
    use retro_sports::games::hangtime::{app::LADDER, course::VENUES, flight::*};
    let k: usize = std::env::var("ST").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let sk: f32 = std::env::var("SK").ok().and_then(|s| s.parse().ok()).unwrap_or(0.6);
    for me in 0..4 {
        for s in 0..16u64 {
            let seed = s * 7 + 3 + me as u64 * 101 + k as u64 * 1009;
            let mut entries: Vec<Entry> = vec![(me, Ctrl::Cpu, sk, Style::Normal)];
            for j in 0..3 {
                entries.push(((me + 1 + j) % 4, Ctrl::Cpu, LADDER[k][j], Style::Normal));
            }
            let r = headless::fly(VENUES[k], &entries, seed);
            if r.ranking()[0] != 0 {
                let line: Vec<String> = r.pilots.iter().map(|p| format!("[{} g{} t{} l{} b{} p{} T{} {:?}]", p.def, p.gates_made(), p.time_bonus, p.landing_pts, p.bonus, p.penalty, p.total(), p.landing)).collect();
                println!("me{} s{}: {}", me, s, line.join(" "));
            }
        }
    }
}

#[test]
#[ignore]
fn race_trace() {
    use retro_sports::games::hangtime::{app::LADDER, course::VENUES, flight::*};
    use retro_sports::engine::input::Pad;
    let k: usize = std::env::var("ST").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let me: usize = std::env::var("ME").ok().and_then(|s| s.parse().ok()).unwrap_or(3);
    let s: u64 = std::env::var("S").ok().and_then(|s| s.parse().ok()).unwrap_or(6);
    let seed = s * 7 + 3 + me as u64 * 101 + k as u64 * 1009;
    let mut entries: Vec<Entry> = vec![(me, Ctrl::Cpu, 0.6, Style::Normal)];
    for j in 0..3 {
        entries.push(((me + 1 + j) % 4, Ctrl::Cpu, LADDER[k][j], Style::Normal));
    }
    let mut r = Race::new(VENUES[k], &entries, seed);
    let mut n = 0;
    let mut g = 0;
    while !r.finished() && n < 60 * 60 * 8 {
        r.update(&[Pad::default(); 2]);
        n += 1;
        let p = &r.pilots[0];
        if n % 60 == 0 || p.gates != g {
            g = p.gates;
            println!("t{:>4} x{:>5.0} y{:>4.0} agl{:>4.0} v{:.2} vy{:+.2} face{:+} st {:?} gates {:b} air {:+.2}", n / 60, p.x, p.y, p.agl(&r.course), p.v, p.vario, p.face, p.state, p.gates, r.air(p.def, p.x, p.y));
        }
    }
}
