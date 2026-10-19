---
id: 92
title: "New-component landings can leave preview's OUT_DIR stale — cargo clippy/test/doc fail until build.rs is…"
status: "Found 2026-09-20 (round 4 integration, Carousel landing onto a pre-existing target dir); recurs for…"
size: "S"
area: build
updated: 2026-10-19
---

# Row 92: New-component landings can leave preview's OUT_DIR stale — cargo clippy/test/doc fail until build.rs is…

## Item

New-component landings can leave `preview`'s `OUT_DIR` stale — `cargo clippy`/`test`/`doc` fail until `build.rs` is touched

## Status / trigger

Found 2026-09-20 (round 4 integration, Carousel landing onto a pre-existing `target` dir); recurs for any future new component landed onto a `target` dir that predates it — remedy known, no construction fix pending

## Size

S

## Detail

**A real trap this integration hit, not a defect in Carousel or in any component's own code:** landing Carousel — this round's brand-new component — into an *existing* `target` dir (one that predated the carousel commits) made `cargo clippy`/`cargo test`/`cargo doc` all fail with `couldn't read .../out/carousel/description.txt` (or equivalent), even though `preview/src/components/carousel/**` was itself complete and correct. **Root cause, read directly from `preview/build.rs`'s own comment, not assumed:** the script deliberately does not watch `src/components` recursively — its own comment explains why (a blanket recursive watch forced a full rebuild on any component's `style.css` edit, compounding the same CSS-hot-reload defect row 80 already root-caused) — so it only ever emits `cargo:rerun-if-changed` for the exact markdown/`component.json` files it already found on some *prior* run; a component it has never seen is invisible to it until something else forces a rerun. **The comment's own "self-healing in practice" framing does not hold the way it reads:** it cites "adding a new component always also means editing `components/mod.rs`, a `.rs` file cargo already tracks directly" as the mechanism — but cargo recompiling `preview` because one of its `.rs` files changed (which it always does) is a different mechanism from cargo *rerunning the build script*, which is governed solely by the script's own declared `rerun-if-changed` list; `mod.rs` is not on that list, so editing it recompiles the crate against the SAME stale `OUT_DIR`, not a fresh one — the comment is not silently wrong (its very next clause already names the real remedy), just optimistically titled for what is actually a loud, blocking failure with a known manual fix, not literal self-healing. **Remedy:** `touch preview/build.rs` or `cargo clean -p preview`, either of which forces cargo to see the build script itself as needing a rerun. **Why none of round 4's three lanes hit this themselves:** each lane built in its own fresh `$CARGO_TARGET_DIR` per this round's own environment brief, so there was no pre-existing `OUT_DIR` to go stale — this trap only appears when an existing target dir predates the new component, exactly the shape of a later integration step, not a lane's own isolated build. **Not fixed here, by design:** the main loop is separately correcting `build.rs`'s own comment (rewording the "self-healing" framing, not this crate's actual behavior); this row exists so the next person who hits the identical `couldn't read .../out/<name>/description.txt` failure after landing a new component finds the remedy in one place rather than re-deriving it
