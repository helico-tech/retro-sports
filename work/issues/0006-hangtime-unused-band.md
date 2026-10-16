---
status: todo
type: debt
created: 2026-10-16
---
# Build warning: unused `band` in Hang Time's dunes

## Problem
`cargo build` warns about an unused closure argument `band` in
`src/games/hangtime/scenery.rs:456` (`dunes_mid`), added in commit
0fd1f47 after the clippy cleanup (issue 0005). The other clippy warnings
in older code are the 9 that issue 0005 left on purpose.

## Notes
Rename it to `_band` (or drop the use). Found while building epic 0007;
out of its scope.
