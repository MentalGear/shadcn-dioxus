---
id: 135
title: "The computed-style snapshot cannot see closed overlays or hover/focus/open states, so phase C's proof has…"
status: "LANDED 2026-10-12 -- playwright/computed-style-snapshot.spec.ts + new computed-style-states.ts: an…"
size: "M"
area: theme
updated: 2026-10-12
---

# Row 135: The computed-style snapshot cannot see closed overlays or hover/focus/open states, so phase C's proof has…

## Item

The computed-style snapshot cannot see closed overlays or hover/focus/open states, so phase C's proof has holes

## Status / trigger

**LANDED 2026-10-12** -- `playwright/computed-style-snapshot.spec.ts` + new `computed-style-states.ts`: an OPEN-STATE section and a HOVER section beside base/pseudo/focus, captured as deltas against a settled baseline (new or changed elements, plus `::before/::after/::placeholder/::marker/::backdrop`), driven by real pointer/keyboard input. Triggers are derived (`aria-haspopup`, collapsed `aria-expanded`, `popovertarget`, `summary`, tooltip `aria-describedby`, `*-trigger`, first focusable in a `data-state=closed` root; one explicit table for plain-onclick overlay demos), tried by click, right-click, hover, focus; closure verified before the next. Coverage over 81 pages: 170 of 183 triggers opened (35 pages), 388 item hovers inside open overlays, 569 rest-state hover targets. **A/A: two full captures byte-identical** (cmp silent, `compare` 0 values; ~16 min, ~405 MB both schemes); a mutation (one hover colour, one tooltip radius) shows 1256 differing values, all in the new sections. Nondeterminism fixed by construction (redirect stub, content-visibility cards, scroll, hover-intent delays, pointer parked after clicks, document reported hidden so motion-gated timers stop, Google Fonts aborted). Output format 3 (14 more properties). Gaps: 13 triggers not opened (reasons in `dev-docs/dev-loop.md`), sidebar block iframes, `:active`/`::selection`/scrollbars/mobile viewport, a 60-target cap on `/`. Row 145 can now be proven against it. Was: **OPEN, first in the approved refactor chain 135 -> 145 -> 111 (next new session, 2026-10-05)** -- recorded 2026-10-04 from `baseline/REPORT.md` (coverage limits)

## Size

M

## Detail

Closed overlays are not in the DOM, so `alert_dialog`, `command`, `context_menu`, `dialog`, `drawer`, `sheet`, `top_layer` and the chart pages that share `dx-chart*` names capture only the docs shell; `:hover` and open states are not covered (one `:focus-visible` pass exists, `dd87a89`); fonts are not loaded offline. Phase A's 0-difference proof therefore covers the in-DOM, rest-state surface; the overlay and hover surface was covered by the token-layer proof (`tools/gen.py`) only. **Fix by construction before phase C repaints overlays:** extend `playwright/computed-style-snapshot.spec.ts` with an open-state pass (open each overlay via its demo trigger, capture content, close) and a hover pass for the interactive controls, with the same A/A byte-identical gate.
