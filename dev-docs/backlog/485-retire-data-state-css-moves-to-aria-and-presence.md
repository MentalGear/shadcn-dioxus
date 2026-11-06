---
id: 485
title: "Retire data-state: our CSS styles from ARIA and presence attributes"
status: "Owner decision 2026-10-29. Depends on row 484, which converted the valued booleans to presence…"
size: "M-L"
area: styling
updated: 2026-11-06
---

# Row 485: Retire `data-state`

## Item

Our own stylesheets and specs select on the ARIA and presence attributes, not on `data-state`, and `data-state` is no longer emitted (owner decision 2026-10-29: Base UI's state-attribute vocabulary is the long-term model). This is a user-facing change: a consumer who selects `data-state` in their own CSS or tests must move. The model is in [`dev-docs/state-attributes.md`](../state-attributes.md); the presence conversion it depends on is [row 484](./484-convert-remaining-valued-boolean-state-attributes-to-presence.md).

## Status / trigger

Owner decision 2026-10-29. Depends on row 484, which converted the valued booleans to presence attributes first (done 2026-11-06).
Done 2026-11-06 on `lane/base-vocab-484`, awaiting merge (see "Result").

## Size

M-L

## Detail

1. **Re-count first (S).** The numbers come from row 481's scan: 94 `data-state` selectors in 29 preview CSS files and 88 Playwright files. Re-count both before the lane starts, and record the counts in the lane's report.
2. **Move the selectors.** Each preview stylesheet selector on `data-state` moves to the selector kind shadcn's `base-nova` classes use for that state (owner decision 2026-10-29): a presence attribute (`data-open`, `data-checked`, `data-active`, ...) in nearly all cases, and an ARIA attribute (`aria-pressed` for Toggle, `aria-invalid` for fields) only where base-nova itself styles from ARIA. Components with no base-nova counterpart use presence attributes. The Playwright specs that select on `data-state` move with them in the same change.
3. **Stop emitting (M).** `state_attrs` keeps only the ARIA and Base UI names. The `state(value)` helper, which wrote the parts that have no Base UI counterpart, is removed, and those parts take presence names by Base UI's convention: Avatar `loading|loaded|error|empty` becomes `data-loading`, `data-loaded`, `data-error`, `data-empty` (verify against Base UI 1.9.0 Avatar, which is not installed in this checkout); the NavigationMenu viewport `visible|hidden` becomes `data-open`/`data-closed`; a Resizable handle `dragging|idle` becomes `data-dragging` (as Slider); the Sidebar `expanded|collapsed` becomes `data-open`/`data-closed` (as Collapsible). The `tier3-radix` oracle compares against Radix's `data-state`; its assertions move to the new attributes in the same change that stops emitting `data-state`, as do the specs (owner decision 2026-10-29).
4. **The gate.** `scripts/check-state-attributes.sh` fails any `data-state` emission, in `state_attrs.rs` as well.
5. **User-facing change.** Document it in the component docs and in `dev-docs/state-attributes.md`, which carries an old-to-new selector table built from its vocabulary table. No release has happened yet, so no release note is needed for this change (owner decision 2026-10-29). The docs pages whose examples select on `data-state` move to the new selectors. After this row, start a `CHANGELOG.md` at the repo root (owner 2026-10-29) for changes from the first release on.
6. **Proof.** The full Playwright suite on a release build. The install check, `scripts/registry-install-smoke.sh`, because `component.rs` and `style.css` change. The docs pages' examples, checked on the rendered pages with the new selectors.

## Result (2026-11-06)

**Re-count at the start (item 1).** `data-state` selectors: 127 hits in 31 preview CSS files (theme, `main.css` and 29 component sheets), `data-state`
mentioned in 92 Playwright files (396 `toHaveAttribute("data-state", ...)` calls plus selectors and `getAttribute` reads), 14 Rust emitters.

**Emission.** `state_attrs` writes presence attributes only: `open` -> `data-open` / `data-closed`, `trigger_open` adds `data-popup-open`,
`checked` -> `data-checked` / `data-unchecked` / `data-indeterminate`, `pressed` -> `data-pressed`, `active` -> `data-active`,
`panel_hidden` -> `data-hidden`, `selected` -> `data-selected`, `progress` -> `data-indeterminate` / `data-progressing` / `data-complete`.
`state(value)` and `open_presence` are deleted (`open` is now what `open_presence` was). The parts that had no Base UI counterpart take
Base UI's name for the analogous part or a `data-<adjective>` presence name: Avatar `data-loading` / `data-loaded` / `data-error` /
`data-empty` (not verified against Base UI 1.9.0, which is not installed in this checkout), Attachment `data-idle` / `data-uploading` /
`data-processing` / `data-error` / `data-done`, the NavigationMenu indicator `data-open` / `data-closed`, a Resizable handle
`data-dragging`, the Sidebar `data-open` / `data-closed`, the Questionnaire progress checkpoints `data-done`. The Toolbar demo button no
longer sets a `data-state` of its own (`aria-pressed` carries it). `data-empty` and `data-hidden` on Questionnaire items and actions
moved to `state_attrs::presence`; they and `data-filled` / `data-visible` are written empty, not `"true"`.

**Selectors.** Every CSS rule moved to the selector kind of the state: presence attributes in nearly all cases, `[aria-pressed="true"]` for
Toggle, ToggleGroup, Toolbar and the docs sort switch (base-nova's Toggle styles from `aria-pressed`), `[data-hidden]` for an inactive Tabs
panel. `[data-state]` with no value (the scrim's reduced-motion rule, the docs sort switch) became `:is([data-open], [data-closed])` and
`[aria-pressed]`. Specs moved with them: `toHaveAttribute("data-state", "open")` is `toHaveAttribute("data-open", "")`, the probe helpers
(`probeAnimations`, `expectFadeKeepsTransform`) take a `null` value to remove an attribute so a close-state probe removes `data-open` and
sets `data-closed`, and the `getAttribute("data-state")` reads in the oracles (`top-layer`, `keyboard-matrix`, `hover-delay-probe`,
`assert-modal-exit`, `assert-backdrop-fade`, `scrim-reduced-motion`, `date-picker`) read the presence pair. Two shared gates were updated:
`check-theme-baseline` (the scrim rules) and `check-spec-at-rest` (an animated component is one whose sheet animates `[data-open]` /
`[data-closed]`; this brings the Accordion into scope and its three frame-sampling reads are allowlisted with reasons).

**The gate.** `check-state-attributes.sh` fails any `"data-state"` emission in `primitives/src` (except `chart/`) and the preview
`component.rs` files, and in `state_attrs.rs` itself; the vocabulary list gained `hidden`, `loading`, `loaded`, `empty`, `idle`,
`uploading`, `processing`, `done`. The self-test covers the failure. The baseline stays empty.

**What stays, and why.** The chart primitives (`primitives/src/chart/`) keep `data-state="open|closed"` on the tooltip (and the
chart CSS, specs and the `tier3-radix` chart oracle that select it) and Recharts' valued `data-active` on bars and arcs: a
conformance surface with its own harness, outside the gate's scope since row 481. The Radix `tier3-radix` oracle compares behaviour,
not attributes, outside the chart spec, so its other specs moved with the attribute swap above.

**User-facing change.** `dev-docs/state-attributes.md` carries the old-to-new selector table. The component docs pages (avatar, data
table, field, hover card, navigation menu, table, tabs, toggle, virtual grid, carousel) say the new names. `CHANGELOG.md` starts at the
repo root with an Unreleased entry (no release has happened, so no release note; owner 2026-10-29).
