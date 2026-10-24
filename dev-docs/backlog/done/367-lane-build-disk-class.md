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

**Root cause, measured.** The warm template (6.8 GB) is `server-dev` 3.0 GB, `wasm-dev` 2.4 GB and ~1.4 GB of dx output; its two
`incremental/` dirs per target total **3.0 GB (44 %)**, written when the template was first built with incremental compilation on.
Every lane command runs with `CARGO_INCREMENTAL=0`, so cargo neither reads nor writes them, yet each lane build copied them and
`lane-slot-target.sh`'s guard (row 356) wanted 10.4 GB free (copy size plus 20 % plus 2 GB). With the container's ~22 GB, the
permanent `deploy` (3.5 GB) and `gates` (3.1 GB) trees left ~10 GB, so the guard refused lane builds in three merge batches in a
row on 2026-10-22 and the main loop deleted trees by hand. The class is "a copied tree's size is not what the consumer reads".

**Landed 2026-10-23 (lane `gatetools`, branch `claude/lane-template-slim`).**

- `lane-slot-target.sh copy` streams the source through `tar --exclude='./*/incremental' --exclude='./*/*/incremental'` (and
  `.tree-owner`), keeping modes, mtimes (cargo freshness, row 216) and hard links, and `check_disk` sizes with
  `du --exclude=incremental`; one function serves `lane-ssg.sh`, the lazy slot-1 warm copies and the guard, so they cannot
  disagree. `LANE_COPY_INCREMENTAL=1` restores the full copy. Effect: 6.8 GB to 3.8 GB per lane build, needing 6.6 GB free instead
  of 10.4 GB. Proven on a real `LANE_SLOTS=1 scripts/lane-ssg.sh` build with 6.7 GB free (the old guard would have refused): 0
  dependency recompiles, `SSG complete` after 393 s.
- `scripts/lane-target.sh slim [--apply]` lists (deletes) the `incremental` dirs in every tree under the root, skipping a tree
  stamped `.keep-incremental`; `--apply` takes every slot lock and the legacy `ssg.lock` non-blocking for the whole delete (a `tar`
  copy reading a tree mid-delete fails) and, if any is held, prints which, deletes nothing and exits 1. `create` uses the same slim
  copy (`LANE_CARGO_INCREMENTAL=1` keeps the full copy and stamps the tree); `prune --apply` runs `slim` afterwards.
- `check-lane-lock.sh` scenarios 10 and 11 prove each promise (dirs absent at both depths, deps kept with equal mtime and an intact
  hard link, the opt-out, the guard on a disk that fits only the slim copy, `slim` without `--apply` deleting nothing, `--apply`
  refusing while a real holder has slot 0). `lane-rules.md`'s new-container setup ends with `slim --apply`.

**One step is the main loop's:** run `scripts/lane-target.sh slim --apply` once on its container with the queue idle (frees ~3.0 GB
of the template for good). **Not done:** a persistent per-slot build tree (no copy at all, one 3.8 GB tree per slot): it was
rejected in the `lane-ssg.sh` header because a target shared across worktrees failed 3 of 5 builds with "stylesheet ... not in dx's
asset manifest", so it needs a verified reset of `dx/` and the `build/` fingerprints per build; revisit only if 6.6 GB free is
still too much.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/367-lane-build-disk-class.md`.*
