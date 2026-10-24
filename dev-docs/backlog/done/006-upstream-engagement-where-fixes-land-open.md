---
id: 6
title: "Upstream engagement — where fixes land (open decision #1) + talking to upstream/fork authors (decision #4)"
status: "CLOSED 2026-10-22: the owner decided on no upstream engagement. Local fixes and workarounds stay as…"
size: "S to start"
area: upstream
updated: 2026-10-22
---

# Row 6: Upstream engagement — where fixes land (open decision #1) + talking to upstream/fork authors (decision #4)

## Item

Upstream engagement — where fixes land (open decision #1) + talking to upstream/fork authors (decision #4)

## Status / trigger

**CLOSED 2026-10-22: the owner decided on no upstream engagement.** Local fixes and workarounds stay as they are; nothing is filed upstream.

**User decision**

## Size

S to start

## Detail

The 4.1 question is now evidence-backed and effectively a ready PR description; upstream `main` idle since 2026-06; consolidation with dignifiedquire/sarendipitee worth more than further cherry-picks. Drafted 2026-09-03, not filed — user decision: [`dev-docs/issues/drafts/dioxus-components-upstream-proposal.md`](../../issues/drafts/dioxus-components-upstream-proposal.md). **2026-09-19 (Chart round) addendum:** a second, distinct upstream question, orthogonal to this row's own DioxusLabs/components fork-consolidation question: `primitives/src/chart/engine/` (the pure scale/curve/stack/geometry/table math, deliberately built with no `dioxus` types and no `use crate::` of this crate's other modules) is a candidate to offer to the `dioxus-community/dioxus-charts` project (0.4.0, MIT OR Apache-2.0, right architecture but zero a11y/theming and only bar/line/pie, per `dev-docs/archive/fork-research/chart-forks-2026-09-19.md` §1) once this repo's own Chart component has landed and proven itself (oracle green, deployed) — not before. Needs the user's own GitHub account, or a session with that repository attached (this session is scoped to `MentalGear/dioxus-components` only). Tracked as its own new row (88), not folded into this row's own DioxusLabs/components-specific text
