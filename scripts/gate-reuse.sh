#!/usr/bin/env bash
# Sourced by scripts/run-gates.sh and scripts/check-gate-reuse.sh (backlog row 320): the store behind "a cargo gate
# whose inputs did not change since it last passed is not run again".
#
# A result is valid for exactly one KEY. The key is a hash of
#   - FORMAT (bump it when this file's semantics change),
#   - scripts/cargo-inputs-hash.sh's output (every file the cargo gates can read, see that script),
#   - the toolchain (`rustc -Vv`, `cargo -V`, `cargo clippy -V`) and the RUSTFLAGS-family environment,
#   - the exact command line of every reusable gate (a changed clippy flag is a different gate),
#   - the bytes of cargo-inputs-hash.sh and of this file (a fix to the selection rules drops every old result).
# Only a PASS is ever stored; a FAIL under a key deletes any stored PASS of that gate (a flake is never trusted).
# One file per (key, gate): $STORE/<key>/<gate>, written through a rename, so concurrent runs never need a lock.
#
#   gate_reuse_store                      -> the store directory
#   gate_reuse_key REPO_ROOT CMDLINES...  -> the key (stdout)
#   gate_reuse_lookup KEY GATE            -> stdout "<tree>\t<iso date>\t<secs>", rc 0 on a stored PASS, else rc 1
#   gate_reuse_record KEY GATE TREE SECS  -> store a PASS
#   gate_reuse_forget KEY GATE            -> delete a stored result
#   gate_reuse_prune                      -> drop keys not written for 21 days

GATE_REUSE_FORMAT=1

gate_reuse_store() {
  printf '%s\n' "${GATE_REUSE_DIR:-${LANE_TGT:-/home/user/tgt}/gate-reuse}"
}

gate_reuse_key() { # $1 = repo root, rest = the reusable gates' command lines
  local root="$1"; shift
  local inputs
  inputs="$("$root/scripts/cargo-inputs-hash.sh" --root "$root")" || return 1
  {
    echo "format $GATE_REUSE_FORMAT"
    echo "inputs $inputs"
    rustc -Vv 2>&1; cargo -V 2>&1; cargo clippy -V 2>&1
    echo "env RUSTFLAGS=${RUSTFLAGS:-} RUSTDOCFLAGS=${RUSTDOCFLAGS:-} ENC=${CARGO_ENCODED_RUSTFLAGS:-} BUILD_TARGET=${CARGO_BUILD_TARGET:-}"
    printf 'cmd %s\n' "$@"
    echo "scripts $(sha1sum < "$root/scripts/cargo-inputs-hash.sh" | cut -c1-40) $(sha1sum < "$root/scripts/gate-reuse.sh" | cut -c1-40)"
  } | sha1sum | cut -c1-40
}

_gate_reuse_file() { printf '%s/%s/%s\n' "$(gate_reuse_store)" "$1" "$2"; }

gate_reuse_lookup() { # prints "<tree>\t<date>\t<secs>"
  local f; f="$(_gate_reuse_file "$1" "$2")"
  [ -f "$f" ] || return 1
  awk -F'\t' 'NR == 1 && $1 == "PASS" && NF >= 5 { print $2 "\t" $3 "\t" $4; ok = 1 } END { exit !ok }' "$f"
}

gate_reuse_record() { # KEY GATE TREE SECS
  local key="$1" gate="$2" tree="$3" secs="$4" dir tmp
  dir="$(gate_reuse_store)/$key"
  mkdir -p "$dir" 2>/dev/null || return 1
  tmp="$(mktemp "$dir/.tmp.XXXXXX")" || return 1
  printf 'PASS\t%s\t%s\t%s\tformat%s\n' "$tree" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$secs" "$GATE_REUSE_FORMAT" > "$tmp"
  mv -f "$tmp" "$dir/$gate"
}

gate_reuse_forget() {
  rm -f "$(_gate_reuse_file "$1" "$2")" 2>/dev/null || true
}

gate_reuse_prune() {
  local s; s="$(gate_reuse_store)"
  [ -d "$s" ] || return 0
  find "$s" -mindepth 1 -maxdepth 1 -type d -mtime +21 -exec rm -rf {} + 2>/dev/null || true
}
