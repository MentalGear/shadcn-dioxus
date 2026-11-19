---
id: 532
title: "Output budgets where none exist: CSS bytes per page and hydration time; an optional step that stops embedding demo source"
status: "BACKLOG, filed 2026-11-19. Next batch; waits for the owner's go. Not started. The hydration budget…"
size: "M: two budgets and their gates; the…"
area: build
updated: 2026-11-19
---

# Row 532: Output budgets for CSS and hydration, and the demo-source step

## Item

No gate budgets the built output. The only budgets are the kernel gzip (12,914 B), `bridge.ts` (61,792 B), JS volume
(487,932) and the slow-spec total; nothing budgets CSS size, hydration time or memory, and the shipped wasm is not
budgeted either. Measured on `0df1782`: wasm 20,635,690 B raw, 5,734,749 B at `gzip -9`; CSS 453,264 B in 5 sheets on
nearly every page, 88.1 to 92.2 % unused at load on component pages; hydration (navigation start to `data-hydrated`)
1,078 to 1,574 ms deployed on component pages. Add, in the existing budget-file idiom (a `scripts/*.budgets.tsv` read by a
ratchet gate), a budget for CSS bytes per page and one for hydration time on a fixed page set.

The wasm-size budget is not built here: [row 371](./371-dependency-evaluation-keep-lift-drop.md) lists "wasm-size
budgets" among what its construction half still builds. This row supplies the measured baseline above to that row.

The optional second step: stop embedding the demo source text in the wasm. 649 Rust files (2,049,994 B) and 85 CSS files
(763,439 B) sit verbatim in the data section through `code!` and `css_highlight!` (`preview/src/components/mod.rs`).
Zeroing the embedded source lowers gzip by 663,325 B (about 0.66 MB, 11.6 %) and brotli by 386,706 B. The owner decides
whether the saving is worth a runtime fetch of the source on the demo pages.

## Status / trigger

BACKLOG, filed 2026-11-19. Next batch; waits for the owner's go. Not started. The hydration budget needs a stable timing method on a release build (the
`@timing` cases are deactivated, [row 494](./494-re-enable-the-timing-cases-after-the-rust-port.md)); until it exists the
CSS budget goes first.

## Size

M: two budgets and their gates; the demo-source step is a separate M if the owner wants it.

## Detail

- **Files:** a new `scripts/check-output-budgets.sh` (header, entry in `run-gates.sh`, `gates-doc.py --write`), its
  `*.budgets.tsv`, and `scripts/lib/ratchet.py` for the shrink-only rule and grants; the hydration page set is the six
  pages measured (home, button, dialog, combobox, sidebar, data_table).
- **Proof:** each budget starts at the measured value, so the gate is green today; a fixture that grows the stylesheet by
  one byte is red; the hydration gate runs on a release build only and says so in its header.
- **Residue:** hydration time varies with the machine (home 2,270 and 3,136 ms in two runs); the budget is a ceiling with
  headroom, not a target.
- Source: [`research/repo-evaluation-2026-11-19.md`](../research/repo-evaluation-2026-11-19.md), section 4.
