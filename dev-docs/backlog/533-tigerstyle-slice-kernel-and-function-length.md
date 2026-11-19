---
id: 533
title: "The TigerStyle slice: kernel lints and assertions, and a shrink-only count of long non-component functions"
status: "BACKLOG, filed 2026-11-19; the owner agreed to the slice. Next batch for its kernel items; waits…"
size: "M: kernel items 1-4 are S; item 5 is S…"
area: tooling
updated: 2026-11-19
---

# Row 533: The TigerStyle slice

## Item

Adopt the narrow slice of TigerStyle that fits this repository (the rest, and why, is section 3 of the
[evaluation](../research/repo-evaluation-2026-11-19.md)).

**Kernel (`decision-kernel`):**
1. `too-many-lines-threshold = 70` in `decision-kernel/clippy.toml` plus `#![deny(clippy::too_many_lines)]` (0 violations:
   the longest function is 38 lines).
2. Deny `clippy::unwrap_used`, `expect_used` and `panic`. Today 2 hits under `--all-targets`, `decision.rs:111` (`unwrap`)
   and `:129` (`panic!`), both in tests: the kernel's `clippy.toml` has no allow-in-tests switches (the root one sets only
   `allow-unwrap-in-tests`, and is not read there); add `allow-unwrap-in-tests`, `allow-expect-in-tests` and
   `allow-panic-in-tests` to it. Non-test code has 0.
3. `debug_assert!` pre- and postconditions on the kernel's 20 public functions, the five `extern "C"` exports
   (`dx_abi`, `dx_kinds`, `dx_input`, `dx_output`, `dx_decide`) included. `[profile.kernel]` sets `debug-assertions = false`
   and `panic = "abort"`, so they cost nothing in the committed module.
4. `const _: () = assert!(..)` over the kernel's constants (ABI, `NONE`, buffer sizes); today the tree has one const
   assert, `primitives/src/lib.rs:27`, which checks a cfg.
5. A named cap on `dx_input`'s length (`ffi.rs:45-48` resizes to the caller's `u32`), returning the existing none value
   above it. **This changes the committed kernel wasm bytes** (`scripts/check-decision-kernel.sh` rebuilds and compares), so
   it needs the owner's ratchet grant and the budget update. `kernel.ts` already catches the trap an oversize length causes
   and stands the kind down after three, so the cap closes a trap, not a page failure.

**Function length outside components:** a gate that holds the NUMBER of non-component functions over 70 lines per crate
and lets it only shrink (primitives 17, cli 5 today; kernel 0), through `scripts/lib/ratchet.py`. A function is a
component when it carries `#[component]` or returns `Element`; test code is out. Clippy's `too_many_lines` cannot skip
components (87 of 364 in primitives exceed 70), hence a script.

## Status / trigger

BACKLOG, filed 2026-11-19; the owner agreed to the slice. Next batch for its kernel items; waits for the owner's go. Not started. Items 1-4 need no grant; item 5 waits for it.

**Taken over (2026-11-19).** The gate half, the shrink-only count of long non-component functions, is built by
[row 542](./542-one-generated-item-table-read-by-the-gates.md) as the first consumer of its generated item table. This row
keeps the kernel lints, the assertions, the const assertions and the `dx_input` cap (items 1-5).

## Size

M: kernel items 1-4 are S; item 5 is S plus the grant and a module rebuild; the gate is S-M and is built in row 542.

## Detail

- **Why a count, not a named baseline.** The JavaScript-to-Rust port ([row 434](./434-javascript-in-typescript-modules-not-rust-strings.md);
  row 444 is its gate half and names none of these functions) rewrites, moves and deletes long hooks that wrap page
  scripts. A named per-function baseline needs a grant at every move; a count is indifferent to moves, shrinks as the port
  deletes functions, and still stops ported code from adding a new long one. Of the 17 primitives functions, six are such
  hooks by name (read from names and files, checked against the rows 2026-11-19): `use_dialog_open_driver` and
  `use_animated_open` (`lib.rs`) are named by open [row 456](./456-port-3-ov-overlay-core-escape-dismiss-open-focus.md) (and
  `use_dialog_open_driver` by rows 515 and 523); `use_scroll_lock` (`scroll_lock.rs`) is in `js-interop.md`'s list of hooks
  that call scripts and its script belongs to [row 459](./459-port-3-sl-scroll-lock-l10-l11-l13.md). The other three,
  `use_activity` (`activity.rs`), `bring_into_view` (`keyboard_inset.rs`) and `use_scroll_fade_runtime` (`scroll_area.rs`),
  wrap scripts that rows 451 and 454 already ported; no open row names them, so they are long post-port code and stay in
  the count unless something shortens them. The count form holds for either group.
- **Files:** `decision-kernel/clippy.toml`, `decision-kernel/src/lib.rs`, `ffi.rs`, `decision.rs`, `kinds.rs`; a new
  `scripts/check-fn-length.sh` (header, entry in `run-gates.sh`, `gates-doc.py --write`) with its baseline file.
- **Proof:** the kernel's clippy arm is green with the denies; the new gate is green at 17 and 5 and red on a fixture that
  adds a long non-component function; item 5 has a unit test that a request above the cap returns none and
  `check-decision-kernel.sh` passes on the rebuilt module.
- **Residue:** one long function can be traded for another at the same count (the diff shows it in review); components are
  out of scope (87 of 364 in primitives, 83 in preview exceed 70); loop bounds in primitives (13 `loop`, 21 `while`) and
  assertion density outside the kernel stay unchecked.
- Source: [`research/repo-evaluation-2026-11-19.md`](../research/repo-evaluation-2026-11-19.md), section 3.
- **Cargo.toml comment (added 2026-11-19):** `Cargo.toml:23` says the one `unsafe_code` exception is
  `preview/src/wasm_libc_shim.rs`; there are two: that file and `decision-kernel/src/ffi.rs:14-17` (`#![allow(unsafe_code, ..)]`
  over the five `extern "C"` exports, with raw pointers at `:45` and `:56`). Fix the comment with item 1 (ledger D-19).
