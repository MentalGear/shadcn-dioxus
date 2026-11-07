---
id: 399
title: "Demo batch from the docs review"
status: "Both halves landed in PR #166 (2026-11-06). The Detail's 2026-10-25 owner items (the demo rename…"
size: "S"
area: docs
updated: 2026-10-25
---

# Row 399: Demo batch from the docs review

## Item

Supported modes with no demo: ScrollArea Both (the default) and `scroll_type`; Progress indeterminate; Dialog non-modal; DataTable reorder with a sortable column; Pie half-pie and padding; Line gaps; Input OTP letters and `on_complete`; Pagination with disabled ends; Resizable controlled sizes; menus with `modal: false`; Collapsible `keep_mounted`; ButtonGroupText; Avatar Rounded.

## Status / trigger

**Both halves landed in PR #166 (2026-11-06).** The Detail's 2026-10-25 owner items (the demo rename audit, the focus-target demos) are not recorded as done, so the row stays open. Found while building them: `ScrollType::Hidden` does not hide the bar (row [497](./done/497-scroll-area-scroll-type-hidden-shows-the-bar.md)); Pagination has no `disabled` prop, so the `disabled_ends` demo marks its unavailable links by hand (an owner decision whether to add the prop).

**First half (lane `demos-399a`).** Seven demos added, one variant directory each: ScrollArea `both` (the default direction) and `scroll_type`; Progress `indeterminate`; Dialog `non_modal`; Collapsible `keep_mounted`; ButtonGroupText `text`; Avatar `rounded`; Pagination `disabled_ends`. Checked in a browser on a lane SSG build: the non-modal page button counts clicks while the dialog is open, the `keep_mounted` field keeps its text across close and reopen, the disabled Pagination links have no `href` and do not navigate, and `indeterminate` sets `data-state="indeterminate"`. **Open primitive bug, not fixed here:** `ScrollType::Hidden` does not hide the bar. The primitive (`primitives/src/scroll_area.rs`, the `owned` attributes) emits `scrollbar-width` as an HTML attribute, not a CSS declaration, so the computed `scrollbar-width` stays `auto` and the bar shows; the `scroll_type` demo's Hidden column shows it. Owner decision needed: fix the primitive (a `style` declaration) or drop the Hidden column until it is fixed. **Pagination has no `disabled` prop:** the demo marks the unavailable links with `aria-disabled`, no `href`, and the disabled opacity, which is a demo convention, not an API. The other half (DataTable reorder, Pie half-pie and padding, Line gaps, Input OTP letters and `on_complete`, Resizable controlled sizes, menus with `modal: false`) belongs to another lane and is not in this branch. Proof on a lane SSG build of this tree (100 routes, no stale HTML): `playwright test:changed` 698 passed, 0 failed (`CHANGED_RANGE` = the session branch; 42 specs plus the matrix specs its diff reaches, axe included). The first run of that suite failed one test: two Pagination `nav` landmarks both labelled "pagination" (axe `landmark-unique`), fixed by giving each its own label.

Original status: BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in the project's shared files, `docs-clarity/2026-10-24-docs-clarity-review.md`.

**Second half (lane demos-399b):** demos added for DataTable reorder with a sortable column (`data_table` `sort_reorder`),
Pie half-pie and padding (`pie_chart` `half`, `padding`), Line gaps (`line_chart` `gaps`), Input OTP letters with
`on_complete` (`input_otp` `complete`: `OtpPattern::Letters`; an alphanumeric demo already existed), Resizable controlled
sizes (`resizable` `controlled`) and ContextMenu `modal: false` (`context_menu` `non_modal`). `DropdownMenu` and `Menubar`
have no `modal` prop, so only the ContextMenu has a non-modal mode to show. Each is exercised once in
`playwright/demo-modes.spec.ts`.
Proof: `demo-modes.spec.ts` 7/7 on a debug SSG build (chromium); `test:changed` over `d040554a...HEAD` 478 passed (27
spec files, including all-routes axe and the chart, resizable, context-menu and data-table specs). The pie padding is
in the drawn path, not in `data-start-angle`/`data-end-angle`, so the spec measures the 4 degree gap between one arc's
end point and the next one's start point; the line gap renders as two `chart-line` paths with five dots; the
non-modal context menu leaves `window.__dxScrollLocked` unset. No mode turned out broken (no `fixme`).

## Size

S

## Detail

One or two lanes, each verified with `dx serve` and `pnpm run test:changed`.

**Added 2026-10-25 (owner, backlog thread):**
- Demo names: rename the Accordion "Until found" demo to "Find in Accordion". More generally, many demo names could be
  more fitting and obvious. Audit every demo title and propose a rename list to the owner, then rename what is approved.
  Specs address a demo's frame by its variant directory (`#component-preview-frame-<variant>`), so change the display
  title rather than the directory where the two are separate.
- "Focus target" demos (Dialog, Alert Dialog and Popover `variants/focus_target`): a mouse open does move focus to
  Cancel, but rings are keyboard-only by design, so a mouse user sees nothing happen. Keep the policy and make the demo
  explicit: a live line inside the panel ("Opened with: mouse, focus on: Cancel") and notes that spell out each
  modality's result.
