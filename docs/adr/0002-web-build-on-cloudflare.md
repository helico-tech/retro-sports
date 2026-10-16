# 0002. Browser build on Cloudflare Workers static assets

## Status
accepted (site and worker renamed by 0003: retro-sports.helico.app)

## Context
The game should also be playable in a browser at dodgeball.helico.app.
macroquad already targets `wasm32-unknown-unknown` with its own JS loader,
and the game draws everything into a `Frame` (ADR 0001), so the same code
runs on the web. The site is static: one HTML page, the loader and the
wasm file.

## Decision
- Build the same binary for `wasm32-unknown-unknown`. `web/build.sh` puts
  `web/index.html`, the wasm and the `mq_js_bundle.js` from the macroquad
  version in `Cargo.lock` into `dist/`. The loader is copied, not
  committed, so it always matches the Rust side.
- Host `dist/` as a Cloudflare Worker with static assets only (no worker
  code), on the custom domain `dodgeball.helico.app` (`wrangler.toml`).
  Deploy by hand with `npx wrangler deploy`.
- `.cargo/config.toml` passes `--allow-undefined` to the wasm linker:
  quad-snd declares its JS audio functions without an import module, and
  newer `rust-lld` rejects them otherwise.
- In the browser, Esc on the title does nothing (there is nothing to quit
  to; stopping the loop would freeze the tab).

## Consequences
- One code base for desktop and web; no web-only game code.
- Keyboard only; phones and tablets cannot play.
- The browser blocks sound until the first key press or click; the loader
  unlocks it then.
- The macroquad loader logs a harmless `register_plugin is not defined`
  error from its unused networking plugin.
- Deploying needs a Cloudflare login (`npx wrangler login`) on the account
  that owns `helico.app`.
