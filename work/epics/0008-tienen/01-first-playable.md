---
status: done
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
- 2026-10-16: simulation (`world.rs`), CPU and probe bots, tuned headless
  first. First runs: every game hit the 5-minute cap. Causes: crosses
  bounced off the receiver's own body (and that touch made the shot
  illegal), deflected balls nobody fetched, crosses too low to head (top
  at 17 px, a head is at 25), and receivers with a negative timing error
  that never swung. Fixed all four: games now take 2.5-3 minutes.
- 2026-10-16: art (`art.rs`, poses and props), backdrops (`scenery.rs`:
  the flats, the school with Mevrouw Knorr's garden, the cage at dusk),
  `render.rs`, `app.rs` (all screens), `audio.rs` (16 sfx, 4 songs),
  `headless.rs`. Looked at every shot; fixed the title logo running off,
  the venue names too wide (dropped "DE"/"HET"), see-through tip boxes,
  the school sign, the HUD hint over the points, bubbles stacking into
  banners, a goal that did not read (now a dark mouth, thicker posts).
- 2026-10-16: island venue HET TRAPVELDJE between Helicodam and the polder
  (a new path stop; CARTS order follows the path). `tests/tienen.rs` (15
  tests). `cargo test --release` passes for all games.
- 2026-10-16: design hole: you only go in goal through your own mistake,
  so a 1P player who never touches the ball cannot lose. Added a target
  per pitch (DOEL 3/4/4 goal points). Probe with 100 games: decent 89% /
  67% / 35%, greedy (bicycle kicks) and lazy keeper clearly worse. Made the
  bicycle kick wider (riskier) and the cage kids skill 0.85.

### Art review 1 (2026-10-16), against dodgeball and Zenuwen shots
- MUST (fixed): heckle bubbles and popups piled up under the banner;
  Kevin's bio ran off his panel; the game-over bubble covered its line;
  the HUD 1P tag covered a keeper's name.
- SHOULD (fixed): a pale goal (darker net, mouth, outlined bar); banners
  over the back-line kids (moved up); a toy-sized car (now 62 px, with a
  driver); a poodle as white as the ball (pink bow, bigger, ball in her
  mouth); popups on the goal (moved right); Gijs is "too big for the goal"
  (now a bigger body); "SCORE 0" next to a total (RUN SCORE / NOT ADDED);
  the island pitch was a grey slab (bigger, with a goal and net).
- SHOULD (open): the continue screen is plain. COULD (open): Lieke's
  helmet, light pools in De Kooi, the two humans on the 2P card, more
  contrast for banners on the flats.

### Gameplay review 1 (2026-10-16), with sim/probe and exploit bots
- MUST (fixed): keeper mashing (now a window and a recovery, punch can
  miss, catch only on the ground); goal-hanging next to the keeper (field
  kids stay out of the area, shorter keeper cooldown after a parry); 2P
  rewarded hiding (now by goal points); stalling on the ball or next to a
  dead ball (SPELEN! after 4 s, dead ball back to the keeper after 5 s);
  ETEN ties went against seat 0.
- SHOULD (fixed): the shot kind was luck (now foot/chest on the ground,
  head or bicycle kick in a jump); human keeping was not feasible (keeper
  assist to the basic spot; dive on B + direction); the probe did not
  play like a human (now in a human seat with human lag); hiding after the
  target (CPU kids cross to a kid who has not touched it in 20 s).
- COULD (fixed): the dribble grab ignored the 1-point rule; the
  billenbrand wiggle was too fast; the goal roof waited 8 s; the whiff
  sound played on every bicycle kick; P2 saw P1's keys.
- Found on the way: the keeper assist first walked to where the shot
  would cross (no reaction time), so a human keeper body-blocked nearly
  everything (1 goal against per game vs 5); now only the basic spot.
- Retuned: catches weaker, hard shots slip through, shorter pauses,
  targets in goals 3/5/6, De Kooi skill 1.0. Numbers in the spec.
- `cargo test` passes (20 Tienen tests), clippy clean in the module and
  the hub, the wasm build compiles. README, AGENTS.md, architecture and
  the island spec list seven games.

## Follow-ups
- [Issue 0060: Tienen polish left](../../issues/0060-tienen-polish-left.md)
