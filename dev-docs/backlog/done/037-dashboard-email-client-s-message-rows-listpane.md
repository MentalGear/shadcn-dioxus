---
id: 37
title: "Dashboard email client's message rows (ListPane, preview/src/dashboard/views/email_client/list_pane.rs)…"
status: "Landed 2026-09-04"
size: "M"
area: tests
updated: 2026-09-04
---

# Row 37: Dashboard email client's message rows (ListPane, preview/src/dashboard/views/email_client/list_pane.rs)…

## Item

**LANDED 2026-09-04, verified by execution (Chromium).** Dashboard email client's message rows (`ListPane`, `preview/src/dashboard/views/email_client/list_pane.rs`) rendered `role="button"` on a `div` that also contained further-interactive controls (per-row star/flag toggle buttons) — axe `nested-interactive`, 6 nodes flagged, left RED and documented in-line (`oracle/tier2-html/touch-focus-zoom.spec.ts`'s "dashboard email client: default state" axe test). Restructured so the accessibility tree matches what a user actually gets: the row container is a plain `div` again (keeping only a mouse-level `onclick` for empty-space clicks, which carries no role and so advertises nothing to assistive technology); the "open thread" action is a real `<button class="ec-row-open">` wrapping subject/sender/snippet/tags, with the star/trash toggles pulled out as its siblings rather than its descendants. The button gets an explicit `aria-label` of sender + subject — left implicit, its computed name would have included the entire preview snippet, an accessible name no one can use. `<button>` is not a `div`, so the row's grid layout needed an explicit UA-styling reset in `email_client.css` (display, padding, border, background, font, line-height, text alignment) plus a `:focus-visible` ring on the shared `--dx-ring` token. Both dashboard axe scans in `touch-focus-zoom.spec.ts` are green, 57/57 in that file — the documented-red comment was removed rather than reworded, since with row 38's landmarks already fixed this was the page's last violation. Combined with row 38, that route now has zero axe violations

## Status / trigger

Landed 2026-09-04

## Size

M

## Detail

Found 2026-09-03 building row 34's axe coverage; a real structural markup question, not a small/mechanical fix, so left for a deliberate pass rather than a same-round patch
