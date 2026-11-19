---
id: 530
title: "`add` installs transactionally: stage, move into place, edit Cargo.toml and mod.rs last, save the lock on every exit"
status: "BACKLOG, filed 2026-11-19. Next batch, waits for the owner's go. Starts after row 529, which…"
size: "M: the install pipeline in add.rs, a…"
area: tooling
updated: 2026-11-19
---

# Row 530: `add` installs transactionally

## Item

`add` changes the project in this order: `ensure_components_module` writes `mod.rs`, then `cargo add` edits `Cargo.toml`
(`cli/src/add.rs:397`), then for each component it copies the files, copies the global assets and edits `mod.rs`, and
`lock.save` runs last (`add.rs:447`). A failure in component N leaves `Cargo.toml` edited, the earlier components copied
and declared, component N copied and not declared, and no lock entry. With `--force`, `remove_dir_all(dest)` runs before the
copy (`add.rs:216`), so a failed copy loses the old directory and the user's edits to it.

Build the order the other way: stage every component into a temporary directory, move each into place only when all of
them staged, edit `Cargo.toml` and `mod.rs` last, and save the lock on every exit path (the error path included, for what
was moved). `--force` keeps the old directory until the new one is in place (rename the old aside, move the new in, delete
the old).

## Status / trigger

BACKLOG, filed 2026-11-19. Next batch, waits for the owner's go. Starts after [row 529](./529-cli-component-names-validated-no-delete-outside-root.md), which
changes the same functions. Not started.

## Size

M: the install pipeline in `add.rs`, a failure-injection test, the install smoke.

## Detail

- **Files:** `cli/src/add.rs` (`add`, `copy_dir`, `copy_global_assets`, the overwrite branch), `cli/src/lock.rs` (save on
  every exit), `cli/src/project.rs` if the staging path needs the crate root.
- **Proof:** a test that fails component N's asset copy (an unreadable file or a read-only target) and asserts that
  `Cargo.toml`, `mod.rs`, the lock and every pre-existing component directory are byte-identical to before; a `--force`
  test whose copy fails and leaves the old directory intact; `scripts/registry-install-smoke.sh` passes for all 83.
- **Residue:** a crash between two renames can still leave one component half-moved; the lock records only what was moved.
- Source: [`research/repo-evaluation-2026-11-19.md`](../research/repo-evaluation-2026-11-19.md), section 4.
