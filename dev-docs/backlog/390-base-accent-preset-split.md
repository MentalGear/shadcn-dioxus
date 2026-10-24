---
id: 390
title: "Base and accent presets own disjoint variables; charts follow the accent"
status: "DECIDED 2026-10-24 (owner): (1) each axis owns its own variables, disjoint and enforced; (2) charts…"
size: "M"
area: theme
updated: 2026-10-24
---

# Row 390: base and accent presets own disjoint variables

## Item

Owner report 2026-10-24: "check if there is clear, encoded distinction between accent and base color, because atm base color (gray) changing even chart colors different and by its own". There was none. `theme-presets.css` copied shadcn's `{...baseColor, ...accent}` merge as cascade order, so both axes wrote `--primary`, `--primary-foreground`, `--chart-1..5` and `--sidebar-primary(-foreground)`, and every accent also wrote `--secondary(-foreground)` as a fixed zinc grey. Charts came from three places: no preset = the library palette; the seven new-style bases = grey ramps; gray and slate (legacy shadcn data) = multicolour. An amber accent on a stone base got a zinc-tinted secondary.

## Status / trigger

DECIDED 2026-10-24 (owner): (1) each axis owns its own variables, disjoint and enforced; (2) charts with no accent picked are the library palette under every base, an accent swaps in its hue ramp. BUILT on `lane/preset-split`.

Ownership table (`OWNERSHIP` in `scripts/gen-theme-presets.mjs`, the one place it is written; the stylesheet header and the gate read it):

| Axis | Owns |
|---|---|
| base | `--background --foreground --card(-foreground) --popover(-foreground) --secondary(-foreground) --muted(-foreground) --accent(-foreground)` (the neutral hover surface) `--destructive --border --input --ring` (the no-accent ring) `--sidebar --sidebar-foreground --sidebar-accent(-foreground) --sidebar-border --sidebar-ring`, `--border-strong`, `--destructive-foreground` (derived once for every base) |
| base, slots | `--dx-base-primary`, `--dx-base-primary-foreground`, `--dx-base-sidebar-primary`, `--dx-base-sidebar-primary-foreground`: the neutral primary that shows with no accent |
| accent | `--primary(-foreground) --chart-1..5 --sidebar-primary(-foreground)`, plus the ring tint (row 385 derives it from `--dx-primary`) |

Dropped from shadcn's data: a base's `--chart-1..5`, an accent's `--secondary(-foreground)`.

Primary construction: the one variable both axes need is a slot. The theme reads `--dx-primary: var(--primary, var(--dx-base-primary, <ramp>))` (same for the foreground and the sidebar-primary pair), so an accent's bare `--primary` overrides the base's slot, a host's own pasted shadcn `--primary` still wins over both, and with no accent the base's neutral primary shows. The alternative, a second attribute that gates the base's primary, would have made the base block depend on the accent's presence; the slot keeps every block static.

Generator: `scripts/gen-theme-presets.mjs` (moved out of the archive) reads `scripts/theme-presets.data.json`, shadcn's data vendored from `shadcn-ui/ui@295a1f11` (extracted from the committed stylesheet, so a regeneration needs no network; `--import <registry dir>` refreshes it from a shadcn checkout), applies the table and fails on a variable the table neither assigns nor drops.

Gate: `scripts/check-theme-preset-axes.sh` (in `run-gates.sh`): every base block writes only base-owned names, every accent block only accent-owned ones, the table gives no name to both axes, every slot is read by a role token, and the committed stylesheet equals the generator's output byte for byte. Its self-test turns six mutations red on every run (a base block with a chart, an accent block with `--secondary`, a base block with a bare `--primary`, an accent block with `--background`, an accent block with a base slot, a slot the theme never reads) plus a hand edit.

Identity with no accent: for all 9 bases x (no accent + 18 accents) = 171 combinations the effective variables old vs new differ only in the intended places: base-only `--chart-1..5` (9 each, now the library palette) and accent `--secondary(-foreground)` (144 = 9 bases x 16 accents whose base secondary is not already zinc's). Every other variable, per base, with no accent: byte-identical values.

Guards: `playwright/theme-preset.spec.ts` ("Axes" block: charts under every base x mode with no accent equal the library palette, with an accent equal its ramp under every base, secondary under an accent equals the base's, the no-accent primary per base equals the values the base had before the split), `theme-preset-contrast`, `ring-contrast` and `accent-states` stay green.

## Size

M

## Detail

See the Status section. A new preset base or accent is a data-file entry; a variable that moves between axes is a one-line change to `OWNERSHIP`, and the gate says which stylesheet lines then differ.
