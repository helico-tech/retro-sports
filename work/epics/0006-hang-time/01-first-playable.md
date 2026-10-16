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
Module `src/games/hangtime/`, split like dodgeball:
- `pilots.rs` — the four pilots (CAPTAIN KITE, DUNE DIRK, SKY GRANNY,
  THE EAGLE): stats (glide, climb, handling, nerve), bios, palettes, wing
  colors and sail pattern.
- `course.rs` — the three venues (HELICO DUNES, ALPINE VALLEY, THUNDER
  CANYON): height map per world pixel, surfaces (sand, grass, rock, water,
  field), thermals, ridge lift (wind on rising slopes), sink zones, gates
  under balloons, launch ramp, landing field with target rings, props,
  storm cloud. `lift(x, y)` is the one lift function everybody uses.
- `flight.rs` — `Race`: pilots driven only by pads (CPU pilots get pads
  from the AI too, so the human and CPU follow the same rules). States:
  wait in line, run (alternate left/right), hop / launch / tumble off,
  fly (bar up/down, turn, tuck), stall, flare, cloudsuck, landed (soft,
  hard, belly, crash, splash, sheep), walk of shame / paddle, done.
  Wildlife and whimsy (storks in thermals, geese V with lift, balloon
  family, sheep, UFO with tractor beam). Scoring: gates, time bonus
  (scaled by gates made, so diving straight in does not pay), landing
  rings, penalties. Banners, popups, sfx. Sunset rule so every flight ends.
- `ai.rs` — CPU brain that outputs a `Pad`: plans the height needed for
  the next gate, finds lift, weaves in thermals, speed-to-fly, flares.
  Skill and style parameters double as the probe bots.
- `art.rs` — sprites built once in `new()`: glider wings from a small 3D
  model (yaw x pitch frames, sail pattern per pilot, palette roles,
  outline), pilots via `person()` with own skeletons (prone, run, flare,
  stand, walk, paddle, tumble, cheer) and own heads (helmet, eagle),
  string bitmaps for storks, geese, gulls, sheep, cows, UFO, balloons.
- `scenery.rs` — per-venue pre-rendered far and mid parallax layers,
  banded sky by altitude, terrain columns drawn from the height map,
  props, animated life.
- `render.rs` — one view of a race (camera, layers, gliders, fx, HUD with
  altimeter and variometer, minimap, banners); split screen through a
  scratch frame and `Frame::copy_band`.
- `app.rs` — scenes: title (+menu, attract demo after 15 s), select,
  intro card, play (+pause), results tally, continue, game over,
  champion. 1P ladder of 3 stages vs 3 CPU pilots (must place 1st);
  2P: both fly the 3 stages split screen, totals decide.
- `audio.rs` — title, flight and win songs; 16 sfx incl. vario beeps.
- `headless.rs` — `shots`, `sim` (CPU pilots), `probe` (dive, cautious,
  decent bots through the ladder).
- `tests/hangtime.rs` — flights end, thermals lift, stalls lose height,
  launch works, determinism, every scene draws, sounds render.

Order: course + flight + AI + sim first, then screens, then art/sound,
then reviews and balance.

## Progress log
- 2026-10-16: Built course (3 venues), flight model (polar, energy/zoom,
  stall, turns, flare), race with launch run, gates, landings, sheep,
  geese, UFO, storm; CPU brain that outputs pads; art (3D-rendered wing
  frames, `person()` pilots incl. an eagle, critters); scenery (parallax
  layers with ground planes, banded sky, terrain columns); all screens;
  audio (16 sfx, 3 songs); headless shots/sim/probe; tests.
- 2026-10-16: Rebased `feat/hangtime` on `feat/retro-sports` (new island
  hub; `Tool` alias in `cart.rs`). `tests/carts.rs` console test fails on
  this branch only because the hub walk between the two registered carts
  is longer than the test waits; logged as issue 0040 (passes once all
  five carts are registered). No engine changes.
- 2026-10-16: CPU tuning: burn mode when too high for a hoop, storm
  immunity after a spit-out (pilots looped in the storm), energy-aware
  final glide with S-turns, skill habits (over-climbing, dawdling,
  over-pulling, late stall reaction), rookie skills below 0 for stage 1.
- 2026-10-16: Perf (native release): new() 2 ms, stage setup 1.3 ms,
  frame 0.09 ms (1P) / 0.18 ms (2P split). wasm build ok.

## Follow-ups
