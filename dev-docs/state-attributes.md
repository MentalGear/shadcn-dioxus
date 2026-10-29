# State attributes: one helper, two vocabularies

Backlog [row 481](./backlog/481-state-attributes-one-helper-emits-data-state-and-presence.md). Source of the vocabulary:
[`research/radix-vs-base-ui-2026-10-28.md`](./research/radix-vs-base-ui-2026-10-28.md), appendix B.

**Owner decision (2026-10-29): Base UI's state-attribute model is the long-term vocabulary.** Our CSS selects each state with the
selector kind shadcn's `base-nova` classes use for that state: a Base UI presence attribute (present when true, absent when false,
e.g. `data-checked`, `data-open`) in nearly all cases, and an ARIA attribute only where base-nova itself styles from ARIA
(`aria-pressed` for Toggle, `aria-invalid` for fields). Components with no base-nova counterpart use presence attributes.
Valued attributes only for genuinely multi-valued states (`data-side`, `data-orientation`); no `="false"` attributes; and
`data-state` retired at the end. A state with no Base UI counterpart reuses Base UI's name for the analogous part where one
exists (Sidebar expanded/collapsed -> `data-open`/`data-closed`, as Collapsible; Resizable handle dragging -> `data-dragging`, as
Slider; NavigationMenu viewport visible/hidden -> `data-open`/`data-closed`); otherwise it takes a `data-<adjective>` presence
attribute (Avatar `data-loading`, `data-loaded`, `data-error`, `data-empty`; verify against Base UI 1.9.0 Avatar, which is not
installed in this checkout). A converter from Base UI selectors to this
format is rejected: users would have to run it, it would not remove the valued-boolean trap, some names have no one-to-one
mapping, and it is upkeep. The conversion is [row 484](./backlog/484-convert-remaining-valued-boolean-state-attributes-to-presence.md)
(valued booleans to presence, Accordion and Collapsible `data-open` first) and [row 485](./backlog/485-retire-data-state-css-moves-to-aria-and-presence.md)
(retire `data-state`). Until they land, the valued booleans and `data-state` in the tables below are still emitted.

## The rule

A component never writes a state attribute (`data-state`, `data-checked`, `data-open`, `data-disabled`, ...) by hand. It spreads
what one function of `primitives/src/state_attrs.rs` returns:

```rust
let owned = attributes!(button {
    role: "checkbox",
    ..state_attrs::checked(checked()),     // data-state + data-checked | data-unchecked | data-indeterminate
    ..state_attrs::disabled(props.disabled),
    ..state_attrs::readonly(props.read_only),
});
```

`scripts/check-state-attributes.sh` fails a file that writes a name of the table below as a literal key, and a `data-*` attribute given
the constant `false` or `"false"`. In an `rsx!` element the spread goes after the last named attribute and before the children;
`attributes!` takes it anywhere.

## Two vocabularies from one call

| Vocabulary | Shape | Who selects on it |
|---|---|---|
| Radix | `data-state="checked"`, `"open"`, `"on"`, `"active"`, ... | the shipped stylesheets, the `tier3-radix` oracle, 88 Playwright files; kept |
| Base UI | a presence attribute: `data-checked=""` while true, **absent** while false | shadcn's `base-nova` class strings and Tailwind's `data-open:` variants |

A presence attribute never takes a boolean value: `data-x=false` would match the presence selector `[data-x]`. The presence helpers
emit an empty value or nothing.

## The vocabulary

| Name | Helper | Emitted as | Base UI part |
|---|---|---|---|
| `data-state` | `open`, `trigger_open`, `checked`, `pressed`, `active`, `panel_hidden`, `selected`, `progress`, `state` | Radix values | all of them |
| `data-checked` / `data-unchecked` | `checked`, `presence(Flag::Checked \| Unchecked)` | presence | Checkbox, Switch, Radio, CheckboxItem, RadioItem |
| `data-indeterminate` | `checked(Checked::Indeterminate)`, `progress` | presence | Checkbox, Progress |
| `data-open` / `data-closed` | `open`, `trigger_open`, `open_valued` | presence; `data-open` stays valued (below) on Accordion and Collapsible | Dialog, Popover, Tooltip, Menu, Select, Collapsible, Accordion |
| `data-popup-open` | `trigger_open`, `popup_open` | presence | triggers of popups |
| `data-panel-open` | `panel_open` | presence | Accordion and Collapsible triggers |
| `data-pressed` | `pressed` | presence | Toggle |
| `data-active` | `active`; `valued`/`presence_true` where legacy | presence on Tabs | Tabs.Tab, NavigationMenu.Link |
| `data-readonly` | `readonly` | presence | Checkbox, Switch, Radio, DatePicker |
| `data-highlighted` | `presence(Flag::Highlighted, ..)` | presence on menu items and Select options | Menu.Item, Select.Item |
| `data-selected` | `selected`, `presence(Flag::Selected, ..)` | presence on Select options and selected rows | Select.Item, rows |
| `data-disabled` | `disabled` | **valued** `true` \| `false` (legacy) | every part |
| `data-invalid`, `data-dragging` | `valued`, `presence_true` | **valued** (legacy) | Field, Slider |
| `data-hidden` | `panel_hidden` | presence on inactive Tabs panels | Tabs.Panel |
| `data-complete` / `data-progressing` | `progress` | presence | Progress |

`data-state` values the helper writes: `open`/`closed`, `checked`/`unchecked`/`indeterminate`, `on`/`off`, `active`/`inactive`,
`selected`, `loading`/`indeterminate` (Progress), and through `state(value)` the parts with no Base UI counterpart (Avatar
`loading|loaded|error|empty`, the NavigationMenu viewport `visible|hidden`, a Resizable handle `dragging|idle`, the Sidebar
`expanded|collapsed`).

## Valued booleans are legacy

`data-disabled`, `data-open` (Accordion, Collapsible), `data-selected`/`data-highlighted` (Combobox option, Command item, Tag, Carousel
item), `data-active` (Input OTP slot, NavigationMenu link, Pagination, Sidebar), `data-invalid` (Field) and `data-dragging` are still
emitted `true | false`, because stylesheets and specs select `[data-disabled="true"]` (about 150 selectors) and some assert
`data-disabled="false"`. They go through `state_attrs::valued`, `disabled`, `open_valued` and `presence_true` (Questionnaire:
`true` while it holds, absent otherwise, so `[data-x]` and `[data-x="true"]` both match exactly the true case). The gate counts each
call per file in `scripts/check-state-attributes.baseline.tsv` (shrink-only); a new component uses `presence`.

Moving a flag to presence is one edit in `state_attrs.rs` (`valued` -> `presence`) plus the selectors and specs that read the old
value (`rg 'data-disabled="?(true|false)'`). `data-state` goes last, after the CSS and the `tier3-radix` oracle have moved.

## Not in the vocabulary (left as they are)

Component-specific attributes keep their own handling: `data-orientation`, `data-direction`, `data-side`, `data-align`, `data-slot`,
`data-index`, `data-position`, `data-variant`, `data-size`, `data-type`, `data-status`, `data-required`, `data-multiple`,
`data-visible`, `data-removed`, `data-collapsed`, `data-placeholder`. The valued booleans `data-nested-open`, `data-streaming`,
`data-task`, `data-show-on-hover`, `data-scroll-anchor` and `data-preserve-scroll-on-prepend` become presence attributes in row
484. The chart
primitives (`primitives/src/chart/`) keep Recharts' `data-active`/`data-state` (a conformance surface with its own harness). The
Message Scroller's `data-active` is written by its page script. `SwitchThumb` renders no state attribute (it has no context yet).
