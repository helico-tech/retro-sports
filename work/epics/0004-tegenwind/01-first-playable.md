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
- 2026-10-16: race core first (`race.rs`): strokes from alternating LEFT/RIGHT
  with a cadence limit per bike (9-10 frames, lower while starting, +7 when
  bonked). Too early = wasted (no push, -1% speed, energy, breaks the rhythm);
  a clean stroke right at the limit = PERFECT (+20%). Air drag on
  `speed + headwind`; tuck (hold C) and drafting cut it; side wind drifts
  you toward the canal (hold UP to lean in). Decision: the spec's "cadence
  limit for your speed" became a limit per bike (plus low gear at the start
  and the bonk); a speed-dependent beat made the rhythm unreadable.
- 2026-10-16: added a brake (both pedal keys held for 10+ frames) so riders
  can stop for a bollard or the drawbridge; the CPU uses it too.
- 2026-10-16: first sim/probe runs: races were 140-170 s, the racefiets won
  everything (a faster cadence is free power), Dennis won 92% in Amsterdam,
  mashers never finished. Tuned stroke power, bikes (racefiets power 0.94 for
  its 9-frame cadence), personalities, bonk recovery, the sweeper van and
  the bridge (a sprint can beat the barrier).
- 2026-10-16: art, scenery (cached sky, wrapped parallax strips, road strips
  with cobble columns), render, HUD, app screens, audio, shots. Looked at
  every screen; fixed HUD overlaps, select bios, intro layout, swimmer
  clipping, the sofa lady looking like Oma Riek, banners over the road.
- 2026-10-16: the contract test hung on the old list hub (it presses RIGHT,
  the old hub only moved on UP/DOWN). Rebased `feat/tegenwind` onto
  `ea2aba7` (island hub), like the other game branches.
- 2026-10-16: gameplay pass 1: look-ahead warning signs at the screen
  edge (none on the ligfiets: it can't see over hedges), camera 14 px
  further back, stroke energy raised (perfect 0.24, good 0.16) so food
  matters on the dike, rain showers with umbrella pickups (wet without one;
  an open umbrella is a sail: worse into the wind, a push with it), cheese
  stand and herring cart with snacks, rare sights (stork, party boat, kite
  surfer), GEZELLIG! when you sit on a wheel, gold crank glow in the
  PERFECT window. Racefiets: normal cadence, suffers on cobbles.

### Art review 1 (2026-10-16), against dodgeball's shots
- MUST A1: the umbrella reads as a hat (13 px). Make it a big held umbrella.
- MUST A2: the rain shower darkens a hard-edged rectangle of sky. Draw a
  rain cloud instead.
- MUST A3: drafting is hard to see (three tiny dashes). Draw a clear
  slipstream: air lines bending round the front rider.
- SHOULD A4: the man with the hammer is a 9 px mallet. Make it a big
  cartoon hammer that bonks the head, with stars.
- SHOULD A5: the raised drawbridge leaves look like bundles of sticks.
  Planked leaves with a rail.
- SHOULD A6: sea spray on the dike is soft grey blobs; crisper white spray.
- SHOULD A7: the party boat crowd looks like candles; real little people.
- SHOULD A8: the shots script stacks crossing sheep in a column (the race
  spawn spreads them); make the shot match the game.
- COULD A9: the crashed bike lying on the road is crude (two ellipses).
- COULD A10: the cow cloud overlaps the logo on the title.
- OK: riders, bikes and wheels read at 1x with outlines; pedalling frames
  are smooth (sheet 20/52); palettes sit well together (Dutch orange HUD
  line, red bike path, green polder); every scene has life (mills, cows,
  geese V, tram, gulls, rain, whitecaps); jokes on every screen.

### Gameplay review 1 (2026-10-16), with sim/probe numbers
- MUST G1 (fixed): mashers never finished (bonked forever). Wasted strokes
  now cost less energy; a masher still loses badly (polder 198 s vs 116 s
  for the decent bot, 68% of presses wasted, 4.6 bonks).
- MUST G2 (fixed): obstacles showed up about 1 s ahead at speed. Warning
  signs and the camera change give about 2 s.
- MUST G3 (fixed): energy never ran out for a good rider; now a
  perfect-rhythm rider bonks on the dike unless they eat or draft.
- SHOULD G4 (fixed): the racefiets won 80% on the dike (its faster cadence
  was free power); now every bike wins 19-44% there with the decent bot.
- SHOULD G5: rhythm feedback is HUD-only plus a tick. Added the crank glow
  on the rider; keep the tick for every good stroke (16 sfx is the cap).
- SHOULD G6 (fixed): Dennis was hopeless everywhere after the cobble
  change; now he wins Amsterdam (38%), Oma the storm (38%), De Vries the
  drafting polder (39%) in CPU races; field spread 7-16 s on average.
- OK: tuck matters in the storm (no-tuck bot 10% wins vs 28%), drafting
  helps placing (drafter bot 70% top 2 on the dike), rhythm beats mashing,
  no exploit found (bell spam is free but harmless, hop spam costs energy
  and pedalling, the umbrella-sail in a tailwind is a fair trick).
- COULD G7: a photo finish when riders cross within a few frames.
- 2026-10-16: fixed A1-A8 (big held umbrella, rain cloud, slipstream lines,
  bonking hammer with stars, planked bridge leaves, crisp spray, real people
  on the party boat, shot script sheep), plus a photo finish banner, geese
  that chase you when you ring at them, tuba and grandfather-clock cargo
  after a continue, no label clutter around a bonked rider.
- 2026-10-16: merged `feat/retro-sports` (island hub, Disc Dogs, Trail Mix)
  as asked by the coordinator; CARTS = dodgeball, discdogs, tegenwind,
  trailmix. All tests pass in debug and release.
- 2026-10-16: clippy clean in the module; spec has "As built" and
  "Balance results"; issue 0020 for the contract test that hangs instead
  of failing.

### Art review 2 (2026-10-16)
- No MUST left. Checked every shot again (title, menu, select, intro
  cards, all three stages, crash, swim, bonk, bridge, showers, stands,
  rare sights, results, continue, game over, champion 1P/2P, demo, split
  screen) and the rider/prop sheets.
- COULD (open): the crashed bike lying on the road is still simple; the
  cow cloud tucks behind the logo on the title.

### Gameplay review 2 (2026-10-16)
- No MUST left. Numbers in the spec's "Balance results": decent bot 96%
  top 2 on the polder, 26% wins on the dike, every bike 15-35% on the dike,
  tuck and draft both pay, mashing loses, no exploit found.
- COULD (open): mashers and very slow players get swept up on the dike
  (they can't reach it in 1P anyway); a tiny on-rider beat light for
  players who never look at the HUD.

## Follow-ups
- Issue 0020: contract test hangs instead of failing when the hub cursor
  never moves.
- COULD items above (crashed-bike sprite, title cow cloud).
