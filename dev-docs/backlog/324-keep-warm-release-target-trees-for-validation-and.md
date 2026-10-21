---
id: 324
title: "Keep warm release target trees for validation and for deploy"
status: "BACKLOG (owner approved 2026-10-21)"
size: "S"
area: process
updated: 2026-10-21
---

# Row 324: Keep warm release target trees for validation and for deploy

## Item

Keep warm release target trees for validation and for deploy.

## Status / trigger

**BACKLOG (owner approved 2026-10-21)**

## Size

S

## Detail

The 2026-10-20 batch-A release build was cold (~22 min, 707 crates) because the only warm release tree carried the deploy's `.base-path-build` marker and `lane-target.sh` refused it as a template. Keep two warm release trees -- one no-base-path for validation, one base-path for deploy -- refreshed after each deploy, and teach `lane-target.sh` to pick by base path. Expected release build ~3-5 min.
