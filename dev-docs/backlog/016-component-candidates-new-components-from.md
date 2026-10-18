---
id: 16
title: "Component candidates — new components from upstream PRs, the fork network, and catalog gaps"
status: "A decision to expand the catalog beyond shadcn (2026-10-05: the in-catalog part is done, see the…"
size: "S–XL per item"
area: upstream
updated: 2026-10-05
---

# Row 16: Component candidates — new components from upstream PRs, the fork network, and catalog gaps

## Item

**Component candidates** — new components from upstream PRs, the fork network, and catalog gaps

## Status / trigger

A decision to expand the catalog beyond shadcn (**2026-10-05: the in-catalog part is done**, see the correction at the end of the notes)

## Size

S–XL per item

## Detail

Ranked queue with per-item triggers in [`component-backlog.md`](../component-backlog.md); three items are ready the moment expansion is wanted (ComboboxMulti, Field+Alert, Password Toggle Field). **2026-09-18 addendum:** the cheap-primitives wave's five catalog items — Table, Data Table, Drawer, Navigation Menu, Resizable — were taken this session; see their own rows in `component-backlog.md` for each construction and rule source (all now "present — landed 2026-09-18"). **2026-09-19 (Chart round) addendum:** the Chart half of row 86's "awaiting the user" pair is now decided and landed (row 88) — in-repo pure-RSX SVG engine, MVP (Area/Bar/Line). Carousel remains `planned, deferred`, still awaiting its own engine decision (CSS scroll-snap in Rust vs. embla-via-JS-interop) per row 86's own account below; its execution plan was approved by the user 2026-09-19 but explicitly not started, pending a fresh go-ahead once this Chart round is merged **2026-10-05 correction (stale-status pass):** the sentences above that Carousel "remains `planned, deferred`" and that Field+Alert, ComboboxMulti and Password Toggle Field are "ready" are history. Carousel landed 2026-09-20 (row 91); Field and Alert landed 2026-09-04 (row 27); ComboboxMulti and Password Toggle Field are "not planned (in this repo)" (not in shadcn; `component-backlog.md`) and stay behind this row's user-gated scope call. Open in the catalog now: `Questionnaire` (row 136), `Direction`'s docs page (inside row 137), Toast depth (row 138) and the shadcn parity wave (row 155); everything else is present (61 of 63 buildable).
