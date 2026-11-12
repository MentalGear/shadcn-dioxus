---
id: 510
title: "The spec selector has no component-to-component edge: a Button change selects none of its consumers' specs"
status: "BUILT on lane/select-edges-510, awaiting merge. The selector reads the component use graph from the…"
size: "S (one helper module of about 170…"
area: tests
updated: 2026-11-12
---

# Row 510: the spec selector has no component-to-component edge

## Item

`playwright/tools/select.mjs` (`pnpm run test:changed`) mapped a changed `preview/src/components/<c>/**` file to the specs
about `<c>` and nothing else. A styled component that other components compose (Button is the obvious one) reached none
of its consumers' specs, while `dev-docs/lane-rules.md` ("Before handback", step 3) asked the lane to add them by hand. A
lane that followed the one-command rule under-tested silently. Before-evidence on main `bdb41d2a`
(`node playwright/tools/select.mjs --explain --files ...`, specs of 286):

| changed file | specs selected | consumer specs that were missing |
|---|---|---|
| `button/component.rs`, `button/style.css` | 66 | `data_table*.spec.ts`, `alert-dialog.spec.ts`, `calendar*.spec.ts`, `combobox.spec.ts`, `date-picker-*.spec.ts` (164 in all) |
| `input/component.rs` | 19 | `color-picker*.spec.ts`, `combobox-*.spec.ts`, `data_table*.spec.ts`, `calendar*.spec.ts` (133) |
| `label/component.rs` | 12 | `combobox*.spec.ts`, `date-picker-*.spec.ts` (131) |
| `separator/component.rs` | 5 | 176 |
| `popover/component.rs` | 19 | `date-picker-*.spec.ts`, `demos-dates-pickers-primitives.spec.ts` (45) |

## Status / trigger

BUILT on `lane/select-edges-510`, awaiting merge. The selector reads the component use graph from the sources; the main
loop closes this row at the merge.

## Size

S (one helper module of about 170 lines, a 30-line change in `select.mjs`, 33 new self-test checks, four mutations).

## Detail

**The construction** (`playwright/tools/component-graph.mjs`; derived on every run, no list). Four edges, each read from text:

- **code**: a component's own files outside `variants/` name another (`crate::components::B`, a `crate::components::{..}`
  group, `super::super::B` from `component.rs`), or its `component.json` lists it in `componentDependencies`. Transitive.
  The Rust scan is the source because it covers every file of the directory; the manifest is unioned in because the installer
  reads it. `scripts/check-installed-paths.sh` keeps the manifest a superset of `component.rs`'s imports; today manifest and
  scan agree exactly on all 84 components, and the self-test keeps it so (a manifest dependency the scan does not see, or a
  Rust dependency the manifest lacks, is red).
- **demo**: a component's `variants/**` name another. The page of the demoing component embeds it; consumers of that
  component do not render its demos, so this edge is one hop and not transitive.
- **css**: a component's `style.css` spells a class of another (`.dx-B`, longest component name that prefixes the class).
  Same one-hop reach. 77 code, 225 demo and 31 css edges in the tree today.
- **site**: a file of the app outside `components/` names a component (the dashboard, the docs shell, the theme picker, the
  home blocks). `site/mod.rs`, which only imports for its `use super::*` children, is resolved to the children that spell
  the imported types; a `#[cfg(test)]` tail is ignored. A file whose route is unambiguous (`SITE_FILE_PAGE` in `select.mjs`:
  dashboard, the four docs pages, the charts gallery and parity pages, the home blocks and gallery) adds that page's own specs and
  at-rest scan; any other site file (docs nav, theme, i18n, component and variant pages) is on every page and adds the BROAD
  set. A site file reaches the change only through the changed component and its code consumers, not through a demo or css
  consumer, whose own page is the only place its demos render.

**Reach.** A change to `component.rs`, `mod.rs`, `style.css` or any other file in the directory selects the component's own specs
plus the specs of every consumer and the at-rest axe scan of each. A change under `variants/`, or to `docs.md`, `notes.md`,
`component.json`, selects only the component's own specs (the demos of its own page) and fans out nowhere. `--explain` says per spec
why: `preview/src/components/button/component.rs: button <- dialog (code) <- top_layer (code)`.

**The cut-off** is the primitive edge's, one mechanism (`addReach` in `select.mjs` serves both): more than `BROAD_AT` (30)
components reached = shared plumbing, the BROAD set is added and the matrix specs are not narrowed. Button reaches 49
components (12 code consumers, 37 through demos and css), so a Button change selects the BROAD set plus the specs of all 49,
which is 230 of 286 specs. Icons reaches 60. Every other component is below the cut-off.

**Proof.** `node playwright/tools/select.mjs --self-test` (section 7 of `select-selftest.mjs`): a synthetic tree (direct,
transitive, group-import, `super::super`, manifest-only, demo-only and css-only consumers; a variants-only change that must not fan
out; a demo consumer that is not a code consumer; a component added later is a consumer with no edit to the selector; a glob
import is reported as unreadable) and the real tree, asserting the class: no unreadable use, manifest equals scan, Button's
consumers include `dialog` (code) and `top_layer` (transitive) and a demo consumer, `button/variants/**` and `docs.md` reach
nothing, a Button change selects `dialog*` and `data_table*` spec files and names the shared-plumbing cut-off, a
dashboard-only component selects the dashboard's spec and no BROAD set. `scripts/check-select-css.sh` adds four mutations
(no code edge, no demo edge, variants fan out, cut-off gone), each of which must turn the self-test red.

**Cost** (spec files selected and estimated run time, `playwright/durations.json`, a stale snapshot of 2026-10-21 on a debug
build with 2 workers; the figures are estimates, and a narrowed matrix file still counts whole):

| component | specs before -> after | est. time before -> after |
|---|---|---|
| icons | 6 -> 264 | 1170 s -> 12348 s |
| button | 66 -> 230 | 4060 s -> 10460 s |
| separator | 5 -> 181 | 955 s -> 9696 s |
| input | 19 -> 152 | 1685 s -> 7747 s |
| label | 12 -> 143 | 1504 s -> 7027 s |
| checkbox | 9 -> 136 | 1059 s -> 6358 s |
| select | 22 -> 115 | 2078 s -> 5687 s |
| field | 5 -> 100 | 920 s -> 5294 s |
| card | 11 -> 74 | 1374 s -> 4966 s |
| dialog | 23 -> 65 | 2284 s -> 3911 s |
| kbd | 7 -> 34 | 1021 s -> 2345 s |

The full suite is 12850 s. Over all 84 components: 65 change their selection, 19 are unchanged (leaves nothing renders). The
median component goes from 7 to 24 specs and from 1096 s to 1997 s; among the 65 that change the median grows by 28 specs (3.8x),
a median of 1371 s. Six components select half the suite or more.

**Is it too broad?** For the widely composed components it is the true set. Button is in 49 components' pages and in the dashboard,
the docs pages and the home blocks, so a Button change is observable in most of the suite; only the BROAD half could be cut (the matrix
specs narrowed by page, as the CSS half does), and that is unsound without knowing every site page that renders Button, which
the class index of an SSG build gives and the source graph does not. A consumer's own spec file is kept whole: nothing proves
that a spec of `dialog` does not look at its Button. The saving comes from the many components below the cut-off, where
the spec files of 3 to 20 consumers replace the whole matrix. What the tool no longer allows is a lane skipping the consumers silently: a lane that wants a
narrower run now chooses it, with `--explain` showing what it leaves out.

**Open.**
- The cut-off components (Button, Icons) take the whole BROAD set. Selecting only the matrix tests of the reached pages needs the
  page set of an SSG build (`site-index.mjs`), as the CSS half does; feeding the component reach into that machinery is the next
  step if Button runs prove too slow.
- The primitive edge has the same blind spot for site files (a primitive used by `site/` or the dashboard directly).
- `SITE_FILE_PAGE` is a hand table of routes (page placement, not dependencies); a site file added later falls to the BROAD set,
  the sound default.
