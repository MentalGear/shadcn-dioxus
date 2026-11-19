---
id: 529
title: "The CLI validates component names and never deletes outside the components root"
status: "BACKLOG, filed 2026-11-19. Next batch, first of the new rows (a destructive bug); waits for the…"
size: "S: one type, its three entry points and…"
area: tooling
updated: 2026-11-19
---

# Row 529: The CLI validates component names and never deletes outside the components root

## Item

`shadcn-dioxus remove` joins the typed name onto the components root and calls `remove_dir_all` (`cli/src/add.rs:517-523`);
`add --force` does the same with the registry's `component.name` (`add.rs:216`); `cli/src/manifest.rs` types
`Component.name` as a bare `String`. `remove -- ..` empties `src/`, and `remove ../../x`, an absolute path or a hostile
registry name deletes that directory. Build a `ComponentName` type that is accepted only as `[a-z0-9_]+` at every entry
(argv, the registry manifest's `name`, the lock file's keys), and a containment check before any `remove_dir_all`: the
resolved target must be a strict child of the components root.

## Status / trigger

BACKLOG, filed 2026-11-19. Next batch, first of the new rows (a destructive bug); waits for the owner's go. Not started.

## Size

S: one type, its three entry points and the guard in `remove` and the `--force` overwrite.

## Detail

- **Files:** `cli/src/add.rs` (`remove`, the overwrite branch at `:216`), `cli/src/manifest.rs` (`Component.name`),
  `cli/src/lock.rs` (the component keys); a new small module for the type is the lane's choice.
- **Reproduction (2026-11-19, built CLI, scratch projects, every run exits 0):** `shadcn-dioxus remove -- ..` empties
  `src/`; `remove ../../victim` and `remove <absolute path>` delete the sentinel directories; `add --path <registry>
  --force --skip-cargo-add ../..` against a local registry whose `component.json` name is `../../victim` deletes
  `victim/SENTINEL` and installs into `victim/`. All 84 `component.json` names of this registry already match
  `[a-z0-9_]+`.
- **Proof:** unit tests for `..`, `../../x`, an absolute path and a hostile registry name (each an error, nothing
  deleted); the reproduction commands above fail with an error and delete nothing; the install smoke
  (`scripts/registry-install-smoke.sh`) still passes for all 83 components.
- **Residue:** the evaluation's code-quality report also names `exclude`, `members` and `components_dir`
  (`cli/src/project.rs:76-78`) as unvalidated; that claim was not re-checked, so the lane reads them and the owner decides
  whether they belong here.
- Source: [`research/repo-evaluation-2026-11-19.md`](../research/repo-evaluation-2026-11-19.md), section 4.
