---
id: 395
title: "`as` keeps the component's children and handlers (Radix asChild semantics)"
status: "IN QUEUE on lane/as-keeps-children. Found by row 387's first docs-clarity review; the evidence…"
size: "M"
area: components
updated: 2026-10-24
---

# Row 395: `as` keeps the component's children and handlers (Radix asChild semantics)

## Item

With `as` set, Item, Marker, the Collapsible trigger, Attachment and others render only the closure; children and `onclick`/`onkeydown` given to the component vanish. Radix `asChild` keeps them.

## Status / trigger

IN QUEUE on `lane/as-keeps-children`. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in the project's shared files, `docs-clarity/2026-10-24-docs-clarity-review.md`. Built with Radix semantics (owner decision relayed by the coordinator: breaking renames are accepted pre-1.0).

## Size

M

## Detail

Owner decision before building: merge children and handlers into the closure's element, or keep the current shape, document it once and warn when both are passed. Chosen: merge (Radix `asChild`).

Root cause: every `as` prop was `Callback<Vec<Attribute>, Element>`, a render callback that receives the attributes alone, so a component had nowhere to put its children, and a component whose handlers were rsx on its own element (Item, AttachmentTrigger) never put them in the attributes either. 20 declarations shared the shape (6 primitives, 14 themed components).

Construction:
- `dioxus_primitives::AsChild { attributes, children }` (`primitives/src/as_child.rs`); every `as` prop is `Option<Callback<AsChild, Element>>` and every component calls it with `AsChild::new(merged, children)`. The type makes it impossible to call `as` without handing over the children.
- Handler props are attributes (`attributes!(div { onclick: .. })`, merged through `merge_attributes`, which chains same-named listeners) in Item and AttachmentTrigger, as Button already did, so the `as` element runs them.
- `scripts/check-as-child.sh` fails any `Callback<Vec<Attribute>, Element>` or `as: |x: Vec<Attribute>|` in Rust sources and component docs, so the shape cannot come back.
- Call sites: `as: |el: AsChild| rsx! { a { href: "#", ..el.attributes, {el.children} } }` with the content written as the component's children. Demos and docs of the components this lane owns are in that form; sidebar, dropdown_menu and the app chrome only had their closure signature changed.
- Not covered: a callback that ignores `el.children` still drops them (the type hands them over; rendering them is the caller's choice, as with any render prop).

Tests: `as_child::tests::as_keeps_children_and_listeners` (primitives), `item::component::tests::as_item_keeps_children_and_handlers` (preview).
