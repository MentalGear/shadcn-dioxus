---
id: 387
title: "Standing docs-clarity review: a technical-writer pass over every component's docs and props for implicit behaviour"
status: "STANDING. Run after every round that adds a component or changes a public prop, and before row…"
size: "M"
area: docs
updated: 2026-10-24
---

# Row 387: Standing docs-clarity review

## Item

A recurring review, run by subagents with a world-class technical-writer persona, over every component's `docs.md`, its variant `notes.md` files and its props (the `#[derive(Props)]` structs in `preview/src/components/<name>/component.rs` and `primitives/src/<name>.rs`, which `preview/build.rs` renders as the props reference). It looks for what a reader cannot learn from the page without reading the source:

- **Implicit behaviour:** passing one prop switches on another behaviour (a header row that pins itself because `render_header` is set).
- **Settings changed by side effect:** a prop that also changes layout, focus, ARIA or another prop's meaning.
- **Undocumented defaults:** a default that matters (direction, size, open state, sticky, virtualization thresholds) and is not stated.
- **Capability limits stated nowhere:** a `bool` where a count or a list is the natural shape (pin one column only), an axis that is not supported.
- **Discoverability:** a name that differs from what shadcn/ui, Radix or TanStack users search for, with no alias or sentence that maps it.
- **Missing demo:** a supported mode with no variant that shows it (horizontal scrolling on ScrollArea).

## Status / trigger

STANDING. Run after every round that adds a component or changes a public prop, and before row 383's review board. Each run writes a ranked findings report and proposes which findings become backlog rows; the owner picks.

## Size

M

## Detail

- **How to run:** four Sonnet subagents in parallel, read-only, each on a disjoint quarter of `preview/src/components/` (alphabetical), each with the brief below. The main loop merges, de-duplicates, ranks and verifies the top findings against the source before they count.
- **Brief (give it verbatim plus the component list):** "You are a world-class technical writer who has documented shadcn/ui, Radix and TanStack. For each component, read its `docs.md`, its variants' `notes.md` and `mod.rs` demos, and its props structs. Report every place where behaviour depends on something the page does not say: implicit behaviour, side effects of a prop, unstated defaults, limits of a prop's shape, names a shadcn/Radix/TanStack user would not find, and supported modes with no demo. Per finding: component, file:line of the code that proves it, what the docs say now (or nothing), what a reader would assume, severity (high = a reader builds the wrong thing; medium = a reader has to read source; low = polish), and the one-sentence fix (doc sentence, demo, rename or alias, API change). Verify each claim in the code; no guesses."
- **Rules for the subagents:** read-only; no builds. If a finding needs a look at the rendered page, `dx serve` (lane-rules.md, "The fast loop"); any change that follows from a finding is its own lane, verified with `pnpm run test:changed`.
- **Output:** a report under the project's shared files and, for accepted findings, new rows (docs-only findings batch into one row per area).
