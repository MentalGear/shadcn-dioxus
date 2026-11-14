---
id: 514
title: "The per-row `mounted` handle has no unconditional owner: keep it with a true reason, or replace it"
status: "BACKLOG. Trigger: the focus-write witness on a real set_focus site (row 446, open item), which…"
size: "S for the decision and the allow row.…"
area: primitives
updated: 2026-11-14
---

# Row 514: the per-row `mounted` handle has no unconditional owner

## Item

`scripts/check-hot-events.allow.tsv` excuses `mounted` handlers in this row's name (owner 2026-11-14; the entry named
[row 451](./done/451-port-3-sc-shared-collection-and-activity.md) before), with the reason that "a container handle and a
batched observe replace the per-row ones". Row 451 moved the observe to the bridge and kept the per-row `onmounted` of
`collection.rs`: it stores the `MountedData` handle that the roving hub's focus writes use and reads nothing from the
page. The removal is claimed only by [row 462](./462-port-roving-focus-78-set-focus-sites.md), which is conditional (it
is built only if the focus-write witness is red) and counts focus writes, not this handle. If the witness is green,
nothing owns the handle and the allow row's reason stays untrue. Other `onmounted` sites exist (`color_picker`,
`checkbox`, `command`, `context_menu`, `nav_link`, the menu items).

## Status / trigger

BACKLOG. Trigger: the focus-write witness on a real `set_focus` site
([row 446](./446-phase-0-6-witness-harness-and-the-first-witnesses.md), open item), which decides whether the
`MountedData` write ban and row 462 are built.

## Size

S for the decision and the allow row. The replacement, if the witness is red, is row 462's work.

## Detail

- **If the witness is green:** the per-row handle stays. The allow row's reason is rewritten to what is true (the
  handle is how a focus write reaches its element; it reads nothing and runs once per mount), and the row names no
  future replacement.
- **If the witness is red:** row 462 replaces the focus writes; this row then removes the per-row handle with them and
  the allow row shrinks to the sites that remain.
- **The allow table is the owner's.** Re-pointing the `mounted` entry from row 451 to this row, and either rewrite of
  its reason, is the owner's edit or needs the owner's approval. Until then row 451 stays open as the entry's owner.
- **Files owned:** `scripts/check-hot-events.allow.tsv` (the `mounted` entry, with the owner's approval),
  `primitives/src/collection.rs` (the `onmounted` only, and only if the witness is red).
- **Proof:** `check-hot-events` green with the entry naming an open row whose reason matches the code.
- Source: [`research/premise-check-447-450-2026-11-13.md`](../research/premise-check-447-450-2026-11-13.md), "The
  per-row `mounted` handle".
