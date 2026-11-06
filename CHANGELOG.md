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
