---
id: 356
title: "Lane disk guard ignores concurrent template copies, so two slots both pass and both die with ENOSPC"
status: "LANDED. lane-slot-target.sh reserve|release|copy (flock on $Q/disk.lock, one file per pid under…"
size: "S: lane-slot-target.sh reservation plus…"
area: tooling
updated: 2026-10-22
---

# Row 356: Lane disk guard ignores concurrent template copies

## Item

`scripts/lane-slot-target.sh check_disk` compares free space with the template size + 20% + 2 GB at the moment a slot starts. With `LANE_SLOTS=2`, two lanes (select-parts in slot 0, disclosure's retry loop in slot 1) both saw 12 GB free, both passed, both started a 6.8 GB copy, and both died with `No space left on device` (2026-10-22, batch 5). The partial slot-1 scratch kept the template's `.tree-owner` (a long-gone worktree), so it then looked like a stale foreign tree and blocked every later build for over an hour.

## Status / trigger

LANDED. `lane-slot-target.sh reserve|release|copy` (flock on `$Q/disk.lock`, one file per pid under `$Q/disk-reserve/`, dead pids swept); `lane-ssg.sh` and the warm slot copy use them; `check-lane-lock.sh` scenario 9 is red on the old script and green now. The `LANE_SLOTS=1` workaround is no longer needed for this.

## Size

S: `lane-slot-target.sh` reservation plus a `check-lane-lock.sh` case.

## Detail

Fix by construction: Make the check a reservation: under one `flock` on `$Q/disk.lock`, subtract the bytes already reserved by live in-flight copies (one file per pid under `$Q/disk-reserve/`, dropped on exit and ignored when its pid is dead) before comparing, and write this copy's reservation before releasing the lock. Also strip `.tree-owner` from the template copy first (or write the new owner before the copy), so a partial scratch never names another worktree. Add a `check-lane-lock.sh` case: two concurrent reservations on a disk sized for one, the second must refuse.
