---
id: 396
title: "Derive item `index` from registration order"
status: "LANDED 2026-10-24 (lane derived-index). Keyboard order is document order; index is an optional…"
size: "L"
area: components
updated: 2026-10-24
---

# Row 396: Derive item `index` from registration order

## Item

Accordion (unread), RadioGroup, DropdownMenu, ContextMenu, Menubar, Select (continuing across groups), Tabs, Combobox and NavigationMenu (`content_index`) take a hand-numbered index that must be unique and ascending; duplicates collide silently.

## Status / trigger

**LANDED 2026-10-24 (lane derived-index).** Keyboard order is document order; `index` is an optional override on every
item of the nine components (Accordion has no `index` at all, TabContent neither). Found by row 387's first
docs-clarity review; the evidence (file:line per finding) is in the project's shared files,
`docs-clarity/2026-10-24-docs-clarity-review.md`. Owner approved the API change 2026-10-24 (pre-1.0, breaking).

## Size

L

## Detail

Owner decision before building (public API change). Derive the order from registration (the collection module already orders items) and keep `index` only as an optional override.

### Construction (2026-10-24)

`collection.rs`: `use_item_index(collection, index)` numbers an item when it is created, unless `index` is set.
Creation order alone is not tree order: Dioxus creates the component siblings nested inside one element of a
template in reverse (`dioxus-core` `load_placeholders` walks them backwards; the unit test
`siblings_nested_in_one_element_are_created_in_reverse` pins it), which is also why NavigationMenu's panel links had
to be numbered by hand (`content_index`). So every derived item renders `data-collection-item="<collection id>:<slot>"`
(the collection id is a hydration-stable `use_unique_id`), and on the client a pass (`querySelectorAll`, document
order) renumbers the items into a fresh range above every index handed out when their order disagrees; the
registered items and the focus are renumbered in the same step. The pass runs when a derived item is created and when
its element mounts; `try_focus_placement` and the listbox's initial focus wait while one is in flight. One explicit
`index` anywhere in a collection turns the pass off for it (the caller owns the order).

Select and Combobox mount their options afresh on every open (`SelectList` switches render arms), so the indices of a
closed list mean nothing to the open one: the listbox's initial focus is an intent (`ListboxFocus`: selected, else
first or last) resolved once the open list's options are registered and ordered.

Subsumed: hand numbering, duplicate indices (Select's two `enumerate()` loops in the `aligned` demo both started at 0),
an item inserted between mounted ones (the pass places it once its element mounts), NavigationMenu `content_index`,
TabContent's index (`aria-controls` is keyed by `value`), Accordion's unread `index`. Not subsumed: a keyed list that
reorders already-mounted items without remounting them keeps their old place (documented on each prop: pass `index`).
Left for their owners: the sidebar and data_table demos still pass explicit `index` (it compiles and is honoured);
Toolbar, ToggleGroup, Navbar, TagGroup, Command and Carousel items still take a required hand-numbered index and can
move to `use_item_index` the same way.

