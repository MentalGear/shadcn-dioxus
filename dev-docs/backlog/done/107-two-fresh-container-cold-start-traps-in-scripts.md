---
id: 107
title: "Two fresh-container cold-start traps in scripts/check-attr-spread-collision.sh and its --update-baseline mode"
status: "LANDED 2026-10-11 -- (a) the analyzer build falls back from --offline to an online build…"
size: "Cosmetic; remedy known, not fixed here"
area: build
updated: 2026-10-11
---

# Row 107: Two fresh-container cold-start traps in scripts/check-attr-spread-collision.sh and its --update-baseline mode

## Item

Two fresh-container cold-start traps in `scripts/check-attr-spread-collision.sh` and its `--update-baseline` mode

## Status / trigger

**LANDED 2026-10-11** -- (a) the analyzer build falls back from `--offline` to an online build (Cargo.lock pins it) with a clear message, and fails loudly naming both causes when neither works (cold registry simulated with an empty `CARGO_HOME`); (b) `--update-baseline` counts non-blank entries, so an empty set reports 0. Gate output unchanged. Was: Found 2026-09-25 (round 8), first fresh-container run of the gate this session

## Size

Cosmetic; remedy known, not fixed here

## Detail

(a) The analyzer is built with `cargo build --offline`, so on a genuinely fresh container (empty local registry cache) it fails with `failed to get \`dioxus-i18n\`` or similar until *some* other `cargo build` in the workspace has first populated `~/.cargo/registry`; running any ordinary `cargo check`/`cargo test` first, before the gate script, works around it. (b) `--update-baseline` reports `wrote 1 entries` even for a genuinely empty baseline — it is counting a trailing blank line in the `.tsv` output as a row. Neither affects the gate's actual detection logic; both are surface-level annoyances a fresh session should expect rather than mistake for a broken gate
