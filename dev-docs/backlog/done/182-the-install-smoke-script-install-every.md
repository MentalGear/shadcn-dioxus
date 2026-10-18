---
id: 182
title: "The install smoke script: install every component into a scratch app and cargo check it"
status: "LANDED 2026-10-10"
size: "S (done)"
area: build
updated: 2026-10-10
---

# Row 182: The install smoke script: install every component into a scratch app and cargo check it

## Item

The install smoke script: install every component into a scratch app and `cargo check` it

## Status / trigger

**LANDED 2026-10-10**

## Size

S (done)

## Detail

`scripts/registry-install-smoke.sh [--cli BIN] [--only a,b,c] [--target TRIPLE] [--keep]`: snapshots the working tree (uncommitted edits included) as a throwaway git repo with the registry URL rewritten to its `file://` URL, renders `templates/starter` as the app, installs every member of the root `component.json` with `dx components add` (or with the `shadcn-dioxus` binary via `--cli`, which also diffs the app against the hand rendering), checks each directory and `pub mod` line landed, then `cargo check` with `CARGO_INCREMENTAL=0` and an absolute scratch target dir. Last run: 75 of 75 components installed with dx 0.7.9 and the app compiled (`cargo check`, 255 s). Not in `run-gates.sh` (minutes, ~6 GB): run it before merging when a `component.json` or a component directory (except `variants/` and `docs.md`) changes; `check-installed-paths.sh` (row 181) is its fast static approximation. The https verification against GitHub itself (rows 161 and 175) is still pending the merge.
