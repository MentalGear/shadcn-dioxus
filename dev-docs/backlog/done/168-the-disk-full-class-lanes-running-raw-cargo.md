---
id: 168
title: "The disk-full class: lanes running raw cargo with incremental compilation on regrow the shared gates target…"
status: "LANDED 2026-10-11 -- scripts/lane-target.sh create now also prints export…"
size: "S"
area: process
updated: 2026-10-11
---

# Row 168: The disk-full class: lanes running raw cargo with incremental compilation on regrow the shared gates target…

## Item

The disk-full class: lanes running raw `cargo` with incremental compilation on regrow the shared gates target dir by ~10-13 GB per hour

## Status / trigger

**LANDED 2026-10-11** -- `scripts/lane-target.sh create` now also prints `export CARGO_INCREMENTAL=${LANE_CARGO_INCREMENTAL:-0}`, so a lane that follows the documented `eval` setup gets both; CLAUDE.md says it next to the `CARGO_TARGET_DIR` rule. The lane-hour `du` before/after measurement in "Done when" was not taken. Was: **OPEN (proposal)** -- recorded 2026-10-05 (audit); scheduled in the next-session order (step 1, a harness fix; the CLAUDE.md note is the main loop's edit)

## Size

S

## Detail

**Evidence (audit, 2026-10-05):** the shared gates target dir regrew by ~10-13 GB per hour while lanes ran raw `cargo` (check, clippy, test) with the default `CARGO_INCREMENTAL=1`; the growth is incremental-compilation state. `scripts/run-gates.sh` already defaults it off (`export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"`, `:117-118`), which is why the gates run is not the grower: the growth comes from lanes outside it. **Construction (not a reminder):** `scripts/lane-target.sh create` already prints the single `export CARGO_TARGET_DIR=<abs path>` line every lane `eval`s; make that output also export `CARGO_INCREMENTAL=0` (opt-out: `LANE_CARGO_INCREMENTAL=1`), so a lane that follows the documented setup gets both and the class needs no discipline; plus one sentence in CLAUDE.md next to the `CARGO_TARGET_DIR` rule (the main loop edits CLAUDE.md, this row's lane does not). **Trade-off, stated:** a lane's own edit-build loop gets slower (a changed crate recompiles whole instead of incrementally) in exchange for a bounded disk footprint; a lane that truly needs the speed opts back in and runs `scripts/lane-target.sh prune` afterwards. **Does not cover:** a caller that exports `CARGO_TARGET_DIR` by hand without `lane-target.sh` (the CLAUDE.md sentence is for them) or `dx` builds, which set their own profile flags. **Done when:** a lane created through the script has `CARGO_INCREMENTAL=0` in its environment, and a before/after `scripts/lane-target.sh du` over one lane-hour shows the growth gone.
