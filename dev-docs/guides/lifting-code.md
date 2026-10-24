# Lifting code from other projects: licensing and provenance

The rules for moving code or ideas from a fork of `DioxusLabs/dioxus-components` (or any other project) into this repo
without importing someone's architecture or losing attribution. The full playbook this was cut from (fetching fork
branches, the three lift shapes, name mapping, the queue, the provenance ledger) is archived in
[`../archive/fork-research/lifting-from-forks.md`](../archive/fork-research/lifting-from-forks.md).

## Legal position

Upstream and the forks mined so far (`dignifiedquire`, `sarendipitee`, `jcgruenhage`) ship `LICENSE-APACHE` and
`LICENSE-MIT`, the same terms as this repo (`MIT OR Apache-2.0`). Lifting is permitted; both licences require that
attribution survives. Re-verify the licence before every lift: a fork can relicense at any time.

## Provenance, by lift shape

- **Cherry-picked commit:** `git cherry-pick` keeps the original author, and `git commit -C <sha>` does the same for a
  partial apply. Never rewrite authorship.
- **Lifted file:** carries a provenance header, since a copied file otherwise loses all trace of its author:

  ```rust
  //! Ported from dignifiedquire/dx-components (MIT OR Apache-2.0),
  //! primitives/src/scroll_lock.rs @ <sha>. Adapted: <what you changed>.
  ```

- **Concept port** (you read the approach and wrote your own): no header needed; credit the source in the commit
  message, and in the ported module's doc comment when it is an algorithm port.

The pinned revision a file was ported from (`aria-practices 7e4034b`, `shadcn-ui/ui@295a1f1`) is provenance and stays in
comments (`dev-docs/history-in-comments.md`).

## Other licences

- **ISC** (`d3-shape`, `d3-array`): the same permissive family. An algorithm port (JS to Rust is necessarily a rewrite)
  needs no header, but cite the source file, SHA and licence in the commit message and the module's doc comment, as
  `primitives/src/chart/engine/scale.rs` and `curve.rs` do. Re-verify the specific licence before treating any other
  ISC/MIT/Apache/BSD source the same way: this is a category, not a blanket clearance.
- **File-level copyleft (MPL-2.0, e.g. `leptos-chartistry`):** never copy, even with attribution. Copying an MPL file's
  content into a crate declared `MIT OR Apache-2.0` would put that content under MPL obligations. Read for the
  algorithm or design shape, re-derive independently against the canonical source, cite the source in the commit
  message. Re-derivation is mandatory here, not just cheaper than lifting (the sibling library's own transcription of
  the Steffen monotone-cubic paper silently dropped a clamp term the canonical source has:
  [`archive/fork-research/chart-forks-2026-09-19.md`](../archive/fork-research/chart-forks-2026-09-19.md) §4.1).
- **Read-never-vendor references:** a project that is only read for shape (`dignifiedquire/dx-components`
  `primitives/src/command.rs`, entangled with that fork's own popper and focus scope) is not a candidate for lifting at
  all.

## Safety checks before any lift

No vendored build hacks, no dependency the fork added only for itself (`primitives/src/top_layer.rs` takes no
dependency for what a short module does), and a regression test that fails on the old behaviour, per the definition of
done in [`../plan.md`](../plan.md).
