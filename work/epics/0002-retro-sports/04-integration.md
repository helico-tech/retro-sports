---
status: done
created: 2026-10-16
---
# 04: Integration

## Goal
Merge the four game branches, check them together on the island, in the
window and the browser, and update README and screenshots.

## Acceptance criteria
- Every game passes `tests/carts.rs` and its own tests.
- Browser check: every game starts and plays from the island.

## Plan
Merge each branch as it lands (registry conflict only), run all tests,
browser-test every venue from the island, measure fps in each demo,
redeploy, README with screenshots, merge to main.

## Progress log
- 2026-10-16: merged Trail Mix, Disc Dogs, Hang Time, Tegenwind; fixed the
  contract test (wait for the hiker, cannot loop forever). Clippy
  cleanup; 324 of 324 shots identical.
- 2026-10-16: Hang Time dunes plane: sand vs field per field, not per
  pixel (it looked speckled).
- 2026-10-16: bug: Esc in the attract demo of dodgeball, Disc Dogs and
  Trail Mix went to the title, whose idle timer restarted the demo at
  once, so you could not leave. Esc now resets the idle timer; new
  contract test `esc_leaves_the_attract_demo`.
- 2026-10-16: browser (headless Chromium): every venue enters, plays and
  returns; 60 fps on the island and in every game's demo; native frame
  cost 60-125 us. Deployed. README rewritten with screenshots.
- Not checked: real speakers, Safari and Firefox, real hands on a keyboard.

## Follow-ups
