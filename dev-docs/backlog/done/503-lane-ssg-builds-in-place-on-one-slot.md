---
id: 503
title: "lane-ssg builds in the warm template under the lock instead of copying it"
status: "DONE 2026-11-07 (lane lanessg-503, landed by merge bae54003). lane-ssg.sh slot 0 builds in…"
size: "S-M"
area: tooling
updated: 2026-11-12
---

# Row 503: lane-ssg builds in place on one slot

## Item

`scripts/lane-ssg.sh` copies the 3.2 GB warm template to a scratch target for every build and refuses to start below
size + 20 % + 2 GB free (about 5.9 GB). With one build slot (`LANE_SLOTS=1`, the default since memory allows only one
SSG build at a time), the builds are already serialized by the slot lock, so the copy buys nothing but disk and copy
time, and on this 38 GB quota (14 GB of it warm trees) it refused lane builds again and again on 2026-11-05..07.

## Status / trigger

DONE 2026-11-07 (lane lanessg-503, landed by merge `bae54003`). `lane-ssg.sh` slot 0 builds in `$LANE_TGT/template` under `lockq/template.lock`
(`lane-slot-target.sh copy` from the template takes the same lock, which also covers `lane-target.sh create`), touches
through `lane-touch.sh` (non-forced, stamp-aware), guards disk with the pub dir + margin and publishes the pub dir
atomically. Proof, `LANE_SLOTS=1`: build A (worktree with MARKERAAA) served 2 files with A, 0 with B; build B (temporary
worktree with MARKERBBB) served B only, stamp = B; a planted `compile_error!` failed in-place (exit 1, previous pub dir
intact, template intact); the next build after the revert (MARKERCCC) served C only, none of the planted text. Free disk
was 2937 MB before, 2527 MB minimum during (other lanes' writes included), 5613 MB after; the build itself adds no
copy. Time saved per build is only the copy: 1 GB of the template copies in 4.9 s, so ~20 s of a 9-12 min build; the
gain is the ~4 GB of disk and the refusals below 5.9 GB free.

## Size

S-M

## Detail

**By construction:** on one slot, build directly in the template under the slot lock, and keep it correct by the
mechanisms that already exist: `lane-touch.sh` touches sources when the tree owner changes (cargo's mtime freshness,
row 216), and the template's `.tree-owner` stamp names the worktree last built. The copy path stays only for slot 1.
Proof: two lanes' builds in sequence on different worktrees each serve their own change (grep a marker in each pub
dir), a failed build leaves the next build correct, and the disk guard needs only the pub dir's ~100 MB.
