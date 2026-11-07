# Demo title audit, 2026-11-07 (row 399)

Scope: every `preview/src/components/*/variants/<variant>/` directory (565 demos; `main` shows no title). Owner
review is pending for everything except the first row.

## How a title is made

A demo's heading is derived from its directory name by `variant_title` (`preview/src/site/variant_page.rs`):
underscores become spaces, a trailing acronym (`api`, `ltr`, `otp`, `rtl`, `ssr`, `ui`, `url`) goes in
parentheses and upper-case, and the CSS (`.dx-component-variant-title`, `text-transform: capitalize`) title-cases
every word. So `virtual_loop_rtl` reads "Virtual Loop (RTL)", and "Find in Accordion" renders as "Find In Accordion".
Specs address the frame by directory (`#component-preview-frame-<variant>`), so a rename must never touch the
directory.

**Mechanism added with this audit:** a demo whose directory name does not make a good title carries a one-line
`variants/<variant>/title.txt`; `preview/build.rs` (`write_variant_titles_index`) turns those into
`variant_title_override(component, variant)` and `demo_title(component, variant)` picks it over the derived
title. Applying any row below is a one-file change (`title.txt`) with no directory rename and no spec edit.

## Applied now

| Component / directory | Was | Now | Reason |
|---|---|---|---|
| accordion / `until_found` | Until Found | Find in Accordion | Owner-approved. "Until found" is the attribute (`hidden="until-found"`); the reader's task is Ctrl+F finding hidden text. |

## Proposed (not applied, owner to approve)

Pattern findings first, then the per-demo list.

1. **The same name in three components.** `until_found` exists in accordion, collapsible and tabs and reads
   identically. Rename all three with the component named: "Find in Collapsible", "Find in Tabs".
2. **Bare generic nouns** (`demo`, `variant`, `variants`, `states`, `types`, `settings`, `form`, `field`,
   `table`, `text`, `group`): the title says nothing about what the demo shows.
3. **Implementation words in a title** (`api`, `tuning`, `force_mount`, `keep_mounted`, `click_only`): fine for
   a developer who knows the prop, opaque to someone browsing. Keep the prop name, add the outcome.
4. **Inconsistent "all of them" naming:** `button/all_variants`, `item/variant`, `marker/variants`. One phrase.
5. **Prefix families that sort and read as noise** (`radar_chart/grid_*`, `chart_tooltip/label_*`,
   `line_chart/dots_*`): readable, listed only where the suffix alone is unclear.

| # | Component / directory | Title now | Proposed | One-line reason |
|---|---|---|---|---|
| 1 | collapsible / `until_found` | Until Found | Find in Collapsible | Same finding as the approved Accordion rename. |
| 2 | tabs / `until_found` | Until Found | Find in Tabs | Same. |
| 3 | message / `demo` | Demo | Full conversation | "Demo" names nothing; the demo is the whole kit in one conversation. |
| 4 | item / `variant` | Variant | All variants | Singular noun reads as one variant; it shows every `ItemVariant`. |
| 5 | marker / `variants` | Variants | All variants | Same phrase as item and button, one convention. |
| 6 | button / `all_variants` | All Variants | All variants | Already the target phrase; keep (listed to fix the family). |
| 7 | avatar / `states` | States | Picture states | States of what: loaded, loading, broken, delayed, none. |
| 8 | tag_group / `states` | States | Disabled and non-selectable | The two states it shows are named in the title. |
| 9 | slider / `edge` | Edge | Thumb at the edge | "Edge" alone could be anything (edge fade, edge case). |
| 10 | scroll_area / `releases` | Releases | Long list (release notes) | A reader cannot tell what a "releases" demo is about. |
| 11 | scroll_area / `fade` | Fade | Edge fade | Names what fades (the scroll edges); matches `carousel/edge_fade`. |
| 12 | textarea / `fade` | Fade | Edge fade | Same. |
| 13 | virtual_grid, virtual_list / `fade` | Fade | Edge fade | Same. |
| 14 | carousel / `api` | API | Slide count and dot picker | The demo builds "Slide 2 of 5" and a dot picker from `use_carousel()`; "API" says nothing. |
| 15 | carousel / `drag_free` | Drag Free | Free drag | Word order of the prop name reads backwards. |
| 16 | carousel / `peek` | Peek | Next slide peeking | One word, no object. |
| 17 | carousel / `edge_fade` | Edge Fade | Edge fade | Case only (see "Case" below). |
| 18 | carousel / `hidden_arrows` | Hidden Arrows | Arrows hide at the ends | Describes the behaviour, not the state. |
| 19 | carousel / `virtual_many` | Virtual Many | Many slides (virtual) | "Virtual Many" is a fragment; the demo is many virtualized slides. |
| 20 | carousel / `virtual_loop` | Virtual Loop | Looping (virtual) | Same. |
| 21 | carousel / `virtual_loop_rtl` | Virtual Loop (RTL) | Looping (virtual, RTL) | Follows 20. |
| 22 | drag_and_drop_list / `tuning` | Tuning | Ghost opacity and drop gap | "Tuning" of what: the two live sliders. |
| 23 | hover_card / `force_mount` | Force Mount | Always in the DOM | The prop name says how, not what the reader sees. |
| 24 | collapsible / `keep_mounted` | Keep Mounted | Content kept when closed | Same reason. |
| 25 | collapsible / `settings` | Settings | Advanced settings | "Settings" is any settings page; the demo is the show-advanced pattern. |
| 26 | separator / `settings` | Settings | In a settings list | Same bare noun; the point is the separator between rows. |
| 27 | questionnaire / `navigation_state` | Navigation State | Disable Next until answered | Names the behaviour. |
| 28 | questionnaire / `animate_height` | Animate Height | Animated height | Grammar, matches the other noun-phrase titles. |
| 29 | questionnaire / `freeform` | Freeform | Free text answers | Reads as a style, not a question type. |
| 30 | questionnaire / `skip` | Skip | Skippable questions | A verb alone. |
| 31 | questionnaire / `resume` | Resume | Resume a saved draft | A verb alone. |
| 32 | input / `form` | Form | Required fields in a form | Bare noun; the demo is required-field validation. |
| 33 | input / `field` | Field | Inside a Field | The component named "Field" makes the title read as the input itself. |
| 34 | input_group / `custom_control` | Custom Control | Any element as the control | Says what is custom. |
| 35 | chart / `bar`, `line`, `stacked` | Bar, Line, Stacked | Bar chart, Line chart, Stacked area and bar | A page of titles that name no noun. |
| 36 | chart_tooltip / `advanced` | Advanced | Formatter with a total row | "Advanced" is a difficulty label, not a topic. |
| 37 | select / `aligned` | Aligned | Item-aligned (default) | Names the `SelectPosition` it shows. |
| 38 | select / `popper` | Popper | Below the trigger (popper) | Same; plain words first. |
| 39 | select / `scroll` | Scroll | Long list with scroll buttons | Bare verb. |
| 40 | tooltip / `delays` | Delays | Open and close delays | Delays of what. |
| 41 | toggle_group / `font_weight` | Font Weight | Font weight picker | Names the pattern, not the property. |
| 42 | button_group / `split` | Split | Split button | The common name of the pattern. |
| 43 | table / `rich` | Rich | Composed cells | "Rich" is vague; the demo composes checkbox, badge and total cells. |
| 44 | table / `reorder`, toggle_group / `reorder`, virtual_list / `reorder` | Reorder | Reorder rows (items) | Three demos with the same bare verb. |
| 45 | skeleton / `card`, `form`, `table`, `avatar` | Card, Form, Table, Avatar | Card skeleton, Form skeleton, ... | The component is "Skeleton"; the title should say what is being skeletoned. |
| 46 | spinner / `in_input` | In Input | Inside an input | Grammar. |
| 47 | virtual_grid / `short` | Short | Short list | Bare adjective. |
| 48 | checkbox / `table` | Table | Select-all in a table | Names the pattern. |
| 49 | card / `terms` | Terms | Terms and conditions | Fragment. |
| 50 | checkpoint / `thread` | Thread | Restore to a checkpoint | The point is the restore behaviour. |
| 51 | dialog, sheet, popover, command, alert_dialog / `overlay` | Overlay | Without the dim (`overlay: false`) | The title alone does not say it removes the dim. |
| 52 | textarea / `button` | Button | With a submit button | Bare noun. |
| 53 | toast / `update` | Update | Update or close a toast | Bare verb. |
| 54 | `click_only` (context menus, menubar, navbar, dropdown_menu, navigation_menu) | Click Only | Opens on click only | Fragment. |
| 55 | `non_modal` (context_menu, dialog, drawer, popover) | Non Modal | Non-modal | Hyphen. |
| 56 | `read_only` (checkbox, date_picker, radio_group, switch) | Read Only | Read-only | Hyphen. |

**Case.** `text-transform: capitalize` title-cases small words ("Find In Accordion", "Date Of Birth", "In Button").
A rule that leaves a, an, in, of, the, to, and, or lowercase mid-title would make every proposed phrase read as
written, and would fix existing titles like `date_picker/date_of_birth` and `kbd/in_button` without per-demo
files. That is a one-line CSS or `variant_title` change, an owner decision.

## Titles that are fine

Everything not listed: single-word nouns that name the thing shown (`sizes`, `disabled`, `invalid`, `rtl`,
`icons`, `groups`, `multiple`, `controlled`, `legend`, `label`, `interactive`, `stacked`, `step`, `linear`,
`placement`, `sides`, `snap_points`) and the descriptive multi-word names (`account_switcher`, `date_of_birth`,
`natural_language`, `load_history`, `opening_position`).
