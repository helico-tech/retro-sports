---
status: done
type: bug
created: 2026-10-16
---
# Contract test loops forever when the hub cursor never moves

## Problem
`tests/carts.rs::console_enters_and_leaves_every_game` walks the hub with
`while con.hub.cursor != i { press right ... }` and has no limit. On a
branch whose hub does not move on RIGHT (the old list hub at 36c156a only
moved on UP/DOWN), the test does not fail, it hangs: `cargo test` never
ends.

## How to reproduce
Register a second cart on top of 36c156a and run `cargo test --test carts`.

## Notes
- Fixed for us by the island hub (`feat/tegenwind` now merges
  `feat/retro-sports`). A step limit with an assert in that loop would turn
  a future hang into a clear failure. Outside the game modules, so left
  for the integration story.

## Resolution
The test now fails after `CARTS.len()` presses instead of looping.
