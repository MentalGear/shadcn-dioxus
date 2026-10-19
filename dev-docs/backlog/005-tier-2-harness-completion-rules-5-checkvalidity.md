---
id: 5
title: "Tier-2 harness completion — rules 5 (checkValidity/reportValidity/willValidate), 7 (label association), 8…"
status: "LANDED 2026-10-19 (rule 8: form attribute forwarded by Switch, RadioGroup, Checkbox; rule 7…"
size: "S (rule 8 remaining)"
area: tests
updated: 2026-09-19
---

# Row 5: Tier-2 harness completion — rules 5 (checkValidity/reportValidity/willValidate), 7 (label association), 8…

## Item

Tier-2 harness completion — rules 5 (`checkValidity`/`reportValidity`/`willValidate`), 7 (label association), 8 (`form` attribute)

## Status / trigger

**LANDED 2026-10-19 (rule 8: `form` attribute forwarded by Switch, RadioGroup, Checkbox; rule 7 earlier; rule 5 subsumed)**

## Size

S (rule 8 remaining)

## Detail

Rule 7 (a `<label>` association focuses and activates the control) landed via the new `playwright/oracle/tier2-html/form-label-association.spec.ts` (147 lines, commit `2101555`, split out of `form-participation.spec.ts` per that file's own rule-7 deferral note) — calibrated against a native reference control first, per this doc's own Calibration policy; 7/7, twice, on `:8080`. **Known fixture gap, found while landing it, not fixed:** the library `Select`'s field caption is a plain `span`, not a `label[for]` (unlike its own native reference and every other control pair in `FormFixture`), so its CALIBRATION case runs but there is no library-subject counterpart to assert — retarget once `Select` gains a real `label[for]`. Rule 5 (`checkValidity()`/`reportValidity()`/`willValidate` as a general API-surface contract) **closed as subsumed, not landed separately**: `form-participation.spec.ts`'s own header already exercises `willValidate` (its rule 3) and `validity.valueMissing` (its rule 4) on the exact controls that matter, and a standalone rule-5 suite would only re-assert the same wiring under a different name — left open only for a future control that calls `checkValidity()`/`reportValidity()` programmatically, which no current fixture needs. Rule 8 (the `form` attribute associating a control rendered outside its `<form>`) **re-scoped, still open**: needs primitive-level `form` attribute forwarding (the same forwarding Phase 1 deliberately deferred for `Switch`/`RadioGroup`/`Select`'s hidden inputs) before a fixture control can even exist to test against — a primitive-level gap, not just the fixture change this row's text previously implied


**Update 2026-10-19 (lane `hygiene`):** rule 8 landed. `Switch`, `RadioGroup` and `Checkbox` (primitives and themed wrappers) take `form: ReadSignal<Option<String>>` (default `None`, so no `form=""` attribute is emitted, which would orphan the input) and put it on the hidden mirror input; the reset listener already resolves `el.form`, so a form reset restores the outside control. `FormFixture` has a "Form attribute" section (controls outside `#outside-form`, plus a native calibration input) and `playwright/oracle/tier2-html/form-attribute.spec.ts` asserts FormData membership, absence when unchecked, and reset. `Select` is not covered (its hidden input is outside this lane's files).
