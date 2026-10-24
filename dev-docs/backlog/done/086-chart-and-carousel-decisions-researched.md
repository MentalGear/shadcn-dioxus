---
id: 86
title: "Chart and Carousel — decisions researched, awaiting the user (SUPERSEDED: both decided and landed, rows 88…"
status: "SUPERSEDED 2026-09-26 (Chart row 88, Carousel row 91) -- was: Both fully researched 2026-09-19…"
size: "— (research done; M–XL to build, per…"
area: charts
updated: 2026-09-26
---

# Row 86: Chart and Carousel — decisions researched, awaiting the user (SUPERSEDED: both decided and landed, rows 88…

## Item

Chart and Carousel — decisions researched, awaiting the user (**SUPERSEDED**: both decided and landed, rows 88 and 91)

## Status / trigger

**SUPERSEDED 2026-09-26 (Chart row 88, Carousel row 91)** -- was: Both fully researched 2026-09-19 (batch 3); **user decision needed** — see `dev-docs/research/chart-2026-09-19.md` §8 and `dev-docs/research/carousel-2026-09-19.md` §8

## Size

— (research done; M–XL to build, per each report's own effort section)

## Detail

**2026-09-26 (audit correction): STALE/SUPERSEDED** — both decisions were made and shipped: Chart at row 88, Carousel at row 91. Kept for its research-history value only. **Carousel:** the backlog's own prior "no a11y reference exists" claim (`dev-docs/archive/rounds/component-backlog.md`'s Carousel row) is corrected — a full, two-example APG reference DOES exist (`content/patterns/carousel/`, pinned commit `7e4034b`, the same commit already vendored elsewhere in this repo); only the tier-3 (Radix) reference is genuinely absent, confirmed by cloning and grepping, not by memory. Engine decision still open: (a) a CSS scroll-snap engine written in Rust (RECOMMENDED — IntersectionObserver as the primary index signal, `scrollend`/`scrollsnapchange` for progressive enhancement, zero new dependencies, RTL-favorable by construction) vs. (c) embla via a JS-interop bridge (this repo's first true npm runtime dependency; `component.json` has no npm-dependency field today). **Chart:** shadcn's own Chart layer is a thin, React-specific children-as-configuration wrapper over `recharts`; a Dioxus port must be data-driven (props) instead, which is also the natural shape of every other charting API surveyed. Recommended: a new, in-repo, pure-RSX SVG chart engine (`primitives/src/chart/`) — satisfies hydration parity and real-DOM accessibility by construction, needs no new dependency, confirmed to compile to wasm32 in a scratch check. Alternatives surveyed and not recommended: `charts-rs` (opaque SVG string output, needs `dangerous_inner_html`), `dioxus-charts` (real RSX but bar/line/pie only, no a11y/theming), `plotters`/`poloto` (same string-output ceiling). Both reports are committed **verbatim** (their content, not their recommendation) at the paths above, each carrying its own full options table, design sketch, effort estimate, and a numbered "Decisions needed from the user" section — **no decision has been made by this docs pass**; `dev-docs/archive/rounds/component-backlog.md`'s own Carousel/Chart rows now point at these reports rather than restate them. **2026-09-19 (Chart round) update:** Chart's own decision is now made and landed — see row 88. Carousel's plan (CSS scroll-snap engine, APG pattern vendored as the tier-1 reference, loop/autoplay deferred, root arrow keys + RTL swap, tablist variant fast-follow) was approved by the user the same day, but execution was explicitly deferred: the user asked to be asked again for the go-ahead once this Chart round is merged. `dev-docs/archive/rounds/component-backlog.md`'s Carousel row updated to "plan approved 2026-09-19, execution pending"
