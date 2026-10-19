---
id: 267
title: "Split oversized source files into modules (carousel.rs, carousel.spec.ts, main.rs, top_layer.rs, calendar.rs, date_picker.rs)"
status: "BACKLOG (owner approved 2026-10-19 on the cleanup plan; Round 6, before the 152b suite prune).…"
size: "M"
area: process
updated: 2026-10-19
---

# Row 267: Split oversized source files into modules

## Item

Pure moves, no behaviour change: `primitives/src/carousel.rs` (361 KB), `playwright/carousel.spec.ts` (182 KB),
`preview/src/main.rs` (180 KB), `primitives/src/top_layer.rs` (169 KB), `primitives/src/calendar.rs` (160 KB) and
`primitives/src/date_picker.rs` (141 KB) become module directories of reviewable size.

## Status / trigger

**BACKLOG (owner approved 2026-10-19 on the cleanup plan; Round 6, before the 152b suite prune). Owner: unassigned**

## Size

M

## Detail

Owner approval: a "yes backlog" doc comment on the large-source-files note in the repo cleanup plan (row 266,
https://claude.ai/code/artifact/19769a0d-67cd-4794-acb3-bc078458b8cb). Order: after the feature lanes that touch these
files have merged (the 2026-10-19 calendar lane edits calendar.rs and date_picker.rs; the Round 4 thread may touch
main.rs and top_layer.rs), inside the Round 6 sweep and before the 152b suite prune, so the prune reviews the split spec
files. One file per commit, full gates green after each, `git log --follow`-friendly moves (no edits in the move commit).
