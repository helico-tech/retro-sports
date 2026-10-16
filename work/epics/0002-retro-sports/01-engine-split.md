---
status: done
created: 2026-10-16
---
# 01: Engine split

## Goal
Shared engine, `Game` trait and cart registry, dodgeball as one cart
(ADR 0003).

## Acceptance criteria
- Dodgeball screenshots byte-identical to before.
- `tests/carts.rs` contract tests exist and pass.

## Plan
Move gfx and sprites to `engine/`, split the synth out of dodgeball's
audio, move Pad and Rng, add `cart.rs`, `console.rs`, a stub hub, lazy
sound banks in `main.rs`, a command line per game.

## Progress log
- 2026-10-16: done. 47 of 47 dodgeball shots identical. Added
  `sprites::person()` (custom skeletons) and `Frame::copy_band` (split
  screen) for the new games. Crate renamed to `retro-sports` 0.3.0.

## Follow-ups
- Clippy has ~38 pre-existing style warnings in dodgeball (new lints).
