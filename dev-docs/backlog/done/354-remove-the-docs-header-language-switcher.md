---
id: 354
title: "Remove the docs header language switcher"
status: "Open (owner 2026-10-21: \"why is there a lang switcher in the docs menu? If there are no translation…"
size: "S"
area: docs
updated: 2026-10-21
---

# Row 354: Remove the docs header language switcher

## Item

Remove the docs header language switcher.

## Status / trigger

Open (owner 2026-10-21: "why is there a lang switcher in the docs menu? If there are no translation i think we can delete it").

## Size

S

## Detail

Facts (verified on main c17bbd78): `LanguageSelect` (`preview/src/main.rs:1059`, mounted at line 720, styled by `assets/language-select.css`) switches `use_init_i18n` (`main.rs:282`) between en-US, fr-FR, es-ES and de-DE from `preview/src/i18n/*.ftl`. Each file is 26 lines: month and weekday names and D/M/Y placeholders, nothing about the site. The only `tid!` users are the `internationalized` variants of `calendar` (weekday, month) and `date_picker` (day, month, year placeholders, month). The site itself is English only. `ssr_parity.rs` mirrors the i18n root (lines 44-61). The `<html lang>` effect (`main.rs:290-298`) follows the active locale.

Plan: drop the header control and its wiring (`LanguageSelect`, `Language`, `language-select.css`, the header's locale plumbing); give the two `internationalized` demos a local locale picker inside the demo (so the i18n formatting hooks are still shown) or a fixed non-English locale; keep `<html lang>` correct (`en` for the page, `lang` on the demo subtree when it is not English). Specs to update: `date-picker.spec.ts:1006` (the switcher test, move it to the demo's own picker) and `all-routes.spec.ts:517` (`<html lang>` follows the switcher); grep `playwright/` for other `Language` combobox users and `ssr_parity.rs` for the i18n root.
