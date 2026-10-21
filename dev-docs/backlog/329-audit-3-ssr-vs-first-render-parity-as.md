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

**Verdict 2026-10-21 (Round 4 thread, lane/ssr-parity-spike 6e037bf4; findings in `dev-docs/research/row-329-ssr-first-render-parity-spike.md` on that branch): holds up, narrower than the audit said.** A native VirtualDom in `cargo test` renders the server frame (`rebuild_in_place` + `dioxus_ssr`) and the client's first frame (`render_immediate` until three quiet rounds, effects run) for 503 demo variants (~9 s) and 94 routes through the real Router (~48 s debug); current divergences are pinned in a reasoned baseline. Catches row 306 (reproduced at fb8c0b65^) and the markup half of row 204; misses row 215's measurement part, row 252 (the eval bridge) and the inline-style interpreter quirk (`check-dynamic-style-shorthand.sh` covers that). Savings well under the audit's ~900 s: `first-paint-text` (~180 s) can retire after a green round; `first-paint-styles` section 2 and `layout-stability` cause 1 shrink partly; `charts-hydration-stability` stays. New findings filed as rows 332 and 333.
