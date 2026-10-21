---
id: 329
title: "Audit 3: SSR-vs-first-render parity as a cargo test (spike first)"
status: "BACKLOG (owner 2026-10-21: do it if it holds up; spike first)"
size: "M"
area: tests
updated: 2026-10-21
---

# Row 329: Audit 3: SSR-vs-first-render parity as a cargo test (spike first)

## Item

Audit 3: SSR-vs-first-render parity as a cargo test (spike first).

## Status / trigger

**BACKLOG (owner 2026-10-21: do it if it holds up; spike first)**

## Size

M

## Detail

Audit section 2 #3: render each route's component tree with `dioxus_ssr` and with a native VirtualDom's first client render and compare, inside `cargo test`; would have caught rows 204, 215, 252 and 306 and could shrink `first-paint-text`, `first-paint-styles` section 2, `layout-stability` cause 1 and `charts-hydration-stability` (~900 s). Unverified: whether a native VirtualDom runs `use_effect` like the web renderer; the spike answers that before anything is built.
