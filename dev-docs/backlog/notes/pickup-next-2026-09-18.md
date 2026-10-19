---
title: "What a fresh session should pick up next (2026-09-18)"
date: 2026-09-18
kind: note
---

### What a fresh session should pick up next (2026-09-18)

1. **Row 72's follow-ups, where cheap** — the Navigation Menu oracle entry, the Select `scroll_lock=false` fixture, and the `dx`/Playwright process notes cost little and have no dependency on anything still open.
2. **`component-backlog.md`'s remaining catalog items** — Carousel and Chart stay `planned, deferred` on their own stated decisions (an a11y-reference/JS-engine decision for Carousel, a charting-library decision for Chart); neither is blocked on anything landed this session.
3. **Phase 6 RTL (row 13)** still waits on a fixture decision — typeahead (row 11) and the portal/class-prop/`CalendarDayState` trio (row 12) landed this session, RTL did not. **[2026-10-05: RTL landed 2026-09-19 (row 13); its open follow-ups are now row 137, "RTL by construction".]**
4. **The user-gated rows are unchanged**: 4, 6, 14, 15, 22, 23, 28, 29, 30, 33, 40, 41b — none of them were sequenced this session and none became unblocked by it. **[2026-10-05 correction: 23 is fixed (`3633b20`), 29 is resolved, 40 and 41b are closed; the user-gated rows now are 4, 6, 14, 15, 16, 22, 28, 30, 33 and the 130/131 filings.]**
5. **Re-check upstream at session start** (row 66) — a read-only fetch (`git fetch https://github.com/DioxusLabs/components.git main:refs/research/upstream-main`), since `main` moved for the first time in ten weeks this round and may move again.
