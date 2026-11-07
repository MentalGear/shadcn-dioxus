---
id: 392
title: "Compile-check the docs.md code samples; samples use the uncontrolled props"
status: "PARTLY LANDED 2026-11-06 (lane docs-samples-392). Every sample the row names is fixed and compiles…"
size: "M"
area: docs
updated: 2026-10-24
---

# Row 392: Compile-check the docs.md code samples; samples use the uncontrolled props

## Item

Docs samples pass a literal to the controlled prop and freeze the component (RadioGroup `value`, Slider `value`, Switch `checked`, Select `value`, Dialog/Drawer `open` with no `on_open_change`, so the dialog cannot close), or do not type-check (Calendar's nonexistent `CalendarDate`, Navbar items without `index`/`to`, Menubar items without `index`, NavigationMenu links without `content_index`).

## Status / trigger

**PARTLY LANDED 2026-11-06 (lane docs-samples-392).** Every sample the row names is fixed and compiles: RadioGroup,
Slider, Switch and Select now take `default_*` (or pair the signal with its `on_*_change`), Select's callback takes
`Option<String>`, Dialog, Drawer and Sheet pair `open` with `on_open_change`, Calendar and DatePicker use `time::Date`
with a signal (the row's `CalendarDate` does not exist), Navbar items and `NavbarNav` take `index` (and items `to`).
Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in the project's shared files,
`docs-clarity/2026-10-24-docs-clarity-review.md`.

Two findings of the row do not hold any more. Row 396 (landed 2026-10-24) made `index` optional on Menubar items and
NavigationMenu links, so the row's samples without `index`/`content_index` compile as they stand; they were left
unchanged. The Menubar and NavigationMenu samples had no type error in the compile check below.

Also fixed, the same two classes in other pages: Sheet (`open` without `on_open_change`), Context Menu, Dropdown Menu
and Menubar (`value: ""` and `{children}` placeholders, undeclared checkbox and radio state), Tag Group and Tabs
(undeclared state, `{children}`), Native Select (undeclared `fruit`), Date Picker (undeclared `selected_date`,
`|| "D"` returns `&str` where `String` is required), Form (`SelectOption` values were `&str`, `item_to_label` was
missing, and `SelectTrigger`/`SelectList`/`SwitchThumb`/`CheckboxIndicator` children do not exist on the themed
components).

Compile check: the 50 `rust` blocks of 19 pages (calendar, date_picker, dialog, drawer, sheet, menubar, navbar,
navigation_menu, radio_group, select, slider, switch, context_menu, dropdown_menu, native_select, tag_group, form,
collapsible, combobox, direction, tabs, top_layer) were pasted into a scratch module with `use` lines for their
components and each as a `#[component]`, then `cargo check -p preview --features web` passed. No gate compiles docs
samples yet. Left as they are: `breadcrumb`'s `Route::Components {}` (the app's own router) and `alert_dialog`'s icon
placeholder, both app-specific fragments.

**Controlled paragraph LANDED (lane controlled-docs-392), by construction.** The explanation is written once, in the
`/docs#controlled` section (`preview/src/controlled_docs.rs`, next to Motion). Each component page gets a generated
"Controlled state" section: `preview/build.rs` (`render_controlled_section`) reads the same props structs as the props
table and `preview/controlled_scan.rs` finds the pairs (`X` + `on_X_change`, with `default_X` when present; the state
prop is also `selected_X`, plus one alias, `DatePickerProps` `value` -> `selected_date`). A half-present pair fails the
build: `default_X` + `on_X_change` with no `X`, or `X` + `default_X` with no `on_X_change`, unless listed with a reason
(`OBSERVERS`: DataTable sort; `UNREPORTED`: the checkbox's `BubbleInput`). `docs_tests::controlled_block_is_on_every_page_with_a_pair`
re-reads the pairs from each page's rendered props table and requires the section exactly when a pair exists, plus the
eleven named pages. Not covered by name: Input (a native element: `value`/`oninput`, no props struct) and Navbar (its
`value` belongs to one item, nothing is controlled), so neither has a pair to list. Spec: `playwright/controlled-docs.spec.ts`
(written, run by the batch verifier). Still open: the gate that compiles every docs `rust` block (the "By construction"
step; a marker opts a block out with a reason), which needs an owner decision on its shape.

## Size

M

## Detail

- Rewrite the samples with `default_*` and add one shared 'Controlled vs uncontrolled' paragraph that every page links to.
- By construction: extract every rust block in `docs.md` into a compile check under the cargo gates (blocks that are fragments get a wrapper; a marker opts a block out with a reason).
