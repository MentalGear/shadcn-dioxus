---
id: 46
title: "Every SSG-prerendered component page is the \"Component not found\" shell. dx build --ssg prerenders…"
status: "Closed 2026-09-19 (batch 3) — fixed by construction, executed, and verified against a real…"
size: "M"
area: build
updated: 2026-09-19
---

# Row 46: Every SSG-prerendered component page is the "Component not found" shell. dx build --ssg prerenders…

## Item

**Every SSG-prerendered component page is the "Component not found" shell.** `dx build --ssg` prerenders `/component/?name=X&` as `<main class="dx-component-demo-not-found">` for every X (confirmed on the deployed bytes with `curl`, 2026-09-03) — a query-string route is not enumerable at build time, so the served HTML is the not-found branch and the client corrects it on hydration

## Status / trigger

**Closed 2026-09-19 (batch 3)** — fixed by construction, executed, and verified against a real production build

## Size

M

## Detail

Construction: the path-segment route this row's own text anticipated — `/component/<name>/` is now enumerable by `Routable::static_routes()`, so `dx build --ssg` prerenders the REAL page for every component instead of the not-found branch; the legacy `?name=X&` links still resolve client-side (full account, dioxus-source citations and commits: row 22's own closure). **2026-09-04:** reinforced, not landed that session (see row 22's own 2026-09-04 addendum). **2026-09-19, verified by execution, not just architecture:** a production-shape build (release+ssg+fullstack) produced **71 routes, 0 shipped not-found shells** (`grep -rl dx-component-demo-not-found` against the built site — zero matches), and hydration-parity's now-extended Rule 5 ("no prerendered page serves the not-found shell") passes across all of them. **One small, unowned defect found and fixed at the same integration** (the SSG lane's own finding, not this row's original claim): the field demo's `<label>` elements carried a literal `class="dx-field-label"` attribute AND a spread `class="dx-label"` from `props.attributes` — two same-named attributes on one element is a WHATWG duplicate-attribute parse error (the same class `conformance-harness.md`'s hydration-parity Rule 4 paragraph already documents; server and client tie-break it differently). Fixed with `merge_attributes` (`5fc1439`, `preview/src/components/label/component.rs`), confirmed by Rule 4/4-extended now passing everywhere, not just the one field demo that surfaced it
