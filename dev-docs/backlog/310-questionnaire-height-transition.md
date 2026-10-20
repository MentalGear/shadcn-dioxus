---
id: 310
title: "Questionnaire: animate the height between items instead of snapping"
status: "OPEN (owner review 2026-10-20; implemented on lane/b-quest, awaiting merge)"
size: "S"
area: questionnaire
updated: 2026-10-20
---

# Row 310: Questionnaire: animate the height between items instead of snapping

## Item

Owner request, 2026-10-20: "questionaire: option to have height css transition, not just snap - default on."

## Status / trigger

OPEN (owner review 2026-10-20; implemented on lane/b-quest, awaiting merge)

## Size

S

## Detail

Inactive items are `hidden`, so the form jumped from one item's natural height to the next. `Questionnaire` takes `animate_height` (default `true`; `false` keeps the snap). `height: auto` cannot transition and `interpolate-size` is Chromium-only, so a ResizeObserver (after layout, before paint) pins the old height inline, then the new one in pixels, and `transition: height` in `style.css` runs between them; the pins are cleared when it ends. `prefers-reduced-motion: reduce` keeps the snap. Duration and easing are theme tokens (`--dx-motion-duration-slower`, `--dx-motion-ease-standard`).

- `preview/src/components/questionnaire/component.rs`, `style.css`, `docs.md`; `QuestionnaireHandle::form_id()` in `primitives/src/questionnaire.rs`.
- `playwright/questionnaire-height.spec.ts`.
