---
status: done
created: 2026-10-16
---
# 01: First playable

## Goal
The whole game per spec: stealth and stealing, all screens, art, sound,
CPU, modes, tools, island venue, reviewed and tuned.

## Acceptance criteria
- See the epic's "Done when".

## Plan
Module `src/games/zenuwen/`, split like Tegenwind:
- `data.rs` — thieves (JOOST, OPA HARRIE, SANNE, DE PROF), bike kinds and
  locks, BOA kinds, stages (PRINSENGRACHT, CENTRAAL STATION, KONINGSDAG)
  with orders, patrols and street layout, ticker and heckles.
- `world.rs` — pure simulation at 60 Hz: the street (x along, y depth),
  thieves (walk, run, pick, ride, hide, casual, swim, busted), parked
  bikes with locks and lock progress, BOA's (patrol, look back, coffee,
  investigate noise, chase, search, write a ticket) with view cones
  blocked by trees, the krul and the glass bin, suspicion, noise events,
  pedestrians and the orange crowd, Sjaak's boat and deliveries, herons,
  the crane boat, timers and the stage result. 2P: a human BOA.
- `bot.rs` — the CPU thief (attract mode, `sim`) and probe bots (decent,
  reckless, sneaky, lazy).
- `art.rs` — string bitmaps with color roles and outline (copied from
  Tegenwind's `Bmp`, games do not depend on each other), bikes (omafiets,
  racefiets, bakfiets, golden, wreck), locks, props, animals, boats;
  `person()` skeletons for walking, running, picking, sawing, riding,
  casual, selfie, swimming, busted, writing a ticket.
- `scenery.rs` — per stage: a pre-rendered strip of canal houses (or the
  station), sky, street bricks, quay, canal with reflections.
- `render.rs` — the street view (y-sorted), view cones, HUD, lock panel,
  banners, popups, split screen.
- `app.rs` — title + menu + ticker, attract demo, select, intro card,
  play + pause, results tally, continue, game over, champion; 1P ladder
  and 2P thief-vs-BOA rounds.
- `audio.rs` — 16 sfx and 4 songs (sneaky title, sneak, chase, fanfare).
- `headless.rs` — shots of every screen and key moments, sprite sheets,
  `sim` and `probe`.
- `tests/zenuwen.rs` — stages end, determinism, locks, catches, scenes
  draw.
- Island: a canal-town venue in the village, path through it.

Order: simulation and bots first (balance headless), then art, screens
and sound, then the island, then reviews and tuning.

## Progress log
- 2026-10-16: spec, epic and story written; branch `feat/zenuwen`.
- 2026-10-16: simulation (`world.rs`) and bots (`bot.rs`, `probe.rs`) first,
  tuned headless before any art. First runs: stage 1 cleared in 26 s, the
  decent bot was busted mostly by BOA's who were only checking a noise.
  Fixes: orders of 4/5/5 bikes; a CPU BOA only fines a rider whose bike is
  hot (a cut lock dangles from it) or known stolen; a fast rider squeezes
  past a surprised BOA (who then chases); a BOA who spots you whistles for
  half a second before running (head start); chase episodes counted once.
  Decision: 2P's BOA player may fine any rider (every bike the thief rides
  is stolen), CPU BOA's only hot or flagged ones.
- 2026-10-16: art (`art.rs`: Tegenwind's `Bmp` copied, bikes, krul, glass
  bin, herons, Bruno, poses), backdrops (`scenery.rs`: canal houses strip,
  Centraal Station, bunting, sky, bricks, quay, canal with reflections,
  houseboats), `render.rs` (y-sorted street, view cones as a stipple, boats,
  HUD with order and street map, lock panel), `app.rs` (all screens, 1P
  ladder, 2P thief vs BOA in split screen), `audio.rs` (16 sfx, 4 songs),
  `headless.rs` shots and sheet. Looked at every shot; fixed bios and tips
  running off their boxes, the lock panel overlap, the 2P strip overlap,
  the station's name too wide, the swimmer drawn too deep.
- 2026-10-16: island venue HELICODAM: a canal through the village (the
  fountain went), the path runs through town between the park and the
  polder; CARTS order follows the path. `tests/zenuwen.rs` (13 tests).
  `cargo test` passes. Debug build caught an overflow (negative screen x
  cast to u32 in the tree sway): fixed.

### Art review 1 (2026-10-16), against dodgeball and Tegenwind shots
- MUST A1: tree canopies and trunks draw over banners and popups. Draw
  banners, popups and prompts in a final pass.
- MUST A2: two speech bubbles on top of each other (garbled text).
- MUST A3: stale BEKEURING lines and popups pile onto prompts and bubbles
  (partly the shot script); hide prompts while a banner is up.
- MUST A4: 2P results labels run into their values ("KLAA200").
- MUST A5: the player is hard to pick out among pedestrians and BOA's: add
  a 1P/2P pointer tag like Tegenwind.
- SHOULD A6: the SJAAK distance label clashes with the tip box and boats;
  draw it on an opaque plate, hide it under the tip box and lock panel.
- SHOULD A7: in-world prompts have no backing plate.
- SHOULD A8: the swimmer reads as someone inside the houseboat.
- SHOULD A9: Sjaak's boat is a grey slab with one bike; his head covers
  the INKOOP sign. Use a piled boat, colour the hull, raise the sign.
- SHOULD A10: Sjaak moors among parked bikes; keep him apart.
- SHOULD A11: the flying heron reads as a knife; bigger, flapping.
- SHOULD A12: the golden bike looks like a yellow racefiets.
- SHOULD A13: results look empty; always say why (TIJD OP!, OPGEPAKT!).
- SHOULD A14: intro bubble and the chief's long name run off.
- SHOULD A15: Bruno is hidden when he sniffs.
- COULD: spec order sizes (fixed in the spec), mini map legend, the cat,
  shoot the coots / crane boat / party boats / organ, selfie phone, the
  Prof's select panel, the title houseboat.
- OK: house flow and look match, the street has depth, stages differ,
  lock panels are clear, cones readable, plenty of jokes.

### Gameplay review 1 (2026-10-16), with probe/sim numbers
- MUST G1 (fixed): a chase never ended: suspicion stayed full after the
  BOA lost you, so it restarted at once. Now `sus` drops to 0.5 on a lost
  chase and a chase only starts when the BOA sees you.
- MUST G2 (fixed): a fine during the throw lost the bike for good. The
  throw can no longer be interrupted.
- MUST G3 (fixed): 2P, the BOA player could camp Sjaak's boat. Sjaak now
  moors away from BOA's and casts off when one walks up.
- SHOULD G4 (fixed): hidden state that leads to fines: GEZOCHT! / HEET!
  badge in the HUD; a known-stolen bike dangles its lock.
- SHOULD G5 (fixed): a BOA's glance back had no warning: a small arrow
  over the head 24 frames before the turn.
- SHOULD G6 (fixed): time limits meant nothing: 120 / 150 / 150 s.
- SHOULD G7 (fixed): the canal rescue now counts how many of a kind the
  order still needs.
- SHOULD G8 (fixed, spec): the lazy bot ignores BOA's; "waiting is not
  free" is shown by the sneaky bot running out of time on stage 3.
- SHOULD G9 (fixed): the stage boss uses its own heckles in a chase.
- COULD (fixed): grace after a fine ends when you start on a lock; an
  innocent hider is pulled out, not fined; a shock ring shows how far the
  KRAK carries; a rare bike in a tree.
- COULD (open): the CPU thief's skill barely changes results (only drives
  the demo); in 2P the BOA can watch the thief's half of the screen.

### After the reviews (2026-10-16)
- Art fixes: popups on a dark plate, prompts under the thief's feet on a
  plate and hidden under banners, one bubble per spot (no stacking),
  1P/2P tags (always in 2P, in 1P at the start and in a crowd), short 2P
  result labels and a reason line (ALLES BEZORGD / OPGEPAKT / TE LAAT),
  Sjaak's boat in wood with a pile and the sign by the bow, the swimmer in
  a patch of water, a bigger flapping heron, the gold bike sparkles and no
  longer shares a colour with a racefiets, shorter intro heckles and the
  chief's name, the Prof's bakfiets off the stat labels, a cat with ears.
- Koningsdag got a sixth BOA (MIRJAM) and the chief sees farther: the
  decent bot clears it 36% (was 50-55%). Numbers in the spec.
- `cargo test` passes (all games), clippy clean in the new code and the
  hub, the wasm build compiles. README, AGENTS.md, architecture and the
  island spec list six games; README has two new screenshots and the new
  island.
- Left open (COULD): Sjaak's sign is hidden when someone stands in front
  of it; the swimmer next to a houseboat still looks a bit like it is in
  the boat; no second art review round.

## Follow-ups
- [Issue 0050: Stalen Zenuwen polish left](../../issues/0050-zenuwen-polish-left.md)
- [Issue 0006: unused `band` warning in Hang Time](../../issues/0006-hangtime-unused-band.md) (found on the way)
