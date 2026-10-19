---
title: "Row 116 -- before/after (2026-10-04)"
date: 2026-10-04
kind: note
---

### Row 116 -- before/after (2026-10-04)

Release SSG with `deploy-preview.sh`'s flags (`--release --ssg --features fullstack --base-path dioxus-components --force-sequential=true`), fresh isolated `CARGO_TARGET_DIR`, 4 cores, nothing else heavy running. Baseline = `2241da8`, after = `8c99edd` (`34d8532` server profile). Source: `/home/user/baseline/RELEASE.md`, `REPORT.md`.

| | 2241da8 | 8c99edd | delta |
|---|---|---|---|
| cold total | 1166 s | 904 s | -262 s (-22 %) |
| cold client phase (640 crates, through wasm-opt/esbuild) | 584 s | ~511 s | -73 s, **noise** (client profile untouched; deps phase alone 225 s vs 110 s) |
| cold server phase (714 crates) | 582 s | ~393 s (incl. 1.8 s SSG) | -189 s (-32 %); `preview` crate 376 -> 233 s |
| warm total (touch `preview/src/main.rs`) | 723 s | 411 s | -312 s (-43 %) |
| warm client | 352 s | ~335 s | noise-level, as expected |
| warm server (+ SSG) | 371 s | 76 s (`preview` crate 73 s, incremental) | -295 s (-80 %) |

The cold win attributable to the profile is the server phase (-189 s); the warm win is larger because `server-release` is incremental for workspace crates, no LTO, 16 CGUs. Debug (not a deploy profile): cold SSG 800 s with the baseline lane's linker shim (13.3 min), warm 123 s; after `7ad8421` no shim is needed (phase-A-copy 408 s, then 210 s / 218 s incremental, 44 s with nothing to compile). `scripts/session-setup.sh`: 7.7 s from scratch, 0.94 s no-op. Tip release build `9f8399d` (no base path, strict link): 916 s. Gates on unmodified `2241da8`: all pass (913 tests, 0 failed).
