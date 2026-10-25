---
id: 267
title: "Split oversized source files into modules (carousel.rs, carousel.spec.ts, main.rs, top_layer.rs, calendar.rs, date_picker.rs)"
status: "LANDED (merge queue batch 22, lane/split-row267): six files split into module directories, public…"
size: "M"
area: process
updated: 2026-10-25
---

# Row 267: Split oversized source files into modules

## Item

Pure moves, no behaviour change: `primitives/src/carousel.rs` (361 KB), `playwright/carousel.spec.ts` (182 KB),
`preview/src/main.rs` (180 KB), `primitives/src/top_layer.rs` (169 KB), `primitives/src/calendar.rs` (160 KB) and
`primitives/src/date_picker.rs` (141 KB) become module directories of reviewable size.

## Status / trigger

**LANDED (merge queue batch 22, lane/split-row267): six files split into module directories, public paths unchanged; the history-comment extension is subsumed by the ratchet at zero (rows 410 and earlier).**

## Size

M

## Detail

Owner approval: a "yes backlog" doc comment on the large-source-files note in the repo cleanup plan (row 266,
https://claude.ai/code/artifact/19769a0d-67cd-4794-acb3-bc078458b8cb). Order: after the feature lanes that touch these
files have merged (the 2026-10-19 calendar lane edits calendar.rs and date_picker.rs; the Round 4 thread may touch
main.rs and top_layer.rs), inside the Round 6 sweep and before the 152b suite prune, so the prune reviews the split spec
files. One file per commit, full gates green after each, `git log --follow`-friendly moves (no edits in the move commit).

**Extension, owner approved 2026-10-20** (cleanup thread, same approval as row 266's playbook steps): while splitting,
rewrite history comments in the ~50 files that have the most (~530 lines; `carousel.rs`, `top_layer.rs`,
`top-layer.spec.ts` and `carousel.spec.ts` lead) to "current reason + row N". Leave explanatory blocks alone. Keep these
rewrites in commits separate from the pure moves.

**Landed 2026-10-25 (lane/split-row267, thread "Split oversized files into modules"):** `primitives/src/{carousel (17
files), top_layer (9), calendar (10), date_picker (10)}/` with `pub` re-exports from `mod.rs`, so every public path is
unchanged; each primitive split starts with a pure `git mv` commit. `preview/src/main.rs` went from 3,984 to 468 lines
plus `preview/src/site/` (12 files), which `select.mjs` maps to the broad set. `carousel.spec.ts` became six specs plus
`carousel-helpers.ts` with identical titles (432) and measured `durations.json` entries. Class fixed by construction:
split-out test files fooled `check-direction-keys.sh` and `check-props-docs.sh`; both now skip `tests.rs`,
`*_tests.rs` and `*/tests/*`. The history-comment extension needed no separate pass: `check-history-comments.sh`
reports 0 baselined lines in 0 files. Evidence on fc623c0b: full gates 90/90, registry-install-smoke 82/82, the
diff-selected specs green.
