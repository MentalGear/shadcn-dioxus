---
id: 246
title: "Snapshot diff buckets are page-level on some routes"
status: "LANDED 2026-10-19 (element-level classifier, user-select, focusK, kbd records)"
size: "S-M"
area: tests
updated: 2026-10-19
---

# Row 246: Snapshot diff buckets are page-level on some routes

## Item

Snapshot diff buckets are page-level on some routes

## Status / trigger

**LANDED 2026-10-19 (element-level classifier, user-select, focusK, kbd records)**

## Size

S-M

## Detail

The verify4 bucket rules match on class, page and property; the route:/ and avatar/attachment/button buckets are page-level and one cascade rule tolerates shifts up to 40 px, so a real regression of the same kind could hide in them. user-select is not among the 71 captured properties, and keyboard-inset variables and keyboard-modality rings are not visible to the harness. Tighten the bucket rules (element-level) and add the missing properties.

## Update 2026-10-19

Landed. **Classifier committed** as `scripts/snapshot-classify.mjs` (reads the new `<B>.diff.json` that `compare` writes; a
legacy `.diff.txt` is parsed best-effort; `--self-test`). Buckets are element-level by construction: a rule must name an
element (`cls` and/or `page`) or an exact value transition (`prop`+`from`+`to`); a px tolerance (`maxPx`, cap 16) needs `cls`
AND `prop`; patterns are anchored; first match wins; unexplained differences are listed per element and exit 1; unused rules
are reported. The old page-level buckets and the 40 px cascade rule cannot be expressed. **Harness additions (format 4):**
`user-select` in the property set; `focusK` section (the focus pass repeated with `data-dx-modality="keyboard"`, only records
that differ from the plain pass); `open:<k>/kbd` records (the `data-dx-keyboard` / `--dx-keyboard-inset: 280px` /
`--dx-keyboard-top` contract applied by hand to each open `<dialog>`). `compare` of different property sets compares the common
ones and notes it; a two-scheme capture writes one file per scheme.

Evidence: seeded 1 px width change on `dx-card` (page `card`) caught by `compare` (1 value) and by the classifier as
UNEXPLAINED against rules for another class and against a zero-tolerance rule for the same element, and explained only by a rule
naming `dx-card` + `width` + `maxPx >= 1` (exit 0). Format-4 A/A byte-identical. Not done: real on-screen-keyboard states and
`:active` / `::selection` remain outside the harness; the rules file for a concrete phase is the main loop's to write.
