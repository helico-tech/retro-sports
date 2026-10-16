# HANG TIME! (hang gliding)

*"The great Helico sky rally."* Id `hangtime`, module
`src/games/hangtime/`. In Dutch: deltavliegen. Find lift, fly the course,
and land on the cross, not on the sheep.

## Flying
Side view; the glider can face left or right. The world is wider and much
taller than the screen; the camera follows the glider. An altimeter and a
variometer (climb/sink) are in the HUD.
- **Launch**: run down the ramp by alternating left/right, press B at the
  edge. Too late: you tumble off and wobble; too early: a short hop.
- **Up/down**: push or pull the bar. Nose down: faster, sinks faster.
  Nose up: slower and flatter, until it STALLS (the nose drops, height
  is lost, a warning flashes).
- **Left/right**: turn around (a short animation that costs some height).
  Weave back and forth to stay in a thermal: the side view's version of
  circling.
- **Hold C**: tuck for a speed run between thermals.
- **B near the ground**: flare to land. Good timing = soft landing.
- **Lift**: thermals (shown by circling storks, rising leaves, cumulus
  clouds on top), ridge lift on slopes facing the wind, sink behind
  ridges and over lakes. The variometer beeps faster and higher as you
  climb, the classic sound.

## The task
Fly a course through turnpoints (gates under big hot-air balloons), then
land on the target circle. Score: turnpoints, time bonus, landing ring
(bullseye 500), minus crashes. Land short and you walk the glider in
(slowly, sadly), losing time.

## Whimsy
- Geese in a V: fly in their slipstream for free lift (HONK HONK).
- A stork carrying a bundle. A hot-air balloon family that waves.
  A cow-shaped cloud. A sheep on the landing field (BAAAD LANDING).
- A lake landing: splash, then paddle ashore with the glider like a duck.
- Stage 3's storm cloud sucks you up (CLOUDSUCK!) and spits you out high
  and dazed.
- A rare UFO that gives a little tractor-beam lift.

## Pilots (select: glide, climb, handling, nerve)
Four pilots with gliders in loud 90s colors, e.g. CAPTAIN KITE ("HAS NOT
TOUCHED THE GROUND SINCE TUESDAY"), DUNE DIRK, SKY GRANNY, THE EAGLE (an
actual eagle in sunglasses? decide on the select screen). Pilots are
`person()` in a prone hanging pose under a hand-drawn glider sprite.

## Stages and modes
1P ladder against three CPU pilots flying the same course at the same
time; the best score goes on, and you must win stage 3.
1. HELICO DUNES: Dutch coast, sea breeze ridge soaring over the dunes,
   lighthouse, kite surfers. Easy.
2. ALPINE VALLEY: thermals, cows, a chapel, sink over the lake.
3. THUNDER CANYON: strong thermals, big sink, the storm cloud.
2P: split screen, both on the same course. Attract mode: CPU flights.

## CPU and balance
`sim`: CPU pilots (completion rate, scores, landing accuracy). `probe`:
human bots (dive-everything, cautious, decent). Targets:
- A decent probe completes stage 1 almost always and wins stage 3 about
  30%.
- Each flight about 2-3 minutes.
- Flying fast and diving straight in does not beat using thermals.

## Sound
Songs: a dreamy title song (slow), a breezy flight song, a fanfare.
Effects: variometer beeps (several pitches up, a low sink tone), stall
alarm, wind gusts, launch run, flare, landing thud, crash, splash, goose
honk, checkpoint ding, cheer, menu blip and confirm.

## How it plays (as built)
- Flight model per frame: trim 0.80 px, polar sink 0.075 + 0.30 (v - 0.66)^2
  (best glide ~10:1 at trim with the tailwind), speed changes trade height
  1:1 through an energy term (zoom), stall below ~0.48, turns 15-24 frames
  and 2-3 px of height. Stats: glide = sink, climb = lift, handling = turn
  time, nerve = stall speed, flare window and tuck speed.
- Score: hoops 4 x 250, time bonus 10 per second under 4:00 times the share
  of hoops made, landing rings 500 / 300 / 150 (9 / 22 / 40 px), 50 in the
  field; geese 25, UFO 100, super launch 50; penalties belly/hard 50,
  sheep 100, splash 100 (+20 s paddling), crash 200 (no ring points).
  Landing outside the field: walk of shame, 2 frames per px to the field.
- Sunset at 3:30 of flight: no lift, gentle sink; every flight ends.
- 1P: you plus three CPU rivals on the same course; you must place first.
  CPU skill per stage: [-0.2, -0.1, 0.05], [0.05, 0.2, 0.35],
  [0.15, 0.3, 0.45] (below 0 = rookie habits). 2P: both humans plus two
  CPUs (skill 0.5), three stages, totals decide.

## Balance results
Tools: `cargo run --release -- --sim hangtime 25`,
`cargo run --release -- --probe hangtime 30` (120 races per bot per stage).
Probe bots are the CPU brain in your seat: dive (never climbs, tucked
straight in), cautious (climbs high everywhere, never speeds up), lazy
(skill 0.2), decent (0.6), good (0.8).

| Target | Result |
|---|---|
| Decent completes (wins) stage 1 almost always | 94% |
| Decent wins stage 3 about 30% | 29% (stage 2: 57%) |
| Good: rises with skill | 96% / 73% / 43% |
| Lazy | 73% / 22% / 12% |
| Flights about 2-3 minutes | decent 1:57 / 2:03 / 2:21; CPU 1:55-2:35 |
| Diving straight in does not beat thermals | dive bot: 0 hoops, never reaches the field, 0% wins |
| Cautious (climb everything) does not pay | 41% / 12% / 0%, 3:30+ in the canyon |
| CPU completion | 100% of flights end; 94-100% land in the field (skill >= 0.2) |
| CPU landing accuracy | 23-43 px off the cross (skill 0.8 -> 0.2) |
| Pilots balanced | averages within ~4% (Dune Dirk slightly ahead) |
| Pumping the bar | no gain over trim (checked) |
