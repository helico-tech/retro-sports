---
status: done
created: 2026-10-16
---
# 02: Island hub

## Goal
The selector from [docs/specs/island.md](../../../docs/specs/island.md).

## Acceptance criteria
- Walk between venues, enter each game, come back to the same venue.
- Looks like the games (house style), has its own song.

## Plan
Draw the island procedurally once (cached Frame), animate the sea,
clouds, gulls and blimp on top, dioramas per venue keyed by cart id,
walker with `SpriteBank` run poses along a path, iris wipe on enter.

## Progress log
- 2026-10-16: island drawn per pixel from a wobbly ellipse (grass, sand,
  foam, shallows, sand bank), village, mountain with trail, polder with
  tulips and windmill, park with pond, beach court, cliffs with
  lighthouse. Animated: sea glints, foam, sailboat, Nessie (rare),
  dithered see-through clouds with shadows, gulls, blimp with HELICO SOFT
  banner, a mini scene per venue. Hiker (12x16, 4 frames) walks the
  dotted path; markers and pennants per venue; iris wipe in and out;
  calypso island song. Shots via `--shots <dir> hub`.

## Follow-ups
