# shadcn demo parity audit -- the owner's second audit, checked against the tree (2026-10-11)

Source: the owner's row-188 text (below), compared with the row-187 report ([`shadcn-demo-gaps-2026-10-11.md`](./shadcn-demo-gaps-2026-10-11.md)) and with the preview tree **at git HEAD `a6611b3`** (read with `git show HEAD:<path>`). A parallel batch is adding demo variants for checkbox, input, input_group, button_group, radio_group, switch, toggle, toggle_group, tabs, progress, skeleton, spinner, tooltip, textarea, collapsible, card, empty, item, breadcrumb and button; none of that is counted as existing here. Tracked-feature rows: 133 (input-group states), 137 (RTL), 138 (toast), 155 (parity wave, groups 1-10), 156 (drawer), 166 (hover/tooltip delays), 169 (popover non-modal).

## 1. Row 188, verbatim

Row 188 -- demo parity audit (2026-10-11)

Original examples we lack, by priority. Checkbox is the reference case: the original has 9 examples (description, controlled, invalid, Field, disabled, group, table row-select, RTL); ours is one bare Checkbox. main demo files may hold more than their names suggest, so med/low gaps need a check; checkbox, input, toggle, switch, button_group and input_group were confirmed single-demo.

| Priority | Component | Missing original examples |
|---|---|---|
| high | checkbox | labeled + description, controlled, invalid, disabled, group list, table row-select, RTL |
| high | input | Field + label + description, disabled, invalid, file, inline search button, required, grid, input-group / button-group composition, form |
| high | input_group | inline-end / block-start / block-end addons, text, button, kbd, dropdown, spinner, textarea footer, custom input |
| high | button_group | orientation, nested, separator, split button, input wrap, dropdown / select / popover combos, RTL |
| high | radio_group | description via Field, choice-card, fieldset + legend, disabled, invalid |
| high | switch | description, choice-card, disabled, invalid, sizes, RTL |
| high | toggle | outline, with text, sizes, disabled, RTL |
| med | tabs | line variant, vertical, disabled trigger, icon triggers |
| med | dropdown_menu | shortcuts, icons, destructive, avatar account-switcher, groups + submenus, radio icons |
| med | progress | label + value, controlled via slider, RTL |
| med | toast (sonner) | promise loading / success / error, typed icon variants |
| med | dialog | custom close, no close, sticky footer, scrollable content |
| med | alert_dialog | small size, media icon, destructive, small + media |
| med | toggle_group | outline, sizes, spacing, vertical, disabled, font weight |
| med | collapsible | controlled, settings panel, nested file tree |
| med | skeleton | card, avatar, form, table loading layouts |
| med | combobox | multi-select chips, clear button, groups, custom items, invalid, auto-highlight, input-group addon |
| med | item | avatar, icon, link, dropdown, header, RTL |
| med | breadcrumb | custom separator, dropdown, collapsed ellipsis, router-link render, RTL |
| med | empty | outline, gradient background, avatar, avatar group, input-group search, RTL |
| med | button | spinner loading, as-link, rounded, button-group |
| med | drawer | non-modal, nested, snap points, swipe handle, side positions |
| med | calendar | presets, month / year dropdown caption, date + time picker, cell size |
| med | date_picker | typed input, time picker, date-of-birth dropdown, natural-language |
| med | spinner | sizes, in button, in badge, in input-group, RTL |
| med | select | scrollable long list, disabled, invalid, align-item-with-trigger |
| med | hover_card | trigger delays, alignment |
| med | tooltip | keyboard-shortcut tooltip, disabled-button tooltip |
| med | card | small size, image cover, spacing |
| med | textarea | Field label + description, submit-button composition |
| low | separator, label, slider, popover, avatar, context_menu, aspect_ratio, scroll_area, sheet, badge, command, alert, field, input_otp, kbd, native_select, table, navigation_menu, accordion, menubar, resizable | small gaps (vertical / disabled / controlled / icons / RTL variants); see the audit if picked up |

Ours is richer than the original for carousel, data_table, sidebar and chart. Skipped: form (original has no demos). No original page: navbar, toolbar, virtual_list, drag_and_drop_list, color_picker, tag_group, message, bubble, marker, attachment, message_scroller, top_layer, chart_tooltip, and the area / line / pie / radar / radial charts.

## 2. Differences from the row-187 report

Row 187: 25 richer / 36 equal-or-better, 61 pages. Row 188: 7 high + 23 med + 21 low = 51 components flagged. All 25 of 187's components appear in 188, so 188 is a superset; the 26 extra are below.

**Components 188 flags that 187 called equal-or-better (26, one per page).**
- high (1): **toggle** (187: equal; 188: missing outline, text, sizes, disabled, RTL). The biggest disagreement; 188's claim holds, `Toggle` has no `variant`/`size` (row 155 group 3).
- med (11): toast, dialog, alert_dialog, drawer, calendar, select, hover_card, item, breadcrumb, button, toggle_group.
- low (14): label, slider, popover, aspect_ratio, sheet, badge, command, alert, kbd, native_select, table, navigation_menu, menubar, resizable.
- 188 also puts `separator`, `avatar`, `context_menu`, `scroll_area`, `field`, `input_otp`, `accordion` (all in 187's 25) in its low tier; see priority below.

**Components 187 called equal-or-better that 188 does not mention either way:** pagination (neither list), attachment/bubble/marker/message/message_scroller (187 "equal"; 188 "no original page", so these two agree in effect), carousel/data_table/sidebar/chart family (agree: ours richer). **187 flagged and 188 does not drop any.** 188's "skipped: form" matches 187 (not in either table).

**Priority disagreements.**
| Component | 187 | 188 |
|---|---|---|
| toggle | equal | high |
| input_group | M, listed 8th | high |
| button_group | S, listed 12th | high |
| tabs | top item (real panels, vertical, disabled) | med |
| accordion | top item (FAQ, multiple, disabled) | low |
| empty, avatar | top item (team avatar group) | empty med, avatar low |
| dropdown_menu, context_menu | top item together | dropdown_menu med, context_menu low |
| input_otp | top item ("Verify your login") | low |
| field | M | low |
| progress, spinner, tooltip, skeleton, collapsible, textarea, card | S | med (same size, higher tier than 187's "small ones") |

**Per-component items.** Only components where the lists differ.
| Component | 188 adds (187 lacked) | 187 had (188 lacks) |
|---|---|---|
| checkbox | controlled, table row-select, RTL | -- |
| input | file, required, grid, input-group/button-group composition | -- |
| switch | RTL | -- |
| radio_group | -- (same set; 188's "description via Field" = 187's "legend/description") | -- |
| tabs | line variant, icon triggers | realistic panel copy ("Overview/Analytics/...") |
| combobox | auto-highlight | disabled (it already exists, see 3) |
| date_picker | natural-language | labelled fields, presets (188 moves presets to calendar) |
| input_group | custom input | counter text on the textarea (188: "textarea footer", same) |
| button_group | orientation, nested, separator, RTL | -- |
| empty | outline, gradient background, RTL, input-group search | offline user, 404 search (188 has "input-group search") |
| card | small size, spacing, image cover | terms card with scroll text + Decline/Accept |
| spinner | sizes, RTL | empty-state spinner |
| textarea | Field label + description | invalid Field message |
| collapsible | controlled | -- |
| skeleton | card, avatar | -- |
| progress | controlled via slider, RTL | -- |
| accordion | -- (low, unspecified) | FAQ copy, multiple, bordered/card variants |
| dropdown_menu | -- (same six) | -- |
| separator, avatar, context_menu, field, input_otp, scroll_area | unspecified ("small gaps") | 187's specific demos (settings list, AvatarGroup/badge, icons/shortcut/destructive, payment form, verify-login card, release list) |
| 188-only components | toggle, toggle_group, toast, dialog, alert_dialog, drawer, calendar, select, hover_card, item, breadcrumb, button + 14 low | -- |

**Where 188 is stale at HEAD.** 188 says toggle, switch, etc. are "confirmed single-demo": true at HEAD for checkbox, input, input_group, button_group, switch and toggle (each has only `variants/main`). But HEAD already has `toggle_group/variants/{main,rtl}` (so "RTL" is not missing for toggle_group), `radio_group/{main,rtl}`, `tabs/{main,rtl,until_found}`, `item/{group,image,main,size,variant}` and `button/{icon,main,size}` (so item "icon/image" and button "sizes" are partly shown). The batch owner should read those before adding duplicates.

## 3. Verification of "main demo files may hold more than their names suggest" (non-batch MED and LOW)

Files are `preview/src/components/<c>/variants/<v>/mod.rs` at HEAD. "Exists" means the example is already shown in some form.

| Component | 188's "missing" item | State at HEAD |
|---|---|---|
| dropdown_menu | groups + submenus | submenus **exist** (`main`: "More tools" sub; `checkboxes`: "More options" sub; `click_only_submenu`; `rtl`). A group part does not. |
| dropdown_menu | radio ... | radio group **exists** (`radio_group`, `rtl`) but without icons. Checkbox items **exist** (`checkboxes`), not listed by 188. Disabled item exists (`main` Undo). |
| dropdown_menu | shortcuts, icons, destructive, avatar switcher | none exist (`main` is text-only). |
| toast | promise, typed icon variants | `main` is one "Info (60s)" button. Success/error/warning/info exist in the API and CSS (`toast/style.css:180-195`, tinted, no icon); only info is demoed. |
| dialog | custom close | **exists**: `main` hand-writes a `.dx-dialog-close` button, and nests a second dialog. Also `overlay` (no overlay, non-modal). |
| dialog | no close, sticky footer, scrollable | none exist. |
| alert_dialog | destructive | **exists**: `main` is "Delete item" and the Action is hard-coded destructive (`alert_dialog/style.css:238`). Also `overlay`. Size/media do not. |
| combobox | disabled (187 only) | **exists** (`disabled`: a disabled option and a disabled combobox). `controlled`, `dynamic` also exist. |
| combobox | multi-select, clear, groups, invalid, auto-highlight, addon | none exist; no part or prop for them (see 4). |
| drawer | swipe handle | **exists** (`DrawerHandle` in `main`, `overlay`). |
| drawer | side positions | partial: `main` has the default Bottom plus a Top drawer; `DrawerSide` has Top/Right/Bottom/Left, so Left/Right are one demo. |
| calendar | month / year dropdown caption | **exists in every demo** (the styled `Calendar` always renders both selectors; `calendar/docs.md:24`). Row 155 group 8 changes the default to a label. |
| calendar | presets, date+time, cell size | none exist. Variants: main, simple, range, multi_month, unavailable_dates, internationalized, rtl. |
| date_picker | typed input | **exists**: `DatePickerInput` has day/month/year segments you type into (`primitives/src/date_picker.rs`, docs "entered it from the keyboard"). Variants: main, range, multi_month, unavailable_dates, internationalized, keep_open. |
| date_picker | time picker, DOB dropdown, natural-language | none exist. |
| select | disabled | option-level **exists** (`main`: Orange); trigger-level (`Select { disabled }`) not demoed. |
| select | long list | no. Variants: main (groups + labels), multi (`SelectMulti`), rtl. |
| hover_card | alignment, delays | one variant; `HoverCardContent` has `side` and `align` props, delays do not exist (row 166). |
| separator | vertical | no (`main` is horizontal only; `horizontal: bool` prop exists). |
| label | with checkbox | no (`main` is input + label). |
| slider | vertical / disabled / controlled / RTL | RTL **exists**, controlled **exists** (`dynamic_range` drives `value`), range **exists**, multiple thumbs via `RangeSlider` **exists**. Vertical and disabled do not (props `horizontal`, `disabled` exist). |
| popover | align, with form, RTL | with form **exists** (`main`: Dimensions inputs); `non_modal`, `overlay` **exist**. Side/align demos, RTL do not. |
| avatar | group, badge, dropdown | `main` has basic, rounded, loading, error, large, **sizes exist**; group/badge/dropdown do not. |
| context_menu | icons, shortcuts, destructive, groups | none; submenu, checkbox, radio, RTL **exist** (`checkboxes`, `radio_group`, `click_only_submenu`, `rtl`). |
| aspect_ratio | image, portrait, square | one 4:3 gradient. |
| scroll_area | horizontal | no (`main`, `rtl` are vertical; `ScrollDirection::Horizontal` exists). RTL **exists**. |
| sheet | sides, no-close | four sides **exist** (`main` Top/Right/Bottom/Left buttons) and `overlay`. No-close / RTL do not. |
| badge | variants, icon | 4 variants + an icon badge **exist**; spinner, count, link do not. |
| command | dialog, groups, shortcuts | **exist** (`main`: CommandDialog, two groups, Kbd shortcuts; `overlay`). Inline (non-dialog) and RTL do not. |
| alert | default + destructive | **exist** with icons. Action button, no-icon, custom colour do not. |
| field | select/radio/slider/switch/textarea/choice-card/payment | only `main` (profile form: legend, group, invalid, horizontal checkbox, responsive, separators). |
| input_otp | disabled, separator, controlled readout | disabled (`otp-disabled`) and `InputOtpSeparator` **exist**. Patterns, verify-login form do not. |
| kbd | group, inline | **exist**. In button / tooltip / input-group do not. |
| native_select | basic | `main` only (label + 5 options); disabled / optgroup / invalid do not. |
| table | -- | `main` has caption, header, body, footer and a total row: **equal** to shadcn's basic table. |
| navigation_menu | -- | `main`, `click_only`: featured link, list items with descriptions; RTL does not exist. |
| accordion | disabled item | **exists** (`main`: item 2 is `disabled`); `until_found` has real FAQ copy; multiple mode has no demo. |
| menubar | -- | `main`, `checkboxes`, `radio_group`, `click_only`, `rtl` **exist**; icons / shortcuts / destructive do not. |
| resizable | vertical, handle | **exist**: `main` has a vertical nested group, a collapsible panel and grip handles; `rtl` exists. Equal. |

**Net correction to the gap list.** Already present in some form, so remove or shrink them: dropdown_menu submenus/radio/checkboxes; dialog custom close; alert_dialog destructive; combobox disabled; drawer swipe handle and half of the side positions; calendar month/year dropdowns; date_picker typed input; slider controlled/range/RTL; popover "with form"; sheet sides; scroll_area/resizable/menubar/context_menu RTL; accordion disabled item; badge icon. Effectively **equal**, no demo work: table, resizable.

## 4. Remaining gaps not covered by the parallel batch

Class: **(a)** demo-only, buildable from existing props and parts (props checked in `component.rs`/primitives); **(b)** needs a component part or prop, with the tracking row; **exists** = already shown (see 3). RTL items are tracked by row 137 (preview LTR/RTL toggle gives every page an RTL example); a hand-built `rtl` variant is possible but 137 supersedes it, so they are classed (b) 137 here.

| Component | Gap | Class | Tracked in |
|---|---|---|---|
| dropdown_menu | icons in items | (a) | -- (lucide icons as item children; no part needed) |
| dropdown_menu | avatar account-switcher | (a) | -- |
| dropdown_menu | radio with icons | (a) | -- |
| dropdown_menu | shortcuts | (b) | 155 g2 |
| dropdown_menu | destructive | (b) | 155 g2 |
| dropdown_menu | groups (the part) | (b) | 155 g2 |
| dropdown_menu | RTL | exists | -- |
| toast | typed variants (success/error/warning/info buttons) | (a) | -- |
| toast | promise loading/success/error | (b) | 138 |
| toast | typed icons | (b) | untracked (138 adds a `loading` type, not icons; CSS is tint only) |
| dialog | no close button | (a) | -- (omit the hand-written button) |
| dialog | sticky footer, scrollable content | (a) | 155 g4 would make `DialogHeader`/`DialogFooter` parts; plain `div` + overflow works today, best written after g4 |
| dialog | built-in close (`showCloseButton`) | (b) | 155 g4 |
| alert_dialog | small size | (b) | untracked (`AlertDialogRootProps` has no `size`) |
| alert_dialog | media icon | (b) | untracked (no `AlertDialogMedia` part) |
| alert_dialog | small + media | (b) | untracked (same two) |
| alert_dialog | non-destructive action (Continue) | (b) | untracked (`.dx-alert-dialog-action` is hard-coded destructive; 188 does not list it) |
| combobox | multi-select chips | (b) | untracked (primitive has no multiple mode; 187 also says so) |
| combobox | clear button | (b) | untracked (no clear in `primitives/src/combobox/`) |
| combobox | groups | (b) | untracked (parts are combobox, empty, input, list, option only) |
| combobox | custom items | (a) | -- (option children are free-form) |
| combobox | invalid | (b) | 155 g1 (invalid ring; check combobox is one of its ~12 controls) |
| combobox | auto-highlight | (b) | untracked (no prop; highlight is internal) |
| combobox | input-group addon | (b) | untracked (no addon slot on the input part) |
| drawer | side positions (Left/Right) | (a) | -- |
| drawer | non-modal, nested, snap points | (b) | 156 |
| drawer | swipe handle | exists | -- |
| calendar | presets | (a) | -- (Calendar + preset buttons setting `selected_date`) |
| calendar | date + time picker | (a) | -- (Calendar + time `Input`) |
| calendar | month/year dropdown caption | exists; layouts (b) | 155 g8 (`caption_layout`) |
| calendar | cell size | (b) | untracked (cell is a fixed `2rem`, `calendar/style.css:171`; needs a `--dx-calendar-cell-size` token) |
| date_picker | time picker | (a) | -- (DatePicker + time `Input`) |
| date_picker | natural-language | (a) | -- (an `Input` + small hand parser + `Calendar` in a `Popover`; no component change) |
| date_picker | DOB with explicit dropdowns | (b) | 155 g8 |
| date_picker | typed input | exists | -- |
| select | disabled | (a) | -- (`Select { disabled }` exists) |
| select | scrollable long list | (b) | untracked (`select/style.css:102-104`: the listbox "never scrolls (no max-height)"; shadcn has max-height + scroll buttons) |
| select | invalid | (b) | 155 g1 |
| select | align-item-with-trigger | (b) | untracked (deliberately not built: `select/context.rs:57-59`, always anchor positioning) |
| hover_card | alignment | (a) | -- (`align`/`side` props) |
| hover_card | trigger delays | (b) | 166 |
| separator | vertical, settings list | (a) | -- (`horizontal: false`) |
| label | with checkbox | (a) | -- |
| slider | vertical, disabled | (a) | -- (`horizontal`, `disabled` props) |
| popover | side/align demos | (a) | -- (`side`, `align` props) |
| popover | RTL | (b) | 137 |
| avatar | dropdown trigger | (a) | -- |
| avatar | group +N, status badge | (b) | 155 g9 |
| context_menu | icons | (a) | -- |
| context_menu | shortcuts, destructive, groups | (b) | 155 g2 |
| aspect_ratio | image / portrait / square | (a) | -- (image via `asset!`) |
| scroll_area | horizontal gallery, release list | (a) | -- (`ScrollDirection::Horizontal`) |
| sheet | no close button | (a) | -- (omit `SheetContentClose`) |
| sheet | typed `side` prop | (b) | 155 g7 (not a demo gap; today a `data-side` attribute) |
| sheet | RTL | (b) | 137 |
| badge | with spinner / count | (a) | -- |
| badge | ghost, link, as-link | (b) | 155 g6 |
| command | inline (non-dialog) | (a) | -- (`Command` renders standalone) |
| command | RTL | (b) | 137 |
| alert | no-icon, custom colour | (a) | -- |
| alert | action button | (b) | 155 g9 (`AlertAction`) |
| field | select/radio/slider/switch/checkbox/textarea/choice-card/fieldset composite | (a) | -- (all components exist; choice-card is a pattern; invalid ring waits on g1 for the extra) |
| input_otp | verify-login form (DemoForm), resend | (a) | -- |
| input_otp | PIN / alphanumeric patterns | (b) | 157 (numeric keypad hard-coded; cold, needs owner approval) |
| kbd | in button / tooltip / input-group | (a) | -- |
| native_select | disabled, optgroup | (a) | -- |
| native_select | invalid / size | (b) | 155 g1 / g7 |
| table | with checkbox / badge columns | (a) | -- (optional; basic table is already equal) |
| navigation_menu | icons | (a) | -- |
| navigation_menu | RTL | (b) | 137 |
| accordion | FAQ copy, multiple | (a) | -- (155 g9 lists the same "Multiple" demo; do it once) |
| accordion | bordered / card variants | (b) | untracked (187 only; CSS has no variant) |
| menubar | icons | (a) | -- |
| menubar | shortcuts, destructive, groups | (b) | 155 g2 |
| resizable | standalone vertical, handle-less | (a) | -- |

**Batch-component gaps that the in-flight demo batch cannot finish** (they need a part or prop; the batch can only show what exists). The brief named several of these as untracked; two are in fact tracked:
| Component | Gap | Class | Tracked in |
|---|---|---|---|
| toggle | outline variant, sizes | (b) | **155 g3** (tracked, not untracked) |
| toggle_group | outline, sizes, spacing | (b) | **155 g3** (decision A, spacing default 2) |
| switch | sizes | (b) | **155 g7** (tracked, not untracked) |
| input_group | block-start / block-end addons | (b) | untracked (`InputGroupAlign` is `Start`/`End` only) |
| input_group | invalid / disabled states | (b) | 133 (via 155 g1) |
| card | small size | (b) | untracked (`Card` has no size prop) |
| tabs | line variant, vertical | (b) | 155 g5 (and row 160 for `Ghost`) |
| item | `xs` size | (b) | 155 g7 |
| breadcrumb | dropdown ellipsis, RTL | (b) | the `span` Ellipsis is already done at HEAD (`breadcrumb/component.rs:123`); RTL 137 |
| breadcrumb, button | router-link / as-link render | (b) | 155 g6 (`r#as`) |
| progress | label + value readout | (b) | 155 g9 (`ProgressLabel`, `ProgressValue`) |
| radio_group, textarea, checkbox, input, switch | invalid ring | (b) | 155 g1 |
| tooltip | delays (if listed) | (b) | 166 |
| button_group | orientation, nested, separator, split | (a) | -- (`orientation_vertical`, `ButtonGroupSeparator`, `ButtonGroupText` exist) |
| button | spinner loading, rounded, button-group | (a) | -- (`Spinner` exists; `border_radius` attribute) |
| spinner | sizes | (a) | -- (no size prop; style attribute) |

### Counts (section 4 main table)

75 gap rows: **35 (a) demo-only**, **36 (b) needs a part or prop** (22 tracked, 14 untracked), **4 already exist** (dropdown_menu RTL, drawer swipe handle, date_picker typed input, calendar dropdown caption).

Tracked (b), by row: 155 g2 x5 (menu shortcuts, destructive, groups across three menus), 137 x4 (RTL: popover, sheet, command, navigation_menu), 155 g1 x2 (combobox, select invalid) plus one combined g1/g7 (native_select), 155 g9 x2, 155 g8 x2 (calendar layouts, DOB dropdowns), 155 g4, g6, g7 (sheet `side`), 138, 156, 166, 157 one each.

Untracked (b), 14: toast typed icons; alert_dialog size, media, small + media, non-destructive action (4); combobox multi-select, clear, groups, auto-highlight, input-group addon (5); calendar cell size; select scrollable long list; select align-item-with-trigger; accordion bordered/card variants.

Second table (batch components that need parts): 16 rows: 3 demo-only (button_group, button, spinner), 11 tracked (155 g3 x2, g7 x2, g5, g6, g9, g1, 133, breadcrumb RTL/137, 166), 2 untracked (input_group block addons, card size).

## 5. Suggested next batch (demo-only items, by lane = component directories)

Each lane owns only its own `preview/src/components/<c>/variants/**` plus new files; none touches `component.rs`, `style.css` or `primitives/`. Disjoint, so all lanes dispatch in one batch. Before starting, grep `playwright/` for any old demo text the lane replaces (CLAUDE.md note), keep forms as `DemoForm` and bundle any image with `asset!` (demo gates), and re-check how a new variant dir is registered with `preview/build.rs` as the batch in flight did.

| Lane | Owns | Demos (all from existing props) |
|---|---|---|
| A. menus | dropdown_menu, context_menu, menubar | icons in items; avatar account-switcher (dropdown_menu); radio items with icons. Leave shortcuts/destructive/groups for after 155 g2. |
| B. overlays | dialog, alert_dialog, sheet, drawer | dialog: no-close, scrollable content, sticky footer (prefer after 155 g4); sheet: no-close; drawer: Left/Right side buttons. |
| C. floating | toast, hover_card, popover | toast: four typed buttons; hover_card: align/side variants; popover: side/align variants. |
| D. dates | calendar, date_picker | calendar: presets, date+time; date_picker: time picker, natural-language input (Input + hand parser + Popover/Calendar). |
| E. pickers | select, combobox, native_select | select: disabled trigger; combobox: custom items (icon + secondary text); native_select: disabled, optgroup. |
| F. primitives | separator, label, slider, aspect_ratio, scroll_area, kbd, badge, resizable | vertical separator + settings list; label with checkbox; vertical and disabled slider; image/portrait/square ratios; horizontal scroll gallery + release list; kbd in button/tooltip; badge with spinner/count; standalone vertical resizable. |
| G. compositions | field, input_otp, accordion, avatar, command, navigation_menu, alert, table | field composite form (select/radio/slider/switch/textarea/choice cards) using the batch's new control demos' patterns; input_otp verify-login card; accordion FAQ + `multiple` (the only owner of the g9 demo); avatar dropdown trigger; command inline; navigation_menu icons; alert no-icon/custom colour; table with checkbox/badge columns. |

Not in this batch: everything marked (b) above. Candidate follow-ups by size, in the order that unblocks the most demos: 155 g2 (menu item parts: dropdown_menu, context_menu, menubar), 155 g9 (avatar group/badge, progress parts, alert action), 155 g1 (invalid ring for combobox, select, native_select and the batch's controls), then the untracked ones that need an owner decision or a row: combobox multi-select/clear/groups, alert_dialog size/media/non-destructive action, select scroll + item-aligned, input_group block addons, card size, calendar cell-size token, toast icons, accordion variants.
