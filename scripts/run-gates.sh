#!/usr/bin/env bash
# The ONE way to run every pre-commit gate, in order, with timings and a summary. What each gate checks is
# dev-docs/gates.md, generated from the gate list below and the headers of scripts/check-*.sh. Every workflow
# is `workflow_dispatch`-only (CI freeze, dev-docs/backlog.md row 33), so nothing else runs these.
#
#   CARGO_TARGET_DIR=/abs/isolated/dir scripts/run-gates.sh [options]
#
#   --only REGEX      run only gates whose name matches REGEX (egrep)
#   --skip REGEX      skip gates whose name matches REGEX
#   --fast            skip the cargo gates (what a lane runs: every static check, fmt and stylelint, in seconds)
#   --stop-on-fail    stop at the first red gate (default: run everything, report all reds)
#   --log-dir DIR     per-gate logs + summary.tsv (default: $CARGO_TARGET_DIR/run-gates)
#   --list            print the gate names and exit
#   --no-reuse        run every cargo gate even when its result is stored for an identical cargo-input hash
#                     (also GATE_REUSE=0); see guard 6
#
# Exit codes: 0 = every gate ran and passed; 1 = at least one gate red; 2 = bad usage /
# environment; 3 = green but PARTIAL (--only/--skip/--stop-on-fail hid some gates) -- a partial
# run can therefore never be mistaken for the full gate.
#
# A new gate: write the script's header (its first comment paragraph is the summary gates.md prints), add one
# line to the gate list under its group, run `scripts/gates-doc.py --write`. Nothing else names a gate: not
# CLAUDE.md, not lane-rules.md.
#
# The gate list is grouped by `# group: <title>` lines; the comment lines above an entry that is not a
# scripts/check-*.sh describe it (gates.md prints them), a check script is described by its own header.
#
# Guards -- each one closes a failure class by construction instead of relying on the caller:
#  1. CARGO_TARGET_DIR must be set and ABSOLUTE, and must not be the shared repo-root `target/`
#     (dev-docs/backlog.md rows 98/100; check-attr-spread-collision.sh also builds into it).
#  2. A clippy gate is red if its log contains a "generated N warning(s)" summary even when cargo
#     exited 0 (a command run without `-D warnings`, or a cargo that replays cached diagnostics as warnings).
#  3. The summary names the tree the gates ran on (HEAD + a hash of the uncommitted diff), so a
#     green cannot be quoted for a commit it was not taken on.
#  4. Any scripts/check-*.sh that is not in the list below fails the run, so a newly added check
#     cannot silently stay out of the gate set.
#  5. stylelint runs with `pnpm exec`, which never downloads: a worktree without node_modules fails loudly
#     (`cd preview && pnpm install --frozen-lockfile`, or symlink the main checkout's node_modules --
#     backlog rows 72, 359) instead of fetching an unvetted version.
#  6. Reuse (backlog row 320). The cargo gates (CARGO_GATES: check-primitives-features, check-attr-spread-collision,
#     the three clippy arms, `cargo test`, `cargo test` on the server arm, check-rustdoc) are ~90 % of a full run, and a
#     CSS-, spec- or docs-only batch cannot change their result. So a PASS is stored per (cargo-input hash,
#     toolchain, command line) -- the key of scripts/gate-reuse.sh, whose inputs are scripts/cargo-inputs-hash.sh's
#     working-tree file set -- and a later run with the SAME key reuses it, printing `reused from <tree>`; the
#     summary lists every reused gate with the tree it really ran on and the final line says how many. Only a
#     PASS is stored, a FAIL deletes the stored PASS, a changed byte in any input is a different key, and
#     --no-reuse (or GATE_REUSE=0) forces the real run. fmt is never reused (it is under a second). The store is
#     $GATE_REUSE_DIR, default ${LANE_TGT:-/home/user/tgt}/gate-reuse.
set -uo pipefail

# The cargo gates: the slow ones, whose PASS is reused for an identical cargo-input key (guard 6) and which --fast leaves out.
CARGO_GATES='^(clippy-|test(-server)?$|check-rustdoc$|check-primitives-features$|check-attr-spread-collision$)'

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
# shellcheck source=scripts/gate-reuse.sh
. "$repo_root/scripts/gate-reuse.sh"

die() { echo "run-gates: $*" >&2; exit 2; }

only="" skip="" fast=0 stop_on_fail=0 list=0 log_dir="" reuse=1
[ "${GATE_REUSE:-1}" = 0 ] && reuse=0
while [ $# -gt 0 ]; do
  case "$1" in
    --only) only="${2:?--only needs a REGEX}"; shift 2 ;;
    --skip) skip="${2:?--skip needs a REGEX}"; shift 2 ;;
    --fast) fast=1; shift ;;
    --stop-on-fail) stop_on_fail=1; shift ;;
    --log-dir) log_dir="${2:?--log-dir needs a DIR}"; shift 2 ;;
    --list) list=1; shift ;;
    --no-reuse) reuse=0; shift ;;
    -h|--help) sed -n '2,24p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) die "unknown option: $1 (try --help)" ;;
  esac
done
[ "$fast" = 0 ] || skip="${skip:+($skip)|}$CARGO_GATES"

# --- the gate list, by group. name | command (run with bash -c from the repo root) -------
CLIPPY_FLAGS='--all-targets -- -D warnings'
gates=(
  # group: Rust and rsx source rules
  "check-preview-composition|scripts/check-preview-composition.sh"
  "check-cfg-axis|scripts/check-cfg-axis.sh"
  "check-hooks-in-closures|scripts/check-hooks-in-closures.sh"
  "check-self-subscribing-effects|scripts/check-self-subscribing-effects.sh"
  "check-component-call|scripts/check-component-call.sh"
  "check-test-harness-controlled|scripts/check-test-harness-controlled.sh"
  "check-attr-spread-collision|scripts/check-attr-spread-collision.sh"
  "check-direction-keys|scripts/check-direction-keys.sh"
  "check-render-keys|scripts/check-render-keys.sh"
  "check-render-clock|scripts/check-render-clock.sh"
  "check-collection-derive|scripts/check-collection-derive.sh"
  "check-raw-text-interpolation|scripts/check-raw-text-interpolation.sh"
  "check-dynamic-style-shorthand|scripts/check-dynamic-style-shorthand.sh"
  "check-as-child|scripts/check-as-child.sh"
  "check-wrapper-props|scripts/check-wrapper-props.sh"
  "check-wrapper-spread|scripts/check-wrapper-spread.sh"
  # group: Timers, listeners, motion and focus
  "check-uncleared-intervals|scripts/check-uncleared-intervals.sh"
  "check-blocking-scroll-listeners|scripts/check-blocking-scroll-listeners.sh"
  "check-js-listeners|scripts/check-js-listeners.sh"
  "check-motion-gating|scripts/check-motion-gating.sh"
  "check-infinite-animations|scripts/check-infinite-animations.sh"
  "check-open-focus|scripts/check-open-focus.sh"
  "check-theme-mode-writer|scripts/check-theme-mode-writer.sh"
  # group: Theme and component CSS
  "check-dx-class-prefix|scripts/check-dx-class-prefix.sh"
  "check-cross-component-overrides|scripts/check-cross-component-overrides.sh"
  "check-css-literals|scripts/check-css-literals.sh"
  "check-css-logical-properties|scripts/check-css-logical-properties.sh"
  "check-css-vars-defined|scripts/check-css-vars-defined.sh"
  "check-hover-mix-space|scripts/check-hover-mix-space.sh"
  "check-coarse-sizing|scripts/check-coarse-sizing.sh"
  "check-press-under|scripts/check-press-under.sh"
  "check-anchored-keyframes|scripts/check-anchored-keyframes.sh"
  "check-duplicate-keyframes|scripts/check-duplicate-keyframes.sh"
  "check-theme-baseline|scripts/check-theme-baseline.sh"
  "check-theme-preset-axes|scripts/check-theme-preset-axes.sh"
  "check-cursor-baseline|scripts/check-cursor-baseline.sh"
  "check-popup-boundary|scripts/check-popup-boundary.sh"
  "check-variant-scope|scripts/check-variant-scope.sh"
  "check-popup-root-reset|scripts/check-popup-root-reset.sh"
  "check-component-root-reset|scripts/check-component-root-reset.sh"
  "check-destructive-text|scripts/check-destructive-text.sh"
  "check-css-state-recipes|scripts/check-css-state-recipes.sh"
  "check-focus-outline-reset|scripts/check-focus-outline-reset.sh"
  "check-hover-media|scripts/check-hover-media.sh"
  "check-scroll-behavior|scripts/check-scroll-behavior.sh"
  "check-select-css|scripts/check-select-css.sh"
  "check-component-fonts|scripts/check-component-fonts.sh"
  "check-demo-wrapper-width|scripts/check-demo-wrapper-width.sh"
  # group: Docs site, demos and delivery
  "check-eager-head-document|scripts/check-eager-head-document.sh"
  "check-css-delivery|scripts/check-css-delivery.sh"
  "check-internal-hrefs|scripts/check-internal-hrefs.sh"
  "check-demo-forms|scripts/check-demo-forms.sh"
  "check-demo-remote-assets|scripts/check-demo-remote-assets.sh"
  "check-docs-no-demo-list|scripts/check-docs-no-demo-list.sh"
  "check-docs-own-components|scripts/check-docs-own-components.sh"
  "check-props-docs|scripts/check-props-docs.sh"
  "check-virtual-docs-parity|scripts/check-virtual-docs-parity.sh"
  "check-ssg-verify|scripts/check-ssg-verify.sh"
  # group: Registry and installed components
  "check-component-manifests|scripts/check-component-manifests.sh"
  "check-component-catalog|scripts/check-component-catalog.sh"
  "check-installed-paths|scripts/check-installed-paths.sh"
  "check-registry-url|scripts/check-registry-url.sh"
  "check-cargo-dep-versions|scripts/check-cargo-dep-versions.sh"
  "check-markdown-parser|scripts/check-markdown-parser.sh"
  "check-js-bundle|scripts/check-js-bundle.sh"
  "check-cargo-locks|scripts/check-cargo-locks.sh"
  # group: Playwright specs
  "check-playwright-fixture-import|scripts/check-playwright-fixture-import.sh"
  "check-spec-viewport-bound|scripts/check-spec-viewport-bound.sh"
  "check-spec-at-rest|scripts/check-spec-at-rest.sh"
  "check-spec-font-state|scripts/check-spec-font-state.sh"
  "check-timing-relative|scripts/check-timing-relative.sh"
  "check-spec-goto-hydrated|scripts/check-spec-goto-hydrated.sh"
  "check-demo-text-locators|scripts/check-demo-text-locators.sh"
  "check-slow-spec-budget|scripts/check-slow-spec-budget.sh"
  "check-chromium-path|scripts/check-chromium-path.sh"
  # group: Repo docs and tooling
  "check-backlog-index|scripts/check-backlog-index.sh"
  "check-history-comments|scripts/check-history-comments.sh"
  "check-comment-refs|scripts/check-comment-refs.sh"
  "check-history-names|scripts/check-history-names.sh"
  "check-supply-chain-doc|scripts/check-supply-chain-doc.sh"
  "check-gates-doc|scripts/check-gates-doc.sh"
  "check-lane-lock|scripts/check-lane-lock.sh"
  "check-gate-reuse|scripts/check-gate-reuse.sh"
  "check-gate-kit|scripts/check-gate-kit.sh"
  "check-lane-target|scripts/check-lane-target.sh"
  "check-merge-drivers|scripts/check-merge-drivers.sh"

  # group: Rust build, lint, test and style
  # Every Rust file is rustfmt-formatted (`cargo fmt --all -- --check`). Never reused from the store: it takes under a second.
  "fmt|cargo fmt --all -- --check"
  "check-primitives-features|scripts/check-primitives-features.sh"
  # Clippy, default arm: `cargo clippy --workspace --all-targets -- -D warnings` lints the `not(feature = "web")` arm of primitives and every workspace member. A plain `cargo clippy --workspace` never enables primitives' `web` feature or preview's `server` feature, and those two are what ships (`dx build --ssg` compiles the browser client with `web` and the prerender server with `server`), so each arm is its own gate; `--all-features` cannot replace them (preview's `desktop` needs gtk and webkit2gtk, the renderers are alternatives, and it would stop linting the `not(web)` arm). Every clippy gate is also red when its log holds a `generated N warning(s)` line although cargo exited 0.
  "clippy-default|cargo clippy --workspace $CLIPPY_FLAGS"
  # Clippy, `web` arm: primitives and preview with their `web` features, the code the wasm client compiles (over 100 `feature = "web"` cfg sites in primitives alone).
  "clippy-web|cargo clippy -p dioxus-primitives -p preview --features dioxus-primitives/web,preview/web $CLIPPY_FLAGS"
  # Clippy, `server` arm: preview with `server` (the SSG prerender server; implies fullstack and the primitives `web` feature).
  "clippy-server|cargo clippy -p preview --features server $CLIPPY_FLAGS"
  # `cargo test --workspace`: every unit and integration test of the workspace on the plain arm (no `web` or `server` feature); the server arm is `test-server`.
  "test|cargo test --workspace"
  # `cargo test -p preview --features server`: the arm `dx build --ssg` ships, where `ssr_parity` and the web-only effects differ from the plain arm.
  "test-server|cargo test -p preview --features server"
  "check-rustdoc|scripts/check-rustdoc.sh"
  # Stylelint over every stylesheet under `preview/src` and `preview/assets` (run with `pnpm exec`, which never downloads: a worktree without `node_modules` fails loudly).
  "stylelint|cd preview && pnpm exec stylelint \"src/**/*.css\" \"assets/**/*.css\""
)

if [ "$list" = 1 ]; then
  for g in "${gates[@]}"; do printf '%s\t%s\n' "${g%%|*}" "${g#*|}"; done
  exit 0
fi

# --- environment ---------------------------------------------------------------------------
case "${CARGO_TARGET_DIR:-}" in
  "") die "CARGO_TARGET_DIR is not set. Use an ABSOLUTE, isolated dir (scripts/lane-target.sh create <name>); never the shared repo-root target/." ;;
  /*) ;;
  *) die "CARGO_TARGET_DIR must be absolute (dx build --ssg breaks on a relative one, backlog row 98): $CARGO_TARGET_DIR" ;;
esac
[ "${CARGO_TARGET_DIR%/}" != "$repo_root/target" ] || die "CARGO_TARGET_DIR must not be the shared repo-root target/ (backlog row 100): $CARGO_TARGET_DIR"
export CARGO_TARGET_DIR
# Gates are one-shot checks, so the incremental cache is mostly dead weight: it regrows to
# 11-13 GB in about an hour of shared gate runs and fills the session's disk (backlog row 168).
# Default it off; a caller can still opt back in with CARGO_INCREMENTAL=1.
export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"
log_dir="${log_dir:-$CARGO_TARGET_DIR/run-gates}"
mkdir -p "$log_dir" || die "cannot create log dir $log_dir"
summary_tsv="$log_dir/summary.tsv"
: > "$summary_tsv"

# --- guard 4: every scripts/check-*.sh must be in the list -----------------------------------
unlisted=()
for f in scripts/check-*.sh; do
  [ -e "$f" ] || continue
  listed=0
  for g in "${gates[@]}"; do
    [ "${g#*|}" = "$f" ] && { listed=1; break; }
  done
  [ "$listed" = 1 ] || unlisted+=("$f")
done
if [ "${#unlisted[@]}" -gt 0 ]; then
  echo "run-gates: scripts not in the gate list (add to run-gates.sh, then scripts/gates-doc.py --write): ${unlisted[*]}" >&2
  exit 2
fi

# --- guard 3: which tree is this green about -----------------------------------------------
head_sha="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
dirty_files="$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')"
diff_hash="$( { git diff HEAD 2>/dev/null; git ls-files -o --exclude-standard 2>/dev/null | sort | xargs -r sha1sum 2>/dev/null; } | sha1sum | cut -c1-12 )"
echo "run-gates: tree $head_sha, $dirty_files uncommitted path(s), diff-hash $diff_hash"
echo "run-gates: CARGO_TARGET_DIR=$CARGO_TARGET_DIR  logs=$log_dir"
echo "run-gates: $(cargo --version), $(cargo clippy --version)"

now() { date +%s.%N; }

# Row 216: the cargo gates build in a target other worktrees may also have used, and cargo judges freshness by
# mtime, so touch this worktree's sources once, right before the first cargo gate that actually runs (a
# fast-gates-only run builds nothing and must not stamp the target). lane-touch skips it when the target's
# `.tree-owner` stamp already names this worktree.
touched=0

# Guard 6 state: the reuse key is computed once, lazily, at the first cargo gate that would run.
cargo_cmds=() reuse_key="" reuse_state=unset reuse_pending=() reused=() reused_from=""
tree_label="$head_sha"; [ "$dirty_files" = 0 ] || tree_label="$head_sha+dirty:$diff_hash"
reuse_init() { # sets reuse_state to on|off
  [ "$reuse_state" = unset ] || return 0
  reuse_state=off
  [ "$reuse" = 1 ] || { echo "run-gates: reuse off (--no-reuse / GATE_REUSE=0): every cargo gate runs"; return 0; }
  local g
  for g in "${gates[@]}"; do [[ "${g%%|*}" =~ $CARGO_GATES ]] && cargo_cmds+=("${g%%|*}=${g#*|}"); done
  if ! reuse_key="$(gate_reuse_key "$repo_root" "${cargo_cmds[@]}")" || [ -z "$reuse_key" ]; then
    echo "run-gates: reuse off (could not compute the cargo-input key): every cargo gate runs"; return 0
  fi
  if ! mkdir -p "$(gate_reuse_store)" 2>/dev/null; then
    echo "run-gates: reuse off (cannot create $(gate_reuse_store)): every cargo gate runs"; return 0
  fi
  gate_reuse_prune
  reuse_state=on
  echo "run-gates: cargo-input key $reuse_key (store $(gate_reuse_store))"
}

red=() ran=0 hidden=0
for g in "${gates[@]}"; do
  name="${g%%|*}"; cmd="${g#*|}"
  if { [ -n "$only" ] && ! [[ "$name" =~ $only ]]; } || { [ -n "$skip" ] && [[ "$name" =~ $skip ]]; }; then
    hidden=$((hidden + 1)); continue
  fi
  if [[ "$name" =~ $CARGO_GATES ]]; then
    reuse_init
    if [ "$reuse_state" = on ] && hit="$(gate_reuse_lookup "$reuse_key" "$name")"; then
      IFS=$'\t' read -r from_tree from_date from_secs <<<"$hit"
      printf '>> %-34s PASS  reused from tree %s (ran %s, took %ss)\n' "$name" "$from_tree" "$from_date" "$from_secs"
      printf '%s\tPASS\t0\t0.0\treused from tree %s (%s, %ss)\n' "$name" "$from_tree" "$from_date" "$from_secs" >> "$summary_tsv"
      reused+=("$name")
      case ", $reused_from, " in *", $from_tree, "*) ;; *) reused_from="${reused_from:+$reused_from, }$from_tree" ;; esac
      ran=$((ran + 1)); continue
    fi
    if [ "$touched" = 0 ]; then
      scripts/lane-touch.sh "$CARGO_TARGET_DIR" || die "lane-touch failed"
      touched=1
    fi
  fi
  log="$log_dir/$name.log"
  printf '>> %-34s ' "$name"
  t0="$(now)"
  # `nice` keeps the box usable when other lanes compile; the cargo gates are the slow ones.
  nice -n 10 bash -c "$cmd" > "$log" 2>&1
  rc=$?
  t1="$(now)"
  secs="$(awk -v a="$t0" -v b="$t1" 'BEGIN { printf "%.1f", b - a }')"
  status=PASS why=""
  if [ "$rc" -ne 0 ]; then
    status=FAIL why="exit $rc"
  elif [[ "$name" == clippy-* ]] && grep -Eq 'generated [0-9]+ warnings?' "$log"; then
    status=FAIL why="exit 0 but clippy printed warnings (guard 2)"
  fi
  ran=$((ran + 1))
  printf '%s  %7ss  %s\n' "$status" "$secs" "$why"
  printf '%s\t%s\t%s\t%s\t%s\n' "$name" "$status" "$rc" "$secs" "$log" >> "$summary_tsv"
  if [ "$reuse_state" = on ] && [[ "$name" =~ $CARGO_GATES ]]; then
    if [ "$status" = PASS ]; then reuse_pending+=("$name:$secs"); else gate_reuse_forget "$reuse_key" "$name"; fi
  fi
  if [ "$status" = FAIL ]; then
    red+=("$name")
    echo "   --- last 25 lines of $log"; tail -n 25 "$log" | sed 's/^/   | /'
    if [ "$stop_on_fail" = 1 ]; then hidden=$((hidden + 1)); echo "run-gates: --stop-on-fail"; break; fi
  fi
done

# Guard 6: store the PASSes of the cargo gates that really ran -- only if the cargo inputs are still what the key
# was taken on (a tree edited under a running gate must not vouch for the new bytes).
if [ "${#reuse_pending[@]}" -gt 0 ]; then
  if [ "$(gate_reuse_key "$repo_root" "${cargo_cmds[@]}" 2>/dev/null)" = "$reuse_key" ]; then
    for p in "${reuse_pending[@]}"; do gate_reuse_record "$reuse_key" "${p%%:*}" "$tree_label" "${p#*:}"; done
    echo "run-gates: stored ${#reuse_pending[@]} cargo-gate PASS result(s) under key $reuse_key"
  else
    echo "run-gates: the cargo inputs changed while the gates ran -- nothing stored for reuse"
  fi
fi

echo
echo "================ run-gates summary (tree $head_sha, diff-hash $diff_hash) ================"
awk -F'\t' '{ printf "%-34s %-5s rc=%-3s %8ss  %s\n", $1, $2, $3, $4, $5 }' "$summary_tsv"
total="$(awk -F'\t' '{ s += $4 } END { printf "%.1f", s }' "$summary_tsv")"
echo "gates run: $ran   red: ${#red[@]}   total: ${total}s"
if [ "${#reused[@]}" -gt 0 ]; then
  echo "REUSED (not run now, PASS stored for an identical cargo-input key $reuse_key): ${reused[*]}"
  echo "  taken from tree(s): $reused_from  -- the cargo inputs of $tree_label are byte-identical to theirs"
fi
if [ "${#red[@]}" -gt 0 ]; then
  echo "RED: ${red[*]}"
  exit 1
fi
if [ "$hidden" -gt 0 ] || [ "$ran" -ne "${#gates[@]}" ]; then
  echo "PARTIAL: $ran of ${#gates[@]} gates ran, all green -- this is NOT the full gate"
  exit 3
fi
echo "ALL GREEN: all ${#gates[@]} gates passed on $head_sha (diff-hash $diff_hash)"
[ "${#reused[@]}" -eq 0 ] || echo "  of which ${#reused[@]} cargo gate(s) reused from tree $reused_from (identical cargo inputs, key $reuse_key); --no-reuse for a full run"
