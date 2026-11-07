---
id: 502
title: "Installed components carry #[expect(clippy::disallowed_methods)] that a user's project cannot fulfil"
status: "BACKLOG, found 2026-11-07. Check first: run cargo clippy in the registry-install-smoke app and…"
size: "S"
area: tooling
updated: 2026-11-07
---

# Row 502: Installed components carry repo-only lint expects

## Item

Row 444 lane D bans the page-script sink in this repo's `clippy.toml` and puts an `#[expect(clippy::disallowed_methods,
reason = ...)]` at each existing site. Some sites are in installable components (`form/component.rs`,
`code_block/component.rs`, the sites `scripts/check-installed-paths.sinks.tsv` lists). A user's project has no such
`disallowed-methods` entry, so under the user's own `cargo clippy` the expectation is unfulfilled and warns.

## Status / trigger

BACKLOG, found 2026-11-07. Check first: run `cargo clippy` in the registry-install-smoke app and count the
`unfulfilled_lint_expectations` warnings.

## Size

S

## Detail

**By construction:** an installed file states nothing about this repo's lint configuration. Either the installed sites
move their sink behind a primitive (the port rows remove them anyway: `form` and `code_block` are listed port targets),
or the expect becomes `#[cfg_attr(dx_repo_lints, expect(...))]` under a cfg the repo's clippy runs set and a user's do
not; a gate keeps installed files free of bare repo expects.
