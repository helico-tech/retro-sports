---
status: blocked
created: 2026-10-16
---
# 01: First playable

## Goal
The whole game per spec: core mechanics, all screens, art, sound, CPU,
modes, tools, reviewed and tuned.

## Acceptance criteria
- See the epic's "Done when".

## Plan
Module `src/games/trailmix/`, split like dodgeball:
- `runners.rs` — the four runners and Gerald: stats (climb, descend,
  endurance, agility), bios, palettes.
- `course.rs` — the three stages as a fixed, hand-scripted course per
  stage (deterministic, same every race): height profile in 4 px cells,
  ground type per cell (dirt, grass, rock, mud, scree, snow, water),
  obstacles (roots, rocks, logs, mushrooms, branches, propped trunks,
  streams with stepping stones, sleeping marmots), aid stations, cowbell
  crowds, switchbacks (Gerald's shortcuts), deer crossings, cow and
  marmot spots, fog zone, props (trees, signs, start and finish arch).
- `race.rs` — the simulation, pure and at 60 Hz: runners with pace
  (WALK/JOG/RUN/SPRINT), stamina, bonk, jump, duck/brake, power hike
  (alternate A and C), trip/tumble/faceplant, mud, glissade, aid grabs,
  high fives (HYPE), animals (cows, marmots, deer), Gerald (leaps
  everything, shortcuts, heckles, snacks), CPU AI, banners, popups, fx,
  sfx cues, results.
- `sprites.rs` — all art built once in `App::new()`: runner poses from
  `engine::sprites::person()` with own skeletons (run flat/uphill/
  downhill, sprint, walk, hike, duck, jump, tumble by rotating a curled
  bitmap, faceplant, glissade, bonk, grab, victory), extra details
  (beard, socks, vest) painted on and re-outlined; goat, cows, marmots,
  deer, volunteers, crowd, obstacles and items as string bitmaps with
  role letters and palette swaps, 1px OUTLINE.
- `scenery.rs` — per stage, pre-rendered once: sky bands, far and mid
  mountain strips (wrap 512 px), ground texture tiles; per frame only
  blits with parallax, plus clouds, birds, snow and fog.
- `render.rs` — one view of a race (camera on a runner) into a band of
  the screen: parallax, terrain columns (top face of the trail + textured
  front face), props, obstacles, animals, runners in lanes, fx, banners,
  speech bubbles, HUD (pace icons, legs bar, time, place, course
  profile with runner dots). 2P: two 112 px bands, the second drawn into
  a scratch frame and copied with `Frame::copy_band`.
- `app.rs` — scenes: title (+menu, attract demo after 15 s), select,
  stage intro card, play (pause), results tally, continue, game over,
  champion. 1P ladder: 3 stages vs Gerald + 2 CPU runners, top 2 goes
  on, stage 3 must be won. 2P: a 3-stage cup on points, split screen.
- `audio.rs` — 3 songs (alpine title with yodel leaps, race song,
  alphorn fanfare) and 16 sfx.
- `headless.rs` — `shots` (numbered PNGs of every screen and moments,
  sprite sheets), `sim` (CPU races per stage: times, falls, pacing
  strategies), `probe` (bots: sprinter, steady, smart pacer vs the
  ladder).
- `tests/trailmix.rs` — races finish, sprinting loses to pacing, scenes
  draw, sounds render, determinism, `new()` fast.

Order: core loop (course, race) + sim first, then screens, then art and
sound, then CPU and balance, then reviews.

## Progress log
- 2026-10-16: Core loop first: `course.rs` (three scripted stages, height
  profile in 4 px cells, ground per cell, obstacles, aid, crowds, zones,
  props), `race.rs` (pace, stamina, bonk, jump, duck/brake, poles, trip,
  glissade, mud, water, aid grabs, high fives, cows, marmots, deer, Gerald,
  CPU AI) and a `sim`. First numbers: Gerald won every race by a minute,
  CPUs fell 5-9 times per race, Smart pacing lost to Steady on stage 2.
- 2026-10-16: Fixes from the first sims: hike power decays smoothly and the
  poles set the speed (no SPRINT+poles exploit); bonks recover slowly to 50
  legs; at hiking speed rocks were impossible to clear, so a jump always
  carries you forward at least 1.1 px/frame; CPU jump timing aims the top of
  the arc at the obstacle. CPU falls dropped to 1-2.5 per race.
- 2026-10-16: Art and screens: `sprites.rs` (45 poses from new skeletons,
  beard/vest/socks/apron painted on and re-outlined, tumble by rotating a
  curled bitmap; goat, cows, marmots, deer, items as string bitmaps with
  role palettes), `scenery.rs` (sky, three 512 px parallax strips, ground
  textures, all props cut from a scratch frame with outline), `render.rs`
  (terrain columns: bank, top face, lip, textured front face; lanes;
  fog; tips; HUD with pace pips, legs bar, POLES meter, course profile),
  `app.rs` (all scenes), `audio.rs` (3 songs, 16 sfx), `headless.rs`
  (shots, sim, probe with Lazy/Sprinter/Steady/Hopper/Decent bots).
- 2026-10-16: Rebased `feat/trailmix` onto `feat/retro-sports` (ea2aba7):
  the old placeholder hub could not scroll to a second cart, so the
  contract test hung. The new island hub fixes that, but see issue 0030:
  on this branch alone the walk from the beach to Mount Helico takes
  longer than the contract test waits.
- 2026-10-16: Balance: home-sprint budget was 10x too optimistic (CPU and
  bot bonked just before the line); Gerald's rubber band was +-25% speed,
  now about +-9%; Gerald per stage 0.86 / 0.9 / 0.93, CPU skill per stage
  0.5-0.6 / 0.6-0.7 / 0.75-0.85. Results in the spec.
- 2026-10-16: No engine changes. Everything new lives in the module
  (own outline pass, color sprites, wrapping strips, crop sheets).

- 2026-10-16: Extra whimsy: rare sightings per race (a yeti walking
  through the summit fog, a Helico balloon or a cow-shaped cloud over the
  forest and meadow), a HYPE label in the HUD. The camera looks further
  ahead when you are fast (smoothed per runner).
- 2026-10-16: Checks: `cargo test` and `cargo test --release` pass except
  `tests/carts.rs::console_enters_and_leaves_every_game` (issue 0030: the
  hiker needs 136 frames to walk from the beach to Mount Helico, the test
  waits 120; passes with 200, and with all five games registered).
  `cargo clippy --release --all-targets`: no warnings in the module or its
  tests. `cargo build --release --target wasm32-unknown-unknown` builds.
  `App::new` 4 ms, a 2P split-screen frame (update + draw) 0.14 ms native,
  sound bank 20 ms. Status `blocked` only on issue 0030; the game itself is
  complete.

### Art review (2026-10-16)
Looked at every shot at 3x, the pose sheets and two 12-frame animation
sheets (jump, tumble), next to dodgeball's shots.
- MUST: 1P champion podium took its order from the raw results and could
  crown Gerald. Fixed: the player stands on top.
- MUST: at the start all four racers stood on one spot. Fixed: a queue
  behind the line.
- MUST: intro card: distance and goal text overlapped the tagline. Fixed.
- MUST: rock ground looked like a brick wall. Fixed: natural stones
  (cells around random points, dark cracks, lit top-left edges).
- SHOULD: fog was a flat dither curtain over the trail. Fixed: it washes
  out the background and only thickens on the far right.
- SHOULD: the front of the ground was a big plain brown area. Fixed: lower
  ground line, grass hanging over the lip, roots, buried bones, a fossil,
  a lost pink shoe, ice crystals.
- SHOULD: the deer had one frame. Fixed: two-frame leap.
- SHOULD: the faceplant read as a lump. Fixed: flat on the belly, feet
  kicking in the air.
- SHOULD: the aid station disappeared behind runners. Fixed: tall AID
  banner with bunting.
- SHOULD: pickle juice flashed the whole sprite white. Fixed: green face.
- SHOULD: flat color under the tree line. Fixed: forest trunks, mown
  meadow stripes with flowers, scree dots.
- SHOULD: the results quip never changed. Fixed: six quips.
- COULD (not done): big bust portraits on the intro card (same as
  dodgeball's open art issue 0002); a third cow frame (tail swish); more
  foreground plants.

### Gameplay review (2026-10-16)
Played through `probe` bots and traces (`trace` dev tool), read the sim.
- MUST: bunny hopping (B held) cleared every low obstacle. Fixed: 9-frame
  landing lag and 7% speed loss per landing; the Hopper bot now loses badly.
- MUST: poles at SPRINT pace gave sprint speed at hiking cost. Fixed.
- MUST: rocks on steep climbs could not be jumped at hiking speed. Fixed.
- MUST: the "sprint home" budget made CPUs bonk at the line. Fixed.
- SHOULD: Gerald's rubber band mirrored the player's time. Fixed (milder).
- SHOULD: no feedback for pole plants. Fixed: a tick per alternate plant,
  POLES meter in the HUD on steep ground.
- SHOULD: controls were hard to discover. Fixed: context tips at the
  bottom of the view (steep, cows, deer, snow, aid, fans, low legs, bonk,
  too fast) and a pace hint during the countdown.
- SHOULD: stage 3 should be won about a third of the time. Tuned to 26-28%.
- COULD (not done): rubber band in 2P towards the trailing player; a
  ghost of your best time.

## Follow-ups
- [Issue 0030](../../issues/0030-contract-test-hub-walk.md): contract test
  gives up before the hiker reaches Mount Helico.
- COULD items from the reviews: bust portraits on the intro card, a third
  cow frame, more foreground plants, a 2P rubber band, a ghost runner.
