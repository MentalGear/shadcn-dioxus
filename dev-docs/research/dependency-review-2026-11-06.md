# Dependency review against C23 (row 371)

Date 2026-11-06, lane `dep-review-371`, base main `d040554a`. Read-only: no code and no manifest changed; every outcome below is
a **proposal** for the owner. Rule: [`C23` and D1-D6](../supply-chain/README.md#choosing-a-new-dependency-owner-2026-10-31).
Earlier work this builds on (does not redo): [`dependency-evaluation.md`](./dependency-evaluation.md) (2026-10-23, the risk and blast-radius
columns), row 371's applied trims of 2026-10-25 (`palette` and `time` features, `--locked`, lock-version floors).

## How the numbers were taken

- Today is 2026-11-06, so D1 needs a pinned release published on or before 2026-10-07; D2 a first release on or before 2025-11-06;
  D4 a release on or after 2024-11-06.
- Crates: `https://crates.io/api/v1/crates/<crate>` (versions `created_at`, `downloads`, `license`, `repository`), 2026-11-06. The
  "locked" version is the one in `Cargo.lock`. `build.rs`, `proc-macro` and `unsafe` (raw `\bunsafe\b` line count in `src/`, an upper bound) are read from the
  unpacked sources in `~/.cargo/registry/src`. Direct-dependency counts are `cargo tree -e normal --depth 1 --locked --offline`.
- npm: `registry.npmjs.org/<pkg>` (`time`, `license`, `dependencies`, `hasInstallScript`, `deprecated`) and
  `api.npmjs.org/downloads/point/last-week/<pkg>`, 2026-11-06.
- **GitHub was unreachable** from this lane (`api.github.com` and `gh api` answer "GitHub access to this repository is not enabled for this
  session"), so **no star count and no `archived` flag could be read**. D3 uses the downloads stand-in the rule allows (crates.io total
  >= 1,000,000; npm weekly >= 100,000) and is **unverified** everywhere; D4's "not archived" half is **unverified** everywhere (the
  release-date half is verified).
- D5 "exact pin": the manifests of the workspace carry caret floors equal to the lock version (C02), not `=x.y.z`; exact pins exist only for
  `pulldown-cmark` and `dioxus-icons` (C03). The lock pins every crate for us and not for users. This is a property of the whole Rust
  side, so it is stated once here and not repeated per row (the row says "pin" only where it proposes a change). All npm entries are exact.
  No direct npm package has an install script (`hasInstallScript` unset for all nine; `onlyBuiltDependencies: []`).
- A crate counts as "users compile it" when it is reachable from a `component.json` `cargoDependencies` or from `dioxus-primitives`
  (the PERFORMANCE scope); everything else is SECURITY scope only (docs site, CLI, tests, tooling).

## Summary

25 Rust dependency names (registry, direct, all manifests including the two detached ones), 9 npm packages, 1 cargo-installed tool (`dx`), 0
vendored libraries: 35 entries. The workspace's own crates (`dioxus-primitives`, `dioxus-attributes`, `preview`, `shadcn-dioxus`, `test-harness`,
`attr-spread-collision-checker`, `attr-synth`, `carousel-lab`) are ours and not rated.

| Outcome | Count | Entries |
|---|---|---|
| keep, no change | 25 | rust: `dioxus-rsx`, `dioxus-core`, `dioxus-ssr`, `unic-langid` (while preview keeps i18n), `time`, `tracing`, `serde`, `serde_json`, `palette`, `tokio`, `anyhow`, `toml_edit`, `tempfile`, `proc-macro2`, `quote`, `js-sys`, `web-sys`; npm: `@playwright/test`, `playwright`, `axe-playwright`, `stylelint`, `stylelint-config-standard`, `stylelint-order`, `cytoscape`; tool: `dx` (unverified D3) |
| keep, with a small change | 5 | `dioxus` (feature trim, -16 crates on wasm, proposal P1), `pulldown-cmark` (exact pin of the build-dependency, P5), `syn` (drop unused `extra-traits`, 0 effect on the tree, P6), `clap` (optional feature trim, P7), `gloo-timers` (owner's call: keep or replace, P8) |
| keep under a D6 exception (owner grants, with an end date) | 2 | `dioxus-code` (P3), `@axe-core/playwright` (licence MPL-2.0, P9) |
| lift | 2 | `dioxus-icons` (12 glyphs, ~170 lines, P2; deferred by the owner on 2026-10-25), `stylelint-config-idiomatic-order` (96 lines, P4) |
| replace / drop | 1 | `dioxus-i18n` (D1 and D3 fail; ~90 lines in-repo, or a D6 exception, P10) |


C23 failures (verified from release dates and downloads): `dioxus-code` (D2, D3), `dioxus-icons` (D2, D3), `dioxus-i18n` (D1 until 2026-11-11, D3),
`stylelint-config-idiomatic-order` (D4). Licence: `@axe-core/playwright` is MPL-2.0 (weak copyleft, outside "MIT, Apache-2.0 or ISC class", D5).

### Proposals that change something

| # | Proposal | Effect | Risk |
|---|---|---|---|
| P1 | `dioxus` in `[workspace.dependencies]`: `default-features = false, features = ["lib"]`; `preview`/`test-harness` add what they need (`launch`, `logger`, `devtools`) | wasm closure of `dioxus-primitives --features web` 146 -> 130 unique tree lines (measured with a trial edit, reverted; **not built**) | users' apps enable their own `dioxus` defaults, so the saving reaches an app only if its `dioxus` is also trimmed (the starter's is not); build must be proven by `check-primitives-features.sh` |
| P2 | Lift the 12 glyphs the shipped `component.rs` files use into an in-repo `icons.rs`; `dioxus-icons` stays an opt-in dependency of `preview` (34 glyphs in demos) | removes a `build.rs` (`lazy-js-bundle`, bun) and 25 MB of icon source from 28 components' user builds; fixes D2/D3 for users | low code risk (SVG paths, ISC notice travels with `lifted-from: dioxus-icons@0.1.0`); owner deferred it on 2026-10-25 |
| P3 | `dioxus-code` stays as a recorded D6 exception, end date 2027-05-05 (re-check when it is 12 months old and 1.0k stars is readable) | none | docs-site only; never in a user's build |
| P4 | Lift `stylelint-config-idiomatic-order` 10.0.0 (96 lines, MIT) into `preview/` as a local config; `stylelint-order` stays | removes one package whose last release is 2023-12-09 | low; the config is a property-order list |
| P5 | `preview/Cargo.toml` `[build-dependencies]` `pulldown-cmark = "=0.13.4"` (the `[dependencies]` entry already is) | the docs build and the component parse the same bytes | none; the lock already has 0.13.4 |
| P6 | `dioxus-attributes` and `attr-spread-collision-checker`: drop `extra-traits` from `syn` | none on the lock (dioxus-rsx enables it); both crates `cargo check` without it (measured) | none |
| P7 | `clap` in `cli`: `default-features = false, features = ["derive", "std", "help", "usage", "error-context"]` (drops `color` and `suggestions`) | fewer CLI-only crates, loses colour and "did you mean"; **count unmeasured** (the trial edit did not resolve offline) | owner's taste; CLI only |
| P8 | `gloo-timers` -> ~15-line `setTimeout` future in `primitives/src/sleep.rs` | -1 crate on wasm for every user | the `interval.rs` tests pin semantics; qualifies only on rule 3; **recommend keep** |
| P9 | `@axe-core/playwright` 4.10.2: owner rules whether MPL-2.0 is acceptable for a dev-only test dependency, else drop the axe specs' helper | none until ruled | dev tooling only; never shipped |
| P10 | `dioxus-i18n`: replace with an in-repo locale lookup (the four `.ftl` files are 40 lines each; `mod.rs` is 71 lines; ~90 lines for `tid!`-style lookup + a locale signal) and drop `unic-langid` with it, **or** wait until 2026-11-11 (D1) and record a D6 exception for D3 (82,667 downloads) | removes 2 direct crates and the Fluent crates they bring from the docs build (crate count not isolated: the `cargo tree` closure of `dioxus-i18n` includes `dioxus`) | docs-site only; replacing loses real Fluent plural/format rules the demo might showcase |

Feature flags that pull in unused code (the earlier round trimmed `palette` and `time`; the open ones): `dioxus` defaults (P1: `devtools`,
`logger`, `launch` in a library), `clap` defaults (P7), `syn`'s `extra-traits` (P6, no tree effect), and `dioxus-code` (preview enables
`macro` and `lang-css` only, but `build.rs` still depends on the whole `arborium` theme set; not trimmable from our side).
Checked and fine: `time` (`std`, `local-offset` both used: `OffsetDateTime::now_local` in `calendar/context.rs`; `macros` is dev-only),
`palette` (`std` only), `tokio` (`time` only on native), `gloo-timers` (`futures` needed), `serde` (`derive` used), `unic-langid`
(`macros` for `langid!`), `pulldown-cmark` (`default-features = false`; `html` only as a build-dependency), `toml_edit` (default
`parse`+`display` both used), `web-sys` (`Window` only). `component.json` entries use the object form, which always passes
`--no-default-features` (`cli/src/manifest.rs`), so users get `time` and `palette` without default features.

## Per dependency

Columns: use (call sites); scope; C23 D1 (locked release date, age), D2 (first release), D3 (downloads, unverified stand-in), D4 (latest
release); D5 (build.rs / proc-macro / `unsafe` lines / direct deps / licence); outcome and risk. Dates `yyyy-mm-dd`, downloads total on crates.io.

### Rust, users compile (`dioxus-primitives` and `cargoDependencies`)

| Crate (locked) | Used for | D1 | D2 | D3 | D4 | D5 | Outcome, risk |
|---|---|---|---|---|---|---|---|
| dioxus 0.7.9 | the framework; every component and the primitives | 2026-05-08, 182 d | 2021-01-20 | 3.56 M | 0.8.0-alpha.3 2026-10-16 (latest stable 0.7.9) | no build.rs / `unsafe` 0 / MIT OR Apache-2.0 | keep (framework veto); **P1** feature trim. Risk of trim: a missing feature surfaces as a compile error, caught by the feature gate |
| dioxus-attributes (ours) | `attributes!` macro | n/a | n/a | n/a | n/a | proc-macro; deps dioxus-rsx, proc-macro2, quote, syn | ours |
| time 0.3.47 | `Date`, `Month`, `OffsetDateTime::now_local`, `NumericalDuration` in calendar and date_picker (79 files use `time::`) | 2026-02-05, 274 d | 2014-11-13 | 1,048 M | 0.3.55 2026-08-01 | no build.rs; 105 `unsafe`; MIT OR Apache-2.0 | keep (veto: calendar arithmetic is in our public props); features already minimal. Newer 0.3.55 exists, a bump is a lock decision |
| palette 0.7.6 | `Hsv`, `Srgb`, `Oklch`, `IntoColor` in `color_picker.rs`, `color_naming.rs` | 2024-04-28, 557 d | 2016-01-12 | 37.7 M | 0.7.7 2026-08-02 | no build.rs, `palette_derive` proc-macro; 115 `unsafe`; MIT OR Apache-2.0 | keep (colour-science veto); features already trimmed. 0.7.7 is 96 days old if the owner wants the bump |
| tracing 0.1.44 | `tracing::warn!` and doc examples (37 files) | 2025-12-18, 323 d | 2017-11-27 | 1,016 M | 0.1.44 2025-12-18 | no build.rs; 7 `unsafe`; MIT | keep (0 marginal cost, dioxus brings it) |
| serde 1.0.228 | `derive` for the JS bridge messages (`js_listener`, `virtual/types`, `message_scroller`) | 2025-09-27, 405 d | 2014-12-05 | 1,657 M | 1.0.229 2026-07-18 | build.rs (rustc probe); 2 `unsafe`; MIT OR Apache-2.0 | keep (0 marginal cost) |
| serde_json 1.0.149 | `serde_json::Value` bridge messages (`src/bridge/`), CLI and build.rs parsing | 2026-01-06, 304 d | 2015-08-07 | 1,562 M | 1.0.154 2026-10-11 | build.rs; 12 `unsafe`; MIT OR Apache-2.0 | keep (dioxus brings it) |
| tokio 1.52.3 | `time` only: `src/sleep.rs` on native, tests on a paused clock | 2026-05-08, 182 d | 2016-07-01 | 1,157 M | 1.54.1 2026-10-21 | no build.rs; 1058 `unsafe` (only the timer is compiled); MIT | keep (0 on wasm; already in dioxus's native tree) |
| gloo-timers 0.3.0 | `gloo_timers::future::sleep`, wasm, one site (`src/sleep.rs`; `date_picker/context.rs` mentions it) | 2023-08-11, 1,183 d | 2019-09-13 | 126 M | 0.4.0 2026-03-25 | no build.rs; 0 `unsafe`; MIT OR Apache-2.0 | keep; **P8** replace is optional (-1 crate on wasm) |
| dioxus-icons =0.1.0 | Lucide glyphs; 28 shipped components (`component.rs`) use 12 distinct glyphs, 198 repo files and 34 glyphs in total | 2026-05-08, 182 d | **2026-05-08, 182 d: FAIL** | **41,885: FAIL** | 0.2.0-alpha.3 2026-10-16 | **build.rs** (`lazy-js-bundle`, bun); 0 `unsafe` in `src`; 25 MB package; MIT AND ISC | **lift (P2)**. Risk of keeping: build-time code in every user build from a 6-month-old, 42k-download crate |
| pulldown-cmark =0.13.4 | `markdown` component's parser (untrusted text) | 2026-05-20, 170 d | 2015-06-04 | 199 M | 0.13.6 2026-10-30 (0.13.5 on 2026-10-21 is 16 d old; neither is 30 d old, so the pin stays) | build.rs (tables); 5 `unsafe` (simd, off); MIT | keep, exact pin already (parser veto). **P5** pin the build-dependency too. Re-open on each 0.13.x per row 371 |

### Rust, docs site and tests (`preview`, `test-harness`, `attr-synth`; SECURITY scope only)

| Crate (locked) | Used for | D1 | D2 | D3 | D4 | D5 | Outcome, risk |
|---|---|---|---|---|---|---|---|
| dioxus-rsx 0.7.9 | proc-macro parsing for `dioxus-attributes` and the collision checker | 2026-05-08, 182 d | 2022-12-09 | 3.66 M | 0.8.0-alpha.3 2026-10-16 | no build.rs; 0 `unsafe` | keep (framework) |
| dioxus-core 0.7.8 (attr-synth) | the detached oracle crate's VirtualDom | 2026-05-08, 182 d | 2021-01-19 | 4.24 M | same | 0 `unsafe` | keep |
| dioxus-ssr 0.7.9 | build.rs docs rendering, SSR parity tests | 2026-05-08, 182 d | 2021-09-25 | 1.40 M | same | 0 `unsafe` | keep (dev and build only) |
| dioxus-code 0.1.2 | docs-site syntax highlighting (`preview/build.rs`, `site/code.rs`, `gallery.rs`; 6 files, 28 lines) | 2026-05-11, 179 d | **2026-05-05, 185 d: FAIL** | **14,949: FAIL** | 0.2.0-alpha.3 2026-10-16 | **build.rs** (arborium themes, tree-sitter); 0 `unsafe`; MIT | **D6 exception (P3)**: docs site only. Replace would mean a highlighter of our own (large); not proposed |
| dioxus-i18n 0.6.0 | locale switching in the docs site (`preview/src/i18n/`, `tid!` in two demo variants; 5 files) | **2026-10-12, 25 d: FAIL until 2026-11-11** | 2024-08-31 | **82,667: FAIL** | 0.7.0-alpha.1 2026-10-15 | no build.rs; 0 `unsafe`; MIT | **replace (P10)** or a D6 exception. Risk: a young release of a small crate in the docs build |
| unic-langid 0.9.6 | `langid!` and `LanguageIdentifier` (`preview/src/i18n/mod.rs`, 1 file) | 2025-05-09, 546 d | 2019-06-16 | 29.7 M | 0.9.6 2025-05-09 (18 months; inside 24) | 0 `unsafe`; MIT OR Apache-2.0 | keep; goes with `dioxus-i18n` if P10 replaces it |
| palette 0.7.6 (preview, default features) | `site/blocks.rs`, `color_picker` demo | see palette above | | | | | keep; `preview` declares it with default features, so the workspace build compiles `phf`/`rand` that users do not; trimming preview would make that tree match what users get (docs-site only, small) |
| time 0.3.44 floor, `macros` (preview) | `date!` in two demo variants, parity tests | see time above | | | | | keep (its floor is lower than the lock; `check-cargo-dep-versions.sh` only checks primitives) |
| tokio 1 (preview dev) | `ssr_parity.rs` timers | see tokio | | | | | keep |

### Rust, CLI (`shadcn-dioxus`) and macro helpers

| Crate (locked) | Used for | D1 | D2 | D3 | D4 | D5 | Outcome, risk |
|---|---|---|---|---|---|---|---|
| anyhow 1.0.102 | CLI errors (8 files) | 2026-02-20, 259 d | 2019-10-05 | 1,136 M | 1.0.104 2026-07-18 | build.rs; 100 `unsafe`; MIT OR Apache-2.0 | keep |
| clap 4.5.61 | CLI argument parsing (`main.rs`, derive) | 2026-03-12, 239 d | 2015-03-01 | 1,308 M | 4.6.7 2026-09-14 | 0 `unsafe`; MIT OR Apache-2.0 | keep; **P7** optional feature trim |
| toml_edit 0.25.5 | editing the app's `Cargo.toml` and `Dioxus.toml` (3 files). 0.19.15 and 0.20.2 are transitive copies | 2026-03-17, 234 d | 2017-12-17 | 993 M | 0.25.18 2026-11-04 | 0 `unsafe`; MIT OR Apache-2.0 | keep (a TOML editor is a parser: veto) |
| tempfile 3.27.0 | CLI tests (6 files, dev only) | 2026-03-11, 240 d | 2015-04-14 | 966 M | 3.27.0 2026-03-11 | 4 `unsafe`; MIT OR Apache-2.0 | keep (dev only; the crates.io stand-in is 966 M, far above the threshold) |
| proc-macro2 1.0.106, quote 1.0.45, syn 2.0.117 | `dioxus-attributes` and the collision checker (the proc-macro toolchain every Dioxus build already has) | 2026-01-21 / 03-03 / 2026-02-20 | 2017 / 2016 / 2016 | 1.9 G / 1.9 G / 2.9 G | 2026-07 / 2026-07 / 3.0.9 2026-10-12 | build.rs on proc-macro2 and quote; syn 76 `unsafe`; MIT OR Apache-2.0 | keep (0 marginal cost); **P6** `extra-traits` |
| js-sys 0.3.98, web-sys 0.3.98 | `labs/carousel-lab` only (detached, 3 lines) | 2026-05-07, 183 d | 2018-07-19 | 594 M / 457 M | 0.3.107 2026-10-30 | 32 / 19 `unsafe`; MIT OR Apache-2.0 | keep (lab, never shipped) |

### JS tooling (npm; none ships to users)

| Package (pinned) | Used for | D1 | D2 | D3 weekly | D4 | D5 | Outcome, risk |
|---|---|---|---|---|---|---|---|
| @playwright/test 1.60.0 | the e2e suite, imported by 3 spec helpers and the runner | 2026-05-11, 179 d | 2020-09-24 | 107.8 M | 1.65.0 2026-11-05 | 1 dep; Apache-2.0; no install script | keep |
| playwright 1.60.0 | `scripts/inspect.mjs` (`chromium`) and the browser install | 2026-05-11, 179 d | 2015-01-23 | 163.3 M | 1.65.0 2026-11-05 | 1 dep; Apache-2.0 | keep (same project and version as above; a lone consumer, so dropping it for `@playwright/test`'s re-export is possible, not worth it) |
| @axe-core/playwright 4.10.2 | accessibility audits (`playwright/axe.ts`) | 2025-06-03, 521 d | 2021-06-02 | 22.3 M | 4.14.0 2026-10-15 | 1 dep (`axe-core`); **MPL-2.0** | keep as a D6 decision on the licence (**P9**); a version bump to 4.14.0 is separate |
| axe-playwright 2.1.0 | `all-routes.spec.ts` axe checks | 2025-02-06, 638 d | 2020-06-15 | 848 k | 2.2.2 2025-09-12 | 5 deps; MIT | keep (it and `@axe-core/playwright` overlap; the owner may drop one) |
| stylelint 16.26.1 | `stylelint` gate over component CSS | 2025-11-28, 343 d | 2014-12-11 | 14.1 M | 18.0.0 2026-11-03 | 39 deps; MIT | keep (two majors behind; a bump is a gate re-baseline) |
| stylelint-config-standard 38.0.0 | base ruleset | 2025-04-06, 579 d | 2016-01-11 | 8.29 M | 41.0.0 2026-11-03 | 1 dep; MIT | keep |
| stylelint-config-idiomatic-order 10.0.0 | property order (`preview/.stylelintrc.json`) | 2023-12-09, 1,064 d | 2016-05-05 | 225 k | **2023-12-09, 35 months: FAIL** | 1 dep (`stylelint-order`); MIT; 96-line `index.js` | **lift (P4)** |
| stylelint-order 6.0.4 | peer of the idiomatic-order config (not named in the rc) | 2023-12-08, 1,064 d | 2016-11-03 | 3.32 M | package 8.1.3 2026-11-03 (the pin is old, the package is live) | 2 deps; MIT | keep (needed by the config; bump with stylelint) |
| cytoscape 3.34.3 | code-graph explorer page (`scripts/codegraph/`) | 2026-09-07, 60 d | 2012-10-18 | 25.5 M | 3.34.4 2026-10-15 | 0 deps; MIT | keep (3.34.4 is 22 d old, below D1) |

### Tools installed or run

| Tool | Used for | Facts | Outcome |
|---|---|---|---|
| dx (`dioxus-cli` 0.7.9) | builds the site; prebuilt GitHub release binary in `scripts/session-setup.sh` | crates.io: 2026-05-08 (182 d), first release 2021-01-20, 380,092 downloads (**under the 1 M stand-in; stars unreadable, D3 unverified**), license per repo | keep (the framework's own tool; same project as `dioxus`); the playbook already lists it as an unchecked binary |
| bun, node, pnpm (10.28.0) | JS bundle build, tooling | not measured here (not dependencies of a manifest) | out of this review's scope |

## Not done, and why

- GitHub stars and `archived`: unreachable (above). Re-run with GitHub access to turn D3 from stand-in to verified; no outcome above is
  expected to flip except `dioxus-i18n`, `dioxus-code`, `dioxus-icons` and `dx` (all downloads-stand-in failures).
- P1 and P7 were measured with `cargo tree` only (manifests restored with `git checkout` after each trial); neither was built. A build
  (`check-primitives-features.sh`, SSG) is the proof to run before applying.
- RustSec advisories and `cargo geiger`: still not available (no `cargo-audit`, `cargo-deny`); the `unsafe` counts are upper bounds.
- Transitive crates were not rated against D1-D4 (the rule covers new dependencies; the closure gate is C16/C17, planned).
