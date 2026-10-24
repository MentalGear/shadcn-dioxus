---
id: 136
title: "Questionnaire: shadcn's multi-step question flow (single/multiple choice, freeform \"other\", skippable…"
status: "LANDED 2026-10-19 (see the update below; the owner lifted the hold on 2026-10-18). Was: ON HOLD…"
size: "L"
area: inputs
updated: 2026-10-19
---

# Row 136: Questionnaire: shadcn's multi-step question flow (single/multiple choice, freeform "other", skippable…

## Item

`Questionnaire`: shadcn's multi-step question flow (single/multiple choice, freeform "other", skippable, previous/next/submit, required + custom validation, resume, conditional items, optional answer shortcuts, native `<form>` serialization) is not built

## Status / trigger

**LANDED 2026-10-19** (see the update below; the owner lifted the hold on 2026-10-18). Was: **ON HOLD (owner 2026-10-14: not started, no schedule).** **BACKLOG** -- recorded 2026-10-04 from the shadcn catalog audit (`dev-docs/archive/research/shadcn-catalog-2026-10-04.md` §3.6); shipped by shadcn 2026-08-05, after our 2026-09-03 catalog list, so in scope by the parity rule. Revisit after the chat kit has proved out (`dev-docs/archive/rounds/component-backlog.md`). **2026-10-05 (owner): backlogged and scheduled LAST in the next-session order; trigger unchanged: after the chat kit proves out**

## Size

L

## Detail

AI-agent-flow specific (agent clarification prompts, onboarding, surveys) and the largest item left in the catalog after `message_scroller`. **Needs a new primitive: a headless state machine** -- navigation over an item collection, validation, answer-shortcut keys, focus management -- shadcn's is ~2.7k non-test lines of headless TS (`packages/react/src/questionnaire/`) plus ~4.3k test lines. The fixed choices are **native** `input[type=radio\|checkbox]` in `fieldset`/`legend`, so our `RadioGroup`/`Checkbox` primitives (custom widgets with hidden mirrors) cannot be reused as-is. A11y: `fieldset`+`legend` per item, `aria-describedby` for description and active error, `aria-invalid`, a named `progressbar`, `aria-keyshortcuts`, inactive items `hidden`+`inert`, focus to the new fieldset on navigation and to the first control on a failed validation. **No APG pattern** for a multi-step wizard, so the oracle is tier-3 (port shadcn's own 2,183-line jsdom and 1,421-line browser suites). Cheap to reverse; nothing in the kit depends on it.

## Update 2026-10-19 (landed)

Built as a pure-Rust state machine plus thin Dioxus parts. `primitives/src/questionnaire/model.rs` owns every decision (validity, navigation, key classification, letter/number shortcuts, Mod+Enter, defaults/resume/reset, conditional items, controlled navigation, focus targets). Its 30 unit tests in `model_tests.rs` port shadcn's jsdom cases. `primitives/src/questionnaire.rs` holds the parts: a `form` (`method="dialog"`, submit and reset always `preventDefault`ed), `fieldset`/`legend` items, native radio/checkbox choices, a freeform input, the error, a named `progressbar`, and Previous/Skip/Next/Submit. DOM effects (focus, `reportValidity`) are queued and drained after the render. The styled component is `preview/src/components/questionnaire/` (Nova tokens, 11 demos: main, multiple, freeform, skip, shortcuts, validation, controlled, resume, conditional, navigation_state, progress).

Construction choices, each closing a class instead of an instance:
- **One `Vec<ItemDef>` declares every item.** `QuestionnaireItem` takes only a `name` and `QuestionnaireChoice` only a `value`, so the parts cannot disagree with the collection; shadcn's runtime registration-mismatch warnings have nothing to catch. `definition_warnings` still reports duplicate names, a bad `default_item`, and defaults that are not choices.
- **Serialization is the DOM.** A skipped or disabled item's controls carry no `name`, and an unselected freeform input has no `name` and `form=""`, so the browser's `FormData` is the answer set.
- **Every `display` rule on a part that can be `hidden` is written `:not([hidden])`.** The navigation buttons sit in a `hidden` wrapper because `.dx-button` sets `display`.

Evidence: unit 32/32 (30 model, 2 SSR/RTL); clippy for primitives, default and `web` arms, `--all-targets -D warnings`, clean; `playwright/questionnaire.spec.ts` 17/17 on a lane SSG build. It covers the JS-off SSR state, focus and error wiring, shortcuts, arrows, RTL mirroring through the page toggle, Mod+Enter, skip, the freeform draft, submit `FormData`, conditional, resume/reset, validation, controlled, navigation state, and axe on every demo plus the error state, light and dark. With `--grep questionnaire`, first-paint-styles, all-routes, popup-style-isolation, docs-structure and rtl-toggle pass 10/10; component-catalog and docs-nav-sort pass 24/24; `registry-install-smoke.sh --only questionnaire` is OK 1/1.

RTL (row 137's `check-direction-keys` arrived with the merge): the root reads `use_direction` and hands the model the LTR-equivalent key, so the model only reasons in LTR. The item's `aria-keyshortcuts` names the mirrored key.

Found on the way: a styled wrapper that forwards `{props.children}` into a primitive whose default text depends on "no children" hides that default, because the forwarded fragment is never the empty placeholder. Progress and Error rendered empty until the wrapper passed `children: props.children` instead.

Not ported: shadcn's animated, card and dialog showcase demos; the choice row's `user-select: none` (the theme-baseline gate owns it).

