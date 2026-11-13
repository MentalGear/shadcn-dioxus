---
id: 445
title: "Phase 0.2: freeze the legacy scripts and sinks (script hashes, per-site sink freeze, js-fix grant)"
status: "DONE in PR #175 (2026-11-12): check-js-volume freezes every legacy script by content hash and every…"
size: "S"
area: tooling
js-grant: none
adds: []
updated: 2026-11-12
---

# Row 445: Phase 0.2: freeze the legacy scripts and sinks (script hashes, per-site sink freeze, js-fix grant)

## Item

The legacy freeze (root doc section 5.3, item 12): a hash per legacy script, a per-site freeze of every remaining sink argument (delete only, never re-point), and a `js-fix:` grant for a legitimate bug fix, so that nothing legacy grows or is copied while the ports wait for their rows. Why: under scope 2 the legacy files stay an open sink until ported; the freeze is what keeps that cost from compounding.

Part of the [row 434](../434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

**DONE in PR #175 (2026-11-12):** `check-js-volume` freezes every legacy script by content hash and every sink site by the hash of what it runs; a change is red except deletion or a `js-fix:` grant. Constructed from the root doc section 5.3 item 12 over the tree as row 444 left it; APPROVED 2026-10-26 (owner: "okay for the plan"; scope 2).

Phase 0 of the row 434 step 3 plan; it started once row 444 landed (it shares `check-js-volume.*`).

### As built

- **What is frozen.** "Legacy" is everything `check-js-volume` counts outside `primitives/src/bridge/` (the typed seam, held by `check-bridge-behaviours`) and the generated bundle `primitives/src/js/` (pinned by `check-js-bundle`). The gate lists it from the tree on every run (the same scan as the counts), so a legacy script or sink that exists and is not in the data is red; there is no hand-kept list. Today: 126 scripts (Rust string groups that read as JS, the hand-written `primitives/src/message_scroller.js`, the two inline `<script>` bodies of `preview/index.html`) and 100 sink sites (every `sink:*` kind, including the five owner-approved allow rows: the allow rows excuse counts, never a freeze entry).
- **Data format.** `scripts/check-js-volume.freeze.tsv`, one line per entry: `class<TAB>path<TAB>kind<TAB>sha256-16<TAB>count<TAB>size<TAB>preview[<TAB># fixed: row N <reason>]`. `script` entries (kind `lit`, `file`, `inline`) are keyed by the hash of the exact text, so any edit is a new, unfrozen script. `site` entries (kind = the sink kind) are keyed by the hash of what the site runs: the whole bracketed argument of a call or block (`eval(<script>)`, `Script { .. }`), the `use ... eval;` statement, the string literal of a string-borne sink, or the source line of a bare token such as `web_sys::window()`; whitespace and rustfmt's bracket and trailing-comma spacing are normalised. The identity is never a line number, so an unrelated edit above a site keeps it; re-pointing `eval(A)` to `eval(B)`, copying a script into another file or adding a site gives a key the data lacks. Two equal sites are `count 2`. The `size` and `preview` columns are for the reader.
- **Comparisons.** (1) the tree's scan against the tree's data: a key the data lacks or a higher count is red, a frozen key gone from the tree is red until lowered; (2) the tree's data against the base's data (`git show <base>:scripts/check-js-volume.freeze.tsv`, the ratchet's base): an entry the base lacks passes only with a `# fixed: row N <reason>` note.
- **The `js-fix:` grant.** Same mechanism as row 444's `js-grant`: the note names row N, and row N must exist AT THE BASE (read with `git show`, `scripts/lib/rows.py`), be open, and carry front matter `js-fix: <path>[, <path>]` naming the file. A grant in the lane's own diff is not at the base, so a lane cannot grant itself. A fix may change a sink site of its file but not add one, and grants no volume raise (that stays a `js-grant`). Fixes print under `FREEZE FIXES` at each run. The column form `js-fix: js_file=20` that `scripts/lib/rows.py` parses grants nothing here.
- **Reconciliation, one command.** After deleting a legacy script or site, or after merging lanes that did, run `scripts/check-js-volume.sh --update-baseline` on the tree: it rewrites the volume baseline and lowers the freeze data, and refuses to raise or re-add either (the freeze half alone is `--update-freeze`). One line per entry keeps lane merges conflict-free; a conflict in the data is resolved by taking either side and rerunning the command. A recorded fix is `scripts/check-js-volume.sh --update-freeze --js-fix "row N <why>"`, refused unless the grant at the base names the file.
- **Proof.** The gate's own `--self-test` (it rides along with the plain run) builds each case on a throwaway repository: (a) an edited frozen script literal, `.js` file and inline `<script>`, and a copied script: red, while the volume ratchet alone stays green for the same edit; (b) a re-pointed sink site: red (ratchet alone green); (c) a second call of a frozen site, a new sink kind and a new legacy file: red; (d) the data hand-edited to match without a note, with a row that is not at the base, with a grant written in the lane's own diff, for another path, from a closed row, from a `js-grant` row, and `--update-freeze` refusing each and a re-add: red or refused with the data file untouched; (e) a script and its site deleted with the data lowered, and a `.js` file deleted: green; the same without lowering: red; (f) an edit under a valid grant, a re-pointed site under a grant and a fixed `.js` file: green and listed; a grant for another file and a fix that adds a site: red; plus controls (an unchanged tree, unrelated edits and a reformatted call, the bridge and bundle not frozen, a deleted or malformed data file).
- **Limits.** A renamed or moved legacy file reads as a deletion plus a new file, so it needs a `js-fix` row naming the new path (the volume baseline carries renames; this data does not). The freeze holds the owner-approved allow rows' sites too (`eager_head.rs`, `theme.rs`, `chrome.rs`, `code.rs`), so editing them needs a `js-fix` row as well. `primitives/src/bridge/` is outside the freeze; its `Program` constants are held by `check-bridge-behaviours`. `scripts/lib/rows.py` documents `js-fix` as the path list the gate reads.

## Size

S

## Detail

- **Files owned:** `scripts/check-js-volume.*` (the gate, its baseline, allow rows and `check-js-volume.freeze.tsv`). Starts after item 0.1 merges: the two share `check-js-volume.*`.
- **Depends on:** [row 444](../444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (shares `check-js-volume.*`).
- **Proof:** The gate's mutation self-test (edit a frozen script, re-point a sink, raise without a grant: each red).
- **Hours (inferred):** about 4 (item 12 said 3; the table's 4 stands).
