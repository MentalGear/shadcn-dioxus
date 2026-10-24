---
id: 414
title: "Spike: formal proofs of one headless model (Kani, optionally Aeneas to Lean)"
status: "OPTIONAL: needs owner approval before it is adopted or scheduled (owner 2026-10-24). Not part of…"
size: "M"
area: research
updated: 2026-10-24
---

# Row 414: Spike: formal proofs of one headless model (Kani, optionally Aeneas to Lean)

## Item

Show whether machine-checked proofs pay off here. Write Kani harnesses for one pure module (the DataTable model or selection state). Optionally, translate the same module to Lean with Aeneas (via Charon), which is the realistic way to use Lean without a rewrite.

## Status / trigger

OPTIONAL: needs owner approval before it is adopted or scheduled (owner 2026-10-24). Not part of the adopted order. A lane must not pick it up until the owner says go. Run after row B, because the property tests produce the specifications.

## Size

M

## Detail

- Kani is a bounded model checker, and it is used in the Rust Foundation and AWS `verify-rust-std` effort. It runs on the host target, so the module must build without `web` and `dioxus` signals, the same split as row B.
- Aeneas handles a subset of safe Rust, and its Lean backend is the most mature. Microsoft uses it for SymCrypt. Expect manual proof work in Lean.
- No tool verifies `rsx!` components, signals, or wasm-bindgen and web-sys calls. Keep the spike to the pure crate.
- **Deliverable:** a note giving the properties proved, the effort spent, the bugs found (if any) and a keep-or-drop recommendation. Nothing goes into the gates until the owner decides.

Source: `/mnt/project-files/backlog-drafts/2026-10-24-optional-safety-rows.md` (draft C), background in `/mnt/project-files/goose-eval/2026-10-24-goose-vs-wasm.md`.
