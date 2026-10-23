---
id: 130
title: "arborium 2.17.0's wasm libc shim lacks the data symbol stderr that tree-sitter's stack.c…"
status: "CLOSED 2026-10-23 (owner): local fix 7ad8421 stays; not filed upstream (no-upstream rule). When a…"
size: "S"
area: upstream
updated: 2026-10-23
---

# Row 130: arborium 2.17.0's wasm libc shim lacks the data symbol stderr that tree-sitter's stack.c…

## Item

arborium 2.17.0's wasm libc shim lacks the data symbol `stderr` that tree-sitter's `stack.c` (`ts_stack_print_dot_graph`) reads

## Status / trigger

**CLOSED 2026-10-23 (owner): local fix 7ad8421 stays; not filed upstream (no-upstream rule). When a fixed arborium lands, the duplicate-symbol link error says to delete the local stderr definition.** **NARROWED 2026-10-19 (backlog review): local fix 7ad8421 in; filing upstream is the owner's decision (row 6)**

## Size

S

## Detail

Root cause and the rejected alternatives (linking `arborium-sysroot` duplicates every other shim symbol; `--allow-undefined` hides real link errors and breaks the x86 server build under `--ssg`) are in row 116's 2026-10-04 addendum. Every debug wasm build of `preview` failed to link on rustc 1.97 before the fix; release links only because lld dead-strips the unreachable function. **To do:** file against arborium with the isolated repro crate (debug wasm link of a crate depending on `arborium_tree_sitter`, no `--allow-undefined`); remove the local definition when a fixed arborium lands (the definition would then duplicate -- the link error says so loudly). Related, also upstream: why `dioxus-code`'s `lang-css` puts the tree-sitter runtime in the browser client at all (only the debug Style tab needs it).
