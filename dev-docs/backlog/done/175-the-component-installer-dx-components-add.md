---
id: 175
title: "The component installer: dx components add resolved dependencies in dx's upstream default registry, not ours"
status: "LANDED 2026-10-10"
size: "S (done)"
area: build
updated: 2026-10-10
---

# Row 175: The component installer: dx components add resolved dependencies in dx's upstream default registry, not ours

## Item

The component installer: `dx components add` resolved dependencies in dx's upstream default registry, not ours

## Status / trigger

**LANDED 2026-10-10**

## Size

S (done)

## Detail

A `--git` flag alone was not enough: bare-name `componentDependencies` still resolved in dx's upstream default registry even under `--git`, and the primitives dependency pointed at DioxusLabs/components. Fixed in 67 `component.json` files (object-form dependencies with `git`, pointing at our primitives URL), with one `REGISTRY_GIT_URL` constant and a new gate, `scripts/check-registry-url.sh` (in `run-gates.sh`). The command is `dx components add <name> --git https://github.com/MentalGear/shadcn-dioxus`, or `[components.registry] git = ...` in `Dioxus.toml`; the docs pages show it. Verified against a `file://` copy of the repo; **the https verification against GitHub is pending the merge.** See row 161 for the rename that this URL follows. **UPDATE 2026-10-10:** the installer is now four more things: the `shadcn-dioxus` CLI that reads this registry by default (row 179), the starter template (row 180), the missing-`mod.rs` fix with its gate (row 181) and the end-to-end smoke script (row 182). The docs show the CLI first and the dx commands as the alternative (row 184).
