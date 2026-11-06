---
id: 498
title: "Pagination links have a disabled state"
status: "DONE (2026-11-06). Proof below."
size: "S"
area: components
updated: 2026-11-06
---

# Row 498: Pagination links have a disabled state

## Item

`PaginationLink`, `PaginationPrevious` and `PaginationNext` take `disabled: bool`: the platform semantics of an
unavailable link (`aria-disabled="true"`, `role="link"`, no `href`, so not focusable and not navigable, handlers
suppressed) plus a `data-disabled` presence attribute (`state_attrs::presence`) styled with `--dx-opacity-disabled` and
`pointer-events: none`. shadcn's Pagination has no disabled equivalent. The `disabled_ends` demo (row 399) uses the prop
instead of hand-written `aria-disabled` and inline styles.

## Status / trigger

DONE (2026-11-06). Proof below.

## Size

S

## Detail

Spec: `pagination.spec.ts` "disabled Previous/Next: aria-disabled, no href, not focusable, click does not navigate"
(also checks the enabled links keep `href`, tab order and no disabled attributes). `component.rs` changed: run
`scripts/registry-install-smoke.sh`.
