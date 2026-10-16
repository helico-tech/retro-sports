---
status: done
created: 2026-10-16
---
# 01: First playable

## Goal
The whole game per spec: core mechanics, all screens, art, sound, CPU,
modes, tools, reviewed and tuned.

## Acceptance criteria
- See the epic's "Done when".

## Plan
Module `src/games/discdogs/`, split like dodgeball:
- `data.rs`: the four owner+dog pairs (stats, bios, heckles), the three
  stages, ticker jokes. Every text checked against the 5x7 font in a test.
- `sim.rs`: one match (`Match`): 5 tosses per side, alternating. A toss
  is Ready -> Aim -> Power -> Release -> Flight -> Result. Disc physics
  (gravity, lift ~ v^2, drag, wind, floater), dog physics (run, zoomies,
  leap, flip, tumble, swim, stare), hazards (squirrel, seagull, picnic
  blankets with sausages, kite string, pond, sea, gusts), scoring
  (yards x 1/2/3 + perfect/over-the-shoulder/sausage bonuses), tie-break
  ("SUDDEN DOG"). Pure logic, no drawing. 1 yard = 4 px; field 100 yd.
- `ai.rs`: one `Brain` that turns match state into pad presses, used for
  the CPU and for the probe bots (safe / greedy / decent), so CPU and bots
  play through the same input path as a human.
- `sprites.rs`: dogs from a small dog skeleton + hand-drawn head strings
  per breed (normal / open mouth / sad), color roles, 1px outline,
  palette per breed and collar color per side; flip frames are lossless
  90-degree rotations of a curl sprite. Props as string bitmaps (disc,
  seagull, squirrel, sausage, kite kid, sleeper, windsock, trophy).
  Owners via `engine::sprites::person()` with own skeletons (backhand
  wind-up, release, whistle, face-palm, cheer).
- `scenery.rs`: per stage a static sky frame plus pre-rendered parallax
  strips (far 0.3, mid 0.6, ground 1.0) copied each frame, plus an
  animated layer (clouds, water, gulls, kite, crowd, lights, Z's).
- `render.rs`: world, HUD, meters, banners, popups; `app.rs`: scenes
  (title, menu, attract, select, intro card, play, pause, results tally,
  continue, game over, champion); `audio.rs`: 16 sfx, 4 songs;
  `headless.rs`: `shots`, `sim`, `probe`.
- Order: toss loop + sim numbers first, then screens, art, sound, CPU and
  balance, then art and gameplay reviews.

## Progress log
- 2026-10-16: toss loop first. Disc physics tuned by printing flights
  (20-70 yards, 1.0-2.6 s, floater hangs ~30% longer and drifts 2-3x more
  with the wind). Dogs had to be faster than first planned (1.95-2.75
  px/frame) or the CPU could only reach 20-yard tosses. CPU aim bugs
  found with frame traces: re-planning slid the leap forever (fixed:
  commit to a leap 10 frames out), dog faced the wrong way on the leap
  (fixed: plan the facing), dogs swam through the pond (fixed: hop it).
- 2026-10-16: all screens, art, sound, headless tools in. Fast-forwarded
  `feat/discdogs` to `feat/retro-sports` (ea2aba7): the old stub hub only
  moved with up/down, so `tests/carts.rs` (which presses right) hung as
  soon as a second cart was registered. The island hub fixes that; no
  hub or test file edited by this story.
- 2026-10-16: balance pass with `--sim` and `--probe` (see the spec's
  Balance results). Found with the numbers: a tall dog caught the disc
  straight out of its owner's hand (Sir Woofington's 4-yard "catches";
  now no catch in the first 24 frames of flight); the pond turned the
  park into a swim meet (pond 30 px, swimming 1.8 px/f, more air control
  so a hop can brake); squirrel stares lost a third of the flight
  (shorter stares; tapping C now gives a burst of zoomies); always
  flipping was as good as playing safe (flip 44 frames, now clearly
  worse). Ladder skills 0.0 / 0.5 / 0.88; the easy CPU throws short and
  safe and fumbles, rather than missing everything.
- 2026-10-16: ART REVIEW (screens at 1x and 3x, sprite sheets 48/49):
  - MUST: flip frames read as a rotated standing dog -> built a curled
    ball that spins (done). Tumble drew the dog upright (vflip bug) ->
    on its back, legs kicking (done). HUD score overlapped tags and pips,
    windsock overlapped the wind number -> new HUD layout (done). Hedge
    sign text ran off the sign; pier yard numbers painted twice; bowl
    yard signs on top of the ad boards (all done).
  - SHOULD: gull and squirrel too small to read at 1x -> redrawn larger
    (done). Big empty park sky -> Helico blimp, birds, cow cloud (done).
    Afghan fringe drew stripes across the body (done). Continue / game
    over dog too small -> 2x (done). "ZOOMIES!" popup spammed (done).
    Bowl crowd is a flat dot pattern -> issue 0011 (not done).
  - COULD: disc wobble frames are crude; results ranking platforms plain;
    the sleeper is small (left as is).
- 2026-10-16: GAMEPLAY REVIEW (sim, probe, and playing the scripted toss):
  - MUST: free 4-yard catches (fixed); CPU leap re-plan slid forever and
    leapt facing the wrong way (fixed); park pond and squirrel made stage
    1 the hardest (fixed); greedy flipping not worse than safe (fixed);
    an idle human must not stall a match (auto throw after 10 s, test).
  - SHOULD: every action has feedback (bark on leap, whoosh on flip,
    chomp, tick on aim and meter, ding on PERFECT, popups for zoomies,
    sausage, gust, snag, squirrel) - checked. Readable timing: disc
    shadow, wobble when slow, off-screen height arrow - checked. CPU
    mid-skill per-dog spread is wide (Biscuit 74% vs Woofington 37% at
    skill 0.5) -> issue 0010. A on the ground does nothing (no flip
    hint) - left, the intro card explains it.
  - COULD: a landing marker; over-the-shoulder catches are rare (2-9%),
    which suits a bonus.
- 2026-10-16: done. `cargo test` and `cargo test --release` pass (whole
  crate), wasm build ok, no clippy warnings in `src/games/discdogs/`.
  Not checked: how it sounds and feels with real hands and speakers
  (only headless; samples checked for clipping).
- 2026-10-16: no engine changes. Clippy clean in the module, wasm build
  ok, update+draw ~60 us per frame natively, `App::new` ~0.5 ms.

## Follow-ups
- [Issue 0010](../../issues/0010-discdogs-cpu-dog-spread.md): CPU per-dog spread at mid skill.
- [Issue 0011](../../issues/0011-discdogs-bowl-crowd-art.md): richer Grand Bowl crowd.
