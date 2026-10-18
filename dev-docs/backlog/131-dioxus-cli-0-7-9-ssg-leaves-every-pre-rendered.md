---
id: 131
title: "dioxus-cli 0.7.9 --ssg leaves every pre-rendered route except index.html stale when public/ already exists"
status: "OPEN (upstream) -- locally closed by construction (150e97b, scripts/build-ssg.sh); upstream issue…"
size: "S"
area: build
updated: 2026-10-04
---

# Row 131: dioxus-cli 0.7.9 --ssg leaves every pre-rendered route except index.html stale when public/ already exists

## Item

dioxus-cli 0.7.9 `--ssg` leaves every pre-rendered route except `index.html` stale when `public/` already exists

## Status / trigger

**OPEN (upstream)** -- locally closed by construction (`150e97b`, `scripts/build-ssg.sh`); upstream issue not filed

## Size

S

## Detail

Observed 2026-10-04: a second build into the same target dir served the previous build's stylesheet hashes on every component page (`baseline/VERIFY.md`). `build-ssg.sh` wipes `public/` and fails unless every `index.html` is newer than the build start, so this repo cannot hit it; anyone running `dx build --ssg` by hand can. **To do:** file against DioxusLabs/dioxus with the two-build repro; until then keep the CLAUDE.md rule to build SSG only through the wrapper. Same family as rows 98 and 100.
