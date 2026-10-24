---
id: 250
title: "A container reset to a fresh clone loses every unpushed lane commit and all session scratch"
status: "LANDED (construction). dev-docs/lane-rules.md \"Nothing lives only in the container\" (line 165)…"
size: "S"
area: infra
updated: 2026-10-24
---

# Row 250: A container reset to a fresh clone loses every unpushed lane commit and all session scratch

## Item

A container reset to a fresh clone loses every unpushed lane commit and all session scratch

## Status / trigger

**LANDED (construction).** `dev-docs/lane-rules.md` "Nothing lives only in the container" (line 165) makes every lane push `lane/<name>` after every commit and keeps briefs in the repo; the lost lane lock is back (`scripts/lane-lock.sh`, `scripts/check-lane-lock.sh`) and the other lost work is tracked by its own rows (152a done, 242/115 fixes landed).

## Size

S

## Detail

On 2026-10-19 ~08:20 UTC the session container came back as a fresh clone (the third restart that day; the first two
kept the disk). Gone: `/home/user/tgt` (warm build targets, baselines `pw-n4.fails`, snapshots, reports such as the
2026-10-18 backlog review), `/home/user/wt` (every lane worktree), every unpushed commit and every subagent transcript.
Lost work: lane lock (row 221/216/92: the FIFO `scripts/lane-lock.sh` queue with two slots, proven on real runs, never
pushed), lane baseline-red (rows 242/115 fixes), lanes suite-speed (152a) and specs2 (164/241) in progress, and the
main loop's round-0 bookkeeping (closing 18 reviewed rows, rows 247-249). Everything already pushed survived (PR #101,
#102, #103). Construction (owner 2026-10-19): `dev-docs/lane-rules.md` "Nothing lives only in the container" -- lanes
push `lane/<name>` after every commit, the main loop backs up its unpushed commits on `lane/main-loop`, and briefs live
in the repo, not in session scratch. The lost lanes are redone from the reports in the main conversation.
