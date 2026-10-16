# RETRO SPORTS

A collection of 16-bit style sports games on Helico Island, in Rust, in the
spirit of California Games, Ski or Die and Super Dodge Ball. All art and
sound are generated in code. Games: DODGE OR DIE! (dodgeball), DISC DOGS!
(frisbee), STALEN ZENUWEN! (bike lifting in Amsterdam), TEGENWIND! (Dutch
cycling), TRAIL MIX! (trail running), HANG TIME! (hang gliding).

## Build and run
- `cargo run --release` — play (window, keyboard, sound); starts on the
  island selector.
- `cargo test` — contract tests for every game (`tests/carts.rs`) plus
  each game's own tests.
- `cargo run -- --shots <dir> [game]` — scripted PNG screenshots (3x) of
  every screen of every game (or one game, or `hub`). Use these to review
  visuals.
- `cargo run --release -- --<tool> <game> [n]` — a game's headless tools,
  e.g. `--sim dodgeball 1000` (CPU-vs-CPU balance) or `--probe dodgeball 6`
  (scripted "human" bots). Run without arguments to list them.
- `web/build.sh` — browser build into `dist/`; test with
  `python3 -m http.server -d dist`, deploy with `npx wrangler deploy` to
  https://retro-sports.helico.app (ADR 0002).

## Controls (all games)
- P1: arrows, Z (or ,) = A, X (or .) = B, C (or /) = C, Enter start.
- P2: WASD, F, G, H, Tab.
- Esc: back to the game's title, then back to the island (quits from the
  island on desktop). F1: scanlines on/off.

## Map
- [docs/architecture.md](docs/architecture.md) — engine, carts, key rules.
- [docs/house-style.md](docs/house-style.md) — look, flow and controls
  every game follows.
- [docs/specs/](docs/specs/) — one design per game, plus the island.
- [docs/adr/](docs/adr/) — decisions.
- [work/queue.md](work/queue.md) — what to pick up next.
- [work/epics/](work/epics/) and [work/issues/](work/issues/).
