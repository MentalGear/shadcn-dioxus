---
id: 98
title: "dx build --ssg fails with No such file or directory (os error 2) when CARGO_TARGET_DIR is a relative path"
status: "Hit 2026-09-20 (round 5) by a lane following this repo's own CARGO_TARGET_DIR=target-lane-<name>…"
size: "Workaround is one word: make the path…"
area: process
updated: 2026-09-20
---

# Row 98: dx build --ssg fails with No such file or directory (os error 2) when CARGO_TARGET_DIR is a relative path

## Item

`dx build --ssg` fails with `No such file or directory (os error 2)` when `CARGO_TARGET_DIR` is a relative path

## Status / trigger

Hit 2026-09-20 (round 5) by a lane following this repo's own `CARGO_TARGET_DIR=target-lane-<name>` convention

## Size

Workaround is one word: make the path absolute. ~20 min lost across two failed builds before it was traced

## Detail

Traced into `dioxus-cli-0.7.9/src/build/builder.rs`'s `pre_render_static_routes`: it runs `Command::new(&server_exe).current_dir(server_exe.parent().unwrap())` while `server_exe` is still a **relative** `PathBuf`. `current_dir` chdirs the child before exec, so the still-relative program path is then resolved against the new cwd rather than the original one, and the binary can never find itself. **This is a direct hazard for `CLAUDE.md`'s own lane discipline**, which tells every lane to set an isolated `CARGO_TARGET_DIR` and shows it as a bare relative name — any lane that then does an SSG live-check walks into this. Passing the same directory as an absolute path fixes it with no rebuild cost, since the relative-path attempt has already populated the cache. Upstream-reportable: the fix is to canonicalise `server_exe` before setting `current_dir`.
