---
id: 111
title: "Migrate the theme to shadcn's system: semantic role tokens, then the Nova style"
status: "LANDED 2026-10-14 (phases A-C; decision 2 applied by row 203, dx-components-theme.css dark…"
size: "M (phase A) + S/M (phase B foundations)…"
area: theme
updated: 2026-10-19
---

# Row 111: Migrate the theme to shadcn's system: semantic role tokens, then the Nova style

## Item

Migrate the theme to shadcn's system: semantic role tokens, then the Nova style

## Status / trigger

**LANDED 2026-10-14 (phases A-C; decision 2 applied by row 203, dx-components-theme.css dark --dx-border 10% / --dx-input 15%; closed 2026-10-19).** Was: **PHASE C LANDED 2026-10-14**; decision 2 APPROVED 2026-10-14 (shadcn white 10%/15%; row 203), shown to the owner side by side -- ours #232323 is 1.14:1 on the #171717 card, shadcn's white 10% 1.32:1. **Phase C owner decisions 2026-10-13 (the plan's recommendations):** (1) dark sidebar background `#171717` with the option-C pairing: yes; (2) dark border/input to shadcn's 10%/15% white: decide after seeing it in snapshot S1 (T0 does not change them); (3) Avatar/Item/Empty sizes follow shadcn (default 32px); (4) drop `.dx-tabs-content`'s default 2rem padding (row 132): yes; (5) tinted destructive Button/Badge: no, solid stays the default (a tinted opt-in variant may follow once `--dx-destructive-text` exists); (6) one modal panel fill: `--dx-popover`; (7) `--dx-primary-foreground` dark: unchanged. **Phase C plan written 2026-10-13: [`research/phase-c-plan-2026-10-13.md`](../../archive/rounds/phase-c-plan-2026-10-13.md)** (inventory of what is and is not on Nova, the option-C palette change in the role fallbacks, the `--dx-destructive-text` token of row 134, the axe/contrast pass, a 9-lane batch after a serial theme lane T0, the snapshot proof S0/S0'/S1/S2, and 7 owner questions with defaults). **Owner decision 2026-10-11 on the chart palette: keep our multi-hue palette as the default; shadcn's palettes stay available as themes.** This is already true by construction: each `--dx-chart-N` reads `var(--chart-N, <our hue>)`, so every shadcn preset (`data-theme-base`/`data-theme-accent` in `theme-presets.css`) and any pasted shadcn theme supplies its own `--chart-1..5` (`--dx-chart-6..8` have no shadcn counterpart). Nothing to build. **Accent dark: OWNER DECIDED 2026-10-11, option C (shadcn's pairing):** dark `--dx-accent` `#262626` together with popover and card `#171717` and page `#0a0a0a`, never the accent alone; it lands in phase C with its axe contrast pass. The finding behind it: our dark popover is already `#262626`, so moving only the accent makes a highlighted menu item 1.00:1 against its menu (invisible); it works only as shadcn's pairing (popover and card `#171717`, page `#0a0a0a`), which also lifts muted shortcut text on the highlight from 4.14:1 (below AA) to 5.86:1. Comparison page: claude.ai artifact "Accent Dark Comparison". **Phases A, B and the global border-box step LANDED 2026-10-04; phase C OPEN, owner-approved 2026-10-05 for the next new session (third in the chain 135 -> 145 -> 111; ask the owner the chart-palette and accent-dark questions first)** -- was: **Owner decision 2026-09-26: yes, backlogged** ("we need to migrate our theme to shadcn proper one") -- not started

## Size

M (phase A) + S/M (phase B foundations) + L (phase C, all components)

## Detail

Owner decision 2026-09-26 ("we need to migrate our theme to shadcn proper one"): the carousel-versus-shadcn comparison showed
the gap was theme-wide, not per demo. shadcn's docs render the **Nova** style (`shadcn-ui/ui@98a1fe6`, constants cited at
`295a1f1`), and our theme had no semantic role-token layer: a numbered colour ramp referenced at **968** sites (842 CSS, 126
inline Rust style strings), measured against the old un-prefixed pages. Evidence and tools:
`dev-docs/archive/research/theme-2026-10-03/` (`role-token-map.md`, `border-box.md`, `sites.tsv`, `tools/`),
[`phase-c-plan-2026-10-13.md`](../../archive/rounds/phase-c-plan-2026-10-13.md).

- **Global border-box (`bcfba01`, phase A step):** `.dx-card` was content-box, so a Card at `height: 100%` overflowed by its
  padding and border (two instances in one week: the carousel vertical demo and `CarouselVirtualContent` slides). Two
  instances made a class: one `*, ::before, ::after { box-sizing: border-box }` in the theme, reviewed through the snapshot.
- **Phase A (`7299a4e`, 2026-10-04):** `--dx-<role>` tokens with shadcn-name fallbacks (`--dx-border: var(--border, <ramp>)`),
  ramp names kept as aliases for installers; 884 sites repointed value-preservingly (a site moves only when it resolves to the
  role's exact value in every mode it uses), 78 left for phase B. **Proof:** a computed-style snapshot (light and dark, 37
  colour properties, pseudo-elements, a `:focus-visible` pass) of the phase A tree against unmodified `2241da8`: **0
  differences**, once `9f8399d` made captures deterministic (A/A byte-identical).
- **Phase B (`256d7e8`, `99f00b1`, `0e6d3ac`):** the remaining `-alt` and off-set ramp uses moved to the right shadcn role;
  the Nova foundations (radius, card, button, input, badge, tabs, dialog, select) at `295a1f1`; `--dx-border-strong`; the
  radius gate repaired; new gate `scripts/check-css-vars-defined.sh` (every `var(--x)` is declared or has a fallback), red on
  unmodified main with five never-defined properties.
- **Theme picker and presets (round 2, 2026-10-04):** `preview/assets/theme-presets.css`, generated by
  `tools/gen_presets.mjs` from `shadcn-ui/ui@295a1f1`: 9 bases x 17 accents x 6 radii composed by cascade on three `<html>`
  attributes, with a pre-paint script and a preset contrast sweep (a preset's `--primary` is a fill, 1.5:1 as text in places).
- **Phase C (landed 2026-10-14):** a serial theme lane T0 (the option-C dark palette: accent `#262626` only together with popover
  and card `#171717` and page `#0a0a0a`; sidebar `#171717`; menu-item density; `--dx-destructive-text`;
  `--dx-accent-selected`) plus eight lanes moved about 60 component sheets to Nova, each citing `style-nova.css@295a1f1`.
  Proof: snapshot A/A byte-identical, S0 to S1 13.9k values all in T0's expected classes, S1 to S2 181k values classified;
  the 538 unexplained were three real defects fixed in the same round (rows 197, 199, 200). Registry smoke 75/75.
  Decision 2 (dark border and input to shadcn's white 10% and 15%) applied by row 203.

Owner decisions: chart palette stays our multi-hue default (each `--dx-chart-N` reads `var(--chart-N, <our hue>)`, so any
shadcn preset supplies its own); Avatar/Item/Empty sizes follow shadcn; `.dx-tabs-content`'s default padding dropped (row 132);
solid destructive Button/Badge stays default; one modal panel fill `--dx-popover`. Deliberately not taken: Nova's zero panel
padding on Sheet and Drawer (we keep 16px, row 197), the short month caption for Nova's 196px calendar (we keep 232px). The
`dark-surface-contrast` spec is the hover-state contrast oracle.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/111-migrate-the-theme-to-shadcn-s-system-semantic.md`.*
