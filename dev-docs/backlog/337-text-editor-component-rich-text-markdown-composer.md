---
id: 337
title: "Text editor component (rich text / markdown composer)"
status: "BACKLOG (owner 2026-10-21)"
size: "S-M (A) / M-L (B)"
area: components
updated: 2026-10-21
---

# Row 337: Text editor component (rich text / markdown composer)

## Item

Text editor component (rich text / markdown composer).

## Status / trigger

**BACKLOG (owner 2026-10-21)**

## Size

S-M (A) / M-L (B)

## Detail

Owner 2026-10-21: "a text composer in the sense of text editor". Options, sized:

- **A. Markdown editor (S-M, about one lane-day).** A textarea with a toolbar (bold, italic, link, list, code) that inserts Markdown, plus a live preview through the existing, safe `markdown` component. No new dependency, safe by construction (no raw HTML reaches the DOM), SSG-friendly. Not true WYSIWYG.
- **B. Wrap Tiptap's framework-agnostic core, which runs on ProseMirror (M-L, about 3-5 lane-days).** True WYSIWYG, with undo, paste handling, IME and a11y from a mature engine; toolbar, bubble menu and popovers built from our components, driven through the eval bridge. This is the shadcn-ecosystem choice (minimal-tiptap; Plate is React-only). It adds a third-party JS dependency that every `dx components add` user ships and trusts, so it is a dependency-policy decision (row 217) and needs a change to the `check-js-bundle.sh` model. ~100-200 KB. Pasted HTML must be held to a strict schema or sanitizer.
- **C. Our own contenteditable editor (XL, weeks).** Selection model, document schema, undo, IME, paste sanitisation and a11y from scratch. Not recommended.

Recommendation: A first, B only if WYSIWYG is wanted and the dependency is accepted. Not started; awaiting the owner's choice. (The earlier reading, a chat prompt input like ai-elements' PromptInput, is about one lane-day, size M, and stays in row 265's list.)
