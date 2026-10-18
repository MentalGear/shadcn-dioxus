---
id: 27
title: "shadcn catalog parity round — scoped to the styled-only gaps (user decision 2026-09-03): Alert, Breadcrumb…"
status: "LANDED 2026-09-04, all nine. Each is a themed preview/ component with component.json, docs.md, a…"
size: "S ×9"
area: docs
updated: 2026-09-04
---

# Row 27: shadcn catalog parity round — scoped to the styled-only gaps (user decision 2026-09-03): Alert, Breadcrumb…

## Item

**shadcn catalog parity round — scoped to the styled-only gaps (user decision 2026-09-03)**: Alert, Breadcrumb, Button Group, Empty, Field, Input Group, Kbd, Spinner, Native Select (nine — Typography dropped 2026-09-03: it is a docs page in shadcn and a demo link label in rust-ui, not a component anywhere). The primitive-needing gaps (Resizable, Command, Navigation Menu, Drawer, Table, Input OTP) leave the NEXT queue and stay in [`component-backlog.md`](../../component-backlog.md)'s ranked list with their own triggers

## Status / trigger

**LANDED 2026-09-04, all nine.** Each is a themed `preview/` component with `component.json`, `docs.md`, a `variants/main` demo, registration in `preview/src/components/mod.rs` + the root `component.json` `members`, a Playwright smoke + axe spec, and an entry in `dx-class-migration.spec.ts` — built to post-row-32 conventions (no `#[css_module]`, plain `dx-<name>` classes, per-entry-point `document::Link`) and to row 31a/31b's tokens; all four lints (`check-preview-composition.sh`, `check-cfg-axis.sh`, `check-dx-class-prefix.sh`, `check-css-literals.sh`) green

## Size

S ×9

## Detail

Full classification in [`component-backlog.md`](../../component-backlog.md) "NEXT". Composition notes worth keeping: `Button Group` and `Field` style their contained controls via the plain `button`/`input` ELEMENT selector rather than foreign `.dx-button`/`.dx-input` classes, since `check-dx-class-prefix.sh` forbids a component's stylesheet from selecting another component's classes; `Input Group` wraps the existing themed `Input` and flattens its border via CSS; `Spinner` carries its own `prefers-reduced-motion` override, since row 31a's shared reduced-motion layer covers interactive roots, not animated content. No provenance headers needed — all original work against shadcn/ui's markup shape, nothing substantially copied. **Axe finding, worth recording as evidence the per-component axe pass earns its place:** `button_group`'s own demo scan caught two icon-only buttons ("Previous"/"Next" chevron icons) with no accessible name (`button-name`, critical) — fixed with `aria-label`, matching the sibling "More" button's existing pattern
