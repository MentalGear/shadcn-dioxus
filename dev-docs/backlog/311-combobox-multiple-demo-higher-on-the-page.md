---
id: 311
title: "Combobox multiple demo higher on the component page"
status: "DONE 2026-10-20 (claude/batch-b). Owner request."
size: "S"
area: combobox
updated: 2026-10-20
---

# Row 311: Combobox multiple demo higher on the component page

## Item

Owner (Tom), verbatim: "combobox multiple should have its demo in the main doc page, and on the components page higher in demo examples."

## Status / trigger

DONE 2026-10-20 (claude/batch-b). Owner request.

## Size

S

## Detail

**Reading taken.** The component page (`/component/combobox/`) is one `main` demo, the docs accordion, then the Examples list in `variants/order.txt` order. The site has no other page that orders combobox demos (`/component/` is an index, `/demos` is composed dashboards). So: (1) "higher in demo examples" is `variants/order.txt`, where `multiple`, `multiple_clear` and `multiple_form` were 5th to 7th; they are now first. (2) "in the main doc page" is read as the docs.md intro, which now names `ComboboxMulti` and points to its demos and its "Multiple selection" section, since the `main` demo's locators belong to the single combobox (`combobox.spec.ts`, the first-paint specs) and a second combobox there would make every `getByRole("combobox")` ambiguous. If the owner meant a second combobox inside the main demo frame itself, that is a one-file change to `variants/main/mod.rs` plus its specs; ask before doing it.
