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
Module `src/games/tegenwind/`, split like dodgeball:
- `data.rs` — bikes (4), characters (KEES, ANOUK, OMA RIEK, DENNIS,
  FAMILIE DE VRIES), stages (DE POLDER, AMSTERDAM, AFSLUITDIJK) with wind
  sections, obstacle mixes and cargo.
- `race.rs` — pure simulation at 60 Hz: riders on a two-lane bike path
  (x along the route, y = lane position 0 far .. 1 near, z = hop height).
  Pedal strokes from alternating LEFT/RIGHT with a cadence limit
  (too fast = wasted stroke, breaks the rhythm, costs energy and a bit
  of speed; a clean stroke right at the limit = PERFECT, +20%). Air drag
  against `speed + wind`, less when tucked (hold C) or drafting (same
  lane, close behind). Side wind drifts you toward the canal. Hop (B),
  bell (A), energy with the man with the hammer, food pickups, cargo,
  crashes into the canal, drawbridge, sheep crossings, tourists, geese,
  slow riders to draft (tractor, the lady with the sofa), sweeper van
  for idle humans. CPU AI with personalities.
- `art.rs` — string bitmaps with color roles and auto outline (bikes,
  wheels, cargo, animals, props, windmill), cycling skeletons for
  `person()` with 8 crank frames per bike posture, built once in `new()`.
- `scenery.rs` — per stage: cached sky, wrapped parallax strips (far,
  mid, near, canal), animated windmills, animals, water, rain, spray.
- `render.rs` — race view (world, riders, fx, popups), HUD, banners,
  split screen (each view drawn into a scratch frame, `copy_band`).
- `app.rs` — title + menu + ticker, attract demo, bike select, stage
  intro card with tips, race + pause, results tally + ranking, continue,
  game over, champion. 1P ladder (top 2 to go on, win the last), 2P
  three-stage tour with points.
- `audio.rs` — 16 sfx and 4 songs (street organ title, polka, storm,
  fanfare).
- `headless.rs` — numbered shots of every screen and key moments, sprite
  sheets, `sim` (CPU races) and `probe` (human bots).
- `tests/tegenwind.rs` — races finish, physics sanity, scenes draw.

Order: riding loop first, then screens, art and sound, then CPU and
balance, then the art and gameplay reviews.

## Progress log

## Follow-ups
