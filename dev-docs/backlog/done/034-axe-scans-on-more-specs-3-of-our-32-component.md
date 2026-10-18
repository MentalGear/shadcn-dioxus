---
id: 34
title: "axe scans on more specs — 3 of our 32 component specs run axe vs 12 of their 56; add an axe pass to every…"
status: "Landed 2026-09-03 — 33 of 45 specs now scan (30 new + the 3 pre-existing refactored onto a shared…"
size: "S"
area: tests
updated: 2026-09-03
---

# Row 34: axe scans on more specs — 3 of our 32 component specs run axe vs 12 of their 56; add an axe pass to every…

## Item

axe scans on more specs — 3 of our 32 component specs run axe vs 12 of their 56; add an axe pass to every component spec's default state (and to the gallery, where cross-component landmark duplicates surfaced)

## Status / trigger

**Landed 2026-09-03** — 33 of 45 specs now scan (30 new + the 3 pre-existing refactored onto a shared `playwright/axe.ts` helper), 72 scan-call sites total (up from 4), at load and at each spec's own open/expanded/selected state; plus 4 tier-2 oracle fixtures at states no component spec reaches (every top-layer overlay open, nested dialogs, the form's blocked-submit invalid state, the dashboard email client). Full account: `dev-docs/conformance-harness.md`, "axe (static rules)"

## Size

S

## Detail

From the 2026-09-03 fork comparison; complements the gallery-level oracles (`preview.spec.ts`, hydration-parity). Fixed by construction: row 25 (ContextMenu/Menubar accessible name), plus `SelectList`'s identical (and axe-detectable) gap, a demo-page duplicate landmark (`Navbar`), an unlabelled icon-only close button (`Sheet`), unlabelled `role="img"` avatars (dashboard + sidebar), a duplicate nested `<main>` (dashboard), and two demo headings with no place in the page outline (`Tooltip`, virtual list). Filed, not fixed: rows 36-38 below. A sitewide `color-contrast` exclusion (`CONTRAST_TRACKED_ELSEWHERE`) was necessary — see the harness doc for why it is not a rule-hiding shortcut. **Re-verified landed, 2026-09-19 (batch 3, leftovers lane):** the same source-level scan (axe call sites across all 53 top-level specs, 49/53 scanned, 4 documented exclusions) found no regression
