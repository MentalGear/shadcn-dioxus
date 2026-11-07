---
id: 504
title: "A server-rendered Textarea shows no value: the JS-off page and the hydrated first frame hold an empty field"
status: "BACKLOG. Found by the first-paint fix lane (row 468's section \"First paint of a controlled…"
size: "S"
area: textarea
updated: 2026-11-07
---

# Row 504: a server-rendered Textarea shows no value

## Item

`Textarea { value: description }` renders `<textarea value="sample text">` in the SSR page. A `<textarea>` has no `value` attribute (its value is its text content) and Dioxus SSR does not render the attribute as content, so a JS-off visitor sees an empty field. Hydration does not write the property either, so a hydrated textarea whose owner holds text stays empty until the owner's value next changes (the `<p>` in the `main` demo reads "Description here: sample text" beside an empty field).

## Status / trigger

BACKLOG. Found by the first-paint fix lane (row 468's section "First paint of a controlled Textarea"), 2026-11-07.

## Size

S

## Detail

Facts: measured on the SSG build, `first-paint-styles.spec.ts` for `/component/textarea/`. Row 468's mount report no longer overwrites the owner with the empty DOM (it compares `value` with `defaultValue`), so the owner keeps its text and the page is style-identical JS-off and hydrated; the field's content is the open part. A text child puts the renderer's hydration comments (`<!--node-id-->`) into the content, a textarea shows them as text; `dangerous_inner_html` renders the content but is a JS sink (`check-js-volume`, `check-installed-paths`) the rule does not allow to grow. Options to weigh: a bridge write of a control's `value` at mount (a new closed verb row, needs the owner), an SSR renderer path that omits hydration comments for a textarea's text, or documenting the limit. A form reset of such a field returns it to `""`, so the default must stay empty if the fix writes the value after mount.
