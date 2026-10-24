---
id: 131
title: "dioxus-cli 0.7.9 --ssg leaves every pre-rendered route except index.html stale when public/ already exists"
status: "CLOSED 2026-10-23 (owner): local fix 150e97b stays (build SSG only through scripts/build-ssg.sh)…"
size: "S"
area: build
updated: 2026-10-23
---

# Row 131: dioxus-cli 0.7.9 --ssg leaves every pre-rendered route except index.html stale when public/ already exists

## Item

dioxus-cli 0.7.9 `--ssg` leaves every pre-rendered route except `index.html` stale when `public/` already exists

## Status / trigger

**CLOSED 2026-10-23 (owner): local fix 150e97b stays (build SSG only through scripts/build-ssg.sh); not filed upstream (no-upstream rule).** **NARROWED 2026-10-19 (backlog review): local fix 150e97b in (build-ssg.sh wipes public/); filing upstream is the owner's decision (row 6)**

## Size

S

## Detail

Observed 2026-10-04: a second build into the same target dir served the previous build's stylesheet hashes on every component page (`baseline/VERIFY.md`). `build-ssg.sh` wipes `public/` and fails unless every `index.html` is newer than the build start, so this repo cannot hit it; anyone running `dx build --ssg` by hand can. **To do:** file against DioxusLabs/dioxus with the two-build repro; until then keep the CLAUDE.md rule to build SSG only through the wrapper. Same family as rows 98 and 100.
