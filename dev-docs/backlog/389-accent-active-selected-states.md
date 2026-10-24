---
id: 389
title: "Active and selected states take the picked accent (mark, ink, tint slots)"
status: "DECIDED 2026-10-24 (owner), BUILT on lane/accent-states (audit: 58 rows). Hard requirement kept by…"
size: "M"
area: theme
updated: 2026-10-24
---

# Row 389: active and selected states take the picked accent

## Item

The picked accent (`<html data-theme-accent>`; shadcn's accent presets set `--primary`, charts and sidebar-primary, never `--accent`) reached buttons, badges and links but not the states a user reads as "where am I" or "what is on": the Tabs bar, the current docs-nav row, a pressed Toggle, the highlighted menu row. Owner decision 2026-10-24, scope "must plus optional": those states show the accent, and four marks that already used the accent but were unreadable get fixed. Naming trap: shadcn's `--accent` / `--dx-accent` is the NEUTRAL hover surface, not the picked accent.

## Status / trigger

DECIDED 2026-10-24 (owner), BUILT on `lane/accent-states` (audit: 58 rows). Hard requirement kept by construction and by spec: with no accent picked every touched state paints the colour it painted before, to the byte.

Construction, one theme-level family next to `--dx-primary-ink` (`preview/assets/dx-components-theme.css`); a component reads a slot, never `--dx-primary` raw:

| Slot | With an accent (`:root[data-theme-accent]`) | No accent |
|---|---|---|
| `--dx-primary-mark` | `--dx-primary` with OKLCH L <= 0.58 (light) / >= 0.62 (dark): a non-text mark | `--dx-primary` |
| `--dx-active-mark` | the same mark | `--dx-foreground` |
| `--dx-active-ink` | `--dx-primary-ink` (text on a plain surface) | `--dx-foreground` |
| `--dx-accent-tint` | accent hue, 0.10 (light) / 0.25 (dark) of its chroma, at L 0.97 / 0.269 | `--dx-accent` |
| `--dx-accent-tint-selected` | light: the tint; dark: tint + 15% ink in `--dx-hover-mix-space` | `--dx-accent-selected` |
| `--dx-tint-step-ink` | `--dx-foreground` with its hue missing, so a 15% oklch step keeps the tint's hue (the foreground's explicit `0deg` pulled blue 265 to 279 and emerald 166 to 143) | `--dx-foreground` (the step is today's, hue quirk included, in every base) |
| `--dx-sidebar-accent-tint` | `--dx-accent-tint` | `--dx-sidebar-accent` |
| `--dx-ring-ink` / `--dx-accent-ring-ink` | near-white at L <= 0.575 on the ring colour, else near-black (row 385's tag rule); the second is declared only with an accent | `--dx-ring-ink` is the same formula (the Tag already used it); the second is undeclared |

The slots are declared in the `:root` block with the no-accent value and overridden in the existing `:root[data-theme-accent]` rule (row 385's), so no accent leaves the old chain untouched.

States changed (must): Tabs line bar horizontal and vertical (`--dx-active-mark`), default Tabs active label (`--dx-active-ink` through a `--dx-tabs-active-color` slot; the line variant keeps foreground; a focused active tab keeps it), Navigation Menu current link (`data-active`, new rule, `--dx-accent-tint-selected`; a state that had no rule, so it is the one item without a "same as before" identity), Sidebar active row and sub-row (`--dx-sidebar-accent-tint`; the docs nav is the library Sidebar), Toggle, Toggle Group (pressed = `--dx-accent-tint` + 15% ink) and Toolbar (`--dx-accent-tint-selected`) pressed, Tag Group selected (reads `--dx-ring-ink`, unchanged value), Carousel active dot, `--dx-drop-indicator`, Slider range, Progress fill (`--dx-primary-mark`). Optional: menu / context menu / menubar item highlight (the theme's `[data-dx-menu-item]` rule, `--dx-accent-tint`) and check icon (`--dx-accent-ink`, destructive items keep their ink), Select and Combobox and Command highlight and check, Menubar open trigger, Calendar range band, Table selected row (the theme slot `--dx-table-selected-background` = `--dx-accent-tint`; `virtual_grid/` untouched, it reads the slot), Pagination current page (ink label, mark border), `::selection` and `accent-color` (`main.css`, accent only), docs charts-tab current and sort-toggle pill, `BlockProfile`'s Verified icon (`--dx-accent-ring-ink`, white with no accent).

Not changed: Checkbox / Radio / Switch / Calendar selected-day fills (shadcn-exact), destructive and disabled items, hover on nav and tabs, breadcrumb current page, calendar today, caret, scrollbar, resizable handle, the focus ring, `theme-presets.css`, Navbar item hover (audit row 10; not in the owner's list).

Why these forms. The raw accent is 1.4:1 (light lime, yellow) and 1.5-2.1:1 (15 of 17 accents in dark) against the surfaces it marks, so a mark is clamped (`--dx-primary-ink`'s construction, bounds that also clear the muted track a Slider or Progress sits on); ink on a 10-15% tint measured 4.16-4.45:1 in light, so text on a tint is `--dx-foreground` or `--dx-accent-foreground`, never the ink; a `color-mix(primary 12%, transparent)` tint would paint #e3e3e3 on white where the neutral accent is #f5f5f5, so the tint is the accent's hue at the neutral accent's own lightness (a chroma-0 primary gives the neutral accent) and, with no accent, the neutral accent itself.

Measured (Chromium canvas sRGB, 10 bases x 18 accents x light/dark x default/`wcag` contrast, minimum over all):

| Quantity | Light | Dark |
|---|---|---|
| mark on page / card / popover / muted track (bound 3:1) | 3.53 (taupe + lime, on muted) | 3.51 (olive + fuchsia, on muted) |
| raw `--primary` as that mark (audit) | 1.41 (lime) | 1.50 (indigo) |
| `--dx-active-ink` on page / card / popover (bound 4.5) | 5.49 (green) | 6.44 (violet) |
| `--dx-foreground` / `--dx-accent-foreground` on the tint (bound 7) | 15.2 | 14.2 |
| same on the tint-selected | 15.2 (the tint) | 9.5 |
| sidebar accent foreground on the sidebar tint | 15.2 | 14.2 |
| ink icon on the tint (bound 3) | 5.06 | 5.84 |
| muted foreground on the tint (range band, adjacent month) | 4.52 in `wcag` (3.83 with shadcn's own olive muted foreground, which is 3.8 on the muted fill it replaces) | 5.65 |
| selected Tag ink on its fill (bound 4.5) / Verified icon (bound 3) | 4.67 / 4.67 | 4.51 / 4.75 |
| tint against the popover (neutral accent: 1.090 / 1.185) | 1.081 | 1.165 |

Mark bounds: the audit's 0.58 / 0.62 hold; 0.60 / 0.64 would give 3.34 / 3.99. Tint chroma: 0.35 of the accent's chroma at L 0.97 is outside sRGB for blue and indigo (an sRGB-clipped `oklch(0.97 0.07 264)` reads as cyan, hue 221) and lowers the `wcag` muted foreground on the range band to 4.48:1; 0.12 still gave 4.48, 0.10 gives 4.52, so light is 0.10 and dark (where L 0.269 holds the chroma, Tailwind's own `blue-950` is 0.09) is 0.25. Hue: marks within 12 degrees of the accent's, tints and the dark-yellow ink within 25 (sRGB clip of a pale colour, gamut mapping of dark yellow).

Guards: `playwright/accent-states.spec.ts` (token sweep: no-accent identity per base, default values, marks >= 3:1 on page / card / popover / muted, ink >= 4.5:1, foreground on tint >= 7:1, hue tracks the accent, tint step from the popover, tag ink and verified icon; wiring: every touched component's computed colour equals its slot and, with no accent, the old expression).

## Size

M

## Detail

See the Status section. Tuning is a small diff: `--dx-accent-mark-l-max` / `--dx-accent-mark-l-min` (the mark's bounds, `MARK_L_MAX` / `MARK_L_MIN` in the spec) and `--dx-accent-tint-chroma-light` / `-dark` (how much of the accent's chroma the tint carries).
