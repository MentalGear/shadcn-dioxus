---
id: 484
title: "Convert the remaining valued boolean state attributes to presence attributes (Base UI vocabulary)"
status: "BACKLOG, owner decision 2026-10-29. Row 485 (retire data-state) depends on this row: every state…"
size: "M"
area: primitives
updated: 2026-10-29
---

# Row 484: Valued boolean state attributes become presence attributes

## Item

Base UI's state-attribute model is the library's long-term vocabulary (owner decision 2026-10-29). Our selectors mirror shadcn's `base-nova` classes: each state uses the selector kind base-nova uses for it, which is a presence attribute (present when true, absent when false) in nearly all cases and an ARIA attribute only where base-nova itself styles from ARIA (`aria-pressed` for Toggle, `aria-invalid` for fields). Components with no base-nova counterpart use presence attributes. Valued attributes only for genuinely multi-valued states, and no `="false"` attributes. This row converts the valued booleans that remain, so that a Base UI selector (`[data-disabled]`, Tailwind's `data-disabled:`) matches exactly when the state holds. The model and the gate are in [row 481](./481-state-attributes-one-helper-emits-data-state-and-presence.md) and [`dev-docs/state-attributes.md`](../state-attributes.md).

## Status / trigger

BACKLOG, owner decision 2026-10-29. Row 485 (retire `data-state`) depends on this row: every state gets its ARIA or presence form before `data-state` goes.

## Size

M

## Detail

1. **First: Accordion and Collapsible `data-open` (S).** Both emit `data-open="false"` when closed (the baseline counts `valued:Open` in `primitives/src/accordion.rs` (2) and `primitives/src/collapsible.rs` (3)). A Base UI-style `[data-open]` or Tailwind `data-open:` selector therefore matches a closed item today. Switch both to the presence helper in `primitives/src/state_attrs.rs`: `data-open` present while open, `data-closed` present while closed. Migrate the CSS and specs that read `data-open="true"` or `data-open="false"` in the same change. This item lands before the rest.
2. **The remaining valued booleans (M).** Every call the gate counts in `scripts/check-state-attributes.baseline.tsv`. At the time this row was written the baseline holds 94 calls on 58 file-and-call lines: 82 `valued` calls (the `disabled` and `open_valued` calls are counted as `valued:Disabled` and `valued:Open`), 6 `presence_true` calls (Questionnaire) and 6 `false:` writes (item 3). Each converts to a presence attribute through a `presence` helper, and its CSS selectors and spec assertions move with it in the same change:
   - `data-disabled`: about 150 `[data-disabled="true"]` selectors, and specs that assert `data-disabled="false"` (an assertion of `="false"` becomes an assertion that the attribute is absent);
   - Combobox and Command `data-highlighted` and `data-selected`; Tag and Carousel `data-selected`;
   - Input OTP, NavigationMenu link, Pagination and Sidebar `data-active`;
   - `data-dragging` (Slider, ColorPicker, Drawer);
   - `data-invalid` (Field, Questionnaire);
   - Questionnaire, whose `presence_true` calls become `presence` calls.
3. **The six `false:` writes (S).** The six `false:` entries in the baseline are not constants. Each is a valued boolean written as `if x { "true" } else { "false" }`: `data-streaming` and `data-task` (`preview/src/components/markdown/component.rs`), `data-show-on-hover` (`preview/src/components/sidebar/component.rs`), `data-nested-open` (`primitives/src/drawer.rs`), `data-preserve-scroll-on-prepend` and `data-scroll-anchor` (`primitives/src/message_scroller.rs`). They convert to presence like every other valued boolean: each write emits the presence attribute only while `x` holds, and the selectors in the markdown, sidebar and drawer `style.css` files move to `[data-x]`. The readers move with them: `message_scroller.rs` and `message_scroller.js` check `hasAttribute` instead of `=== "true"`. The JS volume can only shrink (`check-js-volume.sh`).
4. **Done when** the baseline file is empty; `state_attrs::valued`, `disabled`, `open_valued` and `presence_true` are deleted from `state_attrs.rs`; `scripts/check-state-attributes.sh` fails any use of them (the gate's self-test covers the failure); and the "Valued booleans are legacy" section of `dev-docs/state-attributes.md` is rewritten to say that no valued boolean remains.
5. **Proof.** SSR tests per converted attribute: present when true, absent when false (the attribute is not written as `="false"`). The affected Playwright specs. The install check, `scripts/registry-install-smoke.sh`, because `component.rs` and `style.css` change. The state-attributes gate with its self-test.
