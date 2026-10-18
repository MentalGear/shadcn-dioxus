---
id: 180
title: "The starter template: dx new myapp --template https://github.com/MentalGear/shadcn-dioxus --subtemplate…"
status: "LANDED 2026-10-10"
size: "S (done)"
area: build
updated: 2026-10-10
---

# Row 180: The starter template: dx new myapp --template https://github.com/MentalGear/shadcn-dioxus --subtemplate…

## Item

The starter template: `dx new myapp --template https://github.com/MentalGear/shadcn-dioxus --subtemplate templates/starter`

## Status / trigger

**LANDED 2026-10-10**

## Size

S (done)

## Detail

`templates/starter` (a cargo-generate template: `Cargo.toml.liquid`, `Dioxus.toml`, `src/`) declares `mod components;`, depends on `dioxus-primitives` from this repository and sets `[components.registry] git` in its `Dioxus.toml`, so a plain `dx components add <name>` inside the new app reads this registry. `scripts/check-registry-url.sh` pins both URLs to `REGISTRY_GIT_URL`; `shadcn-dioxus new` renders the same files. Follow-up: its own `README.md` still shows the `dx components` commands first (it is outside the docs lane), worth leading with `shadcn-dioxus add`.
