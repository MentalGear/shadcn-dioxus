---
id: 179
title: "The shadcn-dioxus installer CLI: new, init, add, remove, list, update, clean"
status: "LANDED 2026-10-10"
size: "M (done)"
area: build
updated: 2026-10-10
---

# Row 179: The shadcn-dioxus installer CLI: new, init, add, remove, list, update, clean

## Item

The `shadcn-dioxus` installer CLI: `new`, `init`, `add`, `remove`, `list`, `update`, `clean`

## Status / trigger

**LANDED 2026-10-10**

## Size

M (done)

## Detail

`cli/` (package `shadcn-dioxus`, `cargo install --git https://github.com/MentalGear/shadcn-dioxus shadcn-dioxus`), a fork of `dx components` from dioxus-cli 0.7.9 with six dependencies and no async runtime. It reads this registry by default (so no `--git`), resolves a bare-name dependency in the registry of the component that names it, checks destinations before writing, and clones into a temporary directory renamed into the cache when complete. `new <name>` renders `templates/starter` (the project `dx new --subtemplate` makes) without dx or cargo-generate; `init` writes `[components.registry]` into `Dioxus.toml`, creates `src/components`, declares `mod components;` and links the theme, and is idempotent; `update` records a hash of every file it writes in `src/components/.shadcn-dioxus.json` and never overwrites a file you edited (the component is left unchanged and listed, exit 1; `--force` replaces). `cargo test -p shadcn-dioxus` (the tests cover new, init, add, update and the registry). `scripts/check-registry-url.sh` pins the CLI's `DEFAULT_REGISTRY_GIT_URL` and `cli/README.md`'s install command to `REGISTRY_GIT_URL`; `scripts/registry-install-smoke.sh --cli BIN` installs every component through it (row 182). Reference: `cli/README.md`; walkthrough: the README and `/docs`.
