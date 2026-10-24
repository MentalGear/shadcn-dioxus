---
id: 410
title: "History comments: third fresh-seed cold read"
status: "LANDED (merge queue batch 16, lane/history-r8). Trigger for reopening: a cold read above 1 strict…"
size: "S"
area: docs
updated: 2026-10-24
---

# Row 410: History comments: third fresh-seed cold read

## Item

Confirm that about no history comments are left after rounds 6 and 7, including the lines batch 16 added.

## Status / trigger

LANDED (merge queue batch 16, lane/history-r8). Trigger for reopening: a cold read above 1 strict finding per 1,000 comment lines.

## Size

S

## Detail

Seed 8410: all 887 comment lines batch 16 added plus 5,000 random lines of 61,791 gave 2 strict findings in 5,887 lines (0.34 per 1,000, round 7 had 2.2). Fixed those, two more of the same form found by grep, and two borderline ones. `scripts/comment-scan.py` now flags an observation's frequency beside an observation or failure word, "is not the variable" and "was ruled out", each with must-flag and must-not-flag sentences; the round is recorded in `dev-docs/history-in-comments.md`.
