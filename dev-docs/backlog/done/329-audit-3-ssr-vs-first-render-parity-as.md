---
id: 329
title: "Audit 3: SSR-vs-first-render parity as a cargo test (spike first)"
status: "LANDED 2026-10-23"
size: "M"
area: tests
updated: 2026-10-23
---

# Row 329: Audit 3: SSR-vs-first-render parity as a cargo test (spike first)

## Item

Audit 3: SSR-vs-first-render parity as a cargo test (spike first).

## Status / trigger

**LANDED 2026-10-23**

## Size

M

## Detail

Audit section 2 #3: render each route's component tree with `dioxus_ssr` and with a native VirtualDom's first client render and compare, inside `cargo test`; would have caught rows 204, 215, 252 and 306 and could shrink `first-paint-text`, `first-paint-styles` section 2, `layout-stability` cause 1 and `charts-hydration-stability` (~900 s). Unverified: whether a native VirtualDom runs `use_effect` like the web renderer; the spike answers that before anything is built.

**Verdict 2026-10-21 (Round 4 thread, lane/ssr-parity-spike 6e037bf4; findings in `dev-docs/research/row-329-ssr-first-render-parity-spike.md` on that branch): holds up, narrower than the audit said.** A native VirtualDom in `cargo test` renders the server frame (`rebuild_in_place` + `dioxus_ssr`) and the client's first frame (`render_immediate` until three quiet rounds, effects run) for 503 demo variants (~9 s) and 94 routes through the real Router (~48 s debug); current divergences are pinned in a reasoned baseline. Catches row 306 (reproduced at fb8c0b65^) and the markup half of row 204; misses row 215's measurement part, row 252 (the eval bridge) and the inline-style interpreter quirk (`check-dynamic-style-shorthand.sh` covers that). Savings well under the audit's ~900 s: `first-paint-text` (~180 s) can retire after a green round; `first-paint-styles` section 2 and `layout-stability` cause 1 shrink partly; `charts-hydration-stability` stays. New findings filed as rows 332 and 333.

## Resolution (branch claude/ssr-parity-narrowing, 2026-10-23)

The native test (`preview/src/ssr_parity.rs`, batch 1) has been green in every merge batch since (batches 1-11), so the browser halves it subsumes were narrowed. Nothing was deleted outright: each spec keeps a real-browser hydration of the class's own subjects, and `FIRST_PAINT_ROUTES=all` (or `LS_STEP`) restores the old sweep.

- `first-paint-text`: 84 tests -> 5. Keeps Select, Combobox (row 306), Tag Group (row 332) and the not-blind test, plus a test that the kept routes exist. The native test reads a superset (text, `value`, `aria-label`, `class`, `style`, `data-*`) on every demo and component page; new fixture `a_value_written_by_an_effect_is_seen` pins the spec's `input.value` read.
- `first-paint-styles` section 2: 84 routes -> 25 (`SECTION_2`): every `CLIENT_MEASURED` route (a browser measurement is the cause the native test cannot see; the CLS bound and late-sheet check run there) plus Field, Select, Combobox and Tag Group for the style diff. Markup-caused style differences fail in `cargo test`; late sheets are sections 1 and 3. File 175 tests -> 117.
- `layout-stability` cause 1: `/component/avatar/` light 34 widths -> 3 (360, 768, 1280), dark 5 -> 1 (1024). Whether the avatar's pinned `empty` -> `loading` change moves a box is a property of its sheet, not of the width; `/` keeps its full sweep with twelve avatars.

Red evidence (mutation run, reverted): Combobox's input `value` and Resizable's `flex-basis` moved behind a first effect fail `cargo test -p preview ssr_parity` on `combobox/controlled`, `resizable/{main,rtl,vertical}`, `/component/combobox/` and `/component/resizable/` (2 failed, 8 passed); unmutated 10/10 green in 77 s.
