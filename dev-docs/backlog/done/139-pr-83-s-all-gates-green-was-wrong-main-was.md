---
id: 139
title: "PR #83's \"all gates green\" was wrong: main was clippy-red from d9bcfda (primitives/src/navbar.rs:655…"
status: "FIXED 2026-10-04, merged (a17983c, PR #84 / 001a5f0: unwrap_or_else(&*is_open), one line, no…"
size: "S"
area: menus
updated: 2026-10-04
---

# Row 139: PR #83's "all gates green" was wrong: main was clippy-red from d9bcfda (primitives/src/navbar.rs:655…

## Item

PR #83's "all gates green" was wrong: `main` was clippy-red from `d9bcfda` (`primitives/src/navbar.rs:655`, `clippy::redundant_closure`)

## Status / trigger

**FIXED 2026-10-04, merged** (`a17983c`, PR #84 / `001a5f0`: `unwrap_or_else(&*is_open)`, one line, no behaviour change; was uncommitted at time of writing)

## Size

S

## Detail

**Correction to the record.** The 2026-10-04 addenda of rows 111 and 116 and PR #83 (merge `44a103e`) quote "all gates green (... clippy `-D warnings` ...)" as measured "on `20e2fa4`". `20e2fa4` is an ancestor of `d9bcfda` ("open on click so a held tap cannot light-dismiss its own menu", row 40), which added `.unwrap_or_else(\|\| is_open())` to `NavbarTrigger`'s `onclick` (`git log -S` names it as the introducing commit; line 655 sits outside every `cfg` block of the file, so the default clippy run flags it). The quoted green was therefore taken on a tree that did not contain the line that went red; the merged tip's clippy was never what was run. **Same class as row 67** (a gate red on `main` because nothing ran it on the merged tree), different cause: not a missing workflow but a claim about a tree the claimer did not run. **Construction:** `scripts/run-gates.sh` (row 116, round-2 addendum): one entry point, the summary prints HEAD plus a hash of the uncommitted diff so a green names its own tree, exit 0 only when every gate ran, an unlisted `check-*.sh` fails the run. **Honest limit:** that makes a mismatched claim visible, it cannot stop anyone quoting a green from another commit, so a "gates green" in a commit message or addendum should carry the tree line the script prints. **A separate hole found the same day, not the cause of this miss:** `cargo clippy --workspace` never enabled primitives' `web` or preview's `server` feature (104 `feature = "web"` cfg sites and the shipped SSG server code unlinted), which is why the script runs three clippy arms. The earlier addenda are left as written; this row is their correction.
