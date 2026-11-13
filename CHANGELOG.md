# Changelog

Changes that affect people who use the library, for changes from the first release on. No release has happened yet, so the entries
below are collected under "Unreleased" and move under a version heading at the first release.

## Unreleased

### Changed

- **State attributes are presence attributes; `data-state` is gone.** Every component states its state with a Base UI
  presence attribute (`data-open` / `data-closed`, `data-checked` / `data-unchecked` / `data-indeterminate`, `data-active`,
  `data-hidden`, `data-selected`, `data-pressed`, `data-loading`, ...): present (and empty) while the state holds, absent otherwise.
  Booleans are never written as `data-x="true"` or `data-x="false"` any more, so `[data-disabled]` matches exactly when the
  component is disabled. If your own CSS or tests select `data-state="open"`, `[data-disabled="true"]` and the like, move them with the
  old-to-new table in [`dev-docs/state-attributes.md`](./dev-docs/state-attributes.md). The chart primitives keep Recharts'
  `data-active` and `data-state` on bars, arcs and the tooltip.
- **Hover-opened menus close as fast as Base UI's.** NavigationMenu, Navbar and a hover-opened submenu
  (DropdownMenu, ContextMenu) now close 50 ms (0 ms for a submenu) after the pointer leaves, down from 300 ms / 200 ms.
  A pointer moving from the trigger toward the open panel is tracked against the panel's measured position and keeps
  it open while it stays inside that path ("a safe triangle"), so a diagonal move to the panel still reaches it
  before the shorter delay would close it.

### Fixed

- A menu root (DropdownMenu, ContextMenu, Menubar) given an `id` that changes later keeps its outside-press and
  outside-focus dismissal: the dismiss layer follows the current `id` instead of the first one, and returns to the
  generated id when the `id` is dropped.
- On desktop with two windows, a Slider, Resizable handle or ColorPicker drag in the second window now tracks the
  pointer while the first window has a drag in progress, and a form reset in one window no longer queues behind the
  other window's read: the pointer bridge and the reset read are per window. The scroll lock and the
  once-per-page stylesheets (scrollbar gutter, anchored positioning, top-layer ink, toast base) are per window too,
  so a second window's dialog locks its own page and gets its own styles.
- Pressing Escape just after a submenu starts closing (during its close animation) no longer swallows the key: the
  enclosing menu now receives it and closes too, instead of being left open.
- A non-primary pointer button (middle-click, a stylus's barrel button) on a Slider, Resizable handle or
  ColorPicker's draggable area is no longer intercepted; only a primary press starts the drag, so the browser's own
  handling of other buttons (e.g. middle-click autoscroll) reaches the element again.
- Escape no longer closes an open Dialog, Menu or Popover while an IME composition (e.g. typing Japanese or Chinese)
  is in progress; it now only cancels the composition, as the input method expects.
