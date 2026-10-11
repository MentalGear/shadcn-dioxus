# shadcn demo gaps -- 25 components whose shadcn demos are richer than ours (2026-10-11)

Source: the owner's comparison of 61 shadcn/ui documentation pages against our preview demos, recorded 2026-10-11. This is a **demo** comparison (what the example on the page shows), not the config-gap comparison behind backlog row 155 (props and parts). Tracked as backlog row 187; the utilities question that came up with it is row 186.

Result: 25 components have a clearly richer shadcn demo, 36 are equal or better. Most fixes are a demo file (`preview/src/components/<name>/variants/main/mod.rs` or a new variant), not a component change; 20 of the 25 are S and 5 are M (items 6, 7, 8, 9, 11; 6, 7 and 9 also wait on parts that do not exist yet, see "Dependencies" below).

## The table (verbatim from the owner)

| # | Component | What shadcn's demo shows that ours lacks | Suggested demo | Effort |
|---|---|---|---|---|
| 1 | checkbox | Labelled checkbox with description, disabled/invalid states, checkbox group; ours is one bare unlabelled Checkbox | Terms checkbox in Field with label+description; "Show these items" group with disabled/invalid | S |
| 2 | input | Labelled Field with description, invalid/disabled, inline search-with-button, multi-field form; ours one unlabelled input | "Username" Field with description, invalid example, inline search row with Button | S |
| 3 | switch | Labelled switch with description, choice-card, disabled/invalid, sizes; ours bare "Switch Demo" | "Airplane mode" + "Multi-factor authentication" in Field; one disabled | S |
| 4 | radio_group | FieldSet legend/description, choice cards, disabled group, invalid; ours bare option1..3 | "Subscription plan" fieldset + choice-card variant | S |
| 5 | tabs | Realistic panels (Overview/Analytics/Reports/Settings), vertical, disabled tab; ours "Tab 1 Content" | Real panel copy + one disabled tab | S |
| 6 | combobox | Multi-select chips, clear button, groups, custom items, invalid/disabled, input-group addon; ours single-select | Multi-select "Frameworks" chips with clear + groups | M |
| 7 | date_picker | Labelled date fields: DOB with month/year dropdowns, typed input, date+time, presets; ours unlabelled | Labelled Field with typed input + date+time row | M |
| 8 | input_group | inline-end/block addons, text prefix/suffix (https://, .com, USD), buttons, Kbd, dropdown, spinner, textarea counter; ours search+convert only | URL field with prefix/suffix; textarea with 0/280 footer | M |
| 9 | dropdown_menu | Icons, shortcuts, destructive item, radio with icons, avatar account switcher, grouped complex menu | Avatar account switcher with groups, icons, shortcuts, destructive Log out | M |
| 10 | context_menu | Icons, shortcut hints, destructive item, grouped separators | File-row context menu with icons, shortcut, destructive Delete | S |
| 11 | field | Select/radio/slider/switch/checkbox/textarea/choice cards/fieldset/payment form; ours one text profile form | Subscription form: FieldSet, radio plan, switch, select | M |
| 12 | button_group | Split button with dropdown, input+button, select+button, popover; ours prev/today/next | Split "Follow" button + input+Button group | S |
| 13 | empty | Team invite with avatar group, offline user, 404 with search; ours "No messages yet" | "No team members" with avatar group + Invite button | S |
| 14 | avatar | AvatarGroup +N, AvatarBadge status, avatar dropdown trigger; ours one row | Team group with +3 overflow + status badge | S |
| 15 | card | Image-topped event card; terms card with scroll text + Decline/Accept; ours login only | Event card with image + terms card | S |
| 16 | accordion | FAQ with real answers, multiple mode, disabled item, bordered/card variants; ours main is lorem ipsum | Replace main with the FAQ from until_found; add multiple | S |
| 17 | spinner | In Button ("Generating..."), Badge ("Syncing"), InputGroup, empty state; ours one Saving button | Save in progress + Syncing badge | S |
| 18 | separator | Settings menu rows with descriptions; label/value list; ours "One thing / Another thing" | Settings list with separators | S |
| 19 | tooltip | Kbd shortcut tooltip; tooltip on disabled button via span; ours rich content only | Icon Save button with Kbd tooltip; disabled-button tooltip | S |
| 20 | skeleton | Table and form skeletons; ours card + profile row | Table-row + form skeletons | S |
| 21 | collapsible | Nested file tree; settings panel with inputs; ours flat changelog | Nested file-tree collapsible | S |
| 22 | textarea | Field invalid state with message; composer with Send button; ours styling only | Message composer + invalid Field | S |
| 23 | input_otp | "Verify your login" form, resend, support link; PIN/alphanumeric patterns; ours bare passcode | Verify-your-login card with OTP, Resend, Verify | S |
| 24 | progress | Labelled progress with value readout; ours unlabelled auto bar | "Uploading..." with percent readout | S |
| 25 | scroll_area | Realistic list (release tags); ours "item N" placeholders | Release-history list | S |

## Equal or better than shadcn (recorded, no rows)

alert, alert_dialog, aspect_ratio, attachment, badge, breadcrumb, bubble, button, calendar, carousel, command, data_table, dialog, drawer, hover_card, item, kbd, label, marker, menubar, message, message_scroller, native_select, navigation_menu, pagination, popover, resizable, select, sheet, sidebar, slider, table, toast, toggle, toggle_group, and the chart family.

## Dependencies on row 155 (and what is not in it)

Several demos can only be written once the part they show exists. Checked against row 155's nine groups on 2026-10-11:

| Demo items | Needs | Where it comes from |
|---|---|---|
| 9 dropdown_menu, 10 context_menu | `Shortcut`, `Group`, `Separator`, `variant="destructive"`, `inset` | Row 155 **group 2** (menu item parts, `primitives/src/menu_item.rs`). Icons in items and the avatar trigger need nothing new. |
| 13 empty (avatar group), 14 avatar (+N overflow, status badge) | `AvatarGroup` + `AvatarGroupCount`, `AvatarBadge` | Row 155 **group 9** (small parts). |
| 24 progress (percent readout) | `ProgressLabel` + `ProgressValue` | Row 155 **group 9**. Without them the demo can only place a plain text node beside the bar. |
| 16 accordion (multiple mode) | the "Multiple" demo | Row 155 **group 9** lists the same demo (`allow_multiple_open` exists, no demo shows it). Do it once, in one lane. |
| 7 date_picker (DOB with month/year dropdowns) | `caption_layout` (`dropdown` and friends) | Row 155 **group 8** (Calendar, decision B: default flips to `label`, so date_picker needs an explicit dropdown layout anyway). The typed-input and date+time row do not wait. |
| 3 switch ("sizes"), 5 tabs (vertical), 1/2/4/22 (invalid states) | Switch `size`; Tabs vertical; the invalid ring | Row 155 **group 7**, **group 5**, **group 1** (the invalid ring is 145's state recipe). The labelled, described and disabled parts of those demos do not wait; only the "sizes", "vertical" and "invalid" extras do. |
| 8 input_group (states) | InputGroup invalid/disabled styling | Row **133**. |
| **6 combobox (multi-select chips, clear button)** | a multi-select Combobox | **Not in row 155's 27** (the combobox primitive, `primitives/src/combobox/`, has no multiple mode; it is not among the 27 "Add now" gaps, so it is at best one of the 53 "Add later" items in the published comparison artifact). This is a primitive feature, so the M here is a floor and the item needs its own decision and brief before it is scheduled. |

Everything else (the base demos of 1-5, 8, 11, 12, 15, 17-23 and 25, and the non-extra parts of the rest) is demo-only: no component change, one lane per component directory.

## Notes for whoever picks this up

- Each demo goes through the existing gates unchanged: no remote images (`check-demo-remote-assets.sh`: bundle with `asset!` or inline for the event card, #15), every `<form>` a `DemoForm` (#2, #11, #22, #23), no hard-coded `font-family`.
- Several demos also exercise controls that other rows are about to restyle (invalid ring: 145 group 1; Tabs padding: row 132). Write the demos after those rows, or expect to touch them once more.
- The demos on a component's main variant also feed the docs page; the Playwright specs that assert on the old demo text (for example `Tab 1 Content`, `option1`, `One thing`) must move with them. Grep `playwright/` for the old strings before editing.
- Suggested order inside the lane: the demo-only items first (no dependency), then the items that wait on 155 groups 2, 8 and 9 in the same batch as those groups, then #6 only after the owner schedules a multi-select combobox.
