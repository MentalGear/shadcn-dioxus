---
id: 66
title: "Upstream DioxusLabs/components main moved for the first time since 2026-06-29 — three commits on…"
status: "Landed 2026-09-18 (one cherry-pick); re-check upstream at the start of every session with a…"
size: "S"
area: theme
updated: 2026-09-18
---

# Row 66: Upstream DioxusLabs/components main moved for the first time since 2026-06-29 — three commits on…

## Item

**Upstream `DioxusLabs/components` `main` moved for the first time since 2026-06-29 — three commits on 2026-09-07/08, evaluated 2026-09-18.** `49e3f17` (PR #288): `SidebarProvider`'s inline `style` string written on one line instead of a multi-line raw string, so the SSR'd attribute no longer line-breaks — a cosmetic output fix against code that is identical here (`preview/src/components/sidebar/component.rs`); **adopted** by cherry-pick with authorship (`lifting-from-forks.md` §8). `f6ddd44` (PR #291: `.catch(() => {})` on each `Animation.finished`) and `9a75825` (re-check `open` after the exit-animation await, "fix stale open"): both **not taken** — `recommended-implementations.md` §7 predicted exactly this pair (the swallow lets a stale close task hide a reopened element; the follow-up patches that residual), and this repo's `use_animated_open` already carries the `.catch` plus a per-cycle generation counter that is strictly stronger than upstream's `!open` re-check (verified by reading `lib.rs`, not assumed). `plan.md` open decision #1 ("where fixes land") gets a new data point: upstream is active again, so carrying fixes locally has a real rebase cost after all.

## Status / trigger

Landed 2026-09-18 (one cherry-pick); re-check upstream at the start of every session with a read-only fetch (`git fetch https://github.com/DioxusLabs/components.git main:refs/research/upstream-main`)

## Size

S

## Detail

**2026-09-26 (audit correction):** "re-check upstream every session" is a manual practice, not automated — confirmed no workflow in `.github/workflows/` fetches upstream; it depends on a fresh session actually doing it. Fork network re-checked the same day: `Wervice`'s `a704c517` (row 7) is on no branch of that fork any more — only upstream PR #293's head (`refs/pull/293/head`) still carries it, which is where this session fetched it from
