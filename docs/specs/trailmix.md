# TRAIL MIX! (trail running)

*"The ultra trail of Mount Helico."* Id `trailmix`, module
`src/games/trailmix/`. Ski or Die's downhill meets pacing: read the trail,
jump, duck, and don't run out of legs before the summit.

## Running
Side-scroller to the right over sloped terrain (a height profile), with
parallax mountains.
- **Pace**: left/right shift between WALK, JOG, RUN and SPRINT (icons in
  the HUD). Stamina drains faster at high pace and uphill, recovers when
  walking, jogging and running downhill.
- **Power hike**: on steep uphill, alternate A and C to plant the poles:
  faster than running, cheaper than sprinting.
- **B**: jump roots, rocks, streams, sleeping marmots. Jump length grows
  with speed.
- **Down**: duck or slide under branches and fallen trees. On steep
  downhill, hold down to brake; too fast on technical ground and you trip.
- **A**: grab at aid stations (time it as you pass a volunteer) and
  high-five the cowbell crowd for HYPE (a short boost).
- **Terrain**: mud (slow, slippery), scree (slide), stepping stones over
  a stream (jump timing), snow fields (sit down and GLISSADE downhill),
  fog higher up.
- **Falls**: trip and tumble, lose time, FACEPLANT!, and stay muddy for a
  while.

## Runners (select: climb, descend, endurance, agility)
- BIG BEARD BJORN: endless endurance, slow downhill. "HAS NOT SHAVED
  SINCE 1979."
- ULLA ULTRA: all-round. "RUNS TO THE SHOP. THE SHOP IS IN ITALY."
- SPEEDY PETE: fast, fragile. "MOSTLY SOCKS AND OPTIMISM."
- MAMA TRAIL: great descender. "SHE RAN DOWN THIS MOUNTAIN TO GIVE
  BIRTH. TWICE."
Runners are `person()` with new running, climbing, ducking and tumbling
poses.

## Rival
GERALD THE GOAT runs every stage, leaps everything, takes shortcuts and
heckles (BAAAH! banners). Beating Gerald in stage 3 is the real win.

## Stages and modes
1P ladder against Gerald and two CPU runners; top 2 goes on, win the
last. Each stage about 2-3 min.
1. FOREST FLATS: roots, mushrooms, a deer crossing, mud.
2. ALPINE MEADOW: cows with bells blocking the path, whistling marmots,
   streams, switchbacks.
3. SUMMIT PUSH: snow, fog, scree, a glissade down and a sprint into the
   village finish.
Aid stations offer trail mix, bananas, stroopwafels and pickle juice (a
big boost and a funny face). 2P: split screen. Attract mode: CPU race.

## CPU and balance
`sim`: CPU pace strategies, stage times, falls. `probe`: human bots
(sprint-everything, steady, smart pacer). Targets:
- Sprinting everything runs out of stamina and loses to smart pacing.
- Stage times 2-3 min; decent probe top 2 in stage 1 about 90%, beats
  Gerald in stage 3 about 30%.

## Sound
Songs: a heroic alpine title song (a yodel-ish lead), a driving race
song, an alphorn-like fanfare. Effects: footsteps, jump, splash, cowbell,
goat baa, marmot whistle, crowd, gulp, trip, slide, aid ding, a
heartbeat when stamina is low, menu blip and confirm.

## How it plays (as built)
- Pace: WALK / JOG / RUN / SPRINT at 0.72 / 1.12 / 1.5 / 1.95 px per
  frame on the flat. Legs (stamina) per second: -8 / -3.5 / +1.2 / +10 on
  the flat, plus 5 / 11 / 19 / 28 per unit of uphill slope; downhill
  refunds a little. Endurance scales it. At 0 legs: BONK, forced to walk
  at 0.5 until legs are back to 50 (slow: about 4 per second).
- Power hike: on slopes over 0.18, alternate Z and C. Each alternate plant
  adds pole power (decays 3% a frame); the poles then set the speed (up
  to about sprint speed uphill) for 1.5 + 9 x slope legs per second.
- Switchbacks: runners cover ground at 62% speed; Gerald climbs straight
  up the cliff (shortcut) at full speed.
- Jump (X): 2.5-2.9 up, gravity 0.16; at least 1.1 px/frame forward in the
  air; a 9-frame landing lag and 7% speed loss stop bunny hopping.
- Duck / brake (down): 68% speed, under branches and propped trunks,
  deer leap over you, brakes the wobble on fast technical descents
  (rock, scree, snow, mud). Wobble at 1 = WIPEOUT.
- Snow descents: hold down to GLISSADE (up to 3 px/frame, legs recover).
- Aid stations (Z within 15 px of the volunteer): trail mix +28, banana
  +34, stroopwafel +24 and HYPE, pickle juice +45, long HYPE and a green
  face. Fans (Z within 18 px): HIGH FIVE = HYPE 2.5 s (+14% speed, no leg
  cost).
- Falls: roots, rocks, logs and snow rocks trip you (tumble, faceplant,
  get up: about 1.5 s plus lost speed, often muddy). Branches, trunks,
  cows at speed, deer and marmots stun you (about 0.5 s).
- Cows: pass them slower than 0.95 px/frame ("EXCUSE ME!") or get MOOed
  back. Marmots pop up and whistle when you come close.
- Gerald: leaps everything, shortcuts switchbacks, snacks at aid stations
  and now and then (more when far ahead), mild rubber band (about -9% when
  300 px ahead of the players, +9% when 300 px behind), heckles.
- 1P lineup per stage: you, Gerald and two of the other runners; top 2
  goes on in stages 1-2, stage 3 must be won. 2P: a 3-stage cup, points
  10 / 6 / 3 / 1, with Gerald and one CPU runner.

## Balance results
Tuned with `cargo run --release -- --sim trailmix 200` and
`--probe trailmix 25` (25 races x 4 runners per row), 2026-10-16.
Knobs: `GOAT_FAC` = 0.92 / 0.9 / 0.93 and `CPU_SKILL` = 0.5-0.6 /
0.6-0.7 / 0.75-0.85 in `headless.rs`.

| Target | Result |
|---|---|
| Sprinting everything runs out and loses to smart pacing | Sim, same runner and skill: Smart beats Sprinter 100% on every stage (Sprinter 208 / 215 / 253 s, 9-11 bonks; Smart 152 / 167 / 175 s, 0-0.3 bonks). Probe Sprinter bot: last in 100% of races. |
| Stage times 2-3 min | Decent bot 145 / 154 / 160 s. CPU runners 153 / 180 / 205 s (random skill 0.4-0.9). |
| Decent player top 2 in stage 1 about 90% | 99% (wins 88%). |
| Stage 2 in between | top 2 in 89%, beats Gerald 29%. |
| Decent player beats Gerald in stage 3 about 30% | 32% (= wins stage 3, champion). |
| No exploits | Hopper (jump all the time) 0-2% top 2 with 10-20 falls; Steady RUN (no poles, no food) 4% / 21% / 0% top 2; Lazy JOG last. |

Other numbers: CPU runners fall 1.3-2.5 times per race and almost never
bonk (0.01-0.26); the decent bot falls 2-3.5 times (5% missed obstacles).
