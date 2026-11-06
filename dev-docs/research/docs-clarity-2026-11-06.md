# Docs-clarity review, round 2 (2026-11-06)

Row 387's standing review, run as one lane over the component pages that changed since the first round (2026-10-24)
plus the pages row 392 touched: 60 pages (the list is `git log --since=2026-10-24 -- preview/src/components/*/docs.md
preview/src/components/*/variants/`, plus the 392 sweep). Read-only except the one mechanical fix below.

## Method and limits

- Mechanical checks over the 60 pages: component links (`](/component/x/)`), other internal links (`](/docs#motion)`),
  variants named in prose, backticked snake_case and CamelCase names against the code, sample component and prop names,
  controlled literals in samples, prop doc comments (`///`) per props struct, common misspellings.
- A prose read of three pages (dropdown_menu, toast, tooltip) and grep sweeps for implicit-behaviour cues
  ("automatically", "ignored", "overrides", "no effect") over all 60.
- Not done: a full read of all 60 pages by four subagents, the standing brief's method. This lane has no subagent tool,
  so the rest of the review is a sample. Treat the list below as the verified part, not a complete census.

## Findings (ranked)

1. **High. No controlled vs uncontrolled guidance on 13 pages with controlled-capable props.** alert_dialog, date_picker,
   dialog, sheet, popover, hover_card, tooltip, color_picker, input_otp, tag_group, toggle, navbar, input. A reader
   cannot tell from the page whether `open`/`checked`/`value` freezes the component. Row 392's shared "Controlled vs
   uncontrolled" paragraph would close this on every page; it still needs an owner decision on where it lives.
   Size M (one shared paragraph plus one link per page).

2. **Medium. Sheet names components that do not exist.** **Fixed** (sheet/docs.md, one sentence). `sheet/docs.md:51-52` says the Sheet follows shadcn's
   `SheetOverlay`/`SheetContent`, and the repo has neither. A reader who searches for `SheetContent` finds nothing.
   Fix: one sentence, "There is no SheetContent or SheetOverlay here: `Sheet` draws the scrim and the panel, and its
   children are the content." Size S.

3. **Medium. Skeleton has no reduced-motion rule (disclosed, not fixed).** `skeleton/docs.md:16` says so. Verified:
   `preview/src/components/skeleton/style.css` has no `prefers-reduced-motion` block, while progress has one
   (`progress/style.css:80`). Fix: add the block to the shipped stylesheet, so copies do not each need it. A code
   change. **Fixed** (skeleton/style.css gates the pulse with `animation: none` under reduced motion; the docs sentence
   now describes it). Checked in a headless browser on a minimal page with the base and the new stylesheet: the base
   runs the pulse under `reduced`, the new one stops it (`animation: none`, opacity 1). The component-page Playwright
   check was not run: the debug SSG build was refused for disk space (2.4-5.4 GB free, 5.9 GB needed).

4. **Low. Tabs' shadcn names have no mapping to the repo's names.** `tabs/docs.md:32` maps `TabsList` for `variant`, and
   line 64 names shadcn's `TabsContent`, but nowhere is `TabsList` -> `TabList`, `TabsContent` -> `TabContent`,
   `TabsTrigger` -> `TabTrigger` stated in one place. **Fixed** (one sentence under the tabs page's intro).

5. **Low. Toast key casing.** `toast/docs.md:1` wrote `f6`; the key is F6 everywhere else on the page. **Fixed** (the
   only docs edit this round; prose, not a sample, so no compile check was needed).

## Checked and clean

- No dead component links in the 60 pages; `/docs#motion` resolves (the docs page's "Motion" section, slug `motion`).
- Every variant named in prose exists (`date_of_birth` is a date_picker variant, `required` an input variant,
  `clear_form`, `click_only_submenu` and `photo_wall` exist on their pages).
- No sample prop names the code does not know (check is weak: it matches a field name anywhere in the code).
- No frozen controlled literal left in these pages' samples (the row 392 class). Remaining literals are item values,
  form values or display-only props.
- Prop doc coverage: every props struct in the scope with a primitive or component file has `///` on each field,
  except four `attributes` fields (combobox 2, data_table 4, tag_group 1), which are the global-attribute spread.
- Implicit behaviour that is stated: card's `CardAction` switches the header to a two-column grid (card:32); accordion's
  controlled mode ignores `default_open`/`default_value` (accordion:61); date_picker's `close_on_select` is ignored while
  `confirm` is on (date_picker:56); drawer's `is_modal: false` drops the scrim, which matches the code
  (`primitives/src/drawer.rs:129`).
- Modes without a variant: none found. Sidebar's three `collapsible` modes are all shown by the inset variant; toast's six
  positions have the `position` variant; drawer sides, tabs' vertical modes and scroll_area's horizontal mode have variants.
- Typos: none beyond the F6 casing.
