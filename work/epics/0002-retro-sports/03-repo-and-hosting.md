---
status: done
created: 2026-10-16
---
# 03: Repo and hosting

## Goal
New GitHub repo helico-tech/retro-sports, CI and release for the renamed
binary, the site on retro-sports.helico.app, dodgeball.helico.app removed.

## Acceptance criteria
- CI green on the new repo.
- retro-sports.helico.app serves the collection; dodgeball worker deleted.

## Plan
Create the repo and push; rename CI/release/packaging; deploy a new
worker `retro-sports`; delete the `dodgeball` worker once the new site
works; redeploy as games are merged.

## Progress log
- 2026-10-16: github.com/helico-tech/retro-sports created (public), main
  and feat/retro-sports pushed over HTTPS (SSH key has no access). Old
  remote kept locally as `dodgeball-old`; helico-tech/dodgeball untouched.
- 2026-10-16: CI runs `--sim dodgeball` and a wasm build job; release
  packages `retro-sports` binaries and `Retro Sports.app`.
- 2026-10-16: worker `retro-sports` deployed on retro-sports.helico.app
  (island + dodgeball), checked in headless Chromium: island, enter,
  crew select, Esc back, 60 fps, no errors. Worker `dodgeball` deleted;
  dodgeball.helico.app no longer resolves.

## Follow-ups
