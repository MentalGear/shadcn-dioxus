---
id: 93
title: "Literal attribute beside an unmerged props spread with no typed field claiming it — the duplicate-attribute…"
status: "CLOSED 2026-10-19 (bookkeeping: status already said done). Found 2026-09-20 (round 4, f2be1d7's own…"
size: "S–M for the check script itself…"
area: overlays
updated: 2026-10-19
---

# Row 93: Literal attribute beside an unmerged props spread with no typed field claiming it — the duplicate-attribute…

## Item

Literal attribute beside an unmerged props spread with no typed field claiming it — the duplicate-attribute class row 85's own fix swept for but deliberately left unfixed

## Status / trigger

CLOSED 2026-10-19 (bookkeeping: status already said done). Found 2026-09-20 (round 4, `f2be1d7`'s own `..props.attributes`/`..attributes` sweep of `primitives/src/**` and `preview/src/components/**`); **awaiting the repo owner's go-ahead** — the proposed fix-by-construction (a new static `check-*.sh` gate) touches dozens of files across primitives and themed wrappers, so it is deliberately not bundled into this round's PR. **CLOSED 2026-09-25 (round 8)** — the gate (`scripts/check-attr-spread-collision.sh`) had already landed (`ed0ea61`, `f8c2dec`); this round burned its debt register down 788 → 1, see this row's own dated addendum below

## Size

S–M for the check script itself (mirrors the existing seven guards' shape — a source-level scan, no full parser needed once attribute/typed-field names are enumerable per component); separately, M–XL to fix every site the gate then flags, exact count unknown until it is written and run — at minimum the five named `id` instances below, plus an unenumerated "dozens" of `class`/`role`/`data-*`/`type`/`tabindex` sites

## Detail

The class: a literal attribute rendered beside an unmerged props spread (`..props.attributes`) where the props struct has no typed
field claiming that name. A first sweep (`f2be1d7`) found five more unexercised `id` instances (alert-dialog title/description,
virtual-list container, resizable panel, toast title/description, the DnD instructions) and a much wider shape of `class`, `role`,
`data-*`, `type`, `tabindex` literals across dozens of wrappers; three manual passes (`b35d671`, `5fc1439`, `f2be1d7`) each caught
only what some demo page happened to exercise, so the row asked for a construction.

**Root cause (2026-09-20, re-verified against the vendored dioxus 0.7.9 sources).** `dioxus-core`'s dynamic-attribute slot is an
ordered, concatenated list with no per-name uniqueness (`nodes.rs`: literal first, spread second, duplicates permitted) and the
one mount path replays every entry with an unconditional `write_attribute`; `rsx!` can merge only literal-vs-literal pairs it
sees in the token stream, and `dioxus-ssr` writes everything else straight through. Hydration does not repair it either
(`dioxus-web` gates `set_attribute` behind `skip_mutations()` throughout hydration), so the two renderer modes resolve a
collision to opposite values: `dx serve --web` lets the caller's value win, production SSG plus hydrate keeps the library's
literal. A consumer's `Alert { class: "mine" }` works in dev and is silently dropped in production, which is why only the
SSG-backed hydration-parity oracle can show it. A guard prototype counted 623 sites in 66 files (a floor). The full diary is in
[the root-cause note](../../issues/duplicate-attribute-root-cause.md).

**Closed 2026-09-25 (round 8).** The gate had already landed (`ed0ea61` builds `scripts/check-attr-spread-collision.sh` and its
`syn`/`dioxus-rsx` analyzer; `f8c2dec` wires it in); the round burned the debt register from 788 sites (687 primary plus 101
component-forward) down to 1, across four disjoint lanes (overlay and menu primitives 216, remaining primitives 314, preview
wrappers 223, carousel 21; chart came down separately through row 90's struct-update construction).

- **One construction everywhere:** `attributes!()` plus `merge_attributes(vec![defaults, caller, owned])`. `owned` (component
  state, `role`/`aria-*` wiring, managed `tabindex`, `popover`, positioning `style`, an aria-referenced `id`) always wins, which
  preserves each component's pre-fix production behaviour; `defaults` (base `class`, which still concatenates, `data-slot`,
  default `aria-label`, `type="button"`) are caller-overridable; event handlers are never merged.
- **The one remaining register entry** is `PopoverTrigger`'s `id`, a verified false positive (the caller vec is filtered of any
  `id` before the merge).
- **Evidence:** all gates green; full Chromium run at `51a96a6` 1015 passed / 31 failed / 18 skipped, every failure pre-existing,
  a load flake, or by design; an SSG release build showed zero duplicated attribute names across 73 prerendered pages;
  `hydration-parity.spec.ts` 40 passed / 1 failed, the failure being the deliberately red `selftest:unfixed:id`.
- **Side effect:** `merge_attributes` sorts its output by name, so rendered attribute order changed; two string-slicing tests
  (a `calendar.rs` unit test and chart's `tooltip_fragment()`) were re-anchored on the tag start. Row 108 holds the by-construction
  fix for that test-fragility class.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/093-literal-attribute-beside-an-unmerged-props.md`.*
