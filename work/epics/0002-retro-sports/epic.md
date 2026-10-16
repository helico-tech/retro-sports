---
status: done
created: 2026-10-16
---
# Epic 0002: Retro Sports collection

## Goal
Turn Dodge or Die! into RETRO SPORTS: a collection of whimsical 16-bit
sports games on one island selector, in its own GitHub repo
(helico-tech/retro-sports) and on retro-sports.helico.app.

## Scope
Shared engine, game trait and registry, the island selector, rename,
new repo, CI and release, web hosting (and removing dodgeball.helico.app).
The games themselves are epics 0003-0006.

## Done when
- All games are reachable from the island, in the window and the browser.
- `cargo test` passes, including `tests/carts.rs` for every game.
- retro-sports.helico.app serves the collection; dodgeball.helico.app is
  gone; the code is on github.com/helico-tech/retro-sports.

## Stories
- [01-engine-split](01-engine-split.md)
- [02-island-hub](02-island-hub.md)
- [03-repo-and-hosting](03-repo-and-hosting.md)
- [04-integration](04-integration.md)
