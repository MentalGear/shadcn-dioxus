---
id: 367
title: "Lane builds need 10.4 GB free because every lane-ssg copies 6.8 GB of template, 3.0 GB of it dead incremental state"
status: "LANDED 2026-10-23 (tooling). Lane gatetools made the copies slim; branch claude/lane-template-slim…"
size: "S (lane-slot-target.sh, lane-target.sh…"
area: tooling
updated: 2026-10-23
---

# Row 367: Lane builds need 10.4 GB free because every lane-ssg copies the whole template

## Item

`scripts/lane-ssg.sh` copies the warm template (`/home/user/tgt/template`, 6.8 GB) into a fresh scratch tree for every build, and `scripts/lane-slot-target.sh`'s guard (row 356) wants the copy's size + 20% + 2 GB free: 10.4 GB. This container's allowance is ~22 GB, `/home/user/tgt/deploy` (3.5 GB, the warm release tree) and `gates` (3.1 GB) are permanent, and with the template that leaves about 10 GB, so the guard refused lane builds in three merge batches in a row on 2026-10-22 and the main loop deleted trees by hand each time.

## Status / trigger

LANDED 2026-10-23 (tooling). Lane `gatetools` made the copies slim; branch `claude/lane-template-slim` added `scripts/lane-target.sh slim`, made `create` copy slim and `prune` slim the kept trees. One step is the main loop's, on its container: `scripts/lane-target.sh slim --apply` once with the queue idle (frees ~3.0 GB of the template for good). The per-slot persistent tree (proposal 3) is not done: revisit only if 6.6 GB free is still too much.

## Size

S (`lane-slot-target.sh`, `lane-target.sh`, two `check-lane-lock.sh` scenarios). M for the per-slot persistent tree, not done.

## Detail

**Root cause, measured.** The template's 6.8 GB is `x86_64-unknown-linux-gnu/server-dev` 3.0 GB + `wasm32-unknown-unknown/wasm-dev` 2.4 GB + the rest (dx output 281 MB, release stub 21 MB, `wasm-dev`/`server-dev` dx dirs ~1.3 GB, which are the build outputs the next build overwrites). Of the 3.0 + 2.4 GB, `deps/` is 1.4 GB and 0.9 GB, while the two `incremental/` dirs (`preview-*`, `dioxus_primitives-*` in each of the two targets) are 1.6 + 1.4 = **3.0 GB, 44% of the template**. They were written when the template was first built with incremental compilation on. Every lane command runs with `CARGO_INCREMENTAL=0` (`lane-ssg.sh` sets it on the build, `lane-cargo.sh` and `run-gates.sh` default it off), and with it off cargo neither reads nor writes `incremental/`. So each lane build copied 3.0 GB it could not use, paid its copy time, and needed 10.4 GB free instead of ~6.6 GB. The class is "a copied tree's size is not what the consumer reads"; the old guard estimated `du` of the source, so it also could not see the difference.

**Landed (construction).** `lane-slot-target.sh copy` now streams the source through `tar` with `--exclude='./*/incremental' --exclude='./*/*/incremental'` (and `.tree-owner`), keeping modes, mtimes (cargo freshness, row 216) and hard links, and `check_disk` sizes with `du --exclude=incremental`. One function serves `lane-ssg.sh` (scratch copy), the lazy slot-1 `check-s1`/`gates-s1` warm copies and the guard, so the three cannot disagree. `LANE_COPY_INCREMENTAL=1` restores the full copy for a hand-run incremental build. `check-lane-lock.sh` scenario 10 proves: incremental dirs absent at both depths, deps kept with equal mtime and an intact hard link, `.tree-owner` absent, the opt-out keeps them, and the guard passes on a disk that fits the slim copy but not the full tree. Effect: 6.8 GB -> 3.8 GB per lane build (needs 6.6 GB free instead of 10.4 GB). Proven on a real `LANE_SLOTS=1 scripts/lane-ssg.sh` build of this lane with 6.7 GB free (the old guard would have refused): the guard passed, no dependency was recompiled (0 `Compiling` units for deps in the log), `SSG complete` after 393 s, public/ complete.

**Proposed, not done (needs the main loop; the lane must not touch the template).**
1. Free 3.0 GB permanently and keep the guard honest: `rm -rf /home/user/tgt/template/*/*/incremental` once no lane is copying (hold the lock). Nothing reads them while `CARGO_INCREMENTAL=0`; the `release` and `deploy` trees are separate and untouched. After that the guard's `--exclude` is a no-op and the template is 3.8 GB.
2. The same dead weight probably sits in `check`, `gates`, `ssg-lane`/`-s1` copies; `scripts/lane-target.sh prune` could drop `incremental/` from any tree that is not `LANE_CARGO_INCREMENTAL=1`.
3. A persistent per-slot build tree (`$T/ssg-lane` kept between builds, row 216 `lane-touch.sh --force` instead of a fresh copy) would remove the copy entirely and bring the footprint to one 3.8 GB tree per slot, always resident. It was rejected in the `lane-ssg.sh` header: a target shared across worktrees failed 3 of 5 builds with "stylesheet ... not in dx's asset manifest" (dx's asset manifest goes stale across trees), so it needs a verified reset of `dx/` and the `build/` fingerprints per build; larger, riskier, and the slim copy already removes the disk pressure. Revisit only if 6.6 GB is still too much.

**Landed (2026-10-23, proposals 1 and 2 as tooling).** The lane still does not touch the template; it ships the command the main loop runs.
- `scripts/lane-target.sh slim [--apply]` lists (deletes) `<tree>/<profile>/incremental` and `<tree>/<triple>/<profile>/incremental` in every tree under the root (`template`, `check`, `gates`, `ssg-lane*`, `*-s1`, lane copies, the release trees, which normally have none since release builds are not incremental). A tree stamped `.keep-incremental` is skipped. `--apply` takes every slot lock (`LANE_SLOTS` plus any `lockq/slot-*.lock` on disk) and the legacy `ssg.lock` with a non-blocking flock for the whole delete, because a `tar` copy that reads a tree mid-delete fails; if any is held it prints which, deletes nothing and exits 1.
- `scripts/lane-target.sh create` copied the full template with `cp -a`; it now uses the same `lane-slot-target.sh copy` as `lane-ssg.sh` (slim unless `LANE_CARGO_INCREMENTAL=1`, which keeps the full copy and stamps `.keep-incremental` so `slim` leaves that lane's state alone), and sizes its free-disk check without the incremental dirs.
- `scripts/lane-target.sh prune --apply` runs `slim` after its own deletions; with a build running it only warns.
- `check-lane-lock.sh` scenario 11 proves: `create` drops the dirs at both depths and keeps deps, the opt-out copies them and stamps the tree, `slim` without `--apply` deletes nothing, `--apply` refuses (and deletes nothing) while a real `lane-lock.sh` holder has slot 0, then on an idle queue deletes the dirs of the template and of `gates`, keeps deps, the stamped tree and `lockq/`.
- `lane-rules.md`: the new-container setup ends with `slim --apply`, and the command is listed with the session tooling.
