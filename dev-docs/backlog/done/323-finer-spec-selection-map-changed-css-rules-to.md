---
id: 323
title: "Finer spec selection: map changed CSS rules to the classes and routes that use them"
status: "LANDED 2026-10-21 (lane selector): rule-level CSS selection in playwright/tools/select.mjs…"
size: "M"
area: tests
updated: 2026-10-21
---

# Row 323: Finer spec selection: map changed CSS rules to the classes and routes that use them

## Item

Finer spec selection: map changed CSS rules to the classes and routes that use them.

## Status / trigger

**LANDED 2026-10-21 (lane selector): rule-level CSS selection in `playwright/tools/select.mjs`; measured gain is real only for page-local CSS, see Update**

## Size

M

## Detail

Today any change to `main.css`, `main.rs` or a theme sheet makes `playwright/tools/select.mjs` select ~1,800 tests. Splitting `main.css` per page was rejected: it breaks the one render-blocking bundle (`check-css-delivery.sh`), changes cascade order for equal-specificity rules, and the docs-shell rules are shared anyway. Instead `select.mjs` parses the changed CSS rules' selectors into classes and maps classes to the components and routes that render them (from the SSG HTML), selecting only those specs.

## Update 2026-10-21 (lane selector)

Owner's condition: build it only if a measurement shows it holds up. Measured first, then built; both numbers below.

**Baseline (what a CSS edit selected before).** Any `preview/assets/*.css` edit selects the BROAD set: 32 specs, 1,442 tests,
6,570 s of worker time on `durations.json` (about 1.6 h of the lock at two workers on a debug build). 4,659 s of it is four
route sweeps: `all-routes` 2,308, `first-paint-styles` 1,086, `layout-stability` 819, `theme-picker-phone` 446. A whole commit
selects 1,800 to 3,200 tests only because it also touches other files (components, `main.rs`).

**Design.** Rule level, no browser, no build (`playwright/tools/css-rules.mjs`, `css-impact.mjs`, `site-index.mjs`,
`select-titles.mjs`; the soundness argument is the header of `css-impact.mjs`):
1. the diff is taken RULE by RULE against the range base (an edited rule is one change and a `:root` edit traces only the
   declaration that changed; a moved rule is a change; a comment is none);
2. the classes and ids a changed selector REQUIRES (the constraining compound; `:not()`, `:has()`, attribute-only and element
   selectors require nothing) are placed on the pages of the SSG build under test;
3. custom properties and keyframes are traced to the rules that read them (`var(--x`), transitively;
4. the matrix specs whose test titles name the page they visit (18 of them, `select-titles.mjs`) drop the tests of unreached
   pages with a `--grep-invert` pattern; every test whose title names no page is kept; the other BROAD specs stay whole;
5. added on top, because the old mapping missed them: the specs about the component that OWNS a changed class (the theme sheet
   carries hoisted component parts) and every spec that spells a changed class or a rarely spelled (<= 12 specs) property.
Fallback to the whole BROAD set for anything outside that argument: a selector with no class or id, `@font-face`, `@import`,
a nested rule, a token spelled in script or inline style, a class that cannot be placed, no prerendered build to read.

**Class index: from the SSG HTML, re-read on every run, not derived from rsx.** The pages that carry a class are exactly what
the SSG rendered (the home page shows the first demo of every component, the docs shell wraps every component page, a
`format!`-built class is just a string); re-deriving that from Rust text would need a call graph of the preview crate and
would still guess. It is read from `--ssg-dir` / `SELECT_SSG_DIR` (the lane's `pub-<lane>`) or by crawling
`PLAYWRIGHT_BASE_URL` (the build `test:changed` runs against; the crawl finds the same 100 routes as the directory, 0.5 s), so it
cannot be stale against the build being tested and needs no cache. The SSG omits what is not in the DOM at first paint
(closed popups, script-added classes: 517 of the 1,098 classes in the sheets are on no page, 159 of those are spelled in a
source file and 358 nowhere, 337 of them the `dx-utilities` classes nothing in the preview renders): such a class takes the pages of its source siblings, and the broad set when even that is empty. An empty-shell
build (a `dx serve` dev server) is refused, so a client-rendered site cannot make every class look dead.

**Measurement, the last 30 commits that touched `preview/assets`, `main.rs` or `theme.rs` (22 touch a stylesheet).** Columns are
the stylesheets of the commit alone, tests / worker seconds, old then new (page index from a 2026-10-21 debug SSG; the class
index of a historical commit is approximate). `core` = what the same selection costs without the specs that are new coverage.

| commit | sheets | class | old | new | change |
|---|---|---|---|---|---|
| `58ee2762` | dx-components-theme | broad | 1,442 / 6,570 | 1,442 / 6,570 | +0% |
| `b5da5ca8` | dx-components-theme | broad | 1,442 / 6,570 | 1,442 / 6,570 | +0% |
| `de599095` | dx-effects | shell-wide | 1,442 / 6,570 | 1,519 / 6,781 | +3% |
| `98b419d7` | dx-components-theme | broad | 1,442 / 6,570 | 1,442 / 6,570 | +0% |
| `1ba0b2e5` | dx-components-theme,main,theme-presets | broad | 1,442 / 6,570 | 1,490 / 6,685 | +2% |
| `99f0f371` | main | page-local | 1,442 / 6,570 | 356 / 3,179 | -52% |
| `c49ebdca` | hero,main | page-local | 1,442 / 6,570 | 490 / 3,765 | -43% |
| `6b61cbda` | dx-components-theme | broad | 1,442 / 6,570 | 1,442 / 6,570 | +0% |
| `1a7bc3df` | main | shell-wide | 1,442 / 6,570 | 1,422 / 5,916 | -10% |
| `6be1887d` | main | shell-wide | 1,442 / 6,570 | 1,643 / 7,443 | +13% |
| `11db2745` | main | shell-wide | 1,442 / 6,570 | 1,442 / 6,570 | +0% |
| `e55437b5` | main | shell-wide | 1,442 / 6,570 | 1,744 / 7,358 | +12% |
| `736e9600` | main | shell-wide | 1,442 / 6,570 | 1,517 / 6,801 | +4% |
| `c7a52a9c` | main | shell-wide | 1,442 / 6,570 | 2,245 / 9,078 | +38% |
| `b92f3fe7` | main | shell-wide | 1,442 / 6,570 | 1,761 / 7,627 | +16% |
| `591b133f` | main | page-local | 1,442 / 6,570 | 502 / 3,852 | -41% |
| `6f0c7b6d` | main | shell-wide | 1,442 / 6,570 | 1,451 / 6,613 | +1% |
| `4b0cb290` | dx-components-theme,main | broad | 1,442 / 6,570 | 1,460 / 6,666 | +1% |
| `195f670f` | main | shell-wide | 1,442 / 6,570 | 1,371 / 5,753 | -12% |
| `021de76a` | main | shell-wide | 1,442 / 6,570 | 1,342 / 5,676 | -14% |
| `d84da3e9` | dx-components-theme,main | broad | 1,442 / 6,570 | 1,477 / 6,677 | +2% |
| `7ff4cbf6` | dx-components-theme | page-local | 1,442 / 6,570 | 523 / 3,746 | -43% |

| class (commits) | old | new | new, core only |
|---|---|---|---|
| broad: no class to narrow by (7) | 45,990 s | 46,308 s (+1%) | +0% |
| shell-wide: a docs-shell class on 80+ pages (11) | 72,270 s | 75,616 s (+5%) | -5% |
| page-local: one to seven pages (4) | 26,280 s | 14,542 s (-45%) | -57% |
| all 22 | 144,540 s | 136,466 s (-5.6%) | 126,386 s (-12.6%) |

The honest reading: the finer mapping holds up on soundness and on page-local CSS, and does not touch the cost the owner saw.
Seven of 22 commits are broad by construction (`:where([popover][class^="dx-"], ...)`, `[data-dx-menu-item]`, the press wash on
`button[class^="dx-"]:active`, `--dx-foreground` read by inline style), eleven change a docs-shell class that exists on every
page (`.dx-docs-shell-body`, `.dx-navbar-inner`), and a sweep that visits every page must run on every page for those. The
new selection adds coverage (below), which is why the shell-wide row grows 5% while its core shrinks 5%.

**Soundness (never lose a spec the old mapping selected, and catch regressions that actually happened).** Across the 30 commits
the new selection contains every spec the old one did, except `focus-ring.spec` and `keyboard-matrix.spec` for 7ff4cbf6 (a
hover-mix change reaching toggle, alert dialog, calendar, navigation menu, bubble: neither spec has a test on those pages).
Eight culprit commits (found in the fix commits' messages, 3bfab8e9, 2ad95f63, 03e95b83, 84faf813, 1a7bc3df), each run on the
culprit's own tree and diff, page index from today's SSG with the routes that did not exist then removed:

| culprit commit | CSS change | spec that went red | old | new |
|---|---|---|---|---|
| `736e9600` | docs preview frame borderless (main.css) | `tabs.spec` "come from main.css" | missed | caught (names `.dx-component-preview-frame`) |
| `c7a52a9c` | two main.css rules lead with `.dx-button` | `stylesheets-present.spec` | missed | caught (owner component button) |
| `37db3980` | base-nova tokens, light `--dx-destructive` | `attachment.spec`, `bubble.spec`, `menu-item-parts.spec` | missed | caught (names the tokens) |
| `6be1887d` | docs chrome menu | `hydration-parity.spec` Rule 3 | caught | caught |
| `39e0a41d` | phase C dark palette | `all-routes.spec` `bar_chart`/`line_chart: axe (dark)` | caught | caught (both tests kept) |
| `b92f3fe7` | docs sidebar ToggleGroup | `theme-preset-contrast.spec` Toggle probe | caught | caught (24/24 tests) |
| `97f021df` | `--dx-ring-focus` zeroed outside keyboard modality | `textarea.spec:50` | missed | missed (broad: attribute-only readers) |
| `1ba0b2e5` | row 295 press wash | `data_table.spec`, `checkbox.spec` | missed | missed (broad: unconstrained rule) |

Old 3 of 11 specs, new 8 of 11. The three misses are the broad fallback: an unconstrained rule or a token read through an
attribute-only selector reaches every spec that clicks a control, and no mapping can name those specs; only the merge-batch
full run (which found them) can.

**Proposed follow-up, owner decision.** The cost is the four sweeps on shell-wide CSS. Options: (a) leave it (the full suite per
merge batch is the cover); (b) a lane-only page sample for shell-wide CSS (home, docs, demos, charts, button, avatar,
data_table, sidebar, calendar, dialog): 4,246 s instead of 6,570 (-35%, measured with `--pages`), NOT sound in principle and
therefore not built; (c) make the sweeps themselves cheaper (`all-routes` axe at 4.3 s a test is the biggest line). Recommendation:
(c) as its own row, then (b) only if the lock queue still hurts.

Files: `playwright/tools/{css-rules,css-impact,site-index,select-titles,select-selftest}.mjs`, `select.mjs`, `test-changed.sh`,
`scripts/check-select-css.sh` (gate, in `run-gates.sh`). Escape hatches: `--no-css-narrow` / `CHANGED_FULL_CSS=1` (the old
behaviour), `--explain` (why each spec), `--pages c:button,site:home` (diagnostic).
