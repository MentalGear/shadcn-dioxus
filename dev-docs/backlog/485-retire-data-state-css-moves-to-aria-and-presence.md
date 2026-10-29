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
2. **Move the selectors.** Each preview stylesheet selector on `data-state` moves to the selector kind shadcn's `base-nova` classes use for that state (owner decision 2026-10-29): a presence attribute (`data-open`, `data-checked`, `data-active`, ...) in nearly all cases, and an ARIA attribute (`aria-pressed` for Toggle, `aria-invalid` for fields) only where base-nova itself styles from ARIA. Components with no base-nova counterpart use presence attributes. The Playwright specs that select on `data-state` move with them in the same change.
3. **Stop emitting (M).** `state_attrs` keeps only the ARIA and Base UI names. The `state(value)` helper, which wrote the parts that have no Base UI counterpart, is removed, and those parts take presence names by Base UI's convention: Avatar `loading|loaded|error|empty` becomes `data-loading`, `data-loaded`, `data-error`, `data-empty` (verify against Base UI 1.9.0 Avatar, which is not installed in this checkout); the NavigationMenu viewport `visible|hidden` becomes `data-open`/`data-closed`; a Resizable handle `dragging|idle` becomes `data-dragging` (as Slider); the Sidebar `expanded|collapsed` becomes `data-open`/`data-closed` (as Collapsible). The `tier3-radix` oracle compares against Radix's `data-state`; its assertions move to the new attributes in the same change that stops emitting `data-state`, as do the specs (owner decision 2026-10-29).
4. **The gate.** `scripts/check-state-attributes.sh` fails any `data-state` emission, in `state_attrs.rs` as well.
5. **User-facing change.** Document it in the component docs and in `dev-docs/state-attributes.md`, which carries an old-to-new selector table built from its vocabulary table. No release has happened yet, so no release note is needed for this change (owner decision 2026-10-29). The docs pages whose examples select on `data-state` move to the new selectors. After this row, start a `CHANGELOG.md` at the repo root (owner 2026-10-29) for changes from the first release on.
6. **Proof.** The full Playwright suite on a release build. The install check, `scripts/registry-install-smoke.sh`, because `component.rs` and `style.css` change. The docs pages' examples, checked on the rendered pages with the new selectors.
