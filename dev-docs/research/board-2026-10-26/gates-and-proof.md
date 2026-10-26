# Board lens: gates-and-proof (row 434 step 3), 2026-10-26

**Verdict: HOLDS WITH CHANGES.** The architecture (bridge verbs, rules as data, one interpreter) is not what breaks. The enforcement as written does.

- The gate bans spellings (`preventDefault`, rAF, observers) in `ts/` and Rust strings. It does not close the sinks that run JS. Every bypass the brief lists works against a token ban. A sink closure (one `eval` site, clippy `disallowed_methods`, a `git ls-files` universe, an asset/`Script` allowlist) makes the whole class unable to recur, and the token ban becomes defence in depth.
- The planned `call(el, method, args)` verb is itself an eval if it is written the natural way (`el[method](...args)`), and so are `get` and `set`. The tables must be closed on both sides.
- "Shrink-only" is not enforced across commits anywhere in the repo, and every JS gate has a self-service marker. The ratchet must land now, compared against the merge base, before the first port lane.
- Proof gaps: the JSON vectors and `bun test` cover serde field names, not the interpreter, unless it is split into a pure `decide()` and a DOM adapter. The trackpad replay has reconstructed uniform timestamps from one device. Nothing in the plan proves desktop sameness. The timing specs cannot see G latency or per-event interpreter cost.
- Docs: the owner's question has a concrete answer in the code (section 10). Exact wording and placement are in section 11.

Labels: V = verified in the repo or the pinned crate source, W = web source, I = inference.

---

## 1. [blocker] The gate closes spellings, not sinks

**Attacks:** sync-js-root section 5.3 items 3 and 4 ("no `preventDefault`... in `ts/` outside the bridge", "no such token in a Rust string at all"), and the row-434 "Gate" bullet (`check-js-in-rust.sh`).

**Evidence.** A token scan is defeated by every construction the brief lists. Concrete bypasses, with what closes each:

| # | Bypass | Why a token ban misses it | Closes it |
|---|---|---|---|
| B1 | `concat!("prevent", "Default()")`, `format!("{}Default", "prevent")`, `e['prevent'+'Default']()`, template literal pieces | the token never appears as one run of source text (V: the carousel already builds scripts with `concat!`, `primitives/src/carousel/js_macros.rs`) | sink closure (S1 below); for `.ts`, scan the canonicalised form (section 1.3) |
| B2 | `use dioxus::document::eval as run; run("...")` or `use dioxus::{document::eval, prelude::*}` (V: `checkbox.rs:5` and `default_checked.rs:26` already import it bare, so a `document::eval` grep misses them) | the call has no `document::eval(` | `\beval\b` as a Rust identifier, and clippy by path |
| B3 | `document().eval(..)` through the `Document` trait (V: `dioxus-document-0.7.9/src/document.rs:66`) | method syntax | same as B2 |
| B4 | `document::Script { "..inline js.." }` (V: `ScriptProps::script_contents`, `elements/script.rs`, goes through `create_script` to `create_head_element`, `document.rs:92-95`, to eval) | no `eval` in the caller | S2 |
| B5 | `script { dangerous_inner_html: X }` (V: `preview/src/docs_nav.rs:204` does this today), or `dangerous_inner_html: "<img src=x onerror=...>"` | HTML sink, not eval | S3 |
| B6 | rsx string attribute names: `"onclick": "..."`, `a { href: "javascript:..." }`, `Attribute::new("onload", ..)`, `iframe { srcdoc }` | inline handler content attributes | S4 |
| B7 | A hand-written `.js` next to a Rust file, `include_str!`ed and eval'd (V and live: `primitives/src/message_scroller.js`, 1,065 lines, `message_scroller.rs:148`; `check-js-bundle.sh` manages only `ts/` and `js/`, so this file sits outside the bundle chain, the manifest and the new gate) | not under `ts/`, not a Rust string | S5 |
| B8 | A new `primitives/src/ts/foo.ts` (or `ts/rules/foo.ts`) | `ts/rules/` is exempt by directory glob, so bespoke logic is legal there if it is not wired into the enum | file-level allowlist plus reachability (finding 3) |
| B9 | `asset!("/assets/x.js")` plus `document::Script { src }` for a second script | `Script { src }` is how `lib.rs:136` loads focus-trap, so the shape looks legitimate | S2 allowlist of exactly the asset constants |
| B10 | JS in `preview/` or `templates/` or `labs/` (V: existing JS gates scan only `primitives/src` and `preview/src`, `check-js-listeners.sh:117-125`; `labs/carousel-lab/src/main.rs` mentions eval/web_sys; `templates/starter` ships to users) | hand-written scan roots | universe by `git ls-files` (section 1.2) |
| B11 | `web_sys`/`js_sys`/`wasm_bindgen`: `js_sys::eval`, `Function::new_with_args`, `#[wasm_bindgen(inline_js = "...")]`, desktop `webview.evaluate_script` | no `eval` identifier, not a Dioxus API | S6 |
| B12 | `(()=>{}).constructor('code')()`, `setTimeout("code")`, `window['ev'+'al']`, `Reflect.get(window, k)`, `import('data:text/javascript,..')` | none of the planned tokens; `setTimeout` is not on the plan's list at all | the JS-side ban list and canonical form (1.3) |
| B13 | `listen(.., { passive: false })` inside a verb call | the options are data, not source tokens, once `listen` is a typed Rust API; today's `check-blocking-scroll-listeners.sh` regex-reads JS strings and goes blind | passive derived from the effect set (finding 2) |
| B14 | `// listener-ok: reason` and the other self-service markers (V: 18 `listener-ok`, 4 `blocking-ok`, 3 `interval-ok`) | the marker is the escape hatch | the new gates take no inline marker; see finding 4 |

### 1.1 The construction: close the sink, not the spelling

A JS-bearing string can run only through a short, enumerable list of sinks. The list is the gate. Everything else is advisory.

- **S1 (Rust code view, comments removed and string contents blanked):** the identifier `\beval\b`. It matches `document::eval(`, `eval(`, `.eval(` and `use ...::eval`, and an alias is irrelevant because the `use` line itself is flagged. Allowed only in `primitives/src/bridge/transport.rs` (the seam the plan already specifies), with an exact site count of 1 or 2. The same file's `eval` argument must be one of its declared string constants plus `serde_json::to_string` output, so nothing built from user data reaches the sink. Also ban the glob `document::\*`.
- **S2:** `document::Script`, `create_script`, `create_head_element`, `create_link`, a bare `Script\s*\{` and `\bscript\s*\{` in rsx. Allowed: `Script { src: <ASSET_CONST> }` with no string child, for exactly the asset constants on an allowlist (focus-trap on native, the bridge). `preview/src/eager_head.rs` overrides the document and may name them.
- **S3:** `dangerous_inner_html` allowlisted per file and per reason (markdown and docs HTML), and never on a `script`.
- **S4:** `"on[a-z]{3,}"\s*:`, `Attribute::new\(\s*"on`, `href\s*:\s*"javascript:`, `srcdoc`.
- **S5:** `(include_str|include_bytes)!\s*\(\s*"[^"]*\.(js|mjs|cjs|ts|tsx|html)"`, `asset!\(\s*"[^"]*\.(js|mjs|cjs|ts)"`, and a file inventory: `git ls-files '*.js' '*.mjs' '*.cjs' '*.ts' '*.tsx'` outside `playwright/`, `scripts/`, `docs/` (the built site), `primitives/src/js/` (generated) and `primitives/src/ts/`. The one hit today is `message_scroller.js`; it goes in the baseline with its row (W8) and expires with that lane. A new inline `<script` in any tracked `.html` outside `docs/` is counted per file by body hash (`preview/index.html` has two today).
- **S6:** `\b(web_sys|js_sys|wasm_bindgen|gloo_utils|evaluate_script)\b`, `inline_js`, `extern\s+"C"`. The bridge's future web-sys backend (the plan cites PR #5194) is allowed in `transport.rs` only. Also check `Cargo.toml` of `primitives` and `preview` for direct `web-sys`/`js-sys`/`wasm-bindgen` dependencies (none today, V: no hits in any manifest).

Allowlist entries are `path<TAB>sink<TAB>reason<TAB>row`, read by `scripts/lib/allow.py` (reason mandatory, stale entries fail).

### 1.2 Make the compiler say it: clippy `disallowed_methods`

`clippy.toml` already exists (V: `allow-unwrap-in-tests`) and the repo runs three clippy arms with `-D warnings` (CLAUDE.md "Rules for every change"). Add:

```toml
disallowed-methods = [
  { path = "dioxus_document::eval", reason = "browser work goes through the bridge: dev-docs/js-interop.md" },
  { path = "dioxus_document::document::Document::eval", reason = "same" },
  { path = "dioxus_document::elements::script::Script", reason = "same" },  # exact path to be confirmed in lane 3.0
  { path = "js_sys::eval", reason = "same" },
  { path = "js_sys::Function::new_with_args", reason = "same" },
  { path = "js_sys::Function::new_no_args", reason = "same" },
]
```

This resolves by item path, so aliasing, re-exports (`dioxus::document::eval` is `pub use dioxus_document::*`, V: `lib.rs:29` defines `pub fn eval`), `use ... as`, trait-method syntax and glob imports all hit the same def path. It cannot be defeated by splitting text. The one exception is `transport.rs`, carrying `#[expect(clippy::disallowed_methods, reason = "...")]`, so an exception has a reason and goes red when it stops being needed (the repo's convention, `guides/rust-conventions.md`). Limits (I, to be probed by one mutation in lane 3.0): whether `rsx! { Script {} }` expansion trips `disallowed_methods`; clippy is a cargo gate, so it does not run in lanes' `--fast` loop. Hence both layers: the fast python gate in lanes, clippy at the main-loop run.

### 1.3 The JS side: scan the canonical form, ban computed access

For `.ts`, non-generated `.js`, inline html scripts and Rust string literals, scanning source text is the weak form. Scan what `bun build --minify-syntax --no-bundle` emits (the committed bundles are built with exactly `--minify-whitespace --minify-syntax`, `check-js-bundle.sh:70`). The esbuild lineage folds `"a"+"b"` into `"ab"` and rewrites `x["y"]` to `x.y` (I: confirm with a mutation in lane 3.0). That removes B1 for the constant forms. The remaining class is runtime-computed names, closed by banning the access forms, not the names:

- identifiers: `preventDefault|stopPropagation|stopImmediatePropagation|returnValue|cancelBubble|requestAnimationFrame|requestIdleCallback|queueMicrotask|setTimeout|setInterval|ResizeObserver|MutationObserver|IntersectionObserver|PerformanceObserver|addEventListener|eval|Function|importScripts|Worker|Reflect|atob`
- forms: `\bimport\s*\(`, `\bnew\s+Function`, `\.constructor\b`, `\.on[a-z]+\s*=[^=]`, `\bwith\s*\(`
- computed access on globals: `\b(window|globalThis|self|document|top|parent)\s*\[`; computed calls `\]\s*\(` (`x[k](...)`)

The plan's list omits `setTimeout`, `queueMicrotask`, `requestIdleCallback` and the legacy `returnValue`/`cancelBubble` spellings. The audit counts 25 `setTimeout` uses in Rust strings (V: my grep of non-comment lines in `primitives/src` and `preview/src`).

The bridge and the interpreter legitimately contain several of these tokens. They go on a **file-level** allowlist (not a directory) with a per-file line budget (finding 3).

### 1.4 What the gate cannot see, said plainly

Runtime-computed property names reached through an object the gate did not ban (for example a verb table keyed by a Rust-supplied string that the interpreter uses as `obj[name]`). That is why finding 2 is a blocker: the verb tables are the last open door, and they must be closed by construction, not by a gate.

**Change:** implement S1-S6 as `scripts/check-js-sinks.sh`, plus the clippy table, plus the canonical-form scan. Fold the plan's `check-js-in-rust.sh` into it (they are the same job).
**Subsumes:** B1-B12, `check-js-listeners` (and its 18 markers), `check-blocking-scroll-listeners`, `check-uncleared-intervals`, the `listener-ok` markers inside eval strings, and the "scan roots listed by hand" class.
**Does not subsume:** a runtime-built name through a table the gate did not see (finding 2); the pre-wasm class (c) scripts, which stay on the allowlist with a reason.

---

## 2. [blocker] `call`, `get`, `set`, `listen` are an eval unless the tables are closed

**Attacks:** row 434 "Step 3 design": `call(el, method, args)`, `get(el, props)`, `set(el, var, value)`, `listen(target, type, options, fields)`.

**Evidence (I, from the plan's own signatures).** The generic implementation is `el[method](...args)`. With a target that can be `window` or `document` and JSON args, that gives `call(window, "setTimeout", ["code"])` (string argument is eval), `call(window, "eval", [..])`, `call(el, "insertAdjacentHTML", ["<img onerror=..>"])`, `call(document, "write", ..)`, `call(el, "setAttribute", ["onclick", ".."])`. The plan lists eight methods in prose and calls the verbs "generic", so the closed set is not stated as a rule.

**Change (construction):**
1. The method, property, event-type and observer-kind spaces are **closed enums** in Rust (`Call::ShowPopover`, `Call::Focus { prevent_scroll }`, `Get::Rect`, ...) and a **closed object table** in TS: `const CALLS = { showPopover: (el) => el.showPopover(), ... }` dispatched with an own-property check, never `el[name](...)`. Same for `GETS`. `set` accepts only custom properties that start with `--dx-`.
2. Parity between the Rust enums and the TS tables is a **golden-file test** (finding 3), not a text parse.
3. Receiver kinds are part of the table (an element method cannot be called on `window`); a table row says which receivers it accepts.
4. **Passive is derived, not a parameter.** `listen` streams fields, so it cannot cancel: it is always passive for `wheel|touchstart|touchmove|scroll` and the TS ignores a `passive:false` request. A listener is non-passive only when the interpreter installs a B1 rule whose effect set contains `prevent`, and that fact is computed from the rule. This removes B13 and the whole `blocking-ok` class by construction, and retires `check-blocking-scroll-listeners` (which would otherwise go blind when `listen` becomes a typed Rust API).
5. **Event volume is derived too.** `listen` for `pointermove|mousemove|wheel|scroll|touchmove|resize|drag|dragover` requires a frame-coalescing mode and has no raw option. Otherwise `use_window_listener("pointermove")` posts 60-120 IPC messages per second on desktop (the plan's own C5 analysis, 203-205, rejects exactly that).
6. `pub` visibility: the hooks and verbs must be public API of `dioxus-primitives`. Today 25 eval sites live in `preview/` (V: grep), 2 of them in installed components (`form/component.rs`, `code_block/component.rs`). If the bridge is `pub(crate)`, demos and installed wrappers have no sanctioned path and will keep using eval.

**Subsumes:** B13, B14 for listeners, the `blocking-ok` and `interval-ok` classes. **Does not subsume:** a verb that is simply missing (the long tail: clipboard, `scrollTo`, `history`): that needs a cheap path, section 11 "adding a verb". Without one, agents route around the gate, which is the original failure.

---

## 3. [major] Enum-to-switch parity by parsing two languages is the wrong construction

**Attacks:** root doc 5.3 item 1 ("the gate parses both"), plus "no default arm".

**Evidence.**
- A regex parse of a Rust `enum Rule` and a TS `switch` is brittle: `#[serde(rename)]`, `cfg`-gated variants, nested enums, `case 'a': case 'b':` fallthrough, a `HANDLERS = { gate, pan }` table instead of a `switch`, and the region markers the plan wants for concurrent lanes (`bridge/rules.rs` is edited by 3.B1, 3.B5, 3.GS in separate "marked regions", root doc 5.4).
- "No default arm" cannot be checked by the toolchain: `bun build` strips types and does not typecheck, so a `never` exhaustiveness check never runs (I: no `tsc` is installed; `playwright/node_modules` has no `typescript`, V).
- A catch-all variant (`Rule::Custom(String)`, `Rule::Js`) with a free string the interpreter acts on defeats parity while keeping it green.

**Change:** one source of truth plus a golden file, no parsing.
1. A `cargo test` writes/compares the committed `primitives/src/ts/rules.gen.ts` (the kind union, the verb enums, field names) from the Rust types, the way `check-js-bundle` pins generated output. Drift is a failing test with the diff in the message.
2. A Rust `fn sample(r: &Rule) -> &'static str { match r { ... } }` with **no wildcard arm** (clippy `wildcard_enum_match_arm` denied on that fn) names a vector for every variant; adding a variant does not compile until its vector exists.
3. A `bun test` asserts `Object.keys(HANDLERS)` equals the kinds in `rules.gen.ts` (runtime key sets, not text).
4. The gate keeps only: no `String`-typed field in `Rule` that the interpreter treats as code (a field is a selector, an id, an attribute name or a number; fields of "command" type fail); every file under `ts/` appears by name in the budget table below.
5. **File-level budgets** replace the `ts/rules/` directory glob: `scripts/check-js-sinks.budget.tsv` (`path<TAB>max_lines<TAB>row`), shrink-only against the merge base (finding 4). A new `ts/` file needs a row. It must also be imported by the interpreter index (reachability), otherwise it is a dead file hiding code.

**Subsumes:** B8, the catch-all variant, the region-marker merge problem (the enum is still one file, but its parity is test-enforced, so a bad merge fails loudly). **Does not subsume:** whether a rule's semantics match its vector (finding 7).

---

## 4. [major] "Shrink-only" and "allowlist, stale entries fail" are not enforced across commits

**Evidence (V).** Of all `scripts/check-*.sh`, only `check-conflict-markers.sh` and `check-js-listeners.sh` mention `git show`/`merge-base`/`origin/main`, and `check-js-listeners` only in a usage comment. `check-history-comments.sh` compares against `scripts/check-history-comments.baseline.tsv` in the same tree: an agent raises a count and edits the baseline in the same commit and the gate is green (the baseline is two lines, `wc -l`). `scripts/lib/allow.py` guarantees a reason and detects stale entries, not that an entry is *new*. The 18 `listener-ok:` markers are one-line self-service.

**Change:** `scripts/lib/ratchet.py` (beside `allow.py`, imported by every ratchet gate; `check-gate-kit.sh` item 4's grep rule then forbids a hand-written copy).
- `Ratchet(gate, baseline).check(measured)` fails when a key is new or higher than in `git show $(git merge-base HEAD origin/main):<baseline>` (fall back to `main`; with no ref, print `ratchet base unavailable, compared with the tree only` and pass, like `check-js-bundle`'s skipped half).
- A deliberate raise needs `# raised: row N: <reason>` on the changed baseline line, printed in the run-gates summary as `RATCHET RAISES: ...`. This is not prevention (the gate cannot out-vote the owner), it is **visibility**: the lane report template lists them, and the main loop sees them at merge. Same for new allowlist lines (`ALLOWLIST ADDED: n`).
- The new gates take **no inline marker** (`...-ok:`). An exception is a TSV line with a row.
- Per-file rows, no totals line in the TSV, so eight concurrent lanes lowering different files merge without conflicts (they edit different lines; `.gitattributes` merge drivers exist if they ever collide, row 400).

**Subsumes:** the shrink-only promise of `check-history-comments`, `check-comment-refs`, `check-props-docs`, `check-cross-component-overrides` (they can adopt the helper later).

---

## 5. [major] Land the JS-volume ratchet now, before lane 3.0

**Attacks:** the plan's sequencing (the gate arrives in lane 3.0 or 3.B1, after the bridge exists).

**Evidence.** The first port lane is weeks away; 103 blocks were built with no enforcement and nothing stops block 104 meanwhile. Also, the plan puts the gate in two places: "Lane 3.0 adds ... `scripts/check-bridge-rules.sh`" (adopted section) while the lane table gives `scripts/check-bridge-rules.sh` to 3.B1 (root doc 5.4, row 1). Lane ownership of one gate in two lanes is the file-overlap the CLAUDE.md delegation rule forbids.

**Current numbers (V, my count over non-comment lines of tracked `.rs`):** 72 files carry a sink; `eval(` 83 sites; `JsListeners::install(` 45 sites; plus `message_scroller.js` 1,065 lines outside every manifest. Counts vary slightly from the audit's 103 blocks because the audit counts blocks by string extent; the gate needs a deterministic measure, not the audit's.

**Change: step 0, `scripts/check-js-volume.sh` + `scripts/check-js-volume.baseline.tsv` (about 2 agent-hours, one lane, no bridge needed).** It is the first half of the planned `check-js-in-rust.sh` and the first thing to merge.

Per tracked file (universe `git ls-files`, minus `playwright/`, `scripts/`, `docs/`, generated `primitives/src/js/`), measured:

| kind | measure | why this one |
|---|---|---|
| `sink` | count of S1 + `JsListeners::install(` + `Script {` + `include_str!(*.js)` | a new eval site fails at once; a new file with a sink fails |
| `js_lit` | lines inside Rust string literals that read as JS | stops "merge ten evals into one"; sink count alone is gameable |
| `js_file` | lines of hand-written `.ts`/`.js` (not the generated bundle) | stops "move eval into a new `.ts`"; the old plan (one TS module per cluster) would have raised it |
| `inline_script` | body bytes of each inline `<script>` in tracked `.html` | the PREPAINT copies |
| `comment_eval` | comment lines naming `document::eval` or "house pattern" for it | the stale instruction (section 10) |

- "Reads as JS" reuses the one Rust string tokenizer: extend `scripts/comment-scan.py`'s `extract_rust` (it already finds every normal and raw literal with source lines, and has a JS classifier `_EMBED_JS`, `comment-scan.py:137-146`) with a `literals_of` generator, instead of a second scanner. The classifier today requires a `//` or `/*` in the string; the volume gate drops that condition. Never write a second extractor (the row 266 and row 429 classes: `check-gate-kit.sh` item 4).
- **Totals are not stored.** The gate also computes `sum(js_lit) + sum(js_file)` and fails if it rises against the merge base. A port that moves 100 lines from a Rust string into a new `.ts` lowers one and raises the other; the total must still drop, or the allowlist row says why (a pre-wasm class (c) module).
- **The bridge gets a budget, not an exemption:** `bridge.ts`, `rules/*.ts` and `transport.rs`-side JS carry `max_lines` rows (the plan calls the interpreter "small"; a line cap is the proof, and it stops the interpreter becoming the new bespoke dump as B1/B2/B3/B5 grow). Sizing: the nine B-mapped blocks are 1,038 lines of today's JS (root doc executive table), so a first cap of about 600 for the interpreter plus verbs is a decision worth stating.
- It reports `--report` (before/after per file) for the lane report.
- Must pass the `check-gate-kit` conventions (section 6).

**Interaction with porting lanes.** A port lowers its own file's row and runs the gate's `--update-baseline`. Rows are per file, lanes own disjoint files, so merges are clean. The one shared row is the bridge's budget.

**Subsumes:** growth during the months of porting; the unmanaged-`.js` hole; the "every eval site carries its audit class" idea (replaced by a measure that cannot be argued with). **Does not subsume:** quality of what moved.

---

## 6. [major] How the new gates must follow the repo's conventions

**Evidence (V):** `scripts/check-gate-kit.sh` header items 4 and 7, `scripts/lib/allow.py`, `scripts/comment-scan.py`, the `--self-test` pattern in `check-component-call.sh:284-333`, `check-anchor-marker.sh:158`, `check-timing-relative.sh`, and `dev-docs/gates.md` ("A new gate").

1. **One reader per language.** Rust: `comment-scan.py` gains `code_of(path)` (comments removed, string contents blanked to same-length runs so columns survive) and `literals_of(path)`; no gate writes `strip_comments`, `blank_comments` or a second tokenizer (gate-kit item 4 would fail it). Allowlists: `Allowlist` from `allow.py`. Ratchets: the new `ratchet.py`. Shell and python gates import them with `sys.path.insert(0, "scripts/lib")`.
2. **Universe by `git ls-files`**, with an explicit exclusion list, never an inclusion list of roots. Hand-listed roots are why `labs/`, `templates/` and `cli/` were never scanned.
3. **`--self-test` rides along with every real run** (`check-component-call.sh:279-281`): each row of the bypass table (B1-B13) is a fixture that must be red, each control stays green, and a stale allowlist entry is red. A gate whose self-test rots would go green on a regression.
4. **Fast, no cargo, no network.** Pure python; it belongs in lanes' `--fast` set. Clippy is the cargo-side twin and runs at the batch gate.
5. **Header paragraph.** `gates.md` prints the first paragraph of each header, so the first paragraph must say the rule and link `dev-docs/js-interop.md`. `check-gates-doc.sh` then forces `scripts/gates-doc.py --write`.
6. **Register** in `scripts/run-gates.sh` (the "Timers, listeners, motion and focus" group). A gate not in the list fails the run.
7. **Retire, do not stack.** `check-js-listeners`, `check-blocking-scroll-listeners` and `check-uncleared-intervals` each read JS-in-Rust by regex with its own marker. They become either `.ts` scans (while class (c)/(d) code exists) or are deleted when the allowlist reaches zero. Their "NOT covered" paragraphs (observers, `setTimeout` chains, rAF loops: V in `check-js-listeners.sh:47-52` and `check-uncleared-intervals.sh:48-53`) are the gaps the new ban list closes.
8. **Failure messages are the documentation** (section 11): each message names the rule, the replacement, the page anchor, and says an allowlist entry is the owner's decision. Today `check-js-listeners`'s own "Fix:" text tells the agent to write `listen(...)` inside the eval script, which teaches the old way.
9. **Gate ownership.** Put the whole gate in lane 3.0 (or in the step-0 lane above), not in 3.B1 as well; 3.B1 only adds B1 vectors.

---

## 7. [major] Verification: the JSON vectors and `bun test` prove less than claimed

**Attacks:** "JSON conformance vectors run by `cargo test` and `bun test`" (best-practice review, folded into lane 3.0); "Every variant needs a `bun test` unit and at least one Playwright spec" (root doc 187).

**Evidence.**
- `bun test` here would be new infrastructure: `primitives/src/ts/` holds one file and there is no test, no `package.json`, no `tsconfig` (V). The interpreter's predicates need a DOM (`closest(sel)`, `inside(id)`, `can-scroll`, `getComputedStyle`), and bun has none. If the vectors run only against serialisation, they prove the serde field names agree and nothing about decisions.
- bun may be absent. `check-js-bundle.sh:60-72` degrades to "rebuild check skipped" when `bun` is missing or a different version, and `session-setup.sh` does not install bun (V: no `bun` line). A `bun test` gate must say the same, so a green on a machine without bun says "skipped", not "passed".
- The Playwright half is "grep for a `rule:` tag" (root doc 5.3 item 2). A comment `// rule: Gate` satisfies it without executing anything (I).

**Change:**
1. **Split the interpreter into a pure `decide(rule, snapshot, event) -> effects` and a thin DOM adapter that builds the snapshot and applies effects.** Vectors are `{rule, snapshot, event, expected_effects}` triples. Both `cargo test` (through a Rust reference `decide`, needed anyway for the desktop path in finding 8) and `bun test` read the same file. Without the split the vectors are theatre.
2. **Coverage is measured at runtime, not by grep:** in debug builds the interpreter records `window.__dxRuleCoverage` (kind plus effect arm); a Playwright fixture appends it per test; a post-suite check fails if a variant was never executed. Run it inside the deploy suite.
3. Pin the vector file's schema version in the generated `rules.gen.ts` (finding 3), so a stale vector file fails to load, not to match.
4. Report "bun tests: skipped (no bun)" in the gate summary text when skipped.

**Subsumes:** vector/interpreter drift; grep-tag gaming. **Does not subsume:** that the browser's event order, passive and capture semantics match the model; only Playwright (web) proves that.

---

## 8. [major] What proves that desktop behaves the same? Nothing in the plan

**Attacks:** "slow-channel mode", "cargo test replay of the committed trackpad recordings", "one manual desktop pass per batch".

**Evidence.**
- **The trackpad recordings are weak evidence.** `playwright/fixtures/carousel-wheel-recordings.json` (V, read): 11 bursts, 9 real, 2 synthetic; recorded on one MacBook trackpad in Firefox 156; "NO per-event timestamps", events spaced uniformly at `intervalMs` or 17 ms ("Real inter-event jitter is lost"); only the first 80 deltas were logged, three bursts are trimmed. The expected-output file is captured from the current JS (residue doc 3.3), so the replay is a golden against the old implementation on a synthetic uniform timeline. It proves the Rust port computes what the JS computed on that timeline. It says nothing about real jitter, other devices (Windows precision touchpad, libinput, Safari momentum), or batching.
- **The plan's own limit:** the slow-channel harness delays bridge messages by N frames, but `MountedData` reads stay synchronous on web (root doc 5.6), and lane 2C already moved reads onto `get_client_rect`/`get_scroll_offset` (V, row 434 progress). So any code that is correct under N = 2 only because it uses `get` is protected, and any code using `MountedData` directly is not.
- **Desktop is not in CI, and cannot be:** no gtk/webkit2gtk here, workflows are dispatch-only (CLAUDE.md), and the container has only Chromium (V: `ls ~/.cache/ms-playwright` shows `chromium*` and `ffmpeg`; `playwright.config.ts` defines firefox and webkit projects that do not run here).

**Change (what to add, in order of cost):**
1. **Property test for the wheel classifier under jitter and batching.** Replay each recording with (a) jittered inter-event gaps drawn from the spread of the real 60 Hz and 120 Hz streams, (b) delivery in per-frame batches, (c) batches delayed by 0, 1 and 2 frames. Assert the release kind is the same and the release time moves by no more than the injected delay plus one frame. This is the missing proof for the `sample` path, runs in `cargo test` in milliseconds, and exercises the classifier as a pure function of (events, tick schedule). The classifier takes `now` from its input, never from `std::time`/`web_time` (gate: those names are banned in `wheel_band.rs`).
2. **Keep the old JS engine as a test oracle** (a fixture, not shipped) until the desktop measurement has run, so the golden can be regenerated and the live parity row of residue 3.5 can be re-run.
3. **Transport-level fault model, not only delay:** the seam's test mode also reorders `get` replies across verbs and drops nothing. Plus a lint-by-gate: no `get_client_rect|get_scroll_offset|get_scroll_size|get_client_size|scroll_to` outside `primitives/src/bridge/` and a short allowlist with reasons, so the harness covers all reads. It conflicts with lane 2C's direct `MountedData` use; either those moves go through bridge wrappers, or the allowlist names them.
4. **Engine coverage that does exist:** run the A-class (anchor positioning, `overflow-anchor`) and G-class specs on the webkit and firefox projects where a machine has them. Name the matrix in the row: `A/G specs: chromium (CI), webkit, firefox (manual)`. WebKitGTK support for anchor positioning is unverified (root doc 5.5, item 6), so the fallback stays.
5. **A desktop pass is a recorded artifact, not a hope.** `dev-docs/desktop-pass.md` with the checklist (residue 3.5's table), and a stamp file the way `release-suite/<tree>.pass` works for the suite (`lane-rules.md` deploy step): `desktop-pass/<tree>.pass` with machine, OS, webview version and the numbers. The row's status says "web-verified; desktop inferred" until a stamp exists for the tree, and CLAUDE.md's lane report line says the same. Do not let a G or R mapping be marked done on web evidence alone.

**Subsumes:** replay-on-uniform-timeline weakness, the unrecorded desktop pass. **Does not subsume:** WebView2/WKWebView behaviour. Only a machine proves that.

---

## 9. [major] The timing specs will not notice what the bridge changes, and one design choice collides with an existing oracle

**Attacks:** "Risk: desktop and mobile are not in CI", the claim that G costs "zero frames on web", and the delegated interpreter.

**Evidence.**
- **Thresholds are coarse.** `oracle/tier2-html/main-thread.spec.ts` asserts no long task over 50 ms and no interaction over 200 ms (V: spec header). Lane 2C measured 0 long tasks and 16-40 ms max interaction on select/combobox/navigation menu (row 434 progress). An interpreter that adds 5 to 30 ms to every keydown/pointermove passes. The plan states the cost nowhere.
- **INP and long tasks are blind to G.** A gated panel (`opacity: 0` until Rust answers) still produces a painted frame, so Event Timing (`processingEnd`, next paint) looks fine while the user waits. The latency G adds is not a measured quantity anywhere in the plan; the slow-channel run asserts the end state at N = 0 and N = 2, not time-to-visible. Focus into an `opacity: 0` panel interacts with `touch-open-focus` and `modal-initial-focus` (root doc 5.5 item 2).
- **Collision with `listener-inventory.spec.ts`** (V: header). It asserts the multiset of (type, capture, passive) on window/document/html/body after open/close equals the baseline taken BEFORE the first open, "nothing lazily installed for the rest of the session", and no non-passive wheel/touch listener on those targets. The adopted design says "one delegated capture-phase interpreter reading `data-dx-rule*`" and rules with `prevent` on wheel/touch. A document-level interpreter is either installed lazily (fails the inventory) or eagerly and permanently (a capture listener per event type on every page, forever), and a wheel/touch `prevent` rule at document level is non-passive, which blocks compositor scrolling page-wide (the class `check-blocking-scroll-listeners.sh` and root-doc C1 item 3 exist for).
- **Release-only means late.** The timing specs run on release builds (`PW_BUILD_PROFILE`; `lane-rules.md`), the full suite once per deploy, 66-82 min. A lane gets none of that from `test:changed` on a debug SSG build.
- **Registry growth:** a rule registry that holds elements strongly (`Map<id, rule>`, `window.__dxRules`) leaks across route changes and 'element-bound rules die with the element' is only true if the registry is a `WeakMap` or the rule is attached to the element itself.

**Change:**
1. **Delegate only non-scroll events** (`keydown`, `click`, `pointerdown`, `beforeinput`, `dragover`, `focusin`) on one eagerly installed capture listener per type, included in the inventory baseline and documented there. Wheel, touch and any `prevent` on scroll-affecting events are **per-element** listeners created by `rules.install(el, ..)` and removed with the element; the interpreter derives `passive` from the rule (finding 2). Update `listener-inventory.spec.ts`'s expected set in the same lane, deliberately.
2. **A relative cost spec** `bridge-cost.spec.ts` using `playwright/frame-cost.ts` and the `check-timing-relative.sh` convention (assert a ratio against a no-op listener measured on the same page, never an absolute ms): per-event interpreter cost with 0, 10 and 50 installed rules on `keydown`/`pointermove`, and `JSON.parse` of `data-dx-rule*` cached per attribute string (WeakMap). This one is valid on debug as well, since it is relative.
3. **A latency spec for G** (`g-latency.spec.ts`): frames from the opening input to the first frame where the panel's computed `opacity` is above 0, at N = 0 (expect at most 1) and N = 2 (expect at most 3), per G-mapped component. This is the number the plan promises ("zero frames on web") and nobody measures.
4. `tools/select.mjs` maps `bridge.ts`, `ts/rules/`, `bridge/*.rs` and `rules.gen.ts` to a fixed core set (main-thread, scroll-main-thread, listener-inventory, bridge-cost, g-latency) so a lane's `test:changed` includes them. Say in the lane rules that timing results from debug builds are not evidence (already true) and that the main loop runs them on release before merging a bridge-touching batch.
5. A leak spec: mount/unmount a rule host 200 times, assert the registry size is 0 (`window.__dxJsListeners.live` is the precedent, `js_listener.rs:36-40`).

**Subsumes:** the blind spots of INP, the inventory collision, registry leaks. **Does not subsume:** desktop latency (finding 8).

---

## 10. Why 103 bespoke eval blocks? (the owner's question, answered from the repo)

**Evidence (V).**
- The only written convention for "I need the DOM" **was eval**, in the code an agent reads while working: `carousel/drag.rs:76` ("`document::eval` is this crate's established escape hatch for DOM access"), `top_layer/mod.rs:6` ("this crate's `document::eval` house pattern (see `use_form_reset_listener`, `lib.rs`)"), `input_otp.rs:30` ("the *shape* -- a small `document::eval` script"), `popover_sync.rs`, `scroll_lock.rs`, `carousel/context.rs`, `carousel/scroll.rs` and others (about thirty comment lines name it). The neighbour you copy teaches eval.
- Every JS gate regulates the *shape of the string* and so blesses the string. `check-js-listeners.sh`'s own failure text says to register through `listen(...)` inside the script; `js_listener.rs`'s module doc sample is a JS string in Rust; `check-uncleared-intervals` says "the one sanctioned timer is `use_interval`", correct, but the message never says "do not write JS".
- No doc says how to do DOM work. CLAUDE.md, `lane-rules.md` ("Conventions the gates enforce" lists "listeners through `listen`") and `dev-docs/README.md` have no interop page (V: `ls dev-docs` has none; `reference/platform-rationale.md` mentions eval only as "a no-op on native"). A gate page is read after the code is written, "not before it goes red" is the lane-rules wording, and an agent writing new behaviour does not open `gates.md` first.
- No cost for the wrong choice: eval needs no new file, no new verb, no review; a typed alternative did not exist.

**Change:** an authoritative page, a pointer on the path an agent actually reads (the file's neighbours), a gate message that teaches the replacement, and removal of the stale instruction (section 11 item 6).

---

## 11. Docs and instructions: exact wording and placement

### 11.1 `CLAUDE.md`: one bullet under "Rules for every change", after the "Code describes only its current state" bullet

> - **Browser interop goes through the bridge, never a script string** (`dev-docs/js-interop.md`). Rust owns behaviour. To read or change the page, use the typed bridge hooks and verbs (`use_window_listener`, `use_media_query`, `el.show_popover()`, `use_resize`, ...) or install a `Rule` the interpreter already knows; for a conditional `preventDefault` or a drag, the rule is data. If the verb or rule you need is missing, add it (one table line plus a conformance vector, no backlog row for a verb), do not write `document::eval`, `document::Script`, a `<script>`, a `.js`/`.ts` file, or a `listener-ok:`-style marker. `check-js-sinks`, `check-js-volume` and clippy's `disallowed_methods` refuse them, the JS volume only shrinks, and an allowlist entry is the owner's decision, never a lane's.

Also, in the Delegation bullet that names `lane-rules.md`, append: "every lane that adds or changes interactive behaviour is pointed at `dev-docs/js-interop.md` in its prompt."

### 11.2 `dev-docs/lane-rules.md`

Under "Conventions the gates enforce", replace the `listeners through listen` clause with `browser access through the bridge (dev-docs/js-interop.md)` and add this paragraph after the first one:

> **Interactive behaviour: read `js-interop.md` before you write it.** The ladder is Rust handler (a Dioxus `on*` prop, synchronous, `prevent_default()` works) -> platform (CSS: anchor positioning, `scroll-padding`, `overflow-anchor`, `linear()`) -> bridge verb (`listen`, `call`, `get`, `observe`, `match`) -> a rule the interpreter knows. There is no step "write a script". If a step is missing, add the verb or rule; do not route around it. A lane that touches JS reports, in its <= 40 lines: `scripts/check-js-volume.sh --report` before and after, and every allowlist or baseline raise it made (path, reason, row). A raise is a question for the owner in the report, not a decision.

In "Report (<= 40 lines)" add: "JS volume before/after and any ratchet raise (`RATCHET RAISES:` from `run-gates.sh`)."

### 11.3 `dev-docs/README.md`: one row in "Rules and recipes for sessions and lanes", after `motion.md`

> | `js-interop.md` | How Rust talks to the page: the decision ladder, the verbs and rules, what is forbidden and which gate says so, how to add a verb, how to test it (vectors, the slow channel, the desktop pass). Read before writing any interactive behaviour. |

### 11.4 `dev-docs/js-interop.md` (new; outline, about 120 lines)

1. **The rule, in three lines.** Rust owns behaviour. The page gets verbs and rules, never source. Nothing runs a string.
2. **I want to ... use this** (the table agents grep): react to a window/document/element event -> `use_window_listener` / `use_listener`; call `showPopover`, `focus`, `scrollIntoView` -> `el.show_popover()`...; read size/position/computed style -> `el.rect()` / `get`; know when size or visibility changes -> `use_resize` / `use_intersection`; media query -> `use_media_query`; wait or time -> `sleep`/`use_interval` (Rust tasks, no JS timer); cancel an event conditionally (wheel band, Escape in nested layers, OTP filter) -> a `Rule::Gate`; drag/swipe -> `Rule::Pan`; a value that must follow layout -> a CSS rule or `Rule::Publish`; anchoring/scrolling -> CSS first (A), then a gate (G).
3. **Why not a script.** Two lines: async hop, desktop, CSP `unsafe-eval` (web eval is `new Function`); and the cost of 103 blocks.
4. **Forbidden, and the gate that says so** (a 3-column table: thing, gate, replacement): `document::eval`, `document::Script` with content, `script {}`, `dangerous_inner_html` on script, `include_str!(*.js)`, `.js`/`.ts` outside `primitives/src/ts/`, `web_sys`/`js_sys`/`wasm_bindgen`, `setTimeout`/rAF/observers in `ts/` outside the bridge, a marker comment.
5. **Adding a verb** (verb = data, no row): one enum arm in `bridge/verbs.rs`, one table line in `ts/bridge.ts` (receiver kinds), one vector, `cargo test` regenerates `rules.gen.ts`, `scripts/build-js-bundle.sh`. **Adding a rule kind** (language growth = backlog row): the extra steps (pure `decide`, vectors, budget row).
6. **Testing:** vectors (`cargo test`, `bun test`), the slow channel (`window.__dxBridgeDelayFrames = N`, run G specs at 0 and 2), `bridge-cost` and `g-latency`, the desktop pass (`desktop-pass.md`), the words "web-verified" vs "desktop-verified".
7. **Allowlists:** how to read `check-js-sinks.allow.tsv`, why you do not add to it, what the owner needs from you if you think you must (the row, the measured reason).
8. **Pre-wasm class (c):** `PREPAINT_JS` and its copies, the only inline scripts, and which gate keeps the copies identical.

### 11.5 Gate failure messages (each gate, one shape)

> `check-js-sinks: primitives/src/foo.rs:123: `document::eval` runs a script string. Browser work goes through the bridge: dev-docs/js-interop.md#i-want-to--use-this (window event -> use_window_listener, geometry -> el.rect(), conditional cancel -> Rule::Gate). Missing a verb? dev-docs/js-interop.md#adding-a-verb (no backlog row for a verb). Do not add an allowlist entry or a marker: exceptions are the owner's decision, and the JS volume only shrinks.`

Rewrite the "Fix:" paragraphs of `check-js-listeners`, `check-blocking-scroll-listeners` and `check-uncleared-intervals` until they are retired, so none of them tells an agent to write a listener inside a script.

### 11.6 Remove the stale instruction from the code agents read

The comments in item 10 that call eval "the established escape hatch" or "house pattern" are instructions, and they are wrong after this plan. One lane (Haiku-fit: closed file list, verifying gate) rewrites each to "see `dev-docs/js-interop.md`" or deletes it, and `js_listener.rs`'s module doc becomes the bridge's `//!` page with a Rust example (no JS string). The `comment_eval` row of the volume ratchet (section 5) keeps them from coming back. Add `//!` on `primitives/src/bridge/mod.rs` and a one-line pointer atop `primitives/src/lib.rs` ("Browser access: see `bridge`; never `document::eval`, `dev-docs/js-interop.md`"): that is where an agent opening `primitives/src` lands.

**Subsumes:** the information gap, the stale precedent, the lane brief. **Does not subsume:** an agent that does not read; the gates and clippy are the real stop.

---

## 12. Minor findings

- **[minor] The bundle gate's rebuild half is skipped when bun is missing or a different version** (V: `check-js-bundle.sh:60-72`); the sha half then only proves "manifest and files agree", which an agent can satisfy by regenerating the manifest. The new bridge is bigger than `focus-trap.js`; record `bun` in the budget gate's summary (rebuild: ok/skipped) so a skipped rebuild on the merge-batch machine is visible, and pin `bun` in `session-setup.sh` so the rebuild half actually runs.
- **[minor] Concurrent enum regions** (root doc 5.4, last paragraph): three lanes appending to one `bridge/rules.rs` in "marked regions", merged by the main loop. With finding 3's golden file, a bad merge fails loudly; still create the file with all four region headers first (the plan's own fallback), as lane 3.0's last step, not "if uncomfortable".
- **[minor] Transport seam hash:** `transport.rs` declares its eval string constants; the sink gate records their sha256 so a change is a visible diff in the baseline, not a quiet edit.
- **[minor] Installed components:** `dx components add` copies a component's whole directory; `check-installed-paths.sh` already checks external crates against `cargoDependencies`. Add: an installed component's Rust files contain no S1-S6 sink, since the user's app has no `transport.rs` allowlist. Two installed components have eval today (`form`, `code_block`).
- **[minor] The shared `message_scroller.js` baseline row must carry its expiry** (the W8 lane, row cited), or it becomes a permanent exception in the volume gate.

---

## Attacks that failed (what I tried and could not break)

- **Hand-editing a generated `.js` to slip logic past the bundle gate:** blocked by the manifest sha plus the rebuild half when bun matches (`check-js-bundle.sh`). It can be defeated only by also regenerating the manifest, which is a visible diff, and by running on a bun that differs (reported as skipped). Held.
- **Leaking listeners through the new `listen`:** the runtime backstop `listener-inventory.spec.ts` (CDP `DOMDebugger.getEventListeners` on window/document/html/body, before/after twice) would catch it, and `window.__dxJsListeners.live` names the leak. Held, apart from the delegated-interpreter collision in finding 9.
- **`document::Script` inline text as a hidden eval:** confirmed it reaches eval (`document.rs:92-95`), so S2 is needed; but a plain `Script { src: asset!(..) }` is the shape the repo already uses (`lib.rs:136`), so the allowlist is a short list of asset constants, not a ban.
- **`dangerous_inner_html` executing a `<script>` set via innerHTML:** innerHTML-set scripts do not run, but the `script { dangerous_inner_html }` element rendered through SSG HTML does (`docs_nav.rs:204`), and `<img onerror>` runs from either. So S3 stays.
- **A gate on `addEventListener` through an aliased `preventDefault`:** `const pd = e.preventDefault.bind(e)` still contains the token, so the canonical ban list catches aliasing by destructuring or binding. Only computed access escapes, and that is closed by banning computed access.
- **Parity gate can be satisfied by a comment:** the `rule:` tag grep can (finding 7), the golden-file test cannot (it compares generated text with committed text).
- **`concat!` splitting in the Rust gate:** defeated for the sink gate, because the gate keys on the sink identifier and clippy resolves by path, not on the string.
- **Does a Rust string ban work on `///` doc text?** It does not need to: the code view excludes comments, and a doc mention of `preventDefault` stays legal.
- **Whether a typed Rust `listen` can drop the `blocking-ok` class:** it can, by deriving passive from the rule's effects; that survived every attack I tried.
- **The plan's claim that Dioxus handlers are synchronous on web and desktop** (root doc 1.2): I did not re-derive the desktop XHR path; I took the cited lines (`dioxus-desktop` `handleVirtualdomEventSync`) as stated. Not attacked.
- **Not examined:** the layer ladder R/A/G/B itself, the per-block mapping table, CSS anchor positioning support, and Safari 27 `overflow-anchor` quality; other board lenses own those.
