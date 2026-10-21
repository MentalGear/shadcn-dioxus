---
id: 324
title: "Keep warm release target trees for validation and for deploy"
status: "Tooling landed 2026-10-21 (lane/gate-speed dab5418a + follow-up); open on one owner question below.…"
size: "S"
area: process
updated: 2026-10-21
---

# Row 324: Keep warm release target trees for validation and for deploy

## Item

Keep warm release target trees for validation and for deploy.

## Status / trigger

**Tooling landed 2026-10-21 (lane/gate-speed `dab5418a` + follow-up); open on one owner question below. Owner condition "build only if a measurement holds up": it holds, smaller than expected.**

## Size

S

## Detail

The 2026-10-20 batch-A release build was cold (~22 min, 707 crates) because the only warm release tree carried the deploy's `.base-path-build` marker and `lane-target.sh` refused it as a template. Keep two warm release trees -- one no-base-path for validation, one base-path for deploy -- refreshed after each deploy, and teach `lane-target.sh` to pick by base path. Expected release build ~3-5 min.


## Measurement (2026-10-21, quiet box, `CARGO_BUILD_JOBS=2`, `build-ssg.sh release`, same machine, one after the other)

| build | tree | wall | what compiled |
|---|---|---|---|
| cold, no base path | empty `release-template` | **1,309 s** (21.8 min) | server 633 units + client 707 units, all |
| warm, no base path | the same tree after the first build, from another worktree (sources touched, `primitives/src/lib.rs` edited) | **864 s** (14.4 min) | server: `dioxus_primitives` 64 s + `preview` 410 s; client: `dioxus_primitives` 85 s + `preview` 267 s; SSG + verify the rest |

So a warm tree saves **445 s (34 %)**, not the 17-19 min the row assumed (3-5 min total): the two workspace crates
`preview` and `primitives` are built with `lto = true`, `codegen-units = 1`, `opt-level = "z"` (workspace `[profile.release]`)
and recompile in full whatever is warm; only the ~1,300 dependency units are. Raw logs:
`/home/user/tgt/rel-cold.log`, `/home/user/tgt/rel-warm.log`. The 2026-10-20 cold build (~22 min) matches the cold row.
Not measured: a base-path (deploy) build in `release` (it is the same shape; `deploy-preview.sh` now builds there by
default and should show the same ~7 min saving); a build under load from other lanes.

## What changed, and the correction to the row's premise

- Two trees, chosen by base path: `$LANE_TGT/release-template` (no base path, sentinel `.no-base-path-only`) and
  `$LANE_TGT/release` (base path `shadcn-dioxus`, the deploy's). `lane-target.sh create <n> --release [--base-path P]`
  copies the matching one (refuses a tainted or mismatched one), `prune` never removes either, `deploy-preview.sh`
  defaults to `<LANE_TGT>/release` (it used the shared repo `target/`) and refuses the sentinel tree before building.
- **"Refreshed after each deploy" is dropped.** What a warm tree saves is the dependency units; they change only with
  `Cargo.lock`, the toolchain or dx. A per-deploy refresh is a 14-minute warm build that saves the next validation build
  the same 7 minutes it costs. `lane-target.sh warm-release` therefore records `.cargo-lock-sha` and says "is current" and exits
  when the tree was built from the present `Cargo.lock` (`--force` builds anyway). The deploy tree refreshes itself
  on every deploy because that is where the deploy builds.
- `scripts/check-lane-target.sh` (new gate, <1 s, fake trees) pins the selection, the refusals, prune, the skip and the deploy guard.
- The no-base-path tree is built and left in place for the main loop: `/home/user/tgt/release-template` (3.5 GB, `build-ssg: OK -- 100 routes`,
  `.cargo-lock-sha` seeded). The partial `/home/user/tgt/release` (1.1 GB, an aborted 10-20 deploy build) is still the deploy's
  and will complete on the next deploy; it was not touched.

## Open question for the owner

Is ~7 min per release build worth 3.5 GB each of disk, two trees (7 GB; 9 GB free at the time of writing)? The alternative that
would actually shorten the build is a validation-only release profile (`lto = "thin"` and 16 codegen units, say), but then
the build is no longer the shipped one, and the timing specs (`main-thread`) must not run on it. Recommendation: keep the
no-base-path tree only (the validation builds are the frequent ones); the deploy tree already exists.
