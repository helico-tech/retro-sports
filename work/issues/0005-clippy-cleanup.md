---
status: todo
type: debt
created: 2026-10-16
---
# Clippy cleanup after Rust 1.99 lints

## Problem
Newer clippy flags `x % n == 0` (use `is_multiple_of`), range checks and a
few loops across dodgeball, the engine and the hub (~46 warnings). The
code works; it is style only.

## Notes
Do it once, after the four game branches are merged, with
`cargo clippy --fix`, then check dodgeball's shots stay byte-identical.
