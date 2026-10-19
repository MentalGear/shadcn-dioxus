---
id: 136
title: "Questionnaire: shadcn's multi-step question flow (single/multiple choice, freeform \"other\", skippable…"
status: "ON HOLD (owner 2026-10-14: not started, no schedule). BACKLOG -- recorded 2026-10-04 from the…"
size: "L"
area: inputs
updated: 2026-10-14
---

# Row 136: Questionnaire: shadcn's multi-step question flow (single/multiple choice, freeform "other", skippable…

## Item

`Questionnaire`: shadcn's multi-step question flow (single/multiple choice, freeform "other", skippable, previous/next/submit, required + custom validation, resume, conditional items, optional answer shortcuts, native `<form>` serialization) is not built

## Status / trigger

**ON HOLD (owner 2026-10-14: not started, no schedule).** **BACKLOG** -- recorded 2026-10-04 from the shadcn catalog audit (`dev-docs/research/shadcn-catalog-2026-10-04.md` §3.6); shipped by shadcn 2026-08-05, after our 2026-09-03 catalog list, so in scope by the parity rule. Revisit after the chat kit has proved out (`component-backlog.md`). **2026-10-05 (owner): backlogged and scheduled LAST in the next-session order; trigger unchanged: after the chat kit proves out**

## Size

L

## Detail

AI-agent-flow specific (agent clarification prompts, onboarding, surveys) and the largest item left in the catalog after `message_scroller`. **Needs a new primitive: a headless state machine** -- navigation over an item collection, validation, answer-shortcut keys, focus management -- shadcn's is ~2.7k non-test lines of headless TS (`packages/react/src/questionnaire/`) plus ~4.3k test lines. The fixed choices are **native** `input[type=radio\|checkbox]` in `fieldset`/`legend`, so our `RadioGroup`/`Checkbox` primitives (custom widgets with hidden mirrors) cannot be reused as-is. A11y: `fieldset`+`legend` per item, `aria-describedby` for description and active error, `aria-invalid`, a named `progressbar`, `aria-keyshortcuts`, inactive items `hidden`+`inert`, focus to the new fieldset on navigation and to the first control on a failed validation. **No APG pattern** for a multi-step wizard, so the oracle is tier-3 (port shadcn's own 2,183-line jsdom and 1,421-line browser suites). Cheap to reverse; nothing in the kit depends on it.
