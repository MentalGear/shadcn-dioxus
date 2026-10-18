---
id: 225
title: "Email-client demo: a view change keeps the previous view's scroll position"
status: "LANDED 2026-10-18 (lane demos 8c288999), merged on the session branch; deploy pending -- was…"
size: "S"
area: misc
updated: 2026-10-18
---

# Row 225: Email-client demo: a view change keeps the previous view's scroll position

## Item

Email-client demo: a view change keeps the previous view's scroll position

## Status / trigger

**LANDED 2026-10-18 (lane demos 8c288999), merged on the session branch; deploy pending** -- was: **Scheduled 2026-10-18 in `dev-docs/research/next-round-plan-2026-10-15.md` (lane demos)** -- was: **BACKLOG (owner 2026-10-15)**

## Size

S

## Detail

Owner request: in the dashboard email client demo (`preview/src/dashboard/views/email_client/`), switching views (folder, message, list) should either reset the scroll to the top, or, if the user was already in that view before, restore their saved position. **Construction (proposed):** one per-view scroll memory keyed by view id in the email client's state (saved on leave, restored on enter, top for a first visit), applied to the view's own scroll container (not the window), so every view follows the same rule; spec: scroll a list, open a message, go back -> same position; open a different folder first time -> top.
