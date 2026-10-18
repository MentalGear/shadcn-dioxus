---
id: 160
title: "Two small bugs the shadcn comparison found: Tabs Ghost sets data-variant=\"ghost\" that no CSS styles…"
status: "LANDED 2026-10-05 (round 3) -- fixed in the tightening sweep"
size: "S"
area: theme
updated: 2026-10-05
---

# Row 160: Two small bugs the shadcn comparison found: Tabs Ghost sets data-variant="ghost" that no CSS styles…

## Item

Two small bugs the shadcn comparison found: Tabs `Ghost` sets `data-variant="ghost"` that no CSS styles; `BreadcrumbEllipsis` renders an `li` inside an `li`

## Status / trigger

**LANDED 2026-10-05 (round 3)** -- fixed in the tightening sweep

## Size

S

## Detail

(a) `TabsVariant::Ghost` (`preview/src/components/tabs/component.rs`, `to_class` at ~line 67, emitted as `data-variant` at ~line 76) has no rule in `tabs/style.css`, so Ghost looks like the default. Fix: style it, or drop it in favour of the `line` variant row 155 adds (shadcn's equivalent). (b) `BreadcrumbEllipsis` (`preview/src/components/breadcrumb/component.rs`, ~line 117) renders an aria-hidden `li`, and the main demo places it inside a `BreadcrumbItem` `li`: invalid nesting, and it cannot be a menu trigger. Fix: render a `span` (shadcn does). The sweep takes the defects; row 155 (groups 5 and 9) adds the new variant and the trigger-capable form on top, so read it first to avoid fixing the same lines twice. Check the tabs and breadcrumb specs, axe and `check-preview-composition.sh`. **Landed 2026-10-05 (round 3):** (a) Tabs `Ghost` is now styled and has the look of shadcn's `line` variant (transparent list, 4 px gap, no fill on the active trigger, a 2 px `foreground` bar that fades in; `tabs/style.css`), so row 155 group 5's `line` variant can alias Ghost instead of adding a second implementation (its vertical orientation is still group 5's). (b) `BreadcrumbEllipsis` renders a `span`, no longer an `li` inside an `li`. Row 155 groups 5 and 9 build on both: read them before touching the same lines. Verified with `tabs.spec.ts`, `breadcrumb.spec.ts`, the axe sweep (`all-routes.spec.ts`) and `check-preview-composition.sh`.
