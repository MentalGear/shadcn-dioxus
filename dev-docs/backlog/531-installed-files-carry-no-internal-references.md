---
id: 531
title: "Installed component files carry no internal references (row numbers, dev-docs, backlog, scripts paths)"
status: "BACKLOG, filed 2026-11-19. Before the final review (row 383). Not started. Pinning…"
size: "S-M: up to a few hundred lines to…"
area: build
updated: 2026-11-19
---

# Row 531: Installed component files carry no internal references

## Item

The CLI installs each component's directory except `["variants","docs.md","component.json"]` plus its global assets:
256 files (173 `.rs`, 83 `.css`), 34,666 lines. Those files carry references that mean nothing in a user's project:
`row NNN` on 136 lines in 50 components; `dev-docs` on 85 lines in 40 components; `backlog` on 86 in 38; `scripts/` on 61
in 37; `dx_repo_lints` on 13 lines in 3 components. Move the references out of the installed file set (into the
repository's docs or the row files) and add a gate over exactly the installed file set that fails on any of them.

## Status / trigger

BACKLOG, filed 2026-11-19. Before the final review (row 383). Not started. Pinning `dioxus-primitives` in every `component.json` is not this row: all 83
`component.json` carry the git dependency without a rev because `scripts/stamp-release-rev.sh` writes it at release time
([row 217](./done/217-dependency-audit-follow-ups-what-users-pay-for.md), landed); the committed tree is the development
form.

## Size

S-M: up to a few hundred lines to reword across 50 components, one gate (header, entry in `run-gates.sh`,
`gates-doc.py --write`).

## Detail

- **Installed set:** derive it the way `cli/src/add.rs` `copy_dir` does (the member directory minus its `exclude` list,
  plus `globalAssets`); the gate does not keep its own copy of the list.
- **Patterns:** `row [0-9]+`, `dev-docs`, `backlog`, `scripts/`, `dx_repo_lints` (the lint cfg is a repository-only
  switch; the lane decides whether the allow stays as a documented user-visible attribute or moves behind the generator).
- **Proof:** the gate is green on the tree after the rewording and red on a fixture file with each pattern; the install
  smoke passes for all 83 components; the public API snapshot gate stays unchanged (comments only).
- **Rule for the rewording:** a comment that explains a behaviour keeps its explanation and drops the pointer.
- Source: [`research/repo-evaluation-2026-11-19.md`](../research/repo-evaluation-2026-11-19.md), section 4.
