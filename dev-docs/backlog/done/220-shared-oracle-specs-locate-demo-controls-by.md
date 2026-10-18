---
id: 220
title: "Shared oracle specs locate demo controls by their visible demo text, so every demo that mirrors shadcn more…"
status: "LANDED 2026-10-18 (lane specs 4647fc12), merged on the session branch; deploy pending -- was…"
size: "S-M"
area: tests
updated: 2026-10-18
---

# Row 220: Shared oracle specs locate demo controls by their visible demo text, so every demo that mirrors shadcn more…

## Item

Shared oracle specs locate demo controls by their visible demo text, so every demo that mirrors shadcn more closely breaks them

## Status / trigger

**LANDED 2026-10-18 (lane specs 4647fc12), merged on the session branch; deploy pending** -- was: **Scheduled 2026-10-15 in `dev-docs/research/next-round-plan-2026-10-15.md`** (set up, waiting for the owner's go). **OPEN 2026-10-15 (by-construction fix proposed)**

## Size

S-M

## Detail

Three rounds this session: row 212's labelled demos broke `keyboard-matrix`, `main-thread`, `rtl` and `theme-preset-contrast`, and the toggle/toggle-group demos of row 155 group 3 broke the same files again (group 3 moved the shared specs to positional locators). **Construction:** shared oracle specs locate a demo's controls by role plus position inside the demo frame (or by a stable data hook emitted by the demo frame), never by demo copy; add a candidate gate or lint over `playwright/oracle` for text locators on demo controls, with a reasoned allowlist. Subsumes the three instances; per-component specs that assert their own visible labels are unaffected.
