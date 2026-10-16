---
status: in-progress
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

## Follow-ups
