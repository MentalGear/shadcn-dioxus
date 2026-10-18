---
id: 184
title: "Docs: a Motion section and CLI-first install"
status: "LANDED 2026-10-10"
size: "S (done)"
area: build
updated: 2026-10-10
---

# Row 184: Docs: a Motion section and CLI-first install

## Item

Docs: a Motion section and CLI-first install

## Status / trigger

**LANDED 2026-10-10**

## Size

S (done)

## Detail

The owner asked that animations be "properly documented, in general and the new additions". `dev-docs/motion.md` (linked from the README and `dev-docs/README.md`) and the same content on `/docs` under "Motion" (`preview/src/motion_docs.rs`; the tables and code blocks are shown from constants and the unit test `motion_tables_match_motion_md` fails when they stop appearing verbatim in the markdown): tokens and how to change them, reduced motion, compositor-only infinite animations and the shimmer exception, visibility gating (`use_motion`, `use_interval_while`/`use_timeout_while`, `use_entered_view_when`, `use_document_visible`, pause versus stop, the `check-motion-gating.sh` gate and `motion-ok:`), anchored keyframes, modal exits, the kept theme-switch transitions (row 158b), how an app author adds an animated component, and a per-component table (what moves, what it does while unseen, how to tune or disable). A "Motion" subsection (or a link to it) in the `docs.md` of every animated component. Install docs: the README and `/docs` lead with the CLI (`cargo install --git`, `new`, `init`, `add`, `update`), then the `dx new --subtemplate`, `[components.registry]` and `dx components add <name> --git` alternatives, with a warning that plain `dx components add` installs upstream's version. Component pages: the copy button now copies `shadcn-dioxus add <name>` and shows the `dx components add <name> --git ...` line under it (`ComponentInstallCommand`); the gallery cards copy the CLI command only. Row 176 is the reduced-motion gap this writing found.
