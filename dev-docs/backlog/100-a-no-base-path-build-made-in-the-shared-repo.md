---
id: 100
title: "A no---base-path build made in the SHARED repo-root target/ still emits /dioxus-components/ prefixed markup —…"
status: "Hit 2026-09-22 (round 7): all 51 carousel.spec.ts tests reached went red on code that was green an…"
size: "Three rebuilds wasted on a stale-output…"
area: misc
updated: 2026-09-22
---

# Row 100: A no---base-path build made in the SHARED repo-root target/ still emits /dioxus-components/ prefixed markup —…

## Item

A no-`--base-path` build made in the SHARED repo-root `target/` still emits `/dioxus-components/` prefixed markup — every interaction test then fails on a correct component

## Status / trigger

Hit 2026-09-22 (round 7): all 51 `carousel.spec.ts` tests reached went red on code that was green an hour earlier

## Size

Three rebuilds wasted on a stale-output hypothesis before the real channel was found; ~40 min

## Detail

The serve root's pages carried 89 `/dioxus-components/` references (`grep -c` says `3` — it counts LINES, and the page is 223 minified lines; use `grep -o … wc -l`). Hydration then pushState-navigated to a broken `/dioxus-components/dioxus-components` and left a 2.4 KB shell, so `#component-preview-frame` never existed. `dx` is innocent: a build into a brand-new `CARGO_TARGET_DIR` that no `--base-path` build had touched emitted ZERO prefixed refs, and the captured rustc env in `target/dx/.captured-args/*/preview.bin.json` holds only `DIOXUS_APP_TITLE`/`DIOXUS_PRODUCT_NAME`. The channel is the shared `target/` tree, which `scripts/deploy-preview.sh` also writes WITH `--base-path` (line 22) — which is why `rm -rf public`, `cargo clean -p preview` and wiping `web/` all failed: each re-derived contaminated inputs from inside the same tree. **Subsumed by construction** by `CLAUDE.md`'s absolute isolated `CARGO_TARGET_DIR` rule, which row 98 introduced for an unrelated reason (relative paths break `--ssg` outright): two base-path modes can then never share a tree. Honest limit: a second writer was not caught in the act, so the channel is the best-supported account, not a witnessed fact — but the isolated build is witnessed to be clean and the shared one is witnessed to be dirty. Written up in `dev-docs/conformance-harness.md`.
