#!/usr/bin/env bash
# Isolated, warm CARGO_TARGET_DIRs for lanes (backlog row 116 step 3; rows 98/100).
#
#   scripts/lane-target.sh create <name> [--template DIR | --release [--base-path P]] [--root DIR] [--force]
#   scripts/lane-target.sh warm-release [--base-path P] [--root DIR] [--force]
#   scripts/lane-target.sh prune  [--root DIR] [--older-than DAYS] [--apply]
#   scripts/lane-target.sh slim   [--root DIR] [--apply]
#   scripts/lane-target.sh du     [--root DIR]
#
# `create` copies a warm template (scripts/lane-slot-target.sh copy) to <root>/<name> and prints
# `export CARGO_TARGET_DIR=<abs path>` -- use `eval "$(scripts/lane-target.sh create x)"`
# (progress goes to stderr, only the export lines to stdout). The path is always
# absolute (dx --ssg breaks on relative ones, row 98) and never the shared repo `target/`.
# It also exports `CARGO_INCREMENTAL=0` (row 168: incremental state regrows a target dir
# by ~10-13 GB per lane-hour); `LANE_CARGO_INCREMENTAL=1` opts back in, at the cost of
# running `prune` afterwards.
#
# Row 367: every lane command runs with CARGO_INCREMENTAL=0, which neither reads nor writes cargo's
# `incremental/` dirs, so they are dead weight (about 3 of the template's 6.8 GB). `create`
# leaves them out of the copy (the same `lane-slot-target.sh copy` lane-ssg.sh uses) and sizes its disk
# check without them. `slim` deletes them from every tree under the root: once is enough to free the
# template's 3.0 GB for good. A tree created with LANE_CARGO_INCREMENTAL=1 keeps the full copy and gets
# a `.keep-incremental` stamp that `slim` respects. `slim --apply` holds every build slot (and the legacy
# lock) with a non-blocking flock while it deletes, because a copy reading a tree mid-delete fails; with a
# build running it refuses and changes nothing. `prune --apply` runs `slim` after its own deletions and
# only warns when the slots are busy.
#
# The template must be base-path-free (row 100: a tree that ever held a
# `--base-path shadcn-dioxus` build keeps emitting prefixed URLs even after
# `rm -rf public`). Two checks, because neither alone is enough:
#   1. marker `<template>/.base-path-build`, written by scripts/deploy-preview.sh --
#      instant, and catches a tree whose `public/` was already wiped;
#   2. grep of the template's dx `public/` HTML for quoted `/shadcn-dioxus/` (or `/dioxus-components/`) URLs --
#      authoritative on the actual output, catches templates made by hand.
# Defaults: root $LANE_TARGET_ROOT or /home/user/tgt, template $LANE_TARGET_TEMPLATE or <root>/template.
#
# Warm RELEASE trees (row 324). A cold release build is ~22-50 min (700 crates, lto); with the dependencies warm only
# the two workspace crates rebuild (~14 min, measured below). One warm release tree cannot serve both: the deploy's carries
# `.base-path-build`, and the check above (rightly) refuses it for a validation build. So there
# are two, chosen by base path, and neither is ever built with the other's setting:
#   <root>/release-template   NO base path -- validation builds (`create <name> --release`), sentinel `.no-base-path-only`
#   <root>/release            base path `shadcn-dioxus` -- the deploy (scripts/deploy-preview.sh builds in it by default;
#                             `create <name> --release --base-path shadcn-dioxus` copies it)
# `warm-release [--base-path P]` is the REFRESH STEP: an in-place, queued (class lane) `build-ssg.sh release` of THIS
# worktree into the matching tree: the whole cold build the first time (measured: 1,309 s), and afterwards a
# build in which only the workspace crates recompile (measured: 864 s; `preview`, 410 s server + 267 s client, and
# `primitives` are fat-LTO single-codegen-unit crates, so the saving over cold is ~7 min).
# What the warm tree holds is the DEPENDENCY crates, which change only with Cargo.lock, the
# toolchain or dx: so the refresh is only needed when Cargo.lock's hash differs from `.cargo-lock-sha` recorded in the tree
# (`warm-release` says so and skips otherwise; --force builds anyway). Re-running it after every deploy would rebuild
# the workspace crates that every validation build rebuilds anyway. The deploy itself refreshes `release`. `prune` never
# removes either tree. A release tree is ~3.5 GB: `create` refuses when the disk cannot take the copy (--force to try).
set -euo pipefail

root="${LANE_TARGET_ROOT:-/home/user/tgt}"
template="${LANE_TARGET_TEMPLATE:-}"
older_than=2 apply=0 force=0 release=0 base_path="" explicit_template=0

die() { echo "lane-target: $*" >&2; exit 1; }
usage() { sed -n '2,9p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//' >&2; exit 2; }

cmd="${1:-}"; [ -n "$cmd" ] || usage; shift
name=""
if [ "$cmd" = create ]; then name="${1:-}"; [ -n "$name" ] || usage; shift; fi
while [ $# -gt 0 ]; do
  case "$1" in
    --root) root="${2:?}"; shift 2 ;;
    --template) template="${2:?}"; explicit_template=1; shift 2 ;;
    --release) release=1; shift ;;
    --base-path) base_path="${2:?}"; shift 2 ;;
    --older-than) older_than="${2:?}"; shift 2 ;;
    --apply) apply=1; shift ;;
    --force) force=1; shift ;;
    *) usage ;;
  esac
done
root="${root%/}"
case "$root" in /*) ;; *) die "root must be absolute: $root" ;; esac
[ -z "$base_path" ] || [ "$release" = 1 ] || [ "$cmd" = warm-release ] || die "--base-path only goes with --release / warm-release"
[ "$release" = 0 ] || [ "$explicit_template" = 0 ] || die "--release picks the template itself; drop --template"
# The two warm release trees, picked by base path (row 324).
release_tree() { # $1 = base path or empty -> the tree's path
  if [ -n "$1" ]; then echo "$root/release"; else echo "$root/release-template"; fi
}
if [ "$release" = 1 ]; then template="$(release_tree "$base_path")"; fi
template="${template:-$root/template}"; template="${template%/}"

contaminated() { # $1 = tree; prints why and returns 0 if it holds a base-path build
  if [ "$release" = 1 ] && [ -n "$base_path" ]; then
    # The deploy tree is SUPPOSED to hold a base-path build -- of this very base path, and of nothing else.
    [ -e "$1/.no-base-path-only" ] && { echo "$1 is the no-base-path tree (sentinel .no-base-path-only)"; return 0; }
    grep -q -- "--base-path $base_path " "$1/.base-path-build" 2>/dev/null \
      || { echo "$1/.base-path-build does not record a build with --base-path $base_path"; return 0; }
    return 1
  fi
  [ -e "$1/.base-path-build" ] && { echo "marker $1/.base-path-build"; return 0; }
  local d
  for d in "$1"/dx/*/*/web/public; do
    [ -d "$d" ] || continue
    grep -rqE --include='*.html' "[\"'(]/(shadcn-dioxus|dioxus-components)/" "$d" && { echo "base-path URLs in $d"; return 0; }
  done
  return 1
}

# incremental_dirs <tree>: cargo's incremental dirs, at <tree>/<profile>/ and <tree>/<triple>/<profile>/.
incremental_dirs() { find "$1" -mindepth 2 -maxdepth 3 -type d -name incremental -prune 2>/dev/null; }
# slim <apply> <strict>: list (and with apply=1 delete) the incremental dirs of every tree under $root.
# Applying holds every slot lock and the legacy lock non-blocking; busy -> refuse (strict=1: exit 1, else warn).
slim() {
  local apply="$1" strict="$2" q="$root/lockq" legacy="${LANE_LEGACY_LOCK-$root/ssg.lock}" f fd d inc total=0 kb i
  local -A locks=()
  if [ "$apply" -eq 1 ]; then
    mkdir -p "$q"
    # every slot LANE_SLOTS names plus any slot file a run with more slots left behind, each once
    # (a second fd on the same file would conflict with our own first flock)
    for (( i = 0; i < ${LANE_SLOTS:-2}; i++ )); do locks["$q/slot-$i.lock"]=1; done
    for f in "$q"/slot-*.lock; do [ -e "$f" ] && locks["$f"]=1; done
    [ -z "$legacy" ] || locks["$legacy"]=1
    for f in "${!locks[@]}"; do
      exec {fd}>>"$f"
      if ! flock -n "$fd"; then
        echo "lane-target: slim: a build holds ${f#"$root"/}; nothing deleted, run it again when \`scripts/lane-lock.sh --status\` is idle" >&2
        [ "$strict" -eq 1 ] && exit 1; return 0
      fi
    done
  fi
  echo "incremental dirs under $root ($([ "$apply" -eq 1 ] && echo REMOVING || echo "dry run, pass --apply")); unused while CARGO_INCREMENTAL=0 (row 367):"
  for d in "$root"/*/; do
    d="${d%/}"; [ -d "$d" ] || continue; [ "$d" != "$q" ] || continue
    if [ -e "$d/.keep-incremental" ]; then echo "  kept  $d (.keep-incremental)"; continue; fi
    while IFS= read -r inc; do
      kb="$(du -sk "$inc" | cut -f1)"; total=$(( total + kb ))
      printf '  %s MiB  %s\n' "$(( kb / 1024 ))" "$inc"
      [ "$apply" -ne 1 ] || rm -rf "$inc"
    done < <(incremental_dirs "$d")
  done
  echo "  total $(( total / 1024 )) MiB"
}

case "$cmd" in
  create)
    [[ "$name" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]] || die "bad lane name '$name'"
    [ "$name" != template ] || die "'template' is reserved"
    if [ "$release" = 1 ]; then
      [ -d "$template" ] || die "no warm release tree at $template -- build it once (cold, 25-50 min, queued): scripts/lane-target.sh warm-release${base_path:+ --base-path $base_path}"
    else
      [ -d "$template" ] || die "template not found: $template (build one: CARGO_TARGET_DIR=$template dx build ... without --base-path)"
    fi
    why="$(contaminated "$template")" && die "refusing template $template: $why (base-path build, backlog row 100). Build a clean template."
    dest="$root/$name"
    [ ! -e "$dest" ] || die "$dest already exists (prune it or pick another name)"
    keep_inc="${LANE_CARGO_INCREMENTAL:-0}"
    need_kb="$(du -sk $([ "$keep_inc" = 1 ] || echo --exclude=incremental) "$template" | cut -f1)"; free_kb="$(df -Pk "$root" | awk 'NR==2{print $4}')"
    if [ "$force" -ne 1 ] && [ "$free_kb" -lt "$need_kb" ]; then
      die "only $((free_kb/1024)) MiB free under $root, template is $((need_kb/1024)) MiB (reflink may not apply; --force to try)"
    fi
    echo "lane-target: copying $template -> $dest ($((need_kb/1024)) MiB) ..." >&2
    mkdir -p "$root"
    LANE_COPY_INCREMENTAL="$keep_inc" "$(dirname "${BASH_SOURCE[0]}")/lane-slot-target.sh" copy "$template" "$dest" \
      || { rm -rf "$dest"; die "copy of $template failed"; }
    [ "$keep_inc" != 1 ] || : > "$dest/.keep-incremental"
    rm -rf "$dest"/dx/*/*/web/public   # never inherit another build's served output
    rm -f "$dest/.tree-owner"           # the first run in a new target always touches the sources (row 216)
    echo "export CARGO_TARGET_DIR=$dest"
    echo "export CARGO_INCREMENTAL=${LANE_CARGO_INCREMENTAL:-0}"
    ;;
  warm-release)
    # The refresh step: build THIS worktree's release SSG in place, in the queue, in the tree that matches the base path.
    tree="$(release_tree "$base_path")"; here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    lock_sha="$(sha1sum "$here/../Cargo.lock" | cut -c1-40)"
    if [ -d "$tree/dx" ] && [ "$force" -ne 1 ] && [ "$(cat "$tree/.cargo-lock-sha" 2>/dev/null)" = "$lock_sha" ]; then
      echo "lane-target: $tree is current: its dependencies were built from this Cargo.lock ($lock_sha). A refresh would only rebuild the workspace crates, which every build from it rebuilds anyway; --force to build anyway." >&2
      exit 0
    fi
    mkdir -p "$tree"
    if [ -z "$base_path" ]; then
      [ ! -e "$tree/.base-path-build" ] || die "$tree carries .base-path-build (a base-path build got into the no-base-path tree): delete it and rebuild cold"
      : > "$tree/.no-base-path-only"
    fi
    free_gb="$(df -Pk "$root" | awk 'NR==2{print int($4/1024/1024)}')"
    if [ ! -d "$tree/dx" ] && [ "$free_gb" -lt 7 ]; then die "cold release build needs ~5 GB; only ${free_gb} GB free under $root"; fi
    echo "lane-target: warm-release into $tree ($([ -d "$tree/dx" ] && echo "warm: workspace crates only" || echo "COLD: ~25-50 min")), queued as class lane ..." >&2
    LANE_TGT="${LANE_TGT:-$root}" "$here/lane-lock.sh" lane --label "warm-release${base_path:+:$base_path}" -- \
      env CARGO_TARGET_DIR="$tree" CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 BUILD_SSG_NETNS=1 \
      "$here/build-ssg.sh" release ${base_path:+--base-path "$base_path"} || exit 1
    echo "$lock_sha" > "$tree/.cargo-lock-sha"
    ;;
  du)
    [ -d "$root" ] || die "no such root: $root"
    du -sh "$root"/*/ 2>/dev/null | sort -h; du -sh "$root"; df -h "$root" | tail -1
    ;;
  prune)
    [ -d "$root" ] || die "no such root: $root"
    echo "lane dirs under $root idle > ${older_than}d ($([ "$apply" -eq 1 ] && echo REMOVING || echo "dry run, pass --apply")); the template is never pruned:"
    for d in "$root"/*/; do
      d="${d%/}"; [ -d "$d" ] || continue; [ "$d" != "$template" ] || continue
      # The warm release trees (row 324) are expensive to rebuild (25-50 min) and only touched by a deploy / a refresh.
      [ "$d" != "$root/release" ] && [ "$d" != "$root/release-template" ] || continue
      # "Active" = any file touched within N days (depth-bounded, first hit wins).
      [ -z "$(find "$d" -maxdepth 4 -type f -mtime "-$older_than" -print -quit 2>/dev/null)" ] || continue
      printf '  %s  %s\n' "$(du -sh "$d" | cut -f1)" "$d"
      [ "$apply" -ne 1 ] || rm -rf "$d"
    done
    slim "$apply" 0
    df -h "$root" | tail -1
    ;;
  slim)
    [ -d "$root" ] || die "no such root: $root"
    slim "$apply" 1
    df -h "$root" | tail -1
    ;;
  *) usage ;;
esac
