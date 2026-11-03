---
id: 494
title: "Re-enable the @timing cases after the Rust port"
status: "BACKLOG, triggered: when the JavaScript plan's port rows are done (row 434 with rows 444-466, final…"
size: "S-M"
area: playwright
updated: 2026-11-02
---

# Row 494: Re-enable the @timing cases after the Rust port

## Item

The `@timing` cases (row 493) are deactivated: `playwright/tools/timing-switch.sh` has `TIMING_PASS_DEFAULT=off`, so the suite and `test:changed` skip the timing pass and the deploy does not count its missing report as red. This row turns them back on once the JavaScript plan has moved the costs they measure.

## Status / trigger

BACKLOG, triggered: when the JavaScript plan's port rows are done (row 434 with rows 444-466, final port row 467, the legacy removal), re-measure the open cases on a release build and re-enable the timing pass (owner 2026-11-02).

## Size

S-M

## Detail

**Why it is off.** Owner decision 2026-11-02. Measured in row 493 on release builds with nothing else running, the overlay open cases (Combobox, Select, Command, DatePicker) sit on the 50 ms line by code: Combobox open 51 and 69 ms in two of three passes, DatePicker open 52 ms once; the open cases are 48-69 ms solo, and 4 of 5 isolated passes of the 63 `main-thread.spec.ts` cases had a red. With the pass counted as red, every deploy would be blocked by a cost the port is expected to change (the open's work moves from JavaScript into Rust, the overlay port is row 456).

**On re-enable.**
1. Row 490 (the first key press restyles the page through the focus-modality tokens on `:root`) lands before or with this row: it is about half of the Combobox open task.
2. Measure: `TIMING_PASS=on playwright/tools/run-suite.sh --only-chunk timing ...` (or `pnpm exec playwright test --grep @timing --workers=1`) on a release build, nothing else running, at least five passes.
3. For each case that is still red: fix the code, or re-budget the threshold with the owner (a threshold is not changed without the owner).
4. Set `TIMING_PASS_DEFAULT=on` in `playwright/tools/timing-switch.sh`, refresh `playwright/durations.json` (`durations.mjs chunk-*.json --write`, the `chunk-timing.json` is part of the glob), and get one full deploy suite green (`scripts/deploy-preview.sh --suite-only`).
5. The four `carousel-virtual-wheel.spec.ts` cases that bound a frame-sampled stall at 200 ms ("long real flicks never stall at a rendered slice end") carry the tag and are dormant with the pass; their first run in the pass is the re-enable's measurement of them (row 493).
6. Move this row to done in the same commit: `scripts/check-timing-tags.sh` fails while the switch is off and this row is not open.

Related: [row 493](./493-timing-cases-run-alone-on-a-quiet-machine.md) (the tag, the pass, the evidence), [row 490](./490-first-key-press-restyles-the-page-focus-modality-tokens-on-root.md), [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md).
