# Duplicate-attribute class (`id`/`class`/`role`/`data-*`/...): root-cause deconstruction

Why a literal attribute beside a `..attributes` spread produces a duplicate attribute on the server and a different value
on the client, where the cause lives, and what this repo built about it. Written 2026-09-20 by a read-only investigation
lane against `main`@`6b4bd27` for backlog row 93 (done; the live account is
[`../backlog/done/093-literal-attribute-beside-an-unmerged-props.md`](../backlog/done/093-literal-attribute-beside-an-unmerged-props.md)).
Every claim about `dioxus-core`, `dioxus-ssr`, `dioxus-rsx` and `dioxus-web` was re-verified against the vendored sources of
the versions this repo pins (`0.7.9`). The investigation's prototype guard
([`../archive/issues/duplicate-attribute-guard-prototype.py`](../archive/issues/duplicate-attribute-guard-prototype.py)) became
the `syn`-based gate `scripts/check-attr-spread-collision.sh` (with `scripts/attr-spread-collision-checker/`); the per-site
register of what it baselines is [`duplicate-attribute-findings.md`](./duplicate-attribute-findings.md). The upstream report is
[`drafts/dioxus-ssr-duplicate-attributes.md`](./drafts/dioxus-ssr-duplicate-attributes.md). The three instances that
prompted it: `b35d671` (five sites: `ToastRegionRendered`'s `aria_label`, `Progress`, `ContextMenuRoot`/`Trigger`,
`PopoverTrigger`, `SelectTrigger`), `5fc1439` (`Label`'s `class`, through `#[props(extends = GlobalAttributes)]` plus an
explicit `attributes:` assignment: a different entry point, the same mechanism once both reach one `Vec<Attribute>`) and
`f2be1d7` (`dropdown_menu.rs` and `menubar.rs` root `id`/`role`).

## Layer 1 — the `rsx!` macro: cannot see the collision, by construction

`dioxus-rsx`'s own `Element::merge_attributes` (`element.rs:228-301`) merges only **literal-vs-literal** duplicates written
in one element block. Spreads are appended as opaque `AttributeName::Spread` entries *after* that merge (`element.rs:87-103`):
a spread's contents (`props.attributes`) are a runtime `Vec<Attribute>`, an opaque `syn::Expr` at expansion time, so the
macro cannot know what keys it will hold. **No leverage:** a macro-time fix would need a runtime value's keys at compile time.

## Layer 2 — `dioxus-core`'s VNode/diffing: the data-model gap

A dynamic-attribute slot is a `Box<[Attribute]>` defined as an ordered concatenation, `[static named, remaining dynamically
named]` (`nodes.rs:57-88`), **with no per-name uniqueness invariant**. The shared mount path (`write_attrs`,
`diff/node.rs:822-825`) replays every entry as an unconditional `set_attribute`, never resolving to one value per name; the
re-diff (`diff_attributes`, `diff/node.rs:416-498`) assumes sorted-and-unique lists and has no code for two entries with one
name. **This is the earliest point where the decision "a slot resolves to at most one value per name before a renderer sees
it" is never made**; every renderer inherits the raw list.

## Layer 3 — renderers: same list, asymmetric fallout

- **`dioxus-ssr`** is a stateless string writer (`renderer.rs:146-159`, `write_attribute` at `:482-498`): two `id`
  attributes in one slot produce ` id="a" id="b"` in one start tag, and WHATWG tokenization keeps the **first**, discarding
  the caller's override. The one exception is the `style` namespace: CSS-shorthand properties (`padding: "1rem"`) are
  accumulated and folded into one `style="..."` string (`renderer.rs:232-249`), where same-named *properties* collide in
  CSS's own last-wins cascade and never produce a duplicate HTML attribute (which is why the guard excludes them).
- **The web renderer** lowers to `Element.setAttribute`, and a live `NamedNodeMap` holds one `Attr` per name, so two back-to-back
  calls collapse to the **last**: pure CSR looks correct, by accident of DOM semantics.
- **Hydration does not correct the SSR'd value.** `set_attribute` is skipped while hydrating (`skip_mutations`,
  `dioxus-web` `lib.rs:100`, `mutations.rs:181-183`) and `hydrate.ts` never touches attributes, so in the deployed SSG+hydrate
  mode the browser's parse keeps the first value *permanently*. Only a non-hydrating `dx serve --web` session shows the
  coalesced result, which is why three authors testing locally never saw the bug and only the SSG-lane oracle (parsing real
  static HTML, `hydration-parity.spec.ts` Rule 4) can.
- **Native/Blitz** was not traced (most likely mirrors the web renderer: a structured tree, repeated "set" collapses to
  last-wins). Not determined.

## Layer 4 — this repo's authoring surface: the actionable discriminator

The criterion is "a literal attribute beside a spread, where the props struct has **no typed field** claiming that name".
`use_id_or` (`primitives/src/lib.rs`) reconciles an internal id and a caller `Option<id>` into one `Memo`, so a component
whose caller's `id` reaches a **typed field** (`PopoverRootProps::id`) renders one already-resolved `id:` and has nothing
to duplicate; `DropdownMenuProps`/`MenubarMenuProps` have only `attributes: Vec<Attribute>`, so a caller's `id`/`role` has
nowhere to go but the spread. The mechanism (a name claimed by a typed field never also lands in the
`#[props(extends = ...)]` catch-all) rests on `dioxus-core-macro`'s `extends` machinery; whether exclusivity is
compiler-enforced or only conventional was not traced. This layer holds all the leverage this repo has on its own
timeline: layers 1-2 are upstream.

## Layer 5 — upstream status

`drafts/dioxus-ssr-duplicate-attributes.md` is a draft, not filed. The exact function is `render_template`'s
`Segment::Attr` arm plus `write_attribute` (above). Its two proposed directions hold: (1) `dioxus-ssr` dedupes at
serialization, last-wins: a narrow patch inside `renderer.rs`, fixing only the one renderer that cannot self-coalesce;
(2) `dioxus-core` resolves a slot to one entry per `(name, namespace)` when it is built, once for every renderer: the
durable, larger change. The three regressions are one mechanism reached through two repo-level entry points (a raw
`..spread` and the `extends` ad-hoc-key path), both funnelling into the same `Vec<Attribute>`.

## The answer: where does the root cause live?

- **Mechanistically:** `dioxus-core`'s data model (layer 2): a slot is an ordered list with no uniqueness invariant, replayed
  as unconditional "set" instructions. Everything else (the macro's blindness, the SSR/CSR disagreement, the permanence of the
  wrong value under hydration) follows from that one absent step.
- **Practically, for this repo:** its own authoring surface (layer 4): "a default attribute the caller's spread may override"
  and "a default plus a spread that may redefine the same name" look identical at the call site and both compile silently.

Ranked by leverage: (1) a local static guard or construction change (the only layer actionable now, prevents every future
instance here); (2) an upstream `dioxus-core` resolve-on-construct fix (a "file it and wait" lever); (3) a narrow
`dioxus-ssr`-only patch (partial: not native, not the data model); (4) the macro layer (none).

## What the guard excludes, and what was built

The guard flags a literal attribute (bare ident or `"kebab-string"`) in the same brace block as a `..spread`, where the
enclosing component has no typed field or parameter of that exact name. It **excludes** event handlers and `key` (the same
heuristic `dioxus-rsx` uses) and the CSS-shorthand `style` properties (an authoritative denylist extracted from
`dioxus-html`'s `attribute_groups.rs`, 407 idents, regenerated by
`scripts/attr-spread-collision-checker/regenerate-attribute-idents.py` into the two `*-idents.txt` files beside this
document), because those fold into one `style` string. The first prototype was indentation-based and had known
false-negative modes (single-line element bodies, multi-line literal values), so the committed gate is a real `syn` +
`dioxus-rsx` analyzer. The recommendation that was taken: **(a)** the static guard as the immediate gate, paired with the
oracle (`hydration-parity` Rule 4) to tell live bugs from latent sites among the baselined ones, and the per-site fix
`merge_attributes` (caller wins) or `use_id_or` for a single caller-overridable identity. **Not subsumed** by the literal
scan: a `5fc1439`-shaped collision (an `extends` key plus a separately threaded `attributes:` field passed to a *child*
component), which needs either a wider heuristic or the API-level fix; the gate's component-forward pass and baseline cover
the known sites.

## What could not be determined

Whether `dioxus-native`/Blitz coalesces duplicate `set_attribute` calls like the web DOM; whether the `Props` derive refuses
to compile a name claimed by both a typed field and the `extends` set; and an exhaustive hand-verification of the
prototype's 623 original findings (about a dozen across both authoring styles were spot-checked).
