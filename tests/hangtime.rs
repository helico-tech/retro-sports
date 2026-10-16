use retro_sports::games::hangtime::{ai::Style, course::Venue, headless};

#[test]
#[ignore]
fn trace_print() {
    let v = match std::env::var("V").as_deref() { Ok("1") => Venue::Alps, Ok("2") => Venue::Canyon, _ => Venue::Dunes };
    let sk: f32 = std::env::var("SK").ok().and_then(|s| s.parse().ok()).unwrap_or(0.5);
    headless::trace(v, sk, Style::Normal, 3);
}
