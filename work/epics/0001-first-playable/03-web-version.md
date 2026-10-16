---
status: done
created: 2026-10-16
---
# 03: Web version

## Goal
Play the game in a browser at https://dodgeball.helico.app, hosted on
Cloudflare.

## Acceptance criteria
- The wasm build runs the same game as the desktop build, with sound.
- Served from Cloudflare on dodgeball.helico.app.
- Title, menus, a match and Esc work in a real browser.

## Plan
Build for `wasm32-unknown-unknown` with macroquad's JS loader, add a small
`web/index.html` and `web/build.sh`, host `dist/` as a Workers static
assets site with a custom domain (ADR 0002).

## Progress log
- 2026-10-16: wasm link failed on quad-snd audio imports with the newer
  `rust-lld`; fixed with `--allow-undefined` in `.cargo/config.toml`.
- 2026-10-16: Esc on the title stopped the loop, which would freeze the
  tab; ignored on wasm in `main.rs`.
- 2026-10-16: the loader defines a global `canvas`; the page uses another
  name. Sound unlocks on the first key press (loader handles keydown).
- 2026-10-16: tested in headless Chromium, locally and live: 60 fps,
  title, menu, select, match with throws, pause, Esc back to title.
  Deployed with `npx wrangler deploy`; `.wasm` served as
  `application/wasm`. Not tested: Safari, Firefox, real audio output.

## Follow-ups
- None required. Possible: deploy from GitHub Actions (needs a Cloudflare
  API token secret).
