# State attributes: one helper, one vocabulary

Backlog rows [481](./backlog/481-state-attributes-one-helper-emits-data-state-and-presence.md),
[484](./backlog/done/484-convert-remaining-valued-boolean-state-attributes-to-presence.md) and
[485](./backlog/done/485-retire-data-state-css-moves-to-aria-and-presence.md). Source of the vocabulary:
[`research/radix-vs-base-ui-2026-10-28.md`](./research/radix-vs-base-ui-2026-10-28.md), appendix B.

**Owner decision (2026-10-29): Base UI's state-attribute model is the long-term vocabulary.** Our CSS selects each state with the
selector kind shadcn's `base-nova` classes use for that state: a Base UI presence attribute (present when true, absent when false,
e.g. `data-checked`, `data-open`) in nearly all cases, and an ARIA attribute only where base-nova itself styles from ARIA
(`aria-pressed` for Toggle, `aria-invalid` for fields). Components with no base-nova counterpart use presence attributes.
Valued attributes only for genuinely multi-valued states (`data-side`, `data-orientation`); no `="false"` attributes; and
no `data-state`. A state with no Base UI counterpart reuses Base UI's name for the analogous part where one exists (Sidebar
expanded/collapsed -> `data-open`/`data-closed`, as Collapsible; Resizable handle dragging -> `data-dragging`, as Slider;
NavigationMenu viewport visible/hidden -> `data-open`/`data-closed`); otherwise it takes a `data-<adjective>` presence attribute
(Avatar `data-loading`, `data-loaded`, `data-error`, `data-empty`; Attachment `data-idle`, `data-uploading`, `data-processing`,
`data-error`, `data-done`). A converter from Base UI selectors to this format is rejected: users would have to run it, it would not
remove the valued-boolean trap, some names have no one-to-one mapping, and it is upkeep.

Rows 484 and 485 have landed: no valued boolean and no `data-state` remains in the components. The one exception is the chart
primitives (`primitives/src/chart/`): they keep Recharts' `data-active="true|false"` on bars and arcs and `data-state="open|closed"` on
the chart tooltip, a conformance surface with its own harness and outside the gate.

## The rule

A component never writes a state attribute (`data-checked`, `data-open`, `data-disabled`, ...) by hand. It spreads
what one function of `primitives/src/state_attrs.rs` returns:

```rust
let owned = attributes!(button {
    role: "checkbox",
    ..state_attrs::checked(checked()),     // data-checked | data-unchecked | data-indeterminate
    ..state_attrs::presence(state_attrs::Flag::Disabled, props.disabled),
    ..state_attrs::readonly(props.read_only),
});
```

`scripts/check-state-attributes.sh` fails a file that writes a name of the table below as a literal key, any emission of `data-state`
(`state_attrs.rs` included), any call of a deleted valued helper, and a `data-*` attribute given the constant `false` or `"false"`.
In an `rsx!` element the spread goes after the last named attribute and before the children; `attributes!` takes it anywhere.

## One vocabulary

Every state is a Base UI presence attribute: `data-checked=""` while true, **absent** while false. It is what shadcn's `base-nova` class
strings and Tailwind's `data-open:` variants select on. A presence attribute never takes a boolean value: `data-x=false` would match the
presence selector `[data-x]`. The helpers emit an empty value or nothing. ARIA (`aria-pressed`, `aria-invalid`, `aria-expanded`,
`aria-selected`, ...) is emitted beside it where the role calls for it.

## The vocabulary

| Name | Helper | Base UI part |
|---|---|---|
| `data-open` / `data-closed` | `open`, `trigger_open` | Dialog, Popover, Tooltip, Menu, Select, Collapsible, Accordion, NavigationMenu viewport, Sidebar |
| `data-popup-open` | `trigger_open`, `popup_open` | triggers of popups |
| `data-panel-open` | `panel_open` | Accordion and Collapsible triggers |
| `data-checked` / `data-unchecked` | `checked`, `presence(Flag::Checked \| Unchecked)` | Checkbox, Switch, Radio, CheckboxItem, RadioItem |
| `data-indeterminate` | `checked(Checked::Indeterminate)`, `progress` | Checkbox, Progress |
| `data-pressed` | `pressed` | Toggle |
| `data-active` | `active`, `presence(Flag::Active, ..)` | Tabs.Tab, NavigationMenu.Link, Input OTP slots, Pagination, Sidebar |
| `data-hidden` | `panel_hidden` | Tabs.Panel |
| `data-readonly` | `readonly` | Checkbox, Switch, Radio, DatePicker |
| `data-highlighted` | `presence(Flag::Highlighted, ..)` | Menu.Item, Select.Item |
| `data-selected` | `selected`, `presence(Flag::Selected, ..)` | Select.Item, rows, Tag, Carousel items |
| `data-disabled` | `presence(Flag::Disabled, ..)` | every part |
| `data-invalid`, `data-dragging` | `presence(Flag::Invalid \| Dragging, ..)` | Field, Slider, Resizable handle, Drawer |
| `data-complete` / `data-progressing` | `progress` | Progress |
| `data-loading`, `data-loaded`, `data-error`, `data-empty` | `presence(Flag::Loading \| Loaded \| Error \| Empty, true)` | Avatar (no Base UI 1.9.0 counterpart in this checkout to verify against) |
| `data-idle`, `data-uploading`, `data-processing`, `data-done` | `presence(Flag::Idle \| Uploading \| Processing \| Done, true)` | Attachment (`data-error` shared with Avatar) |
| `data-scrollbar-hidden` | `presence(Flag::ScrollbarHidden, ..)` | ScrollArea with `ScrollType::Hidden` (no Base UI counterpart) |

## From `data-state` to the presence form

`data-state` is no longer emitted. A consumer who selects it in their own CSS or tests moves with this table:

| Was | Is |
|---|---|
| `[data-state="open"]` / `[data-state="closed"]` | `[data-open]` / `[data-closed]` |
| `[data-state="checked"]` / `"unchecked"` / `"indeterminate"` | `[data-checked]` / `[data-unchecked]` / `[data-indeterminate]` |
| `[data-state="on"]` / `"off"` (Toggle, ToggleGroup) | `[aria-pressed="true"]` (or `[data-pressed]`) / `[aria-pressed="false"]` |
| `[data-state="active"]` (Tabs.Tab) / `"inactive"` | `[data-active]` / `:not([data-active])` |
| `[data-state="inactive"]` (Tabs.Panel) | `[data-hidden]` |
| `[data-state="selected"]` (rows) | `[data-selected]` |
| `[data-state="loading"]` / `"loaded"` / `"error"` / `"empty"` (Avatar) | `[data-loading]` / `[data-loaded]` / `[data-error]` / `[data-empty]` |
| `[data-state="idle"]` / `"uploading"` / `"processing"` / `"error"` / `"done"` (Attachment) | `[data-idle]` / `[data-uploading]` / `[data-processing]` / `[data-error]` / `[data-done]` |
| `[data-state="visible"]` / `"hidden"` (NavigationMenu viewport) | `[data-open]` / `[data-closed]` |
| `[data-state="dragging"]` / `"idle"` (Resizable handle) | `[data-dragging]` / `:not([data-dragging])` |
| `[data-state="expanded"]` / `"collapsed"` (Sidebar) | `[data-open]` / `[data-closed]` |
| `[data-state="loading"]` / `"indeterminate"` (Progress) | `[data-progressing]` or `[data-complete]` / `[data-indeterminate]` |
| `[data-x="true"]` / `[data-x="false"]` (a boolean) | `[data-x]` / `:not([data-x])` |

## No valued booleans

A boolean state is a presence attribute, present (and empty) while it holds and absent otherwise; `state_attrs::presence` writes it and
there is no valued form. A stylesheet asks for the true case with `[data-disabled]` and the false case with `:not([data-disabled])`; a
Playwright spec asserts `toHaveAttribute("data-disabled", "")` and `not.toHaveAttribute("data-disabled")`. This covers `data-disabled`,
`data-open`/`data-closed` (Accordion, Collapsible), `data-selected`, `data-highlighted`, `data-active`, `data-invalid`, `data-dragging`,
`data-nested-open`, `data-streaming`, `data-task`, `data-show-on-hover`, `data-scroll-anchor` and `data-preserve-scroll-on-prepend`, the
Calendar day cell's `data-selected`/`data-disabled`, and the attributes the page scripts write (the Message Scroller's jump button
`data-active`, the Carousel's `data-dragging`, the Form's `data-invalid` marker). The helpers `valued`, `disabled`, `open_valued` and
`presence_true` are deleted and the gate fails any call to one; its baseline of constant-false writes is empty.

## Not in the vocabulary (left as they are)

Component-specific attributes keep their own handling: `data-orientation`, `data-direction`, `data-side`, `data-align`, `data-slot`,
`data-index`, `data-position`, `data-variant`, `data-size`, `data-type`, `data-status`, `data-required`, `data-multiple`,
`data-visible`, `data-removed`, `data-collapsed`, `data-placeholder`. The chart
primitives (`primitives/src/chart/`) keep Recharts' `data-active`/`data-state` (a conformance surface with its own harness). `SwitchThumb` renders no state attribute (it has no context yet).
