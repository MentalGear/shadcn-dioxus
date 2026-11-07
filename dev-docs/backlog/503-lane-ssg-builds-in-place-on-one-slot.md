---
id: 503
title: "lane-ssg builds in the warm template under the lock instead of copying it"
status: "BACKLOG, 2026-11-07. Next tooling row after the deploy."
size: "S-M"
area: tooling
updated: 2026-11-07
---

# Row 503: lane-ssg builds in place on one slot

## Item

`scripts/lane-ssg.sh` copies the 3.2 GB warm template to a scratch target for every build and refuses to start below
size + 20 % + 2 GB free (about 5.9 GB). With one build slot (`LANE_SLOTS=1`, the default since memory allows only one
SSG build at a time), the builds are already serialized by the slot lock, so the copy buys nothing but disk and copy
time, and on this 38 GB quota (14 GB of it warm trees) it refused lane builds again and again on 2026-11-05..07.

## Status / trigger

BACKLOG, 2026-11-07. Next tooling row after the deploy.

## Size

S-M

## Detail

**By construction:** on one slot, build directly in the template under the slot lock, and keep it correct by the
mechanisms that already exist: `lane-touch.sh` touches sources when the tree owner changes (cargo's mtime freshness,
row 216), and the template's `.tree-owner` stamp names the worktree last built. The copy path stays only for slot 1.
Proof: two lanes' builds in sequence on different worktrees each serve their own change (grep a marker in each pub
dir), a failed build leaves the next build correct, and the disk guard needs only the pub dir's ~100 MB.
