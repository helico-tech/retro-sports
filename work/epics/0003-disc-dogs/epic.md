---
status: done
created: 2026-10-16
---
# Epic 0003: DISC DOGS!

## Goal
Build DISC DOGS! as described in [docs/specs/discdogs.md](../../../docs/specs/discdogs.md),
following [docs/house-style.md](../../../docs/house-style.md).

## Scope
Module `src/games/discdogs/`, its cart entry, sounds, headless shots, `sim`
and `probe`, its own tests.

## Done when
- Every screen of the house style works, in 1P, 2P and attract mode.
- `cargo test` passes (including `tests/carts.rs`); no clippy warnings
  in the module.
- Balance targets in the spec are met and recorded there.
- An art review and a gameplay review found no open MUST items.

## Stories
- [01-first-playable](01-first-playable.md)
