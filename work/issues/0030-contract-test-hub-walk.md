---
status: done
type: bug
created: 2026-10-16
---
# Contract test gives up before the hiker reaches a far venue

## Problem
`tests/carts.rs::console_enters_and_leaves_every_game` presses right once
and waits 120 frames before pressing start. On Helico Island the hiker
walks between venues at 1.3 px/frame; while walking, start is ignored. When
two venues with registered carts are far apart on the path, the walk takes
longer than 120 frames and the test fails with "did not enter <game>".

## How to reproduce
On `feat/trailmix` (only dodgeball and trailmix registered) run
`cargo test --test carts`. The beach (path point 0) to Mount Helico (path
point 9) is about 177 px, 136 frames. With 200 frames of waiting the test
passes (checked locally, not committed).

## Notes
- With all five carts registered the venues are 3 path points apart
  (about 50-80 frames) and the test passes, so this only bites on a branch
  with a subset of the games (e.g. each game branch on its own; hangtime at
  path point 12 will hit it too).
- Fix in the test (wait until `!walking`, or 240 frames) or let the hub
  accept start while walking. Both are outside the game modules, so left
  for the integration story (epic 0002 story 04).

## Resolution
Fixed on feat/retro-sports: the hub has `ready()`, and the test waits for
the hiker to arrive instead of a fixed 120 frames.
