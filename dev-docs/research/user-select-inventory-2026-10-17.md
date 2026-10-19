<!-- Investigation report, 2026-10-17, read-only agent run against the 83758d5 debug SSG and the shadcn clone at 30a58d4; scratch paths under /home/user/tgt are session-local. -->

# user-select inventory (read-only, 2026-10-17)

Repo: /home/user/shadcn-dioxus. Nothing edited or committed.

## 1. Every user-select declaration (preview/src, preview/assets, primitives/src)

All values are `none` (18 CSS declarations in 12 files, 3 Rust in 2). No `user-select: text|all`, `::selection`, `-webkit-user-select` or `-webkit-touch-callout` in any CSS file.

| file:line | selector |
|---|---|
| preview/assets/dx-components-theme.css:1157 | `:where([data-dx-menu-item])` |
| preview/assets/main.css:443 | `.dx-preview-code-theme .dx-code-gutter` |
| preview/src/components/avatar/style.css:32, :154 | `.dx-avatar`, `.dx-avatar-badge` |
| preview/src/components/calendar/style.css:87 | `.dx-calendar-nav-prev, .dx-calendar-nav-next` |
| preview/src/components/calendar/style.css:207, :417, :456, :533 | `.dx-calendar-grid-day-header`, `-grid-weeknum`, `-caption-text`, `-year-select-value` |
| preview/src/components/carousel/style.css:68 | `.dx-carousel-content[data-dragging="true"]` |
| preview/src/components/combobox/style.css:183 | `.dx-combobox-option` |
| preview/src/components/command/style.css:309 | `.dx-command-item` |
| preview/src/components/drag_and_drop_list/style.css:69 | `.dx-drag-and-drop-list-item` |
| preview/src/components/drawer/style.css:136, :236 | `.dx-drawer[data-dragging="true"]`, `.dx-drawer-handle` |
| preview/src/components/menubar/style.css:30 | `.dx-menubar-trigger` |
| preview/src/components/select/style.css:234 | `.dx-select-option` |
| preview/src/components/sidebar/style.css:853 | `.dx-sidebar-menu-badge` |
| primitives/src/context_menu.rs:484, :486 (Rust string) | trigger div inline style: `-webkit-touch-callout: none; user-select: none; -webkit-user-select: none; -webkit-tap-highlight-color: transparent; touch-action: none;` |
| preview/src/components/drag_and_drop_list/variants/main/mod.rs:237 | `ImageAvatar` inline `user-select: none;` |

## 2. Theme touch-action baseline

preview/assets/dx-components-theme.css, comment 803-829, rule 830-867 `{ touch-action: manipulation; }`. Selector list exactly as written, in order, each wrapped in `:where(...)`:

```
button[class^="dx-"]  button[class*=" dx-"]  [role="button"][class^="dx-"]  [role="button"][class*=" dx-"]
a[href][class^="dx-"]  a[href][class*=" dx-"]  input[class^="dx-"]  input[class*=" dx-"]
select[class^="dx-"]  select[class*=" dx-"]  textarea[class^="dx-"]  textarea[class*=" dx-"]
summary[class^="dx-"]  summary[class*=" dx-"]
[tabindex]:not([tabindex="-1"])[class^="dx-"]  [tabindex]:not([tabindex="-1"])[class*=" dx-"]
[role="menuitem"] / [role="menuitemcheckbox"] / [role="menuitemradio"]  (each: [class^="dx-"] and [class*=" dx-"])
[role="option"] / [role="tab"] / [role="switch"] / [role="checkbox"] / [role="radio"]  (same pair each)
[role="slider"] / [role="spinbutton"]  (same pair each)
```

- scripts/check-theme-baseline.sh (~86-100) checks that the rule exists and lists five selectors, not user-select. The press-feedback rule (theme:895-921) repeats a shorter copy.
- Not listed: role="link", role="gridcell" (tag_group.rs:820, 1016). role="combobox" is covered only because the select trigger is a `button`.

## 3. Interactive parts: can they be text-selected today?

Selectable (no declaration reaches them):
- Button `.dx-button`, button_group, toggle, toggle_group items, tabs and accordion triggers
- Checkbox, switch, radio_group items, label `.dx-label`, badge, breadcrumb, pagination, navigation_menu links
- Navbar items, `a.dx-item`, marker, sidebar menu button/action/sub-button, slider thumb
- Calendar day cells `.dx-calendar-grid-cell` (calendar/style.css:219)
- Carousel controls (outside a drag), tag group tags and remove buttons
- Select trigger, native_select, popover/dropdown/tooltip/hover-card triggers, sheet/dialog/alert-dialog actions, toast close, form action, toolbar button, chart toggle, color_picker buttons and thumbs, input_group addon

Not selectable (covered by a declaration):
- Menu items of dropdown_menu, context_menu, menubar (theme:1157, via `data-dx-menu-item`)
- Select options, combobox options, command items, menubar trigger, DnD list items (incl. remove button)
- Calendar nav prev/next, weekday header, weeknum, caption, year value
- Drawer handle; drawer and carousel content only while dragging

shadcn (/home/user/shadcn-ui/ui/apps/v4/registry):
- new-york-v4/ui has `select-none` in 15 files: avatar, calendar, combobox, command, context-menu, dropdown-menu, input-group, kbd, label, menubar, native-select, scroll-area, select, sidebar, slider.
- new-york-v4 button, toggle, tabs, accordion, checkbox, switch, radio-group, badge, pagination, breadcrumb, navigation-menu, sheet, dialog, popover, tooltip, carousel have none. bases/base and bases/radix button.tsx DO have it.
- bases/base/ui/drawer.tsx:155 sets `select-text` on drawer content (overlay is `select-none`). 49 shadcn files total.

## 4. Must stay selectable

- `input`, `textarea`, native `select`, contenteditable date segments, prose and dx- `<a>` links, code text (only `.dx-code-gutter` is none). Form labels are selectable here; shadcn's Label is select-none.

## 5. iOS specifics

- `-webkit-user-select` and `-webkit-touch-callout` appear only in context_menu.rs:484/486. drawer/style.css:131-135 says Safari has supported unprefixed `user-select` since 15.4 (not verified here).
- Long-press callouts on dx- links and images are not suppressed by any stylesheet.

## Suggestion (not applied)

1. Yes: `user-select: none` can join the zero-specificity block at theme:830-867 (second declaration or sibling rule, same list), so the element set lives in one place.
2. Exclude `input`, `textarea`, `select`, `[role="spinbutton"]` (contenteditable segments), `[role="slider"]`, the `[tabindex]` catch-all (prose and code in scrollers and cards), and likely `a[href]` and `summary`.
3. Unverified: a drag starting outside may still sweep through a none element in some engines. Confirm in Chromium and WebKit first.
