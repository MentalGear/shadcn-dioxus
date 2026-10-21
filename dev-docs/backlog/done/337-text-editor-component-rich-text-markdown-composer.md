---
id: 337
title: "Text editor component (rich text / markdown composer)"
status: "CLOSED -- out of scope (owner 2026-10-21, later the same day): not built, not even option A; a…"
size: "S-M (A)"
area: components
updated: 2026-10-21
---

# Row 337: Text editor component (rich text / markdown composer)

## Item

Text editor component (rich text / markdown composer).

## Status / trigger

**CLOSED -- out of scope (owner 2026-10-21, later the same day): not built, not even option A; a Markdown or rich-text editor is a bigger stateful item for a higher-level framework or an external library. See "Out of scope" below.**

## Size

S-M (A)

## Detail

Owner 2026-10-21: "a text composer in the sense of text editor". Options, sized:

- **A. Markdown editor (S-M, about one lane-day).** A textarea with a toolbar (bold, italic, link, list, code) that inserts Markdown, plus a live preview through the existing, safe `markdown` component. No new dependency, safe by construction (no raw HTML reaches the DOM), SSG-friendly. Not true WYSIWYG.
- **B. Wrap Tiptap's framework-agnostic core, which runs on ProseMirror (M-L, about 3-5 lane-days).** True WYSIWYG, with undo, paste handling, IME and a11y from a mature engine; toolbar, bubble menu and popovers built from our components, driven through the eval bridge. This is the shadcn-ecosystem choice (minimal-tiptap; Plate is React-only). It adds a third-party JS dependency that every `dx components add` user ships and trusts, so it is a dependency-policy decision (row 217) and needs a change to the `check-js-bundle.sh` model. ~100-200 KB. Pasted HTML must be held to a strict schema or sanitizer.
- **C. Our own contenteditable editor (XL, weeks).** Selection model, document schema, undo, IME, paste sanitisation and a11y from scratch. Not recommended.

Recommendation: A first, B only if WYSIWYG is wanted and the dependency is accepted. Not started; awaiting the owner's choice. (The earlier reading, a chat prompt input like ai-elements' PromptInput, is about one lane-day, size M, and stays in row 265's list.)

## Out of scope (owner 2026-10-21, supersedes the decision below)

Tom: "update backlog to not build the md editor as we want to keep mostly parity, and its a bigger item that also needs more complex state managment which another higher up framework or external should do as a general rule - we try to keep primitives or shadcn elelemt parity even if a bit more complex like chat". So no option is built. The scope rule this sets is in the [backlog index header](../../backlog.md#scope). The research below stays as a reference for an app or framework that wants an editor on top of the library.

## Owner decision 2026-10-21 (superseded)

**Build option A when it becomes relevant** (no lane until then): a Markdown textarea with a toolbar and a live preview through the existing `markdown` component. Its syntax coverage is exactly what that renderer shows -- CommonMark plus `ENABLE_TABLES`, `ENABLE_STRIKETHROUGH` and `ENABLE_TASKLISTS`, the allowlist `scripts/check-markdown-parser.sh` pins (bold, italic, strikethrough, inline code, links through `SafeHref`, headings, bullet/numbered/task lists, blockquotes, code blocks, rules, tables; no raw HTML, footnotes, heading attributes or metadata) -- so the toolbar can never write syntax the preview will not render. Keep the API engine-agnostic (`value`, `on_change`, `format`, `placeholder`, `name`, `dir`) so a WYSIWYG engine can be swapped in later without breaking callers. Native textarea: IME, mobile input, form participation, RTL and a11y come for free. Open design points for the build: preview on every keystroke (debounced) or on demand; toolbar wraps the selection or inserts at the caret.

## Rust-library research (option B with a Rust engine), 2026-10-21

Full notes: [`text-editor-options-2026-10-21.md`](../../research/text-editor-options-2026-10-21.md). Two young Dioxus WYSIWYG editors exist, both `contenteditable`, MIT/Apache, one maintainer each: **dioxus-editor** (best input model -- intercepts `beforeinput`, has `role=textbox`/aria/placeholder; unpublished, git-only, needs Dioxus >= 0.7.10, ~9k LOC, would likely be vendored) and **taino-edit** (+ `taino-edit-dioxus`; widest features and on crates.io, but reads the DOM after `input` and its adapter has no aria/`dir`/placeholder). Neither is production-grade next to ProseMirror, Lexical or Tiptap; everything else in Rust is a code editor, a Markdown textarea, a GPU/native editor, a wrapper around a JS editor (the Rust ecosystem's usual answer, e.g. maud-ui wraps Tiptap) or a data structure (Loro, yrs and Automerge carry rich-text marks but ship no DOM view). JS engines measured with esbuild, gzip: ProseMirror minimal 66 kB, Tiptap core about 120 kB, Lexical about 122 kB, all MIT -- every `dx components add` user would ship it (a row 217 dependency decision). If WYSIWYG is ever wanted: either the JS engine through the eval bridge as an explicit dependency decision, or first a one-day spike of the two Rust engines in Playwright and on real devices (Gboard, iOS Safari, a Japanese IME, an RTL page), re-checking both repos in 2-3 months. Nothing was run in a browser; input-model verdicts are from reading source.
