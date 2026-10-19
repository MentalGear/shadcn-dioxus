---
id: 299
title: "Accordion: no hover underline on collapsed headers; optional left chevron, used on the docs component page"
status: "OPEN (owner review 2026-10-19)."
size: "M"
area: primitives
updated: 2026-10-19
---

# Row 299: Accordion: no hover underline on collapsed headers; optional left chevron, used on the docs component page

## Item

1. Collapsed accordion headers shouldn't underline on hover.
2. Add an option to put the chevron on the left.
3. Use that variant for the accordion on docs component pages.

## Status / trigger

OPEN (owner review 2026-10-19).

## Size

M

## Detail

1. The underline is shadcn's `hover:underline`, at `accordion/style.css:34-39`, and applies to every trigger.
   - **Target:** scope it to `[data-state="open"]`, or drop it. That is a deliberate deviation from shadcn, so the CSS comment should say so.
   - Owner default (2026-10-19): collapsed ones only.
2. `AccordionTrigger` hard-codes the chevron after the label (`component.rs:47-63`).
   - Add a per-`Accordion` prop, `chevron: Start | End`, default `End`, emitted as `data-chevron`, with the chevron rotating from −90° to 0°.
   - It needs a props doc comment (`check-props-docs`) and a `docs.md` note.
3. Use `chevron: Start` on the docs page accordion (`main.rs:1926-1980`, Installation plus the `docs.md` sections).
   - "Manual installation files" inside it is a `Collapsible`, not an accordion. **Default: give it the same left chevron so the page matches.**
