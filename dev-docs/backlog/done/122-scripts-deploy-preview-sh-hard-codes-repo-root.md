---
id: 122
title: "scripts/deploy-preview.sh hard-codes $repo_root/target/dx/preview/release/web/public and ignores…"
status: "LANDED 2026-10-04 (8c99edd) -- was: OPEN -- found 2026-10-03 when the deploy broke because target…"
size: "S"
area: process
updated: 2026-10-04
---

# Row 122: scripts/deploy-preview.sh hard-codes $repo_root/target/dx/preview/release/web/public and ignores…

## Item

`scripts/deploy-preview.sh` hard-codes `$repo_root/target/dx/preview/release/web/public` and ignores `CARGO_TARGET_DIR`

## Status / trigger

**LANDED 2026-10-04** (`8c99edd`) -- was: **OPEN** -- found 2026-10-03 when the deploy broke because `target` was not a plain directory/symlink; folds into row 116's step 3 but is small enough to take alone

## Size

S

## Detail

Line 13 of the script sets `public_dir="$repo_root/target/dx/preview/release/web/public"` while CLAUDE.md requires every lane to build into an isolated absolute `CARGO_TARGET_DIR` (rows 98/100), so a deploy built from an isolated tree either publishes the wrong tree or finds nothing. The script's own cleanup comment (lines ~41-43) also names `target/` paths. **Fix:** derive `${CARGO_TARGET_DIR:-$repo_root/target}` once (resolve to an absolute path, fail loudly if relative -- row 98's failure mode) and use it for the cleanup list, the build and `public_dir`; add a dry assertion that `public_dir` exists and was written after the build started (mtime), the same "a 200 does not prove it is the latest build" discipline as `dev-docs/dx-serve-hot-reload.md`. **2026-10-04 addendum -- LANDED (`8c99edd`, hardened by `150e97b`):** `deploy-preview.sh` derives `${CARGO_TARGET_DIR:-$repo_root/target}`, rejects a relative one up front (row 98), fails if `public/` holds no file newer than the build start, and tags the tree with `.base-path-build`; the SSG build itself now lives in `scripts/build-ssg.sh` (row 116 addendum), which deploy calls, so the guards exist once.
