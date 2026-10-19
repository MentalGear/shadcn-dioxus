---
id: 212
title: "Small-control demos labelled like shadcn's (not a lone control in a big frame)"
status: "LANDED 2026-10-14 (demo lane, run by a Haiku agent, 738d136)"
size: "S-M"
area: docs
updated: 2026-10-14
---

# Row 212: Small-control demos labelled like shadcn's (not a lone control in a big frame)

## Item

Small-control demos labelled like shadcn's (not a lone control in a big frame)

## Status / trigger

**LANDED 2026-10-14 (demo lane, run by a Haiku agent, 738d136)**

## Size

S-M

## Detail

Owner request. Audit checkbox, radio_group, switch, toggle, toggle_group, slider, progress, label, kbd, badge, spinner, separator, skeleton against their shadcn demos (rows 187/188 first); compose real `Label`/`Field`; demo-parity check that each control demo has a visible label. Item 10. Plan: `dev-docs/research/next-round-plan-2026-10-14.md`. **Landed:** checkbox main "Accept terms and conditions" (Field + FieldLabel), radio_group main Default/Comfortable/Compact (default comfortable; the disabled Green item dropped as in shadcn), switch main "Airplane Mode", toggle main Bookmark icon + text, toggle_group main/disabled aria-labels Toggle bold/italic/strikethrough (a strikethrough icon instead of shadcn's mismatched underline icon), separator main "Radix Primitives" / "An open-source UI component library.", spinner main muted Item "Processing payment..." + "$100.00", skeleton main 250/200px lines + a 250x125 card. New `demo-labels.spec` (visible-label names for checkbox/radio_group/switch; allowlist checkbox/table). **Not covered:** slider and progress (shadcn's are bare too); toggle outline/sm need the parity-wave props (row 155 group 3). Model-fit evidence: `dev-docs/agent-model-fit.md`.
