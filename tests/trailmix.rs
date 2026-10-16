use retro_sports::games::trailmix::headless;

#[test]
#[ignore]
fn sim_dev() {
    headless::sim(40);
}

#[test]
#[ignore]
fn probe_dev() {
    headless::probe(10);
}

#[test]
#[ignore]
fn shots_dev() {
    headless::shots("/private/tmp/claude-501/-Users-avanwieringen-Development-hobby-dodgeball/60845c2d-6e8f-475a-8dbc-b00a389186a6/scratchpad/shots/trailmix");
}
