# Text editor component: is there a Rust library we can use? (research, 2026-10-21)

> **Outcome (owner 2026-10-21):** row 337 is closed as out of scope. The library keeps to primitives and shadcn element parity and builds no editor ([`backlog.md`](../backlog.md#scope)). This note stays as a reference for an app or framework that adds one.

Scope: backlog row 337 (text editor / rich-text composer). Option A (markdown textarea + preview), B (wrap Tiptap/ProseMirror), C (own contenteditable). This note answers "option B-Rust": is there an existing Rust-based library?

Method: crates.io API (names, versions, dates, downloads, licences), the crates' own source downloaded and read (taino-edit-*, dioxus-editor, kode-*, dioxus-nox-markdown, dioxus-code-editor), READMEs and repo pages, docs.rs, `npm view`, and a real esbuild bundle-size measurement for the JS editors. The GitHub REST API was blocked for non-session repos, so last-commit dates come from `git log` of shallow clones where I could clone, otherwise from crates.io release dates. No code was run in a browser. Everything flagged "UNVERIFIED" is a claim I could not check.

Size key for efforts (matches row 337's scale): S = about 1 lane-day, M = 3-5 lane-days, L = 1-2 weeks, XL = weeks+.

## 1. Short answer

- Dioxus-specific WYSIWYG engines **do exist now**, two of them: `dioxus-editor` (danielkov) and `taino-edit` / `taino-edit-dioxus`. Both edit through `contenteditable` in the DOM, both are MIT OR Apache-2.0, both are under 6 months old and single-author. Neither is production-grade by the standard of ProseMirror, Lexical or Tiptap, and neither has been verified here on Android/iOS soft keyboards.
- Nothing else in Rust is a usable web editor: the rest are code editors, markdown textareas, terminal/GPU editors, toy wrappers of JS editors, or data structures (CRDTs, ropes, document models) with no DOM view.
- Mature Rust CRDTs with rich-text marks exist (Loro, yrs, Automerge). They are a document model for collaboration, not an editor, and need a DOM view layer on top. Only relevant if collaboration is wanted, which is not in the brief.
- Recommendation: keep row 337's order. Do **A** (markdown textarea + toolbar + existing safe `markdown` preview) first behind an engine-agnostic `value` API. If true WYSIWYG is wanted, the lowest-risk engine is still **ProseMirror/Tiptap core through the eval bridge (B)**. Run a one-day spike on `dioxus-editor` (and optionally `taino-edit`) as the Rust alternative before committing, and treat both as watch-list items, not foundations.

## 2. Candidate table

| # | Candidate | Repo | Licence | Last release / activity | What it is | DOM, canvas or GPU | Dioxus 0.7 | Verdict |
|---|---|---|---|---|---|---|---|---|
| 1 | **dioxus-editor** (danielkov) | github.com/danielkov/dioxus-editor | MIT OR Apache-2.0 | Tag `v1.0.0` (1.x branch, Dioxus >=0.7.10 <0.8) and `v2.0.0-alpha.1` (main, Dioxus 0.8 alpha). 13 commits on 1.x, 15 on main, all 2026-08-01 to 08-02 (one burst). 1 star. `publish = false`: git dependency only, not on crates.io. README says used in production at slick.chat (a chat product). | Full WYSIWYG editor: transaction/step document model, plugins (keymap, history, markdown shortcuts), Dioxus view. About 9k LOC. | DOM `contenteditable` with `role=textbox`, `aria-multiline`, `aria-label`. Intercepts `beforeinput` and calls preventDefault, then re-renders from the model through Dioxus rsx (so the SSG can render real content). | Native. Needs Dioxus >=0.7.10 (we are on 0.7.9; 0.7.10 exists, released 2026-07-30). | Best fit on paper, least proven. Spike candidate. |
| 2 | **taino-edit** + **taino-edit-dioxus** (juanma-dev) | github.com/juanma-dev/taino-edit | MIT OR Apache-2.0 | crates.io: first release 2026-05-21, 0.7.0 on 2026-09-05 (about 6 weeks ago), 8+ releases in 3.5 months. 116 commits, 15 stars, 4 forks, about 2k downloads of core. Seven crates. | Full editing framework modelled on ProseMirror/Tiptap: typed schema, invertible steps, history, keymap, input rules, 13 extensions, tables, Markdown/HTML/JSON codecs. Core is framework-agnostic and runs headless. | DOM `contenteditable` bridge (`taino-edit-dom`, `web-sys`). Reads DOM changes on `input` events plus own `keydown` keymap. No `beforeinput` (grep of the 0.7.0 source finds none). | Native adapter `taino-edit-dioxus` 0.7.0, depends on `dioxus = "0.7"`. Its `wasm-bindgen 0.2.121` and `web-sys 0.3.98` floors equal our `Cargo.lock` exactly. | Most complete feature set and a real crates.io release train. Weakest input model (see section 3). Spike candidate. |
| 3 | **kode** (`kode-leptos`, `kode-doc`, `kode-core`) | github.com/kyomi-ai/kode | MIT | 22 releases, 0.5.5 on 2026-09-11; 125 commits, 4 stars; 5k downloads. | Leptos code editor plus a tree-based WYSIWYG markdown editor. `kode-doc` is a framework-agnostic document model (about 10k LOC). | `contenteditable`, edits via `beforeinput` + `keydown`, restores selection after re-render. | **Leptos only.** The 5k-line `tree_editor.rs` is Leptos-specific; `kode-doc` pulls `arborium` (tree-sitter). No Dioxus. | Ruled out as a base, useful as design reference (it also uses `beforeinput`). |
| 4 | **dioxus-nox-markdown** | github.com/7A9599F5/dioxus-nox | MIT | 0.13.2, 2026-03-30; 1 star; 153 downloads. README states it was entirely AI-generated and not battle-tested. | Headless markdown editor/preview. Source, inline ("live preview") and read modes over a `crop::Rope` AST. | Hidden transparent `<textarea>` as IME proxy plus rendered DOM; no `contenteditable`. | Native, Dioxus 0.7. | Interesting architecture (accessible, IME-safe), unproven. Markdown, not rich text. |
| 5 | **riparion-editor** | github.com/Riparion/riparion-editor | MIT OR Apache-2.0 | 0.1.0, last commit 2026-06-08; 0 stars. | Block-swap live preview: rendered blocks, the active block becomes a native `<textarea>`. Bring-your-own renderer. | Plain controlled `<textarea>`; no `contenteditable`, no JS. | Native, Dioxus 0.7. | A proven *pattern* for option A+: no caret fight, native IME and a11y. Inline formatting does not render in the line being edited. Not rich text. |
| 6 | **dioxus-code-editor / dioxus-code** | github.com/ealmloff/dioxus-code | MIT | 0.2.0-alpha.3, 2026-10-16; 4k downloads. | Syntax-highlighted code editor: transparent `<textarea>` over a highlighted layer; uses `beforeinput` selection ranges. | DOM textarea overlay. | Native. | A code editor, not a composer. Could serve as the "code block" editor inside a rich editor, no more. |
| 7 | **dioxus_codemirror** | github.com/azriel91/dioxus_codemirror | MIT OR Apache-2.0 | 0.3.1, 2026-06-09. | Wraps CodeMirror 6 through one `document::eval` channel, vendored as a Dioxus folder asset. | DOM (CodeMirror). | Native. | Code editor. Valuable only as **precedent for the JS route**: a Dioxus crate that already ships a JS editor over `document::eval` with vendored assets and no CDN. |
| 8 | **leaf** (`leaf-core`, `leaf-wasm`, `leaf-web`, `leaf-dioxus`) | github.com/diaryx-org/leaf | Apache-2.0 OR MIT | `leaf-core` 0.5.0 (2026-10-03), 38 releases, 772 commits, 2 stars. README is headed "Work in progress!!". | Caret-based rich-text editor over a Markdown/Djot/HTML AST. Frontends: terminal, GPUI desktop, iOS/Android, **web (`leaf-web`, wasm-bindgen + JS)**, and `leaf-dioxus`. | `leaf-web`: one `contenteditable` surface, edits intercepted and mapped to core ops, has `accessibilityValue()` and a live region. But the **Dioxus app uses Dioxus Native (Blitz/wgpu), not the web.** | The web package is a JS API, not a Dioxus component. | Ruled out for now: the Dioxus frontend is native-GPU, the web one is an npm package (README contradicts itself on whether it is published: "not yet published" in one place, "on npm as @diaryx/leaf" in another, UNVERIFIED). Watch-list. |
| 9 | **prosemirror** (fiduswriter/prosemirror-rs) | github.com/fiduswriter/prosemirror-rs | MIT | 0.5.2, 2026-06-01; 167 commits, 3 stars; 330 downloads. | Rust port of `prosemirror-model` and `prosemirror-transform` only, shipped as an npm drop-in for the JS ProseMirror packages. DOM serialisation explicitly out of scope. | No view layer. | n/a | Not an editor. Could be the model under a different view; no reason to when the JS originals run in the browser. |
| 10 | **text-document** (ferntech-eu) | github.com/ferntech-eu/text-document | **MPL-2.0** | 1.13.0, 2026-10-13; 45 releases; 10k downloads. | Qt-style `QTextDocument` model: blocks, format runs, cursors, undo, tables, HTML/Markdown/DOCX I/O, on `ropey`. | Headless; the companion `text-typeset` renders to **GPU quads**. | No DOM view. | Not usable for an accessible web component. |
| 11 | tiptap-rs, quill-wr, leptos-tiptap | dsplce-co/tiptap-rs, ALAWIII/quill-wr, lpotthast/leptos-tiptap | MIT OR Apache-2.0 / MIT | `leptos-tiptap` 0.10.0 2026-07-18 (26k downloads); `tiptap-rs` 0.1.7 2026-08; `quill-wr` 0.2.0 2025-11 | `wasm-bindgen` wrappers around the JS editors. | DOM (the JS editor). | Not Dioxus. | Evidence that the Rust ecosystem's de-facto answer is "wrap Tiptap"; `leptos-tiptap` is the most used. Not Rust editors. |
| 12 | edita, gongcheck-editor, kami | crates.io | MIT/Apache, MIT, MIT | edita 2024-02 (hirola framework), gongcheck 2024-06 (127 LOC), kami 2025-04 (3 LOC) | Abandoned or toy. | n/a | No | Ignore. |
| 13 | unified-markdown-editor-dioxus | github.com/freedomtowin/unified-markdown-editor-dioxus | no licence stated in README (UNVERIFIED) | 13 stars, 2026-07 | A demo app, not a library; cell matrix over `contenteditable`, hand-rolled backspace/enter. | `contenteditable` | Dioxus | Ignore (illustrates how fragile a hand-rolled caret model is). |
| 14 | leptos / yew / sycamore | n/a | n/a | n/a | Leptos: taino-edit-leptos, kode-leptos, leptos-tiptap (above). Yew: only `code-engine` (GPL-3.0 code editor). Sycamore: nothing found. | n/a | n/a | Nothing to port that the Dioxus candidates are not already. Both Leptos engines (taino, kode) are the same designs as the Dioxus ones. |
| 15 | maud-ui | github.com/hgeldenhuys/maud-ui | MIT | 0.21.0, 2026-09-28 | A Rust shadcn-parity UI kit (Maud). Its rich text "editor" is a **TipTap shell hook** in its showcase. | DOM (TipTap) | Not Dioxus | Another Rust shadcn clone that wraps Tiptap instead of building an editor. |

## 3. The two Rust engines worth a spike, compared

Both were read at source level, not run.

| | dioxus-editor 1.x (danielkov) | taino-edit 0.7 (juanma-dev) |
|---|---|---|
| Distribution | Git dependency or vendoring only (`publish = false`); no crates.io release | crates.io, 7 crates, regular releases |
| Dioxus | 1.x: `>=0.7.10, <0.8` (we are on 0.7.9, a bump is needed; the `dx` CLI is 0.7.9, a lib/CLI skew is UNVERIFIED); 2.x: 0.8 alpha | `dioxus = "0.7"`, works on 0.7.9 |
| Size of code | about 9k LOC | about 9.7k (core + extensions) + 1.7k (dom) + 0.6k (adapter) |
| Input model | `beforeinput` intercepted and `preventDefault`ed, the model is authoritative, DOM re-rendered by Dioxus; composition events handled; target-range-aware autocorrect | DOM edit read-back on `input` ("reports the first divergent text run"), own `keydown` keymap for Enter/Backspace etc., composition handled. Out-of-scope list: "mobile touch gestures beyond contenteditable defaults". Android Gboard (keydown 229) behaviour UNVERIFIED |
| SSG / hydration | Content is rendered by Dioxus rsx, so SSR output contains the document | `TainoEditor` renders an **empty `div`** and mounts imperatively in `onmounted`. A host-safe `doc_view_html` exists in `taino-edit-dom` that we could use for pre-hydration content |
| Marks | bold, italic, strike, inline code. Link is a node, not a mark. No underline | bold, italic, link, code, plus case transforms |
| Blocks | paragraph, h1-h6, blockquote, code block, lists, tables, decorator nodes (mentions, embeds as Dioxus components) | paragraph, h1-h3, blockquote, code block, lists, image, align, tables with merge/resize |
| Undo/redo | History plugin | Bounded groupable history |
| Paste | Plain text only (`text/plain`), bare URL becomes link; HTML paste is not imported. Safe by construction | HTML paste parsed through a strict schema-checked, depth-bounded parser (rejects unknown tags) |
| Serialisation | Markdown in/out (pulldown-cmark 0.13, which we already ship). Document model access for JSON: UNVERIFIED | JSON (schema-checked), HTML (escaped serializer), Markdown |
| A11y | `role=textbox`, `aria-multiline`, `aria-label` prop; placeholder prop (`data-placeholder`, host draws it) | Not in the adapter: the rendered div has only a `class`. No `aria-*`, `dir`, placeholder or attribute pass-through. ROADMAP: "basic accessibility yes; certified audit no" (nothing in source backs "basic") |
| RTL | Not handled (no `dir` in view.rs) | Not handled |
| Collab | None | Deferred (Loro integration is "💤") |
| Tests | Rust tests, Playwright e2e suite in `e2e/`; counts UNVERIFIED | 214 host tests, headless-Chromium wasm tests (IME, clipboard, selection, tables) using a locally patched `wasm-bindgen-cli` |
| Maintenance | 2 days of commits, then none visible by 2026-10-21 (on the clone) | commits across May to Sep 2026; last release 2026-09-05 |
| Bus factor | 1, commercial product dependency (slick.chat) | 1, hobby/community project, README invites a community Dioxus adapter |
| Install fit | Needs vendoring into the component dir to pass our `cargo-dep-versions` gate (git deps are not `x.y.z`; whether `dx components add` `cargoDependencies` supports git is UNVERIFIED) | Normal `cargoDependencies` entry; the adapter would have to be replaced by our own wrapper to get aria, `dir`, placeholder and a form hidden input |

Net: dioxus-editor has the better input architecture (beforeinput, authoritative model, SSG-friendly) and a11y hooks, but it is a 2-day-old git-only repo. taino-edit has the broader feature set and a release cadence but the riskier input model and an adapter that exposes no a11y or `dir` hooks. Either way we would end up owning an engine written by someone else, with all the input-method bugs a decade of ProseMirror work has already absorbed.

## 4. CRDT text types with marks (document model under a DOM view)

| Crate | Licence | Latest | Marks | Notes |
|---|---|---|---|---|
| **loro** (loro-dev/loro) | MIT | 1.16.2, 2026-09-21; 945k downloads; 6.2k stars | Yes: `LoroText::mark/unmark/to_delta/apply_delta`, Peritext-style; expand behaviour is documented but not visible in the Rust signatures I read | Best-supported rich-text CRDT. JS bindings `loro-prosemirror` and `loro-codemirror` exist (not Rust, not Dioxus). Undo and cursor/awareness exist. |
| **yrs** (y-crdt) | MIT | 0.28.0, 2026-09-17; 4.2M downloads; 2.2k stars | Yes: `format`, `insert_with_attributes`, `diff`, `apply_delta` | Most deployed CRDT; compatible with the Yjs ecosystem (y-prosemirror etc. are JS). `prosemirror-yjs-html` renders ProseMirror-shaped yrs docs to HTML in Rust (0.1.5, small). |
| **automerge** | MIT | 0.12.0, 2026-09-16 | Yes: `automerge::marks` with `ExpandMark` (Peritext) | Heavier model. |
| diamond-types | ISC | 1.0.0, **2022-08** | Plain text only | Stale; no marks. |
| crdt-richtext | MIT | 0.1.1, **2023-05** | Peritext + Fugue proof of concept | Superseded by Loro; says "do not use in production". |

Relevance: only if collaboration, offline merge or comments become requirements. For a single-user composer with a controlled `value` they add weight (a wasm CRDT) and a second state model to reconcile with the DOM. None ships a DOM view, so they do not remove the hard part (selection, input, IME, paste). Both Rust engines above list Loro as a future `collab` feature; neither has it.

## 5. Text buffers and layout crates

- **ropey** (2.0.0-beta.1, 13.5M downloads), **crop**, **jumprope**, **xi-rope** (0.3.0, 2019): text buffers for large plain-text files. In a browser, text lives in DOM text nodes and the browser owns caret and layout; document sizes for a composer are small. A rope solves a problem we do not have and does nothing for marks, blocks or selection mapping. (`kode-core`, `dioxus-nox-markdown` and `text-document` use them because they render themselves.)
- **cosmic-text** (0.19.0) and **parley** (0.12.0): text shaping, line breaking and bidi for renderers that draw their own glyphs (wgpu, canvas, Vello, Blitz). A DOM editor delegates all of that to the browser, including bidi and IME. They would only matter for a canvas or GPU editor, which is inaccessible by default (no native caret, selection handles, IME, spellcheck, screen reader tree) and so unusable for an accessible web component.

## 6. Native and desktop editors (ruled out)

- **Zed / gpui** (gpui 0.2.2, 2025-10): GPU-rendered, not for the web or DOM. Out.
- **Lapce / floem** (floem 0.2.0, 2024-11): GPU/wgpu UI, own rendering. Out.
- **xi-editor**: crates.io shows `xi-rope` 0.3.0 from 2019; the project is dormant (archived status from memory, UNVERIFIED). Out.
- **Helix**: terminal modal editor; `helix-core` on crates.io is a 0.0.0 placeholder from 2021. Out.
- Dioxus Native / Blitz (`dioxus-native`, `blitz-dom`, alpha): renders to wgpu, not the browser. `leaf-dioxus` and `valin` (Freya) live here. Out for a web component.

## 7. The JS route, for comparison (measured today)

Bundle sizes are `esbuild 0.28.2 --bundle --minify`, `NODE_ENV=production`, gzip -9, IIFE; they include the editor engine plus the extensions named, no toolbar, no Markdown serialiser. Treat them as a ballpark; real sizes depend on which features and the serialisers.

| Stack | Version (npm) | Licence | Raw | gzip | Included in the measurement |
|---|---|---|---|---|---|
| ProseMirror, minimal | prosemirror-view 1.42.6 etc. | MIT | 213 kB | **66 kB** | state, view, model, transform, schema-basic, schema-list, history, keymap, commands, inputrules, DOM parser/serializer |
| Tiptap core + selected extensions | @tiptap/core 3.32.0 | MIT | 376 kB | **119 kB** | doc, paragraph, text, bold, italic, link (pulls linkifyjs), heading, bullet/ordered list, blockquote, code, undo-redo, placeholder |
| Tiptap StarterKit | 3.32.0 | MIT | 387 kB | **122 kB** | the full starter kit |
| Lexical + rich-text, history, list, link | 0.52.0 | MIT | 372 kB | **122 kB** (lexical core alone: 69 kB) | no markdown or HTML modules (adding both: 137 kB) |
| For scale: our current `primitives/src/js/` | n/a | n/a | 1.5 kB | n/a | only `focus-trap.js` |

Licences: ProseMirror, Tiptap core and extensions, Lexical, Plate/platejs, Slate and Milkdown are MIT. Quill 2.0.3 is BSD-3-Clause. Tiptap sells paid "Pro" and Cloud extensions (comments, AI, collaboration cloud); its docs do not state the licence of the React UI components and templates (UNVERIFIED), so use the MIT core and write our own toolbar. Because `dx components add` copies the component into the user's app, the bundle and its licence notices ship in every consumer that installs the component, and it is a third-party dependency those users must trust (row 217's dependency-set policy). The existing precedent: `primitives/src/js/bundle.manifest` and `check-js-bundle.sh` assume small first-party TS; an npm-dependency bundle needs a build recipe (bun/esbuild with a lockfile) and a manifest change.

How the JS route would integrate: `document::eval` already carries our TS bundles; `dioxus_codemirror` is a public precedent (one long-lived eval channel, vendored assets, no CDN). Rust owns the toolbar (Toggle, ToggleGroup, Toolbar, Popover for the link dialog), the form wiring and the value; JS owns the engine and sends back `{json|html|markdown, activeMarks, canUndo}` on change. The SSG renders the initial value through the existing safe `markdown` component or a sanitised HTML string, then swaps in the engine at hydration.

shadcn's own ecosystem: the official `ui.shadcn.com` component list (fetched today) has **no** editor component. Community registries all wrap a JS engine: **Plate** (udecode/plate, platejs 53.3.18, MIT, React-only, built on Slate, installable with the shadcn CLI), **shadcn-editor** (htmujahid, Lexical-based, `npx shadcn add @shadcn-editor/editor-x`), and **minimal-tiptap** (shadcn-minimal-tiptap registry, Tiptap-based). Novel (Tiptap-based) is also common (not verified in search). Nobody in the shadcn lineage builds the engine themselves.

## 8. Realistic paths for our component

Required scope: toolbar, marks, headings, lists, links, undo, placeholder, a controlled value as JSON / HTML / Markdown, form participation, RTL, a11y.

### Path 1. JS ProseMirror or Tiptap-core behind `document::eval` (row 337 option B)
- Effort: **M-L** (about 4-6 lane-days). Pieces: bundle build and gate change (M-part), eval bridge for commands and active state, toolbar from our components, value in/out for the three formats (Markdown needs a serialiser such as `prosemirror-markdown`, size not measured; HTML via DOM serializer; JSON native), hidden input for forms, `dir` pass-through, `role=textbox` / `aria-label` / `aria-multiline` set on the view, SSG fallback, specs.
- Pros: best IME / Android / Safari behaviour (a decade of fixes), paste sanitising by schema, selection API, ecosystem, MIT. Same model other shadcn editors use.
- Cons: 66-120 kB gzip JS shipped to every installer, new dependency class for the registry, bundle gate needs a new model, state lives in two worlds.

### Path 2. A Rust engine behind our own thin wrapper (dioxus-editor 1.x or taino-edit core+dom)
- Effort: **M-L** for a wrapper that vendors or depends on the engine (about 5-8 lane-days, because the spike must also exercise Android/iOS/IME/RTL, and we likely fix issues upstream or locally): own `EditorView` wrapper for aria/`dir`/placeholder/form input, toolbar, value formats (taino has all three; dioxus-editor has Markdown only), SSG pre-render (free in dioxus-editor; `doc_view_html` for taino), specs.
- Pros: no JS dependency, one language, state in Rust, SSG friendly (dioxus-editor), fits `dx components add` as source (dioxus-editor can be vendored, 9k LOC dual-licensed).
- Cons: pre-1.0 engines with one maintainer each; the mobile / IME story is unproven; dioxus-editor needs Dioxus >=0.7.10 and is git-only; taino's adapter exposes no a11y or `dir`; risk of becoming the effective maintainer.
- Gate before committing: 1-day spike (S) that runs both through the Playwright suite plus real-device checks for Gboard, iOS Safari dictation and autocorrect, and a Japanese IME, in an RTL page.

### Path 3. Option A (markdown textarea + toolbar + preview) or its A+ variant (block-swap, riparion pattern)
- Effort: **S-M** (1-3 lane-days). Native textarea gives IME, mobile, form participation, RTL, spellcheck and a11y for free; safe by construction through the existing `markdown` component; output is Markdown only. A+ (rendered blocks, active block is a textarea, as in `riparion-editor`) adds a WYSIWYG feel for about +2-3 days and keeps native input.
- Not true WYSIWYG: bold appears as `**` while typing.

### Not recommended
- Option C or a Rust model plus our own DOM view: **XL**. The hard parts (IME, Android keyboards, selection mapping, paste, undo grouping) are exactly what both Rust engines are still learning.
- A CRDT (Loro, yrs, Automerge) as the model: adds a wasm dependency and does not solve the view; only when collaboration is a requirement (**+L**).
- Canvas / GPU (cosmic-text, parley, gpui, floem, Blitz): inaccessible in the browser; out.

## 9. Recommendation

1. Ship option A now, shaped as the final API: `Editor { value, on_change, format, placeholder, disabled, name, dir }` where the engine is internal. This meets the form, RTL and a11y requirements on day one and keeps the engine choice reversible.
2. If WYSIWYG is a hard requirement, choose Path 1 (ProseMirror, optionally via Tiptap core, MIT) as the default engine, accepted explicitly as a dependency-policy decision under row 217. It is the only route with proven mobile / IME behaviour today.
3. In parallel, run the one-day (S) spike on `dioxus-editor` 1.x and `taino-edit` to see whether a Rust engine can pass the same Playwright and real-device checks. If one does, Path 2 becomes the preferred way to avoid JS, and the wrapper API from step 1 makes the swap local. Re-check both repos in 2-3 months: a Rust engine with sustained commits, a crates.io release (dioxus-editor), and demonstrated Android/IME handling would change the answer.
4. Do not build a CRDT-based or canvas-based editor for this brief.

## 10. What I could not verify

- Browser behaviour of any candidate: nothing was run. All input-model statements come from source and README reading. Android Gboard, iOS Safari and non-Latin IME behaviour for dioxus-editor and taino-edit are UNVERIFIED.
- GitHub REST API (stars history, last commit dates, issue activity) was blocked for repos outside this session. Dates use `git log` of shallow clones (dioxus-editor, riparion-editor) or crates.io release timestamps (taino-edit, kode, leaf-core). Star counts were read from repo pages and may lag.
- Whether `dx components add` and our `check-cargo-dep-versions.sh` accept a git `cargoDependencies` entry (dioxus-editor is not on crates.io). I assumed vendoring.
- Whether Dioxus 0.7.10 libraries work with the `dx` 0.7.9 CLI pinned in this repo.
- Wasm sizes of the Rust engines (not built; building them needs a cold dioxus + wasm build).
- Whether dioxus-editor exposes a serialisable document (JSON) and its exact link/list/table coverage beyond what `view.rs`, `format.rs` and the README show.
- Whether `@diaryx/leaf` is on npm (README contradicts itself); leaf's web renderer was only checked through its README.
- Tiptap's licence for its React UI components and templates; Novel's status.
- xi-editor archive status and the Helix / Lapce / Zed notes rely on crates.io metadata and background knowledge.
- Bundle sizes are for a particular extension selection with esbuild; the Markdown serialisers and a real toolbar bridge are not included.
