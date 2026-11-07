---
id: 484
title: "Convert the remaining valued boolean state attributes to presence attributes (Base UI vocabulary)"
status: "DONE 2026-11-06, landed in PR #166 (lane lane/base-vocab-484; see \"Result\"). Owner decision…"
size: "M"
area: primitives
updated: 2026-11-06
---

# Row 484: Valued boolean state attributes become presence attributes

## Item

Base UI's state-attribute model is the library's long-term vocabulary (owner decision 2026-10-29). Our selectors mirror shadcn's `base-nova` classes: each state uses the selector kind base-nova uses for it, which is a presence attribute (present when true, absent when false) in nearly all cases and an ARIA attribute only where base-nova itself styles from ARIA (`aria-pressed` for Toggle, `aria-invalid` for fields). Components with no base-nova counterpart use presence attributes. Valued attributes only for genuinely multi-valued states, and no `="false"` attributes. This row converts the valued booleans that remain, so that a Base UI selector (`[data-disabled]`, Tailwind's `data-disabled:`) matches exactly when the state holds. The model and the gate are in [row 481](../481-state-attributes-one-helper-emits-data-state-and-presence.md) and [`dev-docs/state-attributes.md`](../../state-attributes.md).

## Status / trigger

DONE 2026-11-06, landed in PR #166 (lane `lane/base-vocab-484`; see "Result"). Owner decision 2026-10-29: row 485 (retire `data-state`) depends on this row.

## Size

M

## Detail

1. **First: Accordion and Collapsible `data-open` (S).** Both emit `data-open="false"` when closed (the baseline counts `valued:Open` in `primitives/src/accordion.rs` (2) and `primitives/src/collapsible.rs` (3)). A Base UI-style `[data-open]` or Tailwind `data-open:` selector therefore matches a closed item today. Switch both to the presence helper in `primitives/src/state_attrs.rs`: `data-open` present while open, `data-closed` present while closed. Migrate the CSS and specs that read `data-open="true"` or `data-open="false"` in the same change. This item lands before the rest.
2. **The remaining valued booleans (M).** Every call the gate counts in `scripts/check-state-attributes.baseline.tsv`. At the time this row was written the baseline holds 94 calls on 58 file-and-call lines: 82 `valued` calls (the `disabled` and `open_valued` calls are counted as `valued:Disabled` and `valued:Open`), 6 `presence_true` calls (Questionnaire) and 6 `false:` writes (item 3). Each converts to a presence attribute through a `presence` helper, and its CSS selectors and spec assertions move with it in the same change:
   - `data-disabled`: about 150 `[data-disabled]` selectors, and specs that assert `data-disabled="false"` (an assertion of `="false"` becomes an assertion that the attribute is absent);
   - Combobox and Command `data-highlighted` and `data-selected`; Tag and Carousel `data-selected`;
   - Input OTP, NavigationMenu link, Pagination and Sidebar `data-active`;
   - `data-dragging` (Slider, ColorPicker, Drawer);
   - `data-invalid` (Field, Questionnaire);
   - Questionnaire, whose `presence_true` calls become `presence` calls.
3. **The six `false:` writes (S).** The six `false:` entries in the baseline are not constants. Each is a valued boolean written as `if x { "true" } else { "false" }`: `data-streaming` and `data-task` (`preview/src/components/markdown/component.rs`), `data-show-on-hover` (`preview/src/components/sidebar/component.rs`), `data-nested-open` (`primitives/src/drawer.rs`), `data-preserve-scroll-on-prepend` and `data-scroll-anchor` (`primitives/src/message_scroller.rs`). They convert to presence like every other valued boolean: each write emits the presence attribute only while `x` holds, and the selectors in the markdown, sidebar and drawer `style.css` files move to `[data-x]`. The readers move with them: `message_scroller.rs` and `message_scroller.js` check `hasAttribute` instead of `=== "true"`. The JS volume can only shrink (`check-js-volume.sh`).
4. **Done when** the baseline file is empty; `state_attrs::valued`, `disabled`, `open_valued` and `presence_true` are deleted from `state_attrs.rs`; `scripts/check-state-attributes.sh` fails any use of them (the gate's self-test covers the failure); and the "Valued booleans are legacy" section of `dev-docs/state-attributes.md` is rewritten to say that no valued boolean remains.
5. **Proof.** SSR tests per converted attribute: present when true, absent when false (the attribute is not written as `="false"`). The affected Playwright specs. The install check, `scripts/registry-install-smoke.sh`, because `component.rs` and `style.css` change. The state-attributes gate with its self-test.

## Result (2026-11-06)

- `state_attrs::valued`, `disabled`, `open_valued` and `presence_true` are deleted. Every call site is `state_attrs::presence(Flag::X, on)`;
  the Accordion and Collapsible `data-open` / `data-closed` pair is `state_attrs::open_presence` (present `data-open` while open,
  `data-closed` while closed, never `="false"`). The gate (`check-state-attributes.sh`) fails any use of the four names, its baseline
  is empty (it was 94 calls on 58 file-and-call lines), and the self-test covers the failure.
- Beyond the baseline's calls: the six `false:` writes (`data-streaming`, `data-task`, `data-show-on-hover`, `data-nested-open`,
  `data-preserve-scroll-on-prepend`, `data-scroll-anchor`) are `x.then_some("")`; the Calendar day cell's `data-selected` and
  `data-disabled` (written through `AttributeValue::Bool` in `calendar/day.rs`, which the gate does not see); the Email dashboard's
  `data-selected` / `data-active`, the Carousel API dots, and the Line and Bar chart interactive toggles' `data-active`.
- Page scripts that wrote or read the valued form moved with them: `message_scroller.js` (`data-active` on the jump button is
  toggled, `data-scroll-anchor` and `data-preserve-scroll-on-prepend` are `hasAttribute` / `dataset !== undefined` checks), the
  Carousel drag script (`data-dragging` is set to an empty value), the Form invalid marker script, and the docs page's
  closed-panel probe (`hasAttribute('data-closed')`).
- CSS: every `[data-x="true"]` selector is `[data-x]` and every `[data-x="false"]` is `:not([data-x])`, in the 29 component sheets,
  the theme, `main.css` and the dashboard sheet. The chart primitives keep Recharts' `data-active="true|false"` on bars and arcs
  (a conformance surface, out of the gate's scope).
- Specs: `toHaveAttribute("data-x", "true")` is `toHaveAttribute("data-x", "")`, `"false"` is `not.toHaveAttribute("data-x")`,
  and the `getAttribute` / `dataset` comparisons are presence checks.
- Not touched: the message scroller's allowlist entry in the gate went away (its `data-active` seed was dropped; the script owns the
  attribute).
