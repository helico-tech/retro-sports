---
status: todo
type: bug
created: 2026-10-16
---
# Console contract test fails when only some carts are registered

## Problem
`tests/carts.rs::console_enters_and_leaves_every_game` presses right once,
waits 120 frames and then presses start. On Helico Island the hiker walks
1.3 px per frame along the path, and start is ignored while walking. With
only `dodgeball` and `hangtime` registered (as on branch `feat/hangtime`),
the walk from BEACH COURT (path point 0) to HELICO CLIFFS (path point 12)
is about 230 px, so the hiker is still walking after 120 frames and the
test fails with "did not enter hangtime".

With all five carts registered every hop is at most about 75 px, so the
test passes once the games are integrated.

## How to reproduce
On `feat/hangtime` (dodgeball + hangtime only): `cargo test --test carts`.

## Notes
Fix in the test (wait until `!hub.walking()` or simply longer), not in a
game. The hangtime branch does not touch the hub or `tests/carts.rs`.
