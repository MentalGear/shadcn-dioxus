# Final sweep: recurring problems and duplicated concerns, deconstructed to a construction

Row 365. Read-only review of `origin/queue/batch-26` at `de2725bd` (2026-10-25). Nothing was built, no cargo ran. Model of the exercise:
[`collection-model-2026-10-22.md`](./collection-model-2026-10-22.md) (row 360). Nothing below is adopted: the owner decides each proposal first.

Marking: **verified** = I read the file and line cited. **measured** = a count I ran (the command is named). **inferred** = my reasoning, not
checked against a running build; each one says what would check it.

## Executive summary

The tree is in good shape against the problems it has already named: the gate allowlists are small (the `*.baseline.tsv` files hold 25 live
sites in total, all one cluster in `sidebar/style.css`; the attr-spread, history and comment-ref baselines are empty or one line) and most classes
from earlier rounds (render clock, listeners, cfg axis, duplicate attributes, the modal `::backdrop`) are closed by construction. What is left
sits in four places, and none of it is in the gates' *verdicts*; it is in the **exceptions and the copies around them**:

1. **Exceptions that name a missing helper** (Playwright). About 124 `goto-ok` / `rest-ok` / `viewport-ok` markers plus 20 locator and 17
   viewport allowlist entries say "this is a different kind of page / element / read" in prose, one line at a time. The kinds are three or four
   and can be helpers, so the marker stops being needed (P2).
2. **A half-finished unification**. Row 145 moved the modal `::backdrop` into the theme; the same scrim is still painted by five component
   sheets for the no-top-layer arm, with three keyframe duplicates sitting on the `check-duplicate-keyframes` allowlist waiting for it; the
   visually-hidden recipe is pasted 12 times in two spellings (P1).
3. **Hand-kept registries that a gate then cross-checks**: 826 hand-forwarded wrapper props guarded by a 25-entry allowlist (P3); 83 component
   manifests whose dependency lists `check-installed-paths` already computes (P5); the anchor engine's list of nine marker classes pasted into
   every rule (P7); slot variables documented by hand for 10 components while 60 are read (P8).
4. **Copies that drifted**. Eighteen gate scripts each embed their own CSS parser, and three of those differ in behaviour (P4). The three menu
   hosts copy the item component and have already diverged in the type of an item's `value` and in the press and focus handling (P10). 46 blocks
   (about 2,400 lines) of JavaScript live in Rust raw strings, so every JS-quality gate has to regex Rust, while a TypeScript bundle chain
   exists and holds one file (P9).

Ranked by value over cost (the two L items are worth more than their rank, they cost more):

| # | Proposal | Size | Makes impossible |
|---|---|---|---|
| P1 | Theme owns the no-top-layer scrim and the visually-hidden recipe | S | a sheet pasting the scrim or the sr-only block |
| P2 | Spec page and read kinds as helpers, markers retired | S-M | a spec reading a non-app page, a static box or a page metric without the helper that states which |
| P3 | Themed wrappers forward with `..props` | M | a primitive prop that the wrapper silently drops |
| P4 | One gate kit: CSS walker and allowlist engine | M | two gates reading the same stylesheet differently |
| P5 | Component manifests derived from source | M | a manifest that disagrees with the files it ships |
| P6 | Scaffolding copies: one test runtime harness, controlled state only through `use_controlled` | S | a 9th copy of the harness or a 4th hand-rolled controlled state |
| P7 | Anchor engine selects on one attribute | M | a new anchored overlay that is unpositioned because it is not in a list |
| P8 | Slot variables derived from the sheet | S-M | a slot read in a sheet but undocumented, or documented but not read |
| P9 | JavaScript lives in TypeScript, not in Rust strings | L | a JS block that is untyped, unlinted and invisible to the gates |
| P10 | One menu-family implementation behind three hosts | L | the three menus drifting in item behaviour |

Examined and not proposed are at the end, so the owner sees what was looked at and why it was left.

---

## P1. Theme owns the no-top-layer scrim and the visually-hidden recipe

**Size: S** (CSS, five wrappers' class lists, one gate extension).

**Invariant.** What the theme owns once is not declared per component (row 145's own rule, enforced for `touch-action`, `::backdrop` and
`user-select` by `check-theme-baseline.sh`). A modal's scrim, and "visible to assistive tech only", are theme concerns.

**Where it is broken (verified).**

- *Scrim wrapper.* `dialog/style.css:15-24`, `alert_dialog/style.css:~19-28` (declaration at `:23`), `command/style.css:52-60` (`:55`),
  `drawer/style.css:27-35` (`:30`), `sheet/style.css:26-34` (`:29`) each declare the same `position: fixed; z-index: var(--dx-z-overlay);
  backdrop-filter: blur(var(--dx-overlay-blur)); background: var(--dx-overlay-scrim); inset: 0; opacity: 0; will-change: opacity` block, the
  same `:has(> dialog) { backdrop-filter: none; background: transparent }` and the same `[data-dx-overlay="off"]` rule. `command/style.css:44-50`
  says in a comment why it could not share `.dx-dialog-backdrop`. Three of them also carry their own opacity keyframes
  (`dialog/style.css:43,57`, `alert_dialog/style.css:47,61`, `command/style.css:79,93`) that are byte-identical to the theme's
  `dx-fade-in/-out` (`dx-components-theme.css:1720,1730`); `check-duplicate-keyframes.sh:44-51` allowlists exactly those three, with the reason
  "moves to the shared pair with the modal panels (row 145)". The theme half of row 145 landed (`dx-components-theme.css:920-950`, "The modal scrim,
  once"); this half did not.
- *Visually hidden.* The same sr-only recipe is hand-written 12 times: `pagination/style.css:163`, `data_table/style.css:131`,
  `code_block/style.css:130`, `command/style.css:35`, `breadcrumb/style.css:81`, `sidebar/style.css:42,709`, `calendar/style.css:465`,
  `chart/style.css:417`, `message_scroller/style.css:172`, `assets/main.css:2295,2426`. Two spellings exist (`clip: rect(0, 0, 0, 0)` in
  data_table, breadcrumb, calendar; `clip-path: inset(50%)` in the rest). Measured: a script over `preview/src/components/*/style.css` grouping
  identical rule bodies of four or more declarations found only these two clusters across three or more components.

**Root cause.** The scrim's *top-layer* half is addressable from the theme (a `dialog[class^="dx-"]::backdrop` selector). The wrapper `div`
that paints the scrim when there is no top layer has no theme-visible name, only a per-component class, so each component had to write it.
Visually hidden has no theme utility at all.

**Construction.**

1. The five wrappers carry `"data-dx-scrim": ""` next to their class (the pattern `"data-dx-root": ""` already follows). The theme gets one
   zero-specificity `:where([data-dx-scrim])` rule: the base block, `:has(> dialog)`, `[data-dx-overlay="off"]` and the closed-state animation
   through the shared `dx-fade-out`. The five component blocks and three keyframe pairs are deleted; the allowlist entries are deleted.
2. The theme gets one `:where([data-dx-visually-hidden])` (the Tailwind/shadcn `sr-only` recipe, one spelling). Components set the attribute;
   the 12 declarations go.

**Subsumes.** The three `check-duplicate-keyframes` allowlist entries; 11 of 12 sr-only blocks (the `.dx-charts-sr-only` rule in `main.css` is
the docs site's own and can use the attribute too). **Does not subsume:** the `::backdrop` rule (already one), sheet-specific panel animation.

**Gate that keeps it closed.** Extend `check-theme-baseline.sh` the way it already guards `::backdrop`: no component sheet may declare
`background: var(--dx-overlay-scrim)` or the visually-hidden recipe (`clip-path: inset(50%)` with `height: 1px`, or `clip: rect(0, 0, 0, 0)`);
the two theme rules must exist. Browser half: the existing computed-style snapshot (`playwright/computed-style-states.ts`) before and after,
which is how row 145 proved "no computed-style difference".

**Inferred, to check first.** That the five wrappers all render the primitive's `data-state` on the same element as the class (true for
alert dialog, `alert_dialog.rs:~140`; the others were not read to that depth).

---

## P2. Spec page kinds and read kinds are helpers, and the exception markers retire

**Size: S-M** (three helper functions, a mechanical sweep of the specs, three gate edits). Spec-only; can run beside any Rust lane.

**Invariant.** A Playwright spec says *what kind of thing it is looking at*, once, in code, rather than once per line in a comment the gate
reads. The gates (`check-spec-goto-hydrated`, `check-spec-at-rest`, `check-spec-viewport-bound`, `check-demo-text-locators`) are right to exist;
the exceptions they are given are the missing construction.

**Where it is broken (measured with `grep -rn "<marker>:" playwright --include=*.ts`).**

| Marker | Count | What the reasons say |
|---|---|---|
| `goto-ok:` | 58 | 30 are "not the app" (23 are "vendored W3C APG reference page ... never sets `html[data-hydrated]`", across `oracle/tier1-apg/menu-roles`, `disclosure-navigation`, `sortable-table`, `select-only-combobox`, `focus-restore-reference`, `carousel`; four `about:blank`; three static fixtures); 23 are "JavaScript is off / wasm aborted / first-paint oracle: nothing hydrates by design" |
| `rest-ok:` | 47 | about 30 are "the trigger / a page sibling / the navbar / a static row does not animate"; the rest are "settled by the helper above" |
| `viewport-ok:` | 19 | "page scroll extent", "page-level sideways overflow", "the size this test restores" |
| allowlists | 20 + 17 | `check-demo-text-locators.sh:63` ALLOW: 16 of 20 entries say "W3C's vendored example page, not a demo"; `check-spec-viewport-bound.sh:66-84` ALLOW: 17 entries, nearly all "scroll extent, not a containment bound" |

The vendored-reference-page exception is therefore written in two gates (about 39 places). `playwright/hydration.ts` has one helper, `gotoHydrated`,
and everything that is not "the interactive app" has to opt out of it by prose.

**Root cause.** The helpers model one kind of page (hydrated app) and one kind of box read (overlay at rest). Every legitimate other kind is a
bare Playwright call plus a comment, so the gate can only say "unless you wrote a reason".

**Construction (one helper per kind, so the gate can allow by construction):**

- `playwright/hydration.ts`: `gotoHydrated` (exists), `gotoPrerendered(page, url)` (no hydration wait, for JS-off, wasm-aborted and first-paint
  oracles; asserts nothing hydrated if the context is JS-off), `gotoReference(page, url)` (a vendored or route-fulfilled page; fails if the URL is
  under the app's origin or base path).
- `playwright/animations.ts`: `staticBox(locator)` next to `measureAtRest`: reads the box, and **checks** that the element and its ancestors have
  no running animation (`getAnimations()`), throwing if they do. The 30 "does not animate" markers become a verified fact instead of a comment.
- `playwright/viewport-box.ts`: `pageMetrics(page)` returning `{ scrollExtent, sidewaysOverflow, gutter }` (the three things the 19 `viewport-ok`
  lines and the 17 allowlist entries actually read); `fixedViewportBox` stays the one overlay bound.

**Subsumes.** About 53 of the 58 `goto-ok` (the other five are individually reasoned), about 30 `rest-ok`, about 15 `viewport-ok`, 16 `copy-ok`-style
allowlist entries (a locator through `gotoReference`'s returned page object needs no exemption because the gate checks the *helper*). **Does not
subsume:** `focus-ok`, `listener-ok`, `key-ok` markers (they sit in Rust and JS, not specs, and each is one-off), nor the `rest-ok` lines whose
reason is "the sampler measures motion itself".

**Gate that keeps it closed.** Same three gates, changed from "reason on the line" to "the call is one of the named helpers": `check-spec-goto-hydrated`
accepts `gotoHydrated|gotoPrerendered|gotoReference` and nothing else; `check-spec-at-rest` accepts `measureAtRest|readAtRest|staticBox`;
`check-spec-viewport-bound` accepts `fixedViewportBox|pageMetrics`. Their self-tests already mutate; add a mutation per helper.

**Inferred.** That `gotoReference` can enforce "not the app" cheaply (URL origin check). The 23 JS-off specs may need two helpers (JS-off vs
wasm-aborted); the sweep will show.

---

## P3. Themed wrappers forward with `..props`, not field by field

**Size: M** (mechanical over 47 wrapper files; a good Haiku lane with `check-wrapper-props` as the verifying gate, reviewed by the main loop).

**Invariant.** A themed wrapper forwards every prop of the primitive it renders (row 391). A prop added to a primitive must reach the wrapper's
users without anyone remembering to edit 83 wrappers.

**Where it is broken (measured).** `grep -E "^\s+[a-z_#]+: props\.[a-z_#]+,?$" preview/src/components/*/component.rs`: **826 forwarded-field lines in
47 files**, of which **816 are `x: props.x`** (identity) and 10 are renamed. Largest: `calendar/component.rs` 69, `date_picker` 60,
`dropdown_menu` 52 (`dropdown_menu/component.rs:21-35` forwards eight fields one by one, `:60-70` five), `context_menu` 51, `virtual_grid` 47,
`combobox` 42, `select` 37. The consequence is `scripts/check-wrapper-props.allow.tsv` (25 live entries, ten of them "Internal part; the
primitive generates its id") and a Rust scanner (`scripts/wrapper-props/main.rs`, with its own unit tests) whose job is to notice a field someone
forgot. The construction already exists in the repo: `chart/component.rs:67,98,126` forward with `chart::ChartContainer { attributes: merged, ..props }`,
and the file says so (`:85-86`, "precedented in this repo, e.g. `primitives/src/toast.rs`'s own `Toast { ..props }`").

**Root cause.** A wrapper that changes one field (the merged `attributes`) writes out all the others because that is what a props struct looks
like in rsx; the struct-update spread is the shorter and drop-proof way, and it is documented in one place only.

**Construction.** A wrapper that only changes `attributes` is `prim::Comp { attributes: merged, ..props }` (clone `props.attributes` first, as
`chart/component.rs:58` notes). A wrapper that changes more than one field keeps explicit fields for the ones it owns and spreads the rest.
Result: a new primitive prop reaches the themed API with no edit, by construction.

**Subsumes.** The ~800 identity-forward lines; the `check-wrapper-props` finding class for every converted wrapper; the 8 `*.id` allowlist
entries whose "reason" is only "the wrapper does not touch it" (the spread forwards them, which is the correct behaviour; the primitive's own
`id` generation is unaffected when the prop is `None`). **Does not subsume:** wrappers that deliberately change a prop (slider's `index` pinned to
0, `select::SelectValue.placeholder` passed conditionally, `color_picker` composing a popover with its own surface props): those stay explicit and
keep their allowlist reasons. The gate keeps running for them.

**Gate that keeps it closed.** `check-wrapper-props.sh` as is (it already treats `..props` as forwarding every prop), plus a new rule inside it: a
wrapper call that names more than N fields all as `x: props.x` fails ("use `..props`"). The allowlist shrinks to the deliberate cases.

**Inferred.** The fraction of the 47 files that convert cleanly. Evidence for a high fraction: 816 of 826 forwarded lines are identity. Rust's
partial-move rules (`props.children` used after a spread, a field read after the spread moved it) are the likely friction; `chart/component.rs`
shows the `.clone()` pattern that handles it. A first lane should convert three wrappers (a small one, `dropdown_menu`, `calendar`) and report the
friction before the sweep.

---

## P4. One gate kit: a shared CSS walker and one allowlist engine

**Size: M** (scripts only; no product code). Do it as one lane; it must be the only lane touching `scripts/check-*.sh` while it runs.

**Invariant.** Two gates that read the same stylesheet read it the same way. A sheet's rules, at-rule context, line numbers and top-level
selector splits are computed once.

**Where it is duplicated (measured with `grep -lE "^def (strip_comments|split_top|split_top_level|parse_rules|style_rules|blocks|walk|rules|css_rule_bodies_for_class|norm)\b" scripts/check-*.sh`).**
Eighteen gate scripts (5,031 lines in total) define their own comment stripper, brace walker or selector splitter inside a bash heredoc:
`check-variant-scope.sh:108-141` (`style_rules`), `check-cross-component-overrides.sh:112-173` (`parse_rules`), `check-popup-boundary.sh:113-160`
(`parse_rules`), `check-press-under.sh:46-62` (`rules`), `check-popup-root-reset.sh:107-117` (`find_rule`), `check-infinite-animations.sh:69-110`
(`blocks`/`walk`), `check-hover-media.sh:73,188` (`walk`), `check-component-root-reset.sh:150`, `check-demo-wrapper-width.sh:86,192`,
`check-theme-preset-axes.sh:43`, `check-focus-outline-reset.sh:59`, `check-scroll-behavior.sh:84`, `check-theme-baseline.sh:143`,
`check-cursor-baseline.sh:93`, and more. `split_top` alone is defined in 10 scripts, `strip_comments` in 10.

**They have drifted (verified by reading).**

- `cross-component-overrides.sh:121-130` and `popup-boundary.sh:121-130` handle a backslash inside a quoted string; `variant-scope.sh:115-123` does
  not (its `style_rules` has no escape branch), so a `\"` in a `content:` or `url()` value ends the quote early.
- `press-under.sh:46-47` strips comments **without keeping the newlines** (`re.sub(..., "", ...)`), while the others replace with the same number of
  newlines "so line numbers survive" (`cross-component-overrides.sh:112-114`): its reported lines are off.
- `popup-root-reset.sh:107-117` finds the rule with `re.finditer(r"([^{}]*\[popover\]...)\{([^{}]*)\}")`, which does not see a rule nested in an
  `@media`/`@supports` block the way the walkers do.

None of these is a known false negative today (I did not search the sheets for one); they are three behaviours for one input, which is how a
gate silently stops seeing a rule after a CSS refactor.

**The allowlist engine is the second copy.** About 40 scripts implement "entry needs a reason, a stale entry fails, `--list` prints" by hand
(`grep -l stale scripts/check-*.sh | wc -l` = 40; the structure differs: `ALLOW` tuples, `ALLOWLIST` dict, `file|callee` keys, TSV baselines, `*-ok:`
markers).

**Root cause.** Each gate was written as a self-contained heredoc (so a lane can add one without touching shared code), and there is no
`scripts/lib`.

**Construction.** `scripts/lib/css.py` (`strip_comments`, `rules(css) -> [Rule(prelude, decls, line, at_ctx)]`, `split_top`, one quote/paren/escape
aware scanner) and `scripts/lib/allow.py` (`Allowlist(entries)`: reason mandatory, `used` tracking, stale report, `--list`). A gate imports them
(`PYTHONPATH=scripts/lib python3 -`, or `python3 -I` with an explicit `sys.path` insert). The scanner has one self-test (fixtures for escapes, nested
at-rules, comments with newlines, `url(data:...;...{)`) in a `check-gate-kit.sh`. Migration is gate by gate; each migrated gate's own
mutation self-test is the regression oracle.

**Subsumes.** The three drifts above; roughly 1,000 duplicated lines; the next gate's boilerplate. **Does not subsume:** the Rust-source scanners
(`comment-scan.py` is already shared; `wrapper-props` and `attr-spread-collision-checker` are Rust `syn` tools) or the shell-only gates.

**Gate that keeps it closed.** `check-gate-kit.sh` (the fixtures) plus a rule inside it: no `scripts/check-*.sh` heredoc may define `strip_comments`
or `split_top` (a grep). `check-gate-reuse.sh` and `cargo-inputs-hash.sh` need the lib added to their declared inputs; the reuse gate's own
"a new reader is red until reviewed" rule will say so.

**Risk.** The 13,957 lines of `check-*.sh` are the repo's safety net; a migration that loosens one is silent. Mitigation: each migrated gate runs
its self-test and the full `run-gates.sh` must be byte-identical in `--list` output before and after (capture the baseline first, as the CLAUDE.md
brief requires).

---

## P5. Component manifests are derived from source, not hand-kept

**Size: M** (one generator, `--check` in `run-gates.sh`, the existing gates become its tests). Scripts plus 83 JSON files; no product code.

**Invariant.** What `dx components add` installs is exactly what the component's files need. A component's manifest and the files it ships
cannot disagree.

**Where it is hand-kept (verified/measured).** `preview/src/components/*/component.json`: 83 manifests. 76 carry the identical
`exclude: ["variants","docs.md","component.json"]` and `globalAssets: [theme]`; the other 7 add one theme asset (`dx-effects.css`). 26 list
`componentDependencies`; all list `cargoDependencies` with the same `dioxus-primitives` git block. 83 `mod.rs` files are the same one sentence in two
spellings (47 on two lines, 36 on one: `cat */mod.rs | sort | uniq -c`). `check-installed-paths.sh:21-53` already *computes* each of these from the Rust
sources to compare with the manifest (every `crate::components::<other>::` in `componentDependencies`, every external crate in `cargoDependencies`
of the closure, `mod.rs` exactly `mod component; pub use component::*;`, `exclude`/`globalAssets` entries existing); `check-cargo-dep-versions.sh`
and `check-registry-url.sh` check more fields; `check-component-catalog.sh` check 2/2b/3 check the same facts again. The failures this has caused are
in `CLAUDE.md`: 27 missing `mod.rs` found only when a user's `cargo check` hit E0583, `kbd` and four more without `cargoDependencies` (E0433), and
the pre-merge smoke script (~6 GB, minutes) exists because the static approximation was not enough.

**Root cause.** The manifest is authored, the Rust files are authored, and a gate verifies they agree after the fact. Agreement is a computation,
not a decision.

**Construction.** `scripts/gen-component-manifests.sh` (or a `props_scan`-style Rust scanner, the repo's precedent: `preview/props_scan.rs`)
writes `componentDependencies` (from `crate::components::<other>` uses outside `variants/`), `cargoDependencies` (crates used, with the version from
`Cargo.lock`, the pin rules of `check-cargo-dep-versions` baked in), `globalAssets` (from the presence of `style.css`, plus a one-line
`assets.txt` only where a component also needs `dx-effects.css`), `exclude`, and `mod.rs`. Hand-authored fields stay: `name`, `description`,
`authors`. `run-gates.sh` runs `gen-component-manifests.sh --check` (byte-identical, the `build-js-bundle`/`check-js-bundle` model: the generated
file is committed, its manifest is the proof).

**Subsumes.** The "manifest says X, source says Y" failure of `check-installed-paths` halves 2-3, `check-cargo-dep-versions`'s
`cargoDependencies` floor check, and `check-component-catalog` checks 2/2b/3 for the fields it derives. **Does not subsume:** the
`registry-install-smoke.sh` end-to-end install (it tests dx itself), the `CATALOG` classification (`Origin::Shadcn|Extra` is a decision, not a
computation), and the `document::Link` line in each component (see "Examined and not proposed").

**Gate that keeps it closed.** `gen-component-manifests.sh --check` in `run-gates.sh`; the existing four gates stay as the cross-checks until the
generator has run clean for a batch, then lose their duplicated halves.

**Inferred.** That the external-crate detection is as easy in a generator as in the gate (the gate's own scan, `check-installed-paths.sh`
second half, is the starting point). If a crate use is behind `cfg`, the generator needs the same rule the gate has.

---

## P6. Scaffolding copies: one test runtime harness, controlled state only through `use_controlled`

**Size: S** (primitives, tests and three hand-rolled signals; one grep gate).

**Invariant.** A concern implemented once in `primitives/src` is used, not copied. Two instances here.

**Where it is duplicated (verified).**

- *The test runtime harness.* `fn with_runtime(f: impl Fn() + 'static)` and a `struct TestHarnessProps { test_fn, result }` are written out in
  `slider.rs:1045,1065`, `pointer.rs:143,163`, `selection.rs:236,258`, `color_picker.rs:748,768`, `direction.rs:287,303` (a variant that returns a
  `Direction`), `calendar/tests.rs:1070,1090`, `date_picker/tests/state.rs:9,29`, `virtual/identity.rs:92`: 8 `with_runtime`, 7 `TestHarnessProps`.
  `slider.rs:1043` says "mirrors `virtual/virtualizer.rs`'s `with_runtime`". `primitives/src/test_support.rs` exists, is `#[cfg(test)]` and holds
  an SSR start-tag parser (row 108), but not this.
- *Controlled-or-uncontrolled state.* `use_controlled` (`lib.rs:256`) has 28 callers. Three hand-roll the same
  "internal signal, memo of `controlled.or(internal)`, callback that sets both": `alert_dialog.rs:119-124` (an exact copy: `use_signal(default_open)`,
  `use_callback` set plus `on_open_change`, `use_memo(props.open or signal)`, while its sibling `dialog.rs:239` calls `use_controlled`),
  `accordion.rs:313`, and `drawer.rs:894-896` (the snap-point state; its `reset` to the default is *different*, so it is a candidate, not an exact copy).

**Root cause.** The harness was written as a private test module twice before anyone looked; `use_controlled` landed after `AlertDialogRoot` was written
and nothing finds the stragglers.

**Construction.** `test_support::with_runtime` (generic over a return value, covering `direction.rs`'s variant) imported by all tests;
`AlertDialogRoot` and `AccordionRoot` call `use_controlled`; `drawer.rs` either takes a `use_controlled` plus its own reset or documents why not.

**Subsumes.** The 8 harness copies and 2 exact controlled-state copies. **Does not subsume:** `drawer.rs` unless the reset can be expressed; the
`VirtualDom::new` calls that are not this harness (111 occurrences overall, most render SSR).

**Gate that keeps it closed.** One more rule in an existing grep gate (`check-hooks-in-closures.sh` or a new line in `check-component-call.sh`'s
neighbour): no `struct TestHarnessProps` outside `test_support.rs`, and no `use_memo(... unwrap_or_else(&*` in a component outside `use_controlled`
(the pattern is in `grep -rnE "unwrap_or_else\(&\*[a-z_]+\)|\.or_else\(&\*[a-z_]+\)"`, which today matches exactly the three sites plus `navbar.rs:696`, a
different thing).

---

## P7. The anchor engine selects on one attribute, not a hand-kept list of nine marker classes

**Size: M** (primitives `top_layer`, 9 overlay primitives, one browser spec run; cross-component).

**Invariant.** A floating panel that asks to be anchored is anchored. Registering a new anchored overlay is not a second edit in a list far from
it.

**Where it is broken (verified).** `primitives/src/top_layer/styles.rs` is the engine's injected stylesheet: the selector group
`.dx-anchor-tooltip[popover], .dx-anchor-hover-card[popover], .dx-anchor-popover[popover], .dx-anchor-popover:modal,
.dx-anchor-dropdown-menu[popover], .dx-anchor-menubar[popover], .dx-anchor-navbar[popover], .dx-anchor-navigation-menu[popover],
.dx-anchor-select[popover], .dx-anchor-combobox[popover]` is written out at `:157-158`, again at `:164-172`, and then once per
`data-side` (top/right/bottom/left at `:208-261` and onward) and per `data-align`, so ten names appear in dozens of selector lines
(`grep -o "dx-anchor-[a-z-]*"` over primitives: `dx-anchor-popover` 53, `dropdown-menu` 29, `navbar` 26, `tooltip` 25 ...). The doc at `:45-66`
explains the reason: a hashed CSS-module class would not match the engine's `@supports` rules
(`dev-docs/issues/css-module-supports-scoping.md`), so every consumer carries a hand-written plain class, and the list lives in the engine. A new
anchored overlay has to be added here by hand. I found no static gate that checks the list is complete: `check-anchored-keyframes.sh` is about
`transform` in keyframes; the only checks are browser specs (`oracle/tier2-html/top-layer.spec.ts` and the per-component ones). A missing entry
renders un-anchored, a visual failure found late. `dx-anchor-context-menu` (twice in the tree) is deliberately not in the list (point anchored).

**Root cause.** The marker is both the *scope* ("anchor me") and the *name* ("I am a tooltip"), so the engine enumerates names.

**Construction.** One attribute, `data-dx-anchored="<kind>"`, written by `anchored_content_attributes` (`top_layer`, already called by 25 sites: the
one writer, like `data-dx-modality`). The engine's rules select `[data-dx-anchored][popover]`; a `kind` selector appears only where kinds really
differ (hover-card's own rules at `styles.rs:268-301`). A new anchored overlay is anchored by calling the function it must already call.

**Subsumes.** The 10-name lists and the omit-one-name failure. **Does not subsume:** per-kind differences (hover card), nor the point-anchored
context menu.

**Gate that keeps it closed.** `check-anchor-marker.sh` (new, tiny): every file that calls `use_anchor_position_fallback` also calls
`anchored_content_attributes`; no `dx-anchor-` class literal outside `top_layer/`. Browser half: the existing top-layer spec.

**Inferred.** That the per-kind CSS differs only in hover-card. I read the first hover-card rules and the grouped selectors, not every one of
the 686 lines; a first pass is `grep` of each non-grouped selector in `styles.rs`.

---

## P8. Slot variables are derived from the sheet, not documented by hand

**Size: S-M** (`preview/build.rs`/a scanner beside `props_scan.rs`, 10 `docs.md` edits, one gate).

**Invariant.** The slot variables a component reads (`var(--dx-label-gap, ...)`, the composition contract row 170 made the rule) are listed
exactly once, and the list is right.

**Where it is hand-kept (measured, heuristic).** A scan of `preview/src/components/*/style.css` for `var(--dx-<component>-<name>, fallback)`
reads of the component's own prefix that its own sheet never declares found **60 slot reads, 11 without a mention in the component's
`docs.md`**: `calendar` (`--dx-calendar-background`, `-border`), `chart` (`--dx-chart-center-primary-max`, `-secondary-max`,
`--dx-chart-legend-item-basis`), `kbd` (`--dx-kbd-background`, `-color`), `drawer` (`--dx-drawer-drag`), `dropdown_menu`
(`--dx-dropdown-menu-trigger-focus-ring`), `select` (`--dx-select-scroll-button-height`), `toggle_group` (`--dx-toggle-group-spacing`). Some are
internal (set by script), which the scan cannot tell. Only 10 `docs.md` files have a "Slot variables" section (`label/docs.md:14-22` is the model).
A second hand list says the same thing again: `check-css-vars-defined.sh:72-96` ALLOWLIST holds 19 slot names with reasons like "slot,
label/docs.md" plus 3 names written by JS.

**Root cause.** A slot is a fallback in CSS, a table in markdown, and an allowlist line in a gate, three spellings of one fact; the Props reference
of each page is generated (`preview/props_scan.rs`), the slot table is not.

**Construction.** Generate the "Slot variables" table at build time from the sheet: a read of the component's own prefix with a fallback, not
declared in the same sheet, is a slot; its row is `name`, `fallback`, and the property it feeds (the declaration the `var()` sits in). A short
`/* slot: <what it is for> */` comment on the first read supplies the description; a slot with no comment fails the gate, a variable set by JS says
`/* slot-internal */`. `check-css-vars-defined.sh` treats a derived slot as defined, so its 19 entries go.

**Subsumes.** The 11 undocumented reads, the hand tables of 10 pages, 19 allowlist entries. **Does not subsume:** slots in the theme's own tokens (the
theme is its own documentation) or custom properties set by JS (3 entries remain).

**Gate that keeps it closed.** `check-props-docs.sh`-style: a slot read with no description fails; a `docs.md` may not declare its own
`## Slot variables` (as it may not declare `## Props reference`).

**Inferred.** The 11 count is a lower-bound heuristic (own-prefix reads only; a slot read through a different prefix, e.g. Field setting
`--dx-label-*`, is counted under label). It is a sizing figure, not a defect count.

---

## P9. JavaScript lives in TypeScript modules, not in Rust strings

**Size: L** (a lane per cluster: carousel, virtual engine, top layer and overlay, the rest; the bundle chain exists). High value, so worth
scheduling even though it ranks late on cost.

**Invariant.** Code in a language has that language's tools: types, lint, one place to read it. The gates that guard JS (`check-js-listeners`,
`check-blocking-scroll-listeners`, `check-uncleared-intervals`, the comment scan of embedded JS) should read JS files, not regex Rust strings.

**Where it is broken (measured).** `grep -rln "document::eval\|eval(" primitives/src` = 45 files. A scan for raw strings of four or more lines that
contain JS tokens found **46 blocks, about 2,420 lines**, plus the carousel's `concat!` macros: `virtual_grid.rs:1954` (`CELL_CONTROLS_JS`),
`:2087` (`FOCUS_CELL_JS`), `virtual/find.rs:135,282`, `virtual/bridge.rs:293` (`SCROLL_BRIDGE_JS`), `virtual/scroll_to.rs:190`,
`select/components/list.rs:102` (`ITEM_ALIGNED_JS`), `top_layer/scroll_clamp.rs:58,141,261`, `top_layer/styles.rs:643`, `collection.rs:17`,
`modality.rs:70,87`, `carousel/{autoplay,wheel,drag,scroll}.rs` (7,431 lines under `carousel/` in all). `modality.rs:70` is a **minified one-liner**
with `/*listener-ok: ...*/` comments inside it. Meanwhile `primitives/src/ts/` holds exactly one file (`focus-trap.ts`) and the whole chain exists
for it: `scripts/build-js-bundle.sh`, `primitives/src/js/bundle.manifest`, `check-js-bundle.sh` (committed `.js`, sha256 manifest, rebuild when the
bun version matches).
The cost shows up as exceptions and copies: `listener-ok:` 18, `blocking-ok:` 4, `interval-ok:` 3 markers; the modality writer exists in three
byte-identical copies that `check-open-focus.sh:34,115` must compare (`modality.rs`, `PREPAINT_JS`, `preview/index.html`), and `PREPAINT_JS` in two
that `check-theme-mode-writer.sh:132` compares.

**Root cause.** `document::eval(r#"..."#)` is the path of least resistance in a crate that must ship no build script, so JS accreted where it was
first needed. The reason for "no build script" (users compile the crate) is already answered by the committed-bundle model.

**Construction.** Every non-trivial script is a function in `primitives/src/ts/<module>.ts` (typed, `tsc --noEmit` in the same bun invocation), bundled to
the committed `primitives/src/js/<module>.js` and included with `include_str!`; the Rust side calls `window.__dx.fn(args)` through one eval wrapper.
`PREPAINT_JS`, `MODALITY_JS` and `preview/index.html`'s copies become `include_str!`/a build-time insert of the same file, so the "identical copies"
gates go away. The JS gates scan `.ts`.

**Subsumes.** The 46 blocks over time; the three-copy and two-copy identity checks; the regex-over-Rust-string complications of four gates; the
`listener-ok` markers that sit inside eval strings. **Does not subsume:** one-line `eval` calls that read one value (a `getBoundingClientRect` for
one element); the carousel needs its own plan because it builds its script with `concat!`/macros.

**Gate that keeps it closed.** `check-js-in-rust.sh`: a raw string with more than N lines or any `addEventListener` in a `.rs` fails unless listed;
the bundle gate already exists.

**Risk (inferred).** Behavioural drift while moving (these scripts are timing-sensitive: scroll bridge, find, wheel band). Each lane's proof is the
specs of the components that use the block, on a release build. Typed access to Dioxus's eval channel (`dioxus.send`) needs a declaration file.

---

## P10. One menu-family implementation behind three hosts

**Size: L, and the riskiest item here** (primitives, public API names preserved; needs the cross-component spec run, not a code read, per
`agent-model-fit.md`).

**Invariant.** The three menus (DropdownMenu, ContextMenu, Menubar) behave the same where APG says they behave the same: item selection, focus,
close-on-select, submenus. A fix lands once.

**Where it is duplicated, and drifted (verified).** `dropdown_menu.rs` 2,399 lines, `context_menu.rs` 2,378, `menubar.rs` 1,581. Same component
list in each (`*Item`, `CheckboxItem`, `RadioGroup`, `RadioItem`, `Separator`, `Group`, `Shortcut`, `Sub`, `SubTrigger`, `SubContent`, `SubItem`; two
`*ContentRendered` web/native arms each at `dropdown_menu.rs:742,893`, `context_menu.rs:691,886`, `menubar.rs:821,980`). Measured: after
normalising the host name, **477 non-comment lines of more than 12 characters are shared between `dropdown_menu.rs` and `context_menu.rs`**
(47% of the smaller file's non-comment lines), 202 between dropdown and menubar. Row 53 extracted three narrow bands (`menu_root.rs`: open-focus
sync, content lifecycle, labelledby) and `menu_item.rs` shares the checkable and radio items and their props
(`dropdown_menu.rs` re-exports them as `pub type DropdownMenuCheckboxItemProps = MenuCheckboxItemProps`); `menu_root.rs:1-60` records that row 53
decided **not** to build one shared component. The remaining copies have drifted:

- *A plain item's `value` has three types:* `dropdown_menu.rs:962` `ReadSignal<T>` (generic `DropdownMenuItem<T>`), `context_menu.rs:1023`
  `ReadSignal<String>`, `menubar.rs:1151` a plain `String` cloned per closure. The generic form is the one the docs promise for Dropdown only.
- *Selection differs:* `DropdownMenuItem` selects on `onclick` (`dropdown_menu.rs:~1111-1135`) and does not clear the focus collection;
  `ContextMenuItem` records `pointerdown` and commits on `pointerup` (the long-press-then-lift guard, `context_menu.rs:~1128-1160`) and calls
  `ctx.focus.clear_focus()` before `set_open(false)`. The long-press guard is plausibly right for a menu opened by long-press and wrong to be
  absent for a touch dropdown; whether the missing `clear_focus` is a bug is **inferred**, not tested.
- *Typeahead text:* Dropdown uses `selection::option_text_value(&value, text_value, "DropdownMenuItem")`; Context and Menubar use
  `text_value.unwrap_or(value)`.

**Root cause.** The three hosts differ in *how they open* (click, right-click or long-press, a bar of triggers) and in nothing a user sees once
open, but the implementation is organised by host, so what is the same is copied.

**Construction.** Split on the axis that really varies. A single `MenuPanel`/`MenuItem`/`MenuSub` family in `menu_*.rs` reads a small
`MenuHostContext` (the fields items actually use: `open`/`set_open`, `focus` collection, `disabled`, `interacted_outside`, `trigger_id`, a
`close_policy`) that each host provides next to its own context. The host files keep only trigger, root and positioning (the genuinely different
parts). Public names stay through re-exports (`pub use menu::Item as DropdownMenuItem`, with `Item<T>` generic once, for all three; Context and
Menubar get the generic value for free). The existing `menu_sub.rs` (442 lines) and `menu_item.rs` (873) are the first step, already shared.

**Subsumes.** The three `value` types, the two selection models (one becomes an explicit `SelectOn::Click | SelectOn::PressRelease`), the
typeahead text rule, and the web/native arm duplication of the shared parts. **Does not subsume:** `Navbar` and `NavigationMenu` (row 53 scoped
them out and the measured overlap is 24-29%), positioning (`top_layer` keeps one call per host), the host triggers.

**Gate that keeps it closed.** A parity spec per behaviour over the three hosts (one spec file iterating hosts: select by click, by keyboard,
by touch tap, close-on-select, typeahead on a valued item) in place of three near-copies; `check-direction-keys.sh` already forces the direction
rule through `direction.rs`. A line-overlap check is not worth a gate.

**Risk and an out.** This reopens a row-53 decision (the owner's call). The cheap half is P10a: unify only `value` typing and the selection
model (S-M) and leave the structure; the full merge is P10b.

---

## Examined and not proposed

- **`sidebar/variants/{main,inset,floating}/mod.rs`**: three files of 540 lines that differ by one line (`SidebarVariant::Sidebar|Inset|Floating`,
  verified with `diff`). Every demo variant is also a code listing the docs page shows to a reader (and `installed_source.rs` rewrites it), so a
  shared body would hide the code a user copies. The same holds for the ~57-fold `chart_data`/`chart_config` helpers, five `thousands`, three
  `site_href` in chart and navigation-menu demos: left as intended duplication. If the owner wants the demos to share, the construction is a
  `variants/shared.rs` that the listing inlines; I did not cost that.
- **`document::Link { rel: "stylesheet", href: asset!(..) }`**: 349 identical lines in 75 `component.rs` files (measured). A macro would not
  remove them (`asset!` needs a literal, called once per component function), and `check-component-catalog` check 2b already guards the failure.
- **Six `focus_next`/`focus_prev` methods** (`toggle_group.rs:87`, `drag_and_drop_list.rs:236`, `navbar.rs:312`, `menubar.rs:240`,
  `radio_group.rs:67`, `collection.rs:463`): all but `collection.rs` and `radio_group.rs` are one-line forwarders into the collection; the roving
  focus is already one construction. `radio_group.rs` adds "select the focused value", which is its behaviour.
- **`check-popup-boundary.sh` ALLOW (35 entries, mostly "leaf part holds only its icon")**: the cluster says the gate flags `svg`/`img`
  descendant rules where no popup can sit. A platform construction may exist: `@scope (.dx-x) to ([popover], dialog) { svg { ... } }` bounds a rule
  by the popup without touching specificity per selector. I did not verify specificity and ordering behaviour of `@scope` against the 139 `svg`
  rules, so it is a **spike**, not a proposal: rewrite one sheet (`button`) and run `popup-style-isolation.spec.ts` and the style-diff.
- **`check-theme-baseline.sh` `USER_SELECT_ALLOW` (17 entries)**: each one is a reasoned "non-interactive or drag surface". The reasons differ, so
  no single construction removes them.
- **`check-cross-component-overrides` baseline (25 lines, one sheet)**: already tracked by row 170, whose text says `DropdownMenuContent` has "no
  props" for placement. It does have them now (`preview/src/components/dropdown_menu/component.rs:60-70` forwards `side`, `align`, `side_offset`;
  `primitives/src/dropdown_menu.rs:742-760`). Sidebar can pass `side`/`align`/`side_offset` instead of the 25 `top/left/margin` declarations at
  `sidebar/style.css:540-583`. **Row 170's remainder should be re-read against that**; this is a small direct follow-up, not a new proposal.
  Inferred: whether the anchor engine places a menu beside (not below) a sidebar footer item exactly as the hand-written CSS does.
- **Gate allowlists that are empty or one entry** (`check-scroll-behavior`, `check-spec-at-rest` ALLOW, `check-destructive-text`,
  `check-dynamic-style-shorthand` ALLOW, the `attr-spread-collision` baselines): nothing to find.

## Suggested order, if the owner accepts any

P1 and P6 first (small, certain, independent file sets). P2 and P4 run beside Rust work (they touch `playwright/` and `scripts/` only), but P4 owns
`scripts/check-*.sh` while it runs. P3 and P5 are mechanical and Haiku-shaped after a Sonnet lane has proved the pattern on three wrappers or
five manifests. P7 and P9 are Sonnet lanes with cross-component spec runs; P9 splits into one lane per script cluster. P10 only after the owner
has decided whether row 53's "do not build one shared component" stands (P10a does not need that decision).
