---
id: 485
title: "Retire data-state: our CSS styles from ARIA and presence attributes"
status: "BACKLOG, owner decision 2026-10-29. Depends on row 484, which converts the valued booleans to…"
size: "M-L"
area: styling
updated: 2026-10-29
---

# Row 485: Retire `data-state`

## Item

Our own stylesheets and specs select on the ARIA and presence attributes, not on `data-state`, and `data-state` is no longer emitted (owner decision 2026-10-29: Base UI's state-attribute vocabulary is the long-term model). This is a user-facing change: a consumer who selects `data-state` in their own CSS or tests must move. The model is in [`dev-docs/state-attributes.md`](../state-attributes.md); the presence conversion it depends on is [row 484](./484-convert-remaining-valued-boolean-state-attributes-to-presence.md).

## Status / trigger

BACKLOG, owner decision 2026-10-29. Depends on row 484, which converts the valued booleans to presence attributes first.

## Size

M-L

## Detail

1. **Re-count first (S).** The numbers come from row 481's scan: 94 `data-state` selectors in 29 preview CSS files and 88 Playwright files. Re-count both before the lane starts, and record the counts in the lane's report.
2. **Move the selectors.** Each preview stylesheet selector on `data-state` moves to an ARIA attribute where one carries the state (`aria-checked`, `aria-expanded`, `aria-pressed`, `aria-selected`, `aria-disabled`), and otherwise to the presence attribute (`data-open`, `data-checked`, `data-active`, ...). The Playwright specs that select on `data-state` move with them in the same change.
3. **Stop emitting (M).** `state_attrs` keeps only the ARIA and Base UI names. The `state(value)` helper, which writes the parts that have no Base UI counterpart (Avatar `loading|loaded|error|empty`, the NavigationMenu viewport `visible|hidden`, a Resizable handle `dragging|idle`, the Sidebar `expanded|collapsed`), needs a new name for each of them. Those names are an owner decision and the lane asks before it chooses them. The `tier3-radix` oracle compares against Radix's `data-state`; whether it keeps that comparison is an owner question that this row does not decide.
4. **The gate.** `scripts/check-state-attributes.sh` fails any `data-state` emission, in `state_attrs.rs` as well.
5. **User-facing change.** Say so in the component docs and in the release notes. The repo tracks no changelog file today, so the lane names the release-notes location and the owner confirms it before the lane writes there. The release notes carry an old-to-new selector table, built from the vocabulary table in `dev-docs/state-attributes.md`. The docs pages whose examples select on `data-state` move to the new selectors.
6. **Proof.** The full Playwright suite on a release build. The install check, `scripts/registry-install-smoke.sh`, because `component.rs` and `style.css` change. The docs pages' examples, checked on the rendered pages with the new selectors.
