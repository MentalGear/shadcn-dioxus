---
id: 172
title: "date-picker.spec.ts :73 and :144 anchor placement (the panel opened above the trigger at 1280x720; offset…"
status: "LANDED 2026-10-05 (round 3, final verification): a test-geometry plus selector defect in the specs…"
size: "S (done)"
area: tests
updated: 2026-10-05
---

# Row 172: date-picker.spec.ts :73 and :144 anchor placement (the panel opened above the trigger at 1280x720; offset…

## Item

`date-picker.spec.ts` `:73` and `:144` anchor placement (the panel opened above the trigger at 1280x720; offset changed after scroll)

## Status / trigger

**LANDED 2026-10-05 (round 3, final verification): a test-geometry plus selector defect in the specs; NOT a product anchoring bug and NOT the sandbox's Chromium 1194 engine**

## Size

S (done)

## Detail

The round-3 question was whether these were rows 21 and 106's environment class or a regression from round 3's date-picker changes. **Neither.** (1) **Geometry:** the panel is 278 px tall and the trigger's bottom edge sits at y=446, so at a 720 px viewport only 274 px remain below it and the native `flip-block` fallback correctly opens the panel ABOVE the trigger; a 150 px scroll frees room and it flips back (this is the jump row 21 measured; the anchoring itself was working). (2) **Selector:** since the header theme picker landed (`b9b8e1f7`), `querySelector('[style*="anchor-name"]')` matched that header button first, so the tests measured the wrong trigger. **Fixed in the specs:** `date-picker.spec.ts` runs the two geometry tests at 1280x900 (`ROOMY_VIEWPORT`), asserts the precondition (room below the trigger exceeds the panel height, so a short viewport fails loudly as itself) and uses the real trigger and panel locators; `oracle/tier2-html/top-layer.spec.ts` Rule 8 (ColorPicker) got the same fix (a 1280x1000 viewport, its own trigger and panel locators) and now PASSES. Rows 21 and 106 no longer carry the "frozen anchor" / engine diagnosis for these cases.
