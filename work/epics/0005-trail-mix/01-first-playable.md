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

## Follow-ups
