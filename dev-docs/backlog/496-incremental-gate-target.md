---
id: 496
title: "Measure incremental compilation for the main loop's gate target"
status: "APPROVED (owner 2026-11-05: item 2 of the build and test-suite review), measure first. Runs when no…"
size: "S"
area: tooling
updated: 2026-11-05
---

# Row 496: Measure incremental compilation for the main loop's gate target

## Item

The six cargo gates (`test`, `test-server`, `check-rustdoc`, `clippy-default`, `clippy-web`, `clippy-server`) are about
2,056 s of a 2,322 s full gate run, and most of it is recompiling the two large workspace crates (`preview`, 65k lines;
`primitives`, 100k lines): the `test` gate's compile was 4 min 58 s of its 6 min 51 s. `run-gates.sh` defaults
`CARGO_INCREMENTAL=0` (the cache regrows to several GB, row 168). Measure whether `CARGO_INCREMENTAL=1` on the main
loop's own gate target (`/home/user/tgt/gates`) only, lanes unchanged, cuts that time for a typical small change, and
what it costs in disk.

## Status / trigger

APPROVED (owner 2026-11-05: item 2 of the build and test-suite review), measure first. Runs when no lane holds the build
lock, so the numbers are not skewed by other builds.

## Size

S

## Detail

**Measurement.** On one tree: a full gate run with incremental on, to warm the cache; then a one-line change in a
`preview` component and one in a `primitives` file. For each change, time the six cargo gates with incremental off and
on, and record the target's disk size before and after. Use the gate-reuse store's `--no-reuse`, so every run is a real
compile.

**Decision.** Adopt it, by an opt-in in `run-gates.sh` for the main loop's target, only if the saving is large
(estimated about half, not measured) and the disk cost fits beside the warm release trees. Otherwise record the numbers
here and close the row.
