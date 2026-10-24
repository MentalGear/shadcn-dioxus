---
id: 116
title: "Build/dev loop efficiency: release SSG build takes ~18 min; lanes rebuild cold"
status: "LANDED 2026-10-04 (steps 1, 2-3 via scripts, 5; step 4 dropped; step 6 timeout only) -- was: Open…"
size: "M"
area: process
updated: 2026-10-04
---

# Row 116: Build/dev loop efficiency: release SSG build takes ~18 min; lanes rebuild cold

## Item

Build/dev loop efficiency: release SSG build takes ~18 min; lanes rebuild cold

## Status / trigger

**LANDED 2026-10-04** (steps 1, 2-3 via scripts, 5; step 4 dropped; step 6 timeout only) -- was: **Open -- owner-requested 2026-10-01; deferred until after the carousel visual fixes**

## Size

M

## Detail

Owner-requested 2026-10-01: the release SSG build took ~18 min and every lane rebuilt cold. Landed 2026-10-04 in `34d8532`,
`8c99edd`, `7ad8421`, `832dbaf`, `150e97b`, plus a round-2 addendum (`scripts/run-gates.sh`). Measured on 4 cores, rustc 1.97.0,
dx 0.7.9, one cargo at a time, isolated absolute `CARGO_TARGET_DIR`s.

**Plan and what came of it:**

1. **Server-only cheap profile, DONE (`34d8532`).** dx 0.7.9 builds the SSG server under `server-release` and the shipped wasm
   under `wasm-release`, both synthesised as `inherits = release`; a user-defined `[profile.server-release]` wins (opt-level 1, no
   LTO, 16 CGUs, incremental), while `wasm-release` stays undefined so the shipped wasm keeps dx's own profile.
2. **Debug SSG for functional specs and 3. warm base-path-free target copies, DONE as scripts.** `scripts/lane-target.sh
   create|prune|du` (`8c99edd`) copies a warm template per lane and refuses a template carrying a base-path build (row 100);
   `deploy-preview.sh` honours `CARGO_TARGET_DIR`. A 6.1 GB template copies in 24 s and there is no reflink on this filesystem,
   so budget the disk.
4. **lld as linker, dropped.**
5. **Environment, DONE (`832dbaf`).** `scripts/session-setup.sh` as the SessionStart hook seeds dx's tools by `curl` (the third
   occurrence of the downloader-ignores-the-proxy-CA trap), `pnpm`/`npm ci`, `cargo fetch`: 7.7 s from scratch, 0.94 s no-op,
   versus ~21 min for `cargo install dioxus-cli`.
6. **Loop, timeout only.** Per-test timeout 300 s to 120 s (`PW_TEST_TIMEOUT`); `main-thread.spec.ts` ignored on known-debug
   builds (`PW_BUILD_PROFILE`, `PW_FORCE_TIMING_SPECS=1` overrides). Debug gave 64/153/141/113 ms long tasks where release gave
   0, so timing specs run only on release. "One full suite pre-merge" stays a convention.

**Two classes found on the way, both fixed by construction.**

- *dx `--ssg` stale public* (third instance after rows 98 and 100): dx 0.7.9 rewrites only `public/index.html` when `public/`
  exists, every other route keeps the previous build's HTML and stylesheet hashes. Construction (`150e97b`):
  `scripts/build-ssg.sh [debug|release] [--base-path P]` is the only place that runs the SSG build; it requires an absolute
  `CARGO_TARGET_DIR`, refuses a no-base-path build into a `.base-path-build` tree, wipes the profile's `public/`, and fails
  unless every `public/**/index.html` is newer than the build start and the URLs match the base path.
- *Every debug wasm build of `preview` failed to link on rustc 1.97* (`undefined symbol: stderr`): dioxus-code's `lang-css`
  pulls tree-sitter's C runtime into the client, arborium 2.17.0's wasm libc shim lacks the data symbol `stderr`, and rustc no
  longer passes `--allow-undefined`; release links only because lld dead-strips the function. Fix (`7ad8421`): define the one
  missing symbol in `preview`. Upstream follow-ups: rows 130, 131.

**Artifact identity.** `8c99edd`'s release output differs from the `2241da8` baseline only in four path-derived strings (hashes,
`dxc-<n>` ids, the hydration-data blob, the css-module suffix): 0 residual differences across 89 differing files once masked.
Not proven byte-for-byte at equal worktree path; the client profile is untouched.

**Round-2 addendum: `scripts/run-gates.sh`, the one gate entry point (2026-10-04).** A re-typed list of gates is how row 139
happened, so the list is a script: every `scripts/check-*.sh`, `cargo fmt`, three clippy arms (default, `web`, `server`: the
plain `--workspace` run lints only the `not(web)` arm and none of preview's `server` code, yet both ship), `cargo test`, then
stylelint. Guards: the target dir must be absolute and not the repo-root `target/`; the summary prints HEAD and a diff hash;
a `check-*.sh` missing from the list fails the run with exit 2; exit 0 is every gate green, 1 a red gate, 2 usage, 3 green but
partial. Verified on `44a103e`: 23/23 gates, release SSG build 14 m 14 s, 93 pages, STALE_HTML=0, full suite 1761 passed / 12
failed (the known rows 113, 21, 106, 114 and the red-by-design Rule 4c selftest, plus load flakes that were green serially).
The round also found and removed one real regression: `.dx-component-variant { content-visibility: auto }` made carousels and
charts measure 0 inside skipped variants (33 tests red), so that rule is gone and `scroll-main-thread.spec.ts` asserts the
opposite; its mechanism is in the scroll-jank research (`dev-docs/archive/research/scroll-jank-2026-10-04.md` section 7.4).
Scroll re-measure (median of 3, release): idle 178 to 45 main ms/s, scroll main busy 6,745 to 2,089 ms, tasks over 16 ms 8 to 0,
live `setInterval` at the end 74 to 0 (section 7.6 of the same note).

## Before / after (2026-10-04)

Release SSG with `deploy-preview.sh`'s flags (`--release --ssg --features fullstack --base-path dioxus-components
--force-sequential=true`), fresh isolated `CARGO_TARGET_DIR`, 4 cores, nothing else heavy running. Baseline = `2241da8`, after =
`8c99edd` (`34d8532` server profile).

| | 2241da8 | 8c99edd | delta |
|---|---|---|---|
| cold total | 1166 s | 904 s | -262 s (-22 %) |
| cold client phase (640 crates, through wasm-opt/esbuild) | 584 s | ~511 s | -73 s, **noise** (client profile untouched) |
| cold server phase (714 crates) | 582 s | ~393 s (incl. 1.8 s SSG) | -189 s (-32 %); `preview` crate 376 -> 233 s |
| warm total (touch `preview/src/main.rs`) | 723 s | 411 s | -312 s (-43 %) |
| warm client | 352 s | ~335 s | noise-level, as expected |
| warm server (+ SSG) | 371 s | 76 s (`preview` crate 73 s, incremental) | -295 s (-80 %) |

The warm win is larger because `server-release` is incremental for workspace crates, no LTO, 16 CGUs. Debug SSG (not a deploy
profile): cold 800 s with a linker shim, no shim needed after `7ad8421` (408 s first, then 210 s incremental, 44 s with nothing
to compile).

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/116-build-dev-loop-efficiency-release-ssg-build.md`.*
