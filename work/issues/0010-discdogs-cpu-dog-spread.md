---
status: todo
type: debt
created: 2026-10-16
---
# Disc Dogs: CPU per-dog spread at mid skill

## Problem
`--sim discdogs 1000` at skill 0.5: Biscuit wins 74%, Sir Woofington 37%.
At 0.0 and 0.9-1.0 the four dogs are within 44-63%. The CPU planner
throws as far as its dog can reach, which favours the fastest dog until
flips (jump stat) start to count at high skill. In 1P this shows up at
stage 2 only: the decent probe wins 98% as Biscuit (facing a CPU Sir
Woofington) but 44% as Sir Woofington (facing a CPU Biscuit).

## Notes
Try: CPU planning margin per dog speed, or stage-2 opponent order, or a
small speed coefficient change (1.75 + 0.2 * speed) followed by a full
re-tune with `--sim` and `--probe`. The zoomies stat barely matters for
the CPU (it rarely needs them).
