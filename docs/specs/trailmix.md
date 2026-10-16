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
