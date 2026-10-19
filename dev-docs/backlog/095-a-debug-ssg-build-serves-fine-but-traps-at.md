---
id: 95
title: "A debug SSG build serves fine but traps at runtime, so interaction specs run against it fail like real…"
status: "Hit 2026-09-20 (round 5) running carousel.spec.ts against a local static serve of dx build --ssg…"
size: "DONE — warning added directly to the…"
area: build
updated: 2026-09-20
---

# Row 95: A debug SSG build serves fine but traps at runtime, so interaction specs run against it fail like real…

## Item

A debug SSG build serves fine but traps at runtime, so interaction specs run against it fail like real regressions

## Status / trigger

Hit 2026-09-20 (round 5) running `carousel.spec.ts` against a local static serve of `dx build --ssg` output

## Size

DONE — warning added directly to the recipe in `dev-docs/conformance-harness.md`, where it is actually read

## Detail

The documented local SSG recipe omits `--release`, and the resulting debug artifact prerenders correct markup (so `hydration-parity.spec.ts` and any static-geometry check are happy) but its wasm traps the moment the app does anything: `RuntimeError: memory access out of bounds` in the console, plus a failed `ws://127.0.0.1:8080/_dioxus?build_id=0` handshake because a debug build still expects a `dx serve` hot-reload socket. HTTP 200, page renders, clicks do nothing. **13 tests across `carousel.spec.ts` and `oracle/tier1-apg/carousel.spec.ts` failed on known-good code** — paging, keyboard, drag — while every geometry assertion in the same run passed, which is exactly the pattern that makes it look like a targeted regression rather than an environment fault. Same family as row 92 (stale `OUT_DIR`) and the `--base-path`-served-at-root trap: an artifact that returns 200 is not evidence that the artifact is the one you meant to test.
