---
id: 512
title: "A spec that located a demo variant by its displayed title broke on a retitle; variants are addressed by slug, enforced by a gate"
status: "DONE in PR #178: a variant is found by its slug (the demo frame id); the gate refuses a spec that…"
size: "S: one spec moved, one gate rule with…"
area: tests
updated: 2026-11-12
---

# Row 512: specs address a demo variant by its slug, never by its title

## Item

Row 506: `tag_group.spec.ts` found a demo through its heading, `getByRole("heading", { name: "states" })`. Row 399's title
audit (78 `variants/<slug>/title.txt` renames) changed that heading and the locator matched nothing; the click retried for the
file's 20-minute timeout and parked a deploy. The instance was fixed in row 506. The class stayed open: the convention
"demo controls located by role and position, never by copy" is enforced by `scripts/check-demo-text-locators.sh`, whose
scope was the SHARED specs only (`playwright/oracle/**` plus three named files). A per-component spec was outside it, so a
title locator passed.

## Status / trigger

DONE in PR #178: a variant is found by its slug (the demo frame id); the gate refuses a spec that finds one through its
title.

## Size

S: one spec moved, one gate rule with six self-test fixtures, one doc comment. No Rust, no markup change.

## Detail

**Premise check (measured on the lane's tree).**

- The identity already exists. `ComponentVariantHighlight` / `BlockComponentVariantHighlight`
  (`preview/src/site/variant_page.rs`) give each variant's demo frame the id `component-preview-frame-<slug>`, `<slug>` being the
  directory name under `variants/` (the main demo is `component-preview-frame`). It is unique per page by construction (the
  code comment there says why). `playwright/demo-frame.ts` already has `demoFrame(page, slug)` and
  `demoRole(page, role, n, slug)`, so no new attribute and no new helper were needed. The title heading is an
  `h3.dx-component-variant-title` inside a `div.dx-component-variant` card, shown from `title.txt` or derived from the
  directory name.
- Locating methods over the 286 spec files and their helpers: demo frame by slug id (`#component-preview-frame-<slug>`):
  120 lines in 51 files; `demoFrame`/`demoRole(s)` with a slug argument: 166 lines; `demoFrame`/`demoRole(s)` in all: 36 and
  41 files. By heading title: 1 spec, `tag_group.spec.ts` (`multiVariant`, `getByRole("heading", { name: "multi" })` inside a
  `.dx-component-variant` filter; `statesVariant` located it by a test id). By title class: `focus-target-line.spec.ts`
  asserts the displayed title on purpose. No spec located a variant by a `title.txt` string through `getByText`/`hasText`
  (checked against all 58 `title.txt` files and every directory-derived title).
- Not a title locator: `preview.spec.ts` (the component page's h1), `component-catalog.spec.ts` / `command.spec.ts` /
  `tap-latency.spec.ts` (gallery cards, a different card), `popover*.spec.ts` ("Dimensions", a dialog's own heading).

**The construction.** `tag_group.spec.ts` addresses `multi` and `states` with `demoFrame(page, slug)`. The gate's second
rule (same script, scope every `.ts` under `playwright/` except `tools/`) fails on `dx-component-variant-title`, on a
`.dx-component-variant` locator filtered by heading or text, and on an h3 or level-less heading locator whose literal equals a
variant's title (title.txt, or the directory name with spaces), read from `preview/src/components/*/variants/` on every run, so
a retitle needs no gate edit. The message names `demoFrame` / `demoRole`. A spec about the title is marked
`// title-ok: <reason>` (two lines in `focus-target-line.spec.ts`). `preview.spec.ts` names its heading `level: 1`.

**Not covered.** A heading locator whose literal is not a variant title (the gate cannot tell it addresses a variant); a
variant found through a control's label inside it (a different question, the first rule governs the shared specs only).
