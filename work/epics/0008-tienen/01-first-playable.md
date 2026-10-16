---
status: in-progress
created: 2026-10-16
---
# 01: First playable

## Goal
The whole game per spec: the rules of tienen, all screens, art, sound,
CPU, modes, tools, island venue, reviewed and tuned.

## Acceptance criteria
- See the epic's "Done when".

## Plan
Module `src/games/tienen/`, split like Stalen Zenuwen:
- `data.rs` — kids (KEVIN, SAMIRA, GIJS, LIEKE and the venue kids) with
  SCHOT, KOP, KEEP, SNEL, venues (GALERIJFLAT, SCHOOLPLEIN, DE KOOI) with
  their kids, skill and events, shot kinds and values, ticker, heckles.
- `world.rs` — pure simulation at 60 Hz: the pitch (x along, y depth, z
  height, goal on the left), the ball (gravity, bounces, posts, bar, net,
  wall), kids (run, jump, kick, header, heel, bicycle kick, keeper dive,
  catch, punch, roll out), the rules (air goals by kind, keeper changes,
  exact zero, overshoot, 1-point keeper), latjetrap and aftellen, events
  (car, poodle, pigeons, the fence), the billenbrand, the result.
- `bot.rs` — CPU field kids and keeper (also the attract demo and `sim`)
  and probe bots (decent, lazy, greedy).
- `art.rs` — `Bmp` copied from Zenuwen (games do not depend on each
  other): ball, goal, car, poodle, pigeons, props; `person()` skeletons
  for kicking, heading, heel, bicycle kick, diving, catching, bending
  over.
- `scenery.rs` — per venue: sky, backdrop, the pitch with chalk, the
  goal, the kerb.
- `render.rs` — the pitch view (y-sorted), ball and shadow, landing
  marker, HUD (points per kid, keeper), banners, popups.
- `app.rs` — title + menu + ticker, attract demo, select, intro card,
  latjetrap, play + pause, billenbrand, results tally, continue, game
  over, champion; 1P ladder and 2P.
- `audio.rs` — 14-16 sfx and 4 songs.
- `headless.rs`, `probe.rs` — shots of every screen, `sim` and `probe`.
- `tests/tienen.rs` — rules, determinism, games end, scenes draw.
- Island: a trapveldje venue.

Order: simulation and bots first (balance headless), then art, screens
and sound, then the island, then reviews and tuning.

## Progress log
- 2026-10-16: spec, epic and story written; branch `feat/tienen`.

## Follow-ups
