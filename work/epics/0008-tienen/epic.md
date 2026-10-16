---
status: in-progress
created: 2026-10-16
---
# Epic 0008: TIENEN!

## Goal
Build TIENEN! (Dutch street football: one goal, one keeper, ten points)
as described in [docs/specs/tienen.md](../../../docs/specs/tienen.md),
following [docs/house-style.md](../../../docs/house-style.md), and give it
a venue on Helico Island.

## Scope
Module `src/games/tienen/`, its cart entry, sounds, headless shots, `sim`
and `probe`, its own tests, a venue on the island (`src/hub/`).

## Done when
- Every screen of the house style works, in 1P, 2P and attract mode.
- `cargo test` passes (including `tests/carts.rs`); no clippy warnings
  in the module.
- Balance targets in the spec are met and recorded there.
- An art review and a gameplay review found no open MUST items.
- The island has a venue for it; docs list seven games.

## Stories
- [01-first-playable](01-first-playable.md)
