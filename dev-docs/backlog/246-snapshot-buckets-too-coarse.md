---
id: 246
title: "Snapshot diff buckets are page-level on some routes"
status: "BACKLOG (validation 2026-10-19)"
size: "S-M"
area: tests
updated: 2026-10-19
---

# Row 246: Snapshot diff buckets are page-level on some routes

## Item

Snapshot diff buckets are page-level on some routes

## Status / trigger

**BACKLOG (validation 2026-10-19)**

## Size

S-M

## Detail

The verify4 bucket rules match on class, page and property; the route:/ and avatar/attachment/button buckets are page-level and one cascade rule tolerates shifts up to 40 px, so a real regression of the same kind could hide in them. user-select is not among the 71 captured properties, and keyboard-inset variables and keyboard-modality rings are not visible to the harness. Tighten the bucket rules (element-level) and add the missing properties.
