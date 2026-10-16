# 0003. One crate, a shared engine and a game per module ("carts")

## Status
accepted

## Context
Dodge or Die! grows into RETRO SPORTS: several small sports games that
share the look, the sound and the controls, behind one game selector, in
one download and on one web page. The games will be built in parallel.

## Decision
- Keep one crate (`retro-sports`). Shared code moves to `src/engine/`
  (framebuffer, font, skeleton sprites, synth, pad, rng, PNG). Each game
  is a module under `src/games/<id>/`.
- A game implements the `cart::Game` trait and exports a `Cart` (id,
  title, blurb, constructor, sound bank, screenshots, headless tools).
  `games::CARTS` lists them; the hub (`src/hub/`) is the selector.
- Sounds stay generated in code. Each game owns a bank of sfx and songs;
  the window layer loads a bank the first time it is used.
- `tests/carts.rs` is the contract every game must pass: survives idle
  and button mashing, sounds exist, Esc leads back to the selector, the
  console can enter and leave it.

Rejected: a cargo workspace with one crate per game. It enforces the
boundaries, but adds manifests and build time for no real gain at this
size; the module rule (engine has no game rules) does the same job.

## Consequences
- One binary, one wasm file, one page for all games.
- Parallel work touches separate folders; only `games/mod.rs` is shared.
- A game that needs a new engine feature builds it locally first, so two
  games do not change the engine at the same time.
- The wasm file grows with every game (now about 1 MB).
- Hosting moves with the name: Cloudflare worker `retro-sports` on
  retro-sports.helico.app; the `dodgeball` worker and dodgeball.helico.app
  are removed. The code moves to github.com/helico-tech/retro-sports.
